//! One-time backfill of the network info column from the legacy
//! latest-certificate indexes, run by the V3 state migration step.
//!
//! A settled row promises that claim history was evaluated through the
//! certificate it names: without a claim row next to it, that history holds no
//! claim. Anything wrong with the settled pointer or its history therefore
//! fails the migration. Pending and proven pointers that do not match a
//! certificate header are skipped with a warning: they are not evidence about
//! claims.

use agglayer_types::{
    Certificate, CertificateHeader, CertificateId, CertificateIndex, Digest, EpochNumber, Height,
    NetworkId, NetworkType, Proof, SettledClaim,
};
use pessimistic_proof::{nullifier_tree::NULLIFIER_TREE_DEPTH, PessimisticProofOutput};
use rocksdb::{Direction, ReadOptions};
use tracing::{error, info, warn};

use crate::{
    columns::{
        certificate_header::CertificateHeaderColumn,
        certificate_per_network::{self, CertificatePerNetworkColumn},
        latest_pending_certificate_per_network::PendingCertificate,
        latest_proven_certificate_per_network::ProvenCertificate,
        latest_settled_certificate_per_network::{
            LatestSettledCertificatePerNetworkColumn, SettledCertificate,
        },
        nullifier_tree_per_network::NullifierTreePerNetworkColumn,
    },
    error::Error,
    storage::{DBError, DB},
    stores::{state::StateStore, EpochStoreReader, PendingCertificateReader},
    types::network_info::{
        self,
        v0::{
            self, network_info_value::Value, LatestPendingCertificateHeight,
            LatestPendingCertificateId, LatestPendingCertificateInfo, LatestProvenCertificateInfo,
        },
    },
};

#[cfg(test)]
mod tests;

type Pointer = (CertificateId, Height);
type Row = (network_info::Key, network_info::Value);

struct NetworkBackfill<'a> {
    db: &'a DB,
    epochs: &'a dyn EpochStoreReader,
    network_id: NetworkId,
}

impl StateStore {
    /// The rows to write into the freshly recreated network info column, from
    /// the legacy indexes in `db` and `pending`, and the settled certificates
    /// in `epochs`. Everything is read before anything is written.
    pub(in crate::stores::state) fn network_info_backfill(
        db: &DB,
        pending: &dyn PendingCertificateReader,
        epochs: &dyn EpochStoreReader,
    ) -> Result<Vec<Row>, Error> {
        let network = |network_id| NetworkBackfill {
            db,
            epochs,
            network_id,
        };
        let failed = |network_id: NetworkId| {
            move |error: &Error| {
                error!(%network_id, %error, "Failed to backfill network info");
            }
        };

        let mut rows = Vec::new();
        for (network_id, PendingCertificate(id, height)) in pending.get_current_pending_heights()? {
            let network = network(network_id);
            if let Some(header) = network
                .unsettled_header("pending", (id, height))
                .inspect_err(failed(network_id))?
            {
                rows.push(network.row(Value::LatestPendingCertificateInfo(
                    LatestPendingCertificateInfo {
                        height: Some(LatestPendingCertificateHeight {
                            height: header.height.as_u64(),
                        }),
                        id: Some(LatestPendingCertificateId {
                            id: header.certificate_id.as_digest().as_slice().to_vec().into(),
                        }),
                    },
                )));
            }
        }
        for ProvenCertificate(id, network_id, height) in pending.get_current_proven_height()? {
            let network = network(network_id);
            if let Some(header) = network
                .unsettled_header("proven", (id, height))
                .inspect_err(failed(network_id))?
            {
                rows.push(network.row(Value::LatestProvenCertificateInfo(
                    LatestProvenCertificateInfo {
                        id: header.certificate_id.as_digest().as_slice().to_vec().into(),
                    },
                )));
            }
        }
        for entry in db.iter_with_direction::<LatestSettledCertificatePerNetworkColumn>(
            ReadOptions::default(),
            Direction::Forward,
        )? {
            let (network_id, SettledCertificate(id, height, ..)) = entry?;
            network(network_id)
                .settled_rows((id, height), &mut rows)
                .inspect_err(failed(network_id))?;
        }

        Ok(rows)
    }
}

