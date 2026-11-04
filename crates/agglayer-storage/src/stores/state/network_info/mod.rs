//! Network information storage implementation.
//!
//! This module implements the `NetworkInfoReader` trait for `StateStore`,
//! providing functionality to read and retrieve network-related information
//! from the database.
//!
//! Every read comes from the network info column and the certificate headers
//! it points to, through one database snapshot. The legacy latest-certificate
//! indexes are not read here: the backfill migration fills the column from
//! them once, and the runtime writers keep it current.
use std::collections::BTreeMap;

use agglayer_types::{
    CertificateId, CertificateStatus, Digest, Height, NetworkId, NetworkInfo, NetworkType,
};
use rocksdb::WriteBatch;
use tracing::warn;

use crate::{
    columns::{certificate_header::CertificateHeaderColumn, network_info::NetworkInfoColumn},
    error::Error,
    schema::CodecError,
    storage::{DBError, Snapshot},
    stores::{expected_type_or_fail, state::StateStore, try_digest, LatestCertificateHeaders},
    types::network_info::{
        self,
        v0::{
            network_info_value::{self, ValueDiscriminants},
            LatestPendingCertificateHeight, LatestPendingCertificateId,
            LatestPendingCertificateInfo, LatestProvenCertificateInfo, SettledClaim,
            SettledLocalExitTreeLeafCount, SettledPessimisticProofRoot,
        },
    },
};

/// The settled row: the certificate it names and the aggregates recorded
/// with it.
struct SettledRow {
    certificate_id: CertificateId,
    pp_root: Option<Digest>,
    let_leaf_count: Option<u64>,
}

/// A network's pointer rows, decoded. A missing row decodes to `None`.
#[derive(Default)]
struct CachedPointers {
    pending: Option<(CertificateId, Height)>,
    proven: Option<CertificateId>,
    settled: Option<SettledRow>,
}

impl CachedPointers {
    /// The row kinds that point to a certificate.
    const KINDS: [ValueDiscriminants; 3] = [
        ValueDiscriminants::LatestPendingCertificateInfo,
        ValueDiscriminants::LatestProvenCertificateInfo,
        ValueDiscriminants::SettledCertificate,
    ];

    /// Decode a pointer row into its slot. The other kinds point to no
    /// certificate and are not decoded here, so a type or claim value this
    /// binary does not know cannot stop a header read.
    fn read(
        &mut self,
        kind: ValueDiscriminants,
        value: Option<network_info::Value>,
    ) -> Result<(), Error> {
        match kind {
            ValueDiscriminants::LatestPendingCertificateInfo => {
                self.pending = StateStore::decode_pending(value)?
            }
            ValueDiscriminants::LatestProvenCertificateInfo => {
                self.proven = StateStore::decode_proven(value)?
            }
            ValueDiscriminants::SettledCertificate => {
                self.settled = StateStore::decode_settled(value)?
            }
            ValueDiscriminants::NetworkType | ValueDiscriminants::SettledClaim => {}
        }

        Ok(())
    }

    fn is_empty(&self) -> bool {
        self.pending.is_none() && self.proven.is_none() && self.settled.is_none()
    }
}

/// All of a network's cache rows, decoded. A missing row decodes to `None`.
#[derive(Default)]
struct CachedRows {
    pointers: CachedPointers,
    network_type: Option<NetworkType>,
    settled_claim: Option<agglayer_types::SettledClaim>,
}

impl CachedRows {
    /// Decode one row into its slot.
    fn read(
        &mut self,
        kind: ValueDiscriminants,
        value: Option<network_info::Value>,
    ) -> Result<(), Error> {
        match kind {
            ValueDiscriminants::NetworkType => {
                self.network_type = StateStore::decode_network_type(value)?
            }
            ValueDiscriminants::SettledClaim => {
                self.settled_claim = StateStore::decode_claim(value)?
            }
            ValueDiscriminants::LatestPendingCertificateInfo
            | ValueDiscriminants::LatestProvenCertificateInfo
            | ValueDiscriminants::SettledCertificate => self.pointers.read(kind, value)?,
        }

        Ok(())
    }

    /// Whether the network has no row at all, which makes it unknown.
    fn is_empty(&self) -> bool {
        self.network_type.is_none() && self.settled_claim.is_none() && self.pointers.is_empty()
    }
}

impl StateStore {
    /// One row of a network's info record, keyed by the kind of value it holds.
    ///
    /// Deriving the key from the value is what keeps the two in step: a value
    /// can never be stored under another kind's key.
    pub(super) fn network_info_row(
        network_id: NetworkId,
        value: network_info_value::Value,
    ) -> (network_info::Key, network_info::Value) {
        let key = network_info::Key {
            network_id: network_id.to_u32(),
            kind: (&value).into(),
        };

        (key, network_info::Value { value: Some(value) })
    }

    pub(super) fn stage_network_info(
        &self,
        network_id: NetworkId,
        value: network_info_value::Value,
        batch: &mut WriteBatch,
    ) -> Result<(), Error> {
        let (key, value) = Self::network_info_row(network_id, value);

        Ok(self
            .db
            .multi_insert_batch::<NetworkInfoColumn>([(&key, &value)], batch)?)
    }

    /// The settled row: every field a settlement knows, so the row is never
    /// partial. The optional aggregates stay unset only when their source is
    /// gone.
    pub(super) fn settled_certificate_value(
        certificate_id: CertificateId,
        ler: agglayer_tries::roots::LocalExitRoot,
        pp_root: Option<agglayer_types::Digest>,
        let_leaf_count: Option<u64>,
    ) -> network_info_value::Value {
        network_info_value::Value::SettledCertificate(network_info::v0::SettledCertificate {
            certificate_id: Some(network_info::v0::SettledCertificateId {
                id: certificate_id.as_digest().as_slice().to_vec().into(),
            }),
            ler: Some(network_info::v0::SettledLocalExitRoot {
                root: agglayer_types::Digest::from(ler).as_slice().to_vec().into(),
            }),
            pp_root: pp_root.map(|root| SettledPessimisticProofRoot {
                root: root.as_slice().to_vec().into(),
            }),
            let_leaf_count: let_leaf_count.map(|settled_let_leaf_count| {
                SettledLocalExitTreeLeafCount {
                    settled_let_leaf_count,
                }
            }),
        })
    }

    pub(super) fn settled_claim_value(
        claim: &agglayer_types::SettledClaim,
    ) -> network_info_value::Value {
        network_info_value::Value::SettledClaim(SettledClaim {
            global_index: Some(network_info::v0::GlobalIndex {
                value: claim.global_index.as_slice().to_vec().into(),
            }),
            bridge_exit_hash: Some(network_info::v0::BridgeExitHash {
                bridge_exit_hash: claim.bridge_exit_hash.as_slice().to_vec().into(),
            }),
        })
    }

    fn decode_network_type(
        value: Option<network_info::Value>,
    ) -> Result<Option<NetworkType>, Error> {
        expected_type_or_fail!(
            value,
            network_info_value::Value::NetworkType(network_type),
            network_info::v0::NetworkType::try_from(network_type)
                .map_err(|_| {
                    Error::Unexpected("Unable to deserialize NetworkType from integer".to_string())
                })?
                .try_into()
                .map_err(|_| Error::Unexpected(
                    "Unable to convert storage NetworkType to agglayer_types::NetworkType"
                        .to_string(),
                ))?,
            "Wrong value type decode, was expecting NetworkType, decoded another type"
        )
    }

    fn decode_pending(
        value: Option<network_info::Value>,
    ) -> Result<Option<(CertificateId, Height)>, Error> {
        expected_type_or_fail!(
            value,
            network_info_value::Value::LatestPendingCertificateInfo(LatestPendingCertificateInfo {
                height: Some(LatestPendingCertificateHeight { height }),
                id: Some(LatestPendingCertificateId { id }),
            }),
            (
                CertificateId::from(try_digest!(&*id, "LatestPendingCertificateId")?),
                height.into(),
            ),
            "Wrong value type decoded, was expecting LatestPendingCertificateInfo, decoded \
             another type"
        )
    }

    fn decode_proven(value: Option<network_info::Value>) -> Result<Option<CertificateId>, Error> {
        expected_type_or_fail!(
            value,
            network_info_value::Value::LatestProvenCertificateInfo(LatestProvenCertificateInfo {
                id
            }),
            CertificateId::from(try_digest!(&*id, "LatestProvenCertificateId")?),
            "Wrong value type decoded, was expecting LatestProvenCertificateId, decoded another \
             type"
        )
    }

    fn decode_settled(value: Option<network_info::Value>) -> Result<Option<SettledRow>, Error> {
        expected_type_or_fail!(
            value,
            network_info_value::Value::SettledCertificate(network_info::v0::SettledCertificate {
                certificate_id: Some(network_info::v0::SettledCertificateId { id }),
                pp_root,
                let_leaf_count,
                ..
            }),
            SettledRow {
                certificate_id: try_digest!(&*id, "SettledCertificateId")?.into(),
                pp_root: pp_root
                    .map(|SettledPessimisticProofRoot { root }| {
                        try_digest!(&*root, "PessimisticProofRoot")
                    })
                    .transpose()?,
                let_leaf_count: let_leaf_count.map(
                    |SettledLocalExitTreeLeafCount {
                         settled_let_leaf_count,
                     }| settled_let_leaf_count,
                ),
            },
            "Wrong value type decoded, was expecting SettledCertificate, decoded another type"
        )
    }