impl NetworkBackfill<'_> {
    fn inconsistent(&self, reason: impl Into<String>) -> Error {
        Error::NetworkInfoBackfill {
            network_id: self.network_id,
            reason: reason.into(),
        }
    }

    fn row(&self, value: Value) -> Row {
        StateStore::network_info_row(self.network_id, value)
    }

    /// The header a pointer names, if it belongs to this network at the
    /// pointer's height.
    fn header(
        &self,
        (certificate_id, height): Pointer,
    ) -> Result<Option<CertificateHeader>, Error> {
        Ok(self
            .db
            .get::<CertificateHeaderColumn>(&certificate_id)?
            .filter(|header| header.network_id == self.network_id && header.height == height))
    }

    /// The header of a pending or proven pointer, skipped with a warning when
    /// it does not match its certificate.
    fn unsettled_header(
        &self,
        kind: &str,
        pointer: Pointer,
    ) -> Result<Option<CertificateHeader>, Error> {
        let header = self.header(pointer)?;
        if header.is_none() {
            warn!(
                network_id = %self.network_id,
                kind,
                certificate_id = %pointer.0,
                height = %pointer.1,
                "Skipping a latest certificate pointer without a matching header"
            );
        }
        Ok(header)
    }

    /// The header of a settled certificate, which must match its pointer.
    fn settled_header(&self, pointer: Pointer) -> Result<CertificateHeader, Error> {
        self.header(pointer)?.ok_or_else(|| {
            self.inconsistent(format!(
                "settled certificate {} has no header for this network at height {}",
                pointer.0, pointer.1
            ))
        })
    }

    /// Push the network type, settled row, and claim of the settled pointer.
    fn settled_rows(&self, pointer: Pointer, rows: &mut Vec<Row>) -> Result<(), Error> {
        let header = self.settled_header(pointer)?;
        let certificate = self.settled_certificate(&header)?;

        // The type follows settlement: a network that never settled has none.
        let network_type = v0::NetworkType::from(NetworkType::from(&certificate.aggchain_data));
        rows.push(self.row(Value::NetworkType(network_type as i32)));

        let claim = match certificate.imported_bridge_exits.last() {
            Some(exit) => Some(exit.into()),
            // Settling an imported bridge exit inserts its nullifier, so a
            // stored empty tree proves that no claim was ever settled. A
            // network that last settled before local state was persisted has
            // no stored tree, and its history decides.
            None
                if StateStore::read_smt::<NullifierTreePerNetworkColumn, NULLIFIER_TREE_DEPTH>(
                    self.db,
                    self.network_id,
                )?
                .is_some_and(|tree| tree.is_empty()) =>
            {
                None
            }
            None => self.latest_settled_claim(&header)?,
        };

        rows.push(self.row(StateStore::settled_certificate_value(
            header.certificate_id,
            header.new_local_exit_root,
            self.pp_root(&header)?,
            self.settled_let_leaf_count(&header)?,
        )));
        if let Some(claim) = &claim {
            rows.push(self.row(StateStore::settled_claim_value(claim)));
        }
        info!(
            network_id = %self.network_id,
            certificate_id = %header.certificate_id,
            height = %header.height,
            has_claim = claim.is_some(),
            "Reconstructed the settled network info"
        );

        Ok(())
    }

    fn epoch_slot(
        &self,
        header: &CertificateHeader,
    ) -> Result<(EpochNumber, CertificateIndex), Error> {
        match (header.epoch_number, header.certificate_index) {
            (Some(epoch), Some(index)) => Ok((epoch, index)),
            _ => Err(self.inconsistent(format!(
                "settled certificate {} has no epoch assignment",
                header.certificate_id
            ))),
        }
    }

    /// The full certificate of a settled header, from its epoch.
    fn settled_certificate(&self, header: &CertificateHeader) -> Result<Certificate, Error> {
        let (epoch, index) = self.epoch_slot(header)?;
        let certificate = self.epochs.get_certificate(epoch, index)?.ok_or_else(|| {
            self.inconsistent(format!(
                "settled certificate {} is missing from epoch {epoch}",
                header.certificate_id
            ))
        })?;
        if certificate.hash() != header.certificate_id {
            return Err(self.inconsistent(format!(
                "epoch {epoch} holds another certificate than {} at index {index}",
                header.certificate_id
            )));
        }

        Ok(certificate)
    }

    /// The last imported bridge exit settled below `settled`, scanning the
    /// settled history downwards one epoch per certificate until one carries
    /// a claim. Any gap in that history fails.
    fn latest_settled_claim(
        &self,
        settled: &CertificateHeader,
    ) -> Result<Option<SettledClaim>, Error> {
        for height in (0..settled.height.as_u64()).rev().map(Height::new) {
            let certificate_id = self
                .db
                .get::<CertificatePerNetworkColumn>(&certificate_per_network::Key {
                    network_id: self.network_id.to_u32(),
                    height,
                })?
                .ok_or_else(|| self.inconsistent(format!("no settled certificate at {height}")))?;
            let header = self.settled_header((certificate_id, height))?;
            if let Some(exit) = self
                .settled_certificate(&header)?
                .imported_bridge_exits
                .last()
            {
                return Ok(Some(exit.into()));
            }
        }

        Ok(None)
    }

    /// The pessimistic root the settled proof committed to. A proof that is
    /// gone, or that the current codec cannot decode, leaves it unset.
    fn pp_root(&self, header: &CertificateHeader) -> Result<Option<Digest>, Error> {
        let (epoch, index) = self.epoch_slot(header)?;
        let proof = match self.epochs.get_proof(epoch, index) {
            Ok(proof) => proof,
            // Written by an older release, before the proof format changed.
            Err(Error::DBError(DBError::CodecError(error))) => {
                warn!(
                    network_id = %self.network_id,
                    certificate_id = %header.certificate_id,
                    %error,
                    "The settled proof cannot be decoded, leaving its pessimistic root unset"
                );
                return Ok(None);
            }
            Err(error) => return Err(error),
        };
        let Some(Proof::SP1(proof)) = proof else {
            return Ok(None);
        };

        Ok(PessimisticProofOutput::bincode_codec()
            .deserialize::<PessimisticProofOutput>(proof.public_values.as_slice())
            .inspect_err(|error| {
                warn!(
                    network_id = %self.network_id,
                    certificate_id = %header.certificate_id,
                    %error,
                    "The settled proof output cannot be decoded, leaving its pessimistic root unset"
                )
            })
            .ok()
            .map(|output| output.new_pessimistic_root))
    }

    /// The local exit tree leaf count when `header` settled. Settlement writes
    /// the tree before it publishes the settled pointer, so a tree that already
    /// moved past the settled root leaves the count unset, as does a network
    /// without a stored tree.
    fn settled_let_leaf_count(&self, header: &CertificateHeader) -> Result<Option<u64>, Error> {
        Ok(StateStore::read_local_exit_tree(self.db, self.network_id)?
            .filter(|tree| tree.get_root() == Digest::from(header.new_local_exit_root))
            .map(|tree| u64::from(tree.leaf_count())))
    }
}