    fn decode_claim(
        value: Option<network_info::Value>,
    ) -> Result<Option<agglayer_types::SettledClaim>, Error> {
        expected_type_or_fail!(
            value,
            network_info_value::Value::SettledClaim(SettledClaim {
                global_index: Some(global_index),
                bridge_exit_hash: Some(bridge_exit_hash),
            }),
            agglayer_types::SettledClaim {
                global_index: try_digest!(&*global_index.value, "GlobalIndex")?,
                bridge_exit_hash: try_digest!(
                    &*bridge_exit_hash.bridge_exit_hash,
                    "BridgeExitHash"
                )?,
            },
            "Wrong value type decoded, was expecting SettledClaim, decoded another type"
        )
    }

    /// A network's cache rows, from `snapshot`.
    fn cached_rows(snapshot: &Snapshot<'_>, network_id: NetworkId) -> Result<CachedRows, Error> {
        let keys = network_info::Key::all_keys_for_network(network_id);
        let mut rows = CachedRows::default();
        for (value, network_info::Key { kind, .. }) in snapshot
            .multi_get::<NetworkInfoColumn>(keys.clone())?
            .into_iter()
            .zip(keys)
        {
            rows.read(kind, value)?;
        }

        Ok(rows)
    }

    /// A network's pointer rows, from `snapshot`.
    fn cached_pointers(
        snapshot: &Snapshot<'_>,
        network_id: NetworkId,
    ) -> Result<CachedPointers, Error> {
        let keys = CachedPointers::KINDS.map(|kind| network_info::Key {
            network_id: network_id.to_u32(),
            kind,
        });
        let mut pointers = CachedPointers::default();
        for (value, network_info::Key { kind, .. }) in snapshot
            .multi_get::<NetworkInfoColumn>(keys.clone())?
            .into_iter()
            .zip(keys)
        {
            pointers.read(kind, value)?;
        }

        Ok(pointers)
    }

    /// Every network's pointer rows, from one scan of `snapshot`. A network
    /// counts as soon as it has any row this binary knows.
    ///
    /// A row kind this binary does not know is skipped with a warning rather
    /// than failing the scan: a newer binary may have written it, and it
    /// points to no certificate this one would read.
    fn all_cached_pointers(
        snapshot: &Snapshot<'_>,
    ) -> Result<BTreeMap<NetworkId, CachedPointers>, Error> {
        let mut networks = BTreeMap::<NetworkId, CachedPointers>::new();
        for row in snapshot.iter::<NetworkInfoColumn>()? {
            let (network_info::Key { network_id, kind }, value) = match row {
                Ok(row) => row,
                Err(DBError::CodecError(CodecError::InvalidEnumVariant(reason))) => {
                    warn!(reason, "Skipping a network info row of an unknown kind");
                    continue;
                }
                Err(error) => return Err(error.into()),
            };
            networks
                .entry(NetworkId::new(network_id))
                .or_default()
                .read(kind, Some(value))?;
        }

        Ok(networks)
    }

    /// The headers the cached pointers name, from `snapshot`.
    ///
    /// A pointer without its header is inconsistent storage: the writers
    /// publish a pointer with or after its header, and the backfill skips a
    /// legacy pending or proven pointer without one and fails on a settled
    /// one.
    fn join_headers(
        snapshot: &Snapshot<'_>,
        network_id: NetworkId,
        pointers: &CachedPointers,
    ) -> Result<LatestCertificateHeaders, Error> {
        let pointers = [
            (
                "pending",
                pointers.pending.map(|(certificate_id, _)| certificate_id),
            ),
            ("proven", pointers.proven),
            (
                "settled",
                pointers
                    .settled
                    .as_ref()
                    .map(|settled| settled.certificate_id),
            ),
        ];
        let mut headers = snapshot
            .multi_get::<CertificateHeaderColumn>(
                pointers
                    .iter()
                    .filter_map(|(_, certificate_id)| *certificate_id),
            )?
            .into_iter();
        let mut header_of = |(kind, certificate_id): (&'static str, Option<CertificateId>)| {
            certificate_id
                .map(|certificate_id| {
                    headers
                        .next()
                        .flatten()
                        .ok_or(Error::DanglingCertificatePointer {
                            network_id,
                            kind,
                            certificate_id,
                        })
                })
                .transpose()
        };

        Ok(LatestCertificateHeaders {
            pending: header_of(pointers[0])?,
            proven: header_of(pointers[1])?,
            settled: header_of(pointers[2])?,
        })
    }

    /// The network's info from `snapshot`: its cache rows, and the headers of
    /// the certificates they point to.
    fn network_info_from_snapshot(
        snapshot: &Snapshot<'_>,
        network_id: NetworkId,
    ) -> Result<Option<NetworkInfo>, Error> {
        let rows = Self::cached_rows(snapshot, network_id)?;
        if rows.is_empty() {
            return Ok(None);
        }
        let CachedRows {
            mut pointers,
            network_type,
            settled_claim,
        } = rows;

        let mut state = NetworkInfo::from_network_id(network_id);
        state.network_type = network_type.unwrap_or(NetworkType::Unspecified);
        state.settled_claim = settled_claim;
        // The live LET may already include the next certificate before its
        // settled row is published. Only the count recorded alongside the
        // settled pointer belongs to this settlement; missing counts stay
        // absent.
        if let Some(settled) = &pointers.settled {
            state.settled_pp_root = settled.pp_root;
            state.settled_let_leaf_count = settled.let_leaf_count;
        }
        // The info serves no proven field, so the proven header is not read:
        // only the headers it serves can fail it.
        pointers.proven = None;
        let headers = Self::join_headers(snapshot, network_id, &pointers)?;
        // Height, epoch, status, error, and the settled local exit root are
        // certificate-header facts; the cached pending height is not
        // authoritative.
        if let Some(header) = headers.settled {
            state.settled_certificate_id = Some(header.certificate_id);
            state.settled_height = Some(header.height);
            state.settled_ler = Some(header.new_local_exit_root);
            state.latest_epoch_with_settlement = header.epoch_number.map(|epoch| epoch.as_u64());
        }
        if let Some(header) = headers.pending {
            state.latest_pending_certificate_id = Some(header.certificate_id);
            state.latest_pending_height = Some(header.height);
            if let CertificateStatus::InError { error } = &header.status {
                state.latest_pending_error = Some(error.as_ref().clone());
            }
            state.latest_pending_status = Some(header.status);
        }

        Ok(Some(state))
    }
}

impl crate::stores::NetworkInfoReader for StateStore {
    fn get_network_info(&self, network_id: NetworkId) -> Result<Option<NetworkInfo>, Error> {
        Self::network_info_from_snapshot(&self.db.snapshot(), network_id)
    }

    fn get_latest_certificate_headers(
        &self,
        network_id: NetworkId,
    ) -> Result<LatestCertificateHeaders, Error> {
        let snapshot = self.db.snapshot();
        let pointers = Self::cached_pointers(&snapshot, network_id)?;

        Self::join_headers(&snapshot, network_id, &pointers)
    }

    fn get_all_latest_certificate_headers(
        &self,
    ) -> Result<BTreeMap<NetworkId, LatestCertificateHeaders>, Error> {
        let snapshot = self.db.snapshot();

        Self::all_cached_pointers(&snapshot)?
            .iter()
            .map(|(network_id, pointers)| {
                Ok((
                    *network_id,
                    Self::join_headers(&snapshot, *network_id, pointers)?,
                ))
            })
            .collect()
    }
}

/// Single-row reads of the cache, for tests. They read one row outside any
/// snapshot and without its header, and the pending height they return is
/// the cached one, which the header overrides at runtime.
#[cfg(any(test, feature = "testutils"))]
impl StateStore {
    fn row(
        &self,
        network_id: NetworkId,
        kind: ValueDiscriminants,
    ) -> Result<Option<network_info::Value>, Error> {
        Ok(self.db.get::<NetworkInfoColumn>(&network_info::Key {
            network_id: network_id.to_u32(),
            kind,
        })?)
    }

    pub fn get_latest_pending_height(
        &self,
        network_id: NetworkId,
    ) -> Result<Option<Height>, Error> {
        Ok(Self::decode_pending(
            self.row(network_id, ValueDiscriminants::LatestPendingCertificateInfo)?,
        )?
        .map(|(_, height)| height))
    }

    pub fn get_latest_pending_certificate_id(
        &self,
        network_id: NetworkId,
    ) -> Result<Option<CertificateId>, Error> {
        Ok(Self::decode_pending(
            self.row(network_id, ValueDiscriminants::LatestPendingCertificateInfo)?,
        )?
        .map(|(certificate_id, _)| certificate_id))
    }

    pub fn get_latest_proven_certificate_id(
        &self,
        network_id: NetworkId,
    ) -> Result<Option<CertificateId>, Error> {
        Self::decode_proven(self.row(network_id, ValueDiscriminants::LatestProvenCertificateInfo)?)
    }

    pub fn get_latest_settled_certificate_id(
        &self,
        network_id: NetworkId,
    ) -> Result<Option<CertificateId>, Error> {
        Ok(
            Self::decode_settled(self.row(network_id, ValueDiscriminants::SettledCertificate)?)?
                .map(|settled| settled.certificate_id),
        )
    }
}

pub(super) mod backfill;

#[cfg(test)]
mod tests;
