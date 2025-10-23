//! Network information storage implementation.
//!
//! This module implements the `NetworkInfoReader` trait for `StateStore`,
//! providing functionality to read and retrieve network-related information
//! from the database.
use agglayer_types::{Height, NetworkId, NetworkInfo};

use crate::{
    columns::network_info::NetworkInfoColumn,
    error::Error,
    stores::{expected_type_or_fail, state::StateStore, try_digest, StateReader as _},
    types::{network_info::{
        self,
        v0::{
            network_info_value, LocalExitTreeLeafCount, PendingCertificateInfo, PessimisticProofRoot, SettledClaim
        },
    }, BasicPendingCertificateInfo},
};

impl crate::stores::NetworkInfoReader for StateStore {
    fn get_network_info(&self, network_id: NetworkId) -> Result<NetworkInfo, Error> {
        let mut state = NetworkInfo::from_network_id(network_id);
        let keys = network_info::Key::all_keys_for_network(network_id);
        self.db
            .atomic_multi_get::<NetworkInfoColumn>(keys.clone())?
            .into_iter()
            .zip(keys)
            .try_for_each(|(maybe_value, network_info::Key { kind, .. })| {
                match kind {
                    network_info_value::ValueDiscriminants::LatestSettledClaim => {
                        state.settled_claim = expected_type_or_fail!(
                            maybe_value,
                            network_info::v0::network_info_value::Value::LatestSettledClaim(SettledClaim { global_index: Some(global_index), bridge_exit_hash: Some(bridge_exit_hash) }),
                            agglayer_types::SettledClaim {
                                global_index: try_digest!(&*global_index.value, "GlobalIndex")?,
                                bridge_exit_hash: try_digest!(&*bridge_exit_hash.bridge_exit_hash, "BridgeExitHash")?,
                            },
                            "Wrong value type decoded, was expecting SettledClaim, decoded \
                             another type"
                        )?;
                    }
                    network_info_value::ValueDiscriminants::LatestSettledCertificate => {
                        let maybe_settled_certificate = expected_type_or_fail!(
                            maybe_value,
                            network_info::v0::network_info_value::Value::LatestSettledCertificate(
                                settled_certificate
                            ),
                            settled_certificate,
                            "Wrong value type decoded, was expecting SettledLer, decoded another \
                             type"
                        )?;

                        if let Some(network_info::v0::SettledCertificateInfo {
                            certificate_id: Some(network_info::v0::CertificateId { id }),
                            pp_root,
                            let_leaf_count,
                            ..

                        }) = maybe_settled_certificate
                        {
                            let certificate_id = try_digest!(&*id, "CertificateId")?
                                .into();
                            if let Some(header) = self.get_certificate_header(&certificate_id)? {
                                state.settled_certificate_id = Some(certificate_id);
                                state.settled_height = Some(header.height);
                                state.settled_ler = Some(header.new_local_exit_root);
                                if let Some(LocalExitTreeLeafCount {
                                    let_leaf_count,
                                }) = let_leaf_count
                                {
                                    state.settled_let_leaf_count = Some(let_leaf_count);
                                } else {
                                    return Err(Error::Unexpected(
                                        "Settled certificate is missing the LET leaf count"
                                            .to_string(),
                                    ));
                                }

                                if let Some(PessimisticProofRoot { root }) = pp_root {
                                    state.settled_pp_root = Some(try_digest!(&*root, "PessimisticProofRoot")?);
                                } else {
                                    return Err(Error::Unexpected(
                                        "Settled certificate is missing the pessimistic proof root"
                                            .to_string(),
                                    ));
                                }
                            }
                        }
                    }
                    network_info_value::ValueDiscriminants::NetworkType => {
                        state.network_type = expected_type_or_fail!(
                            maybe_value,
                            network_info::v0::network_info_value::Value::NetworkType(network_type,),
                            network_info::v0::NetworkType::try_from(network_type)
                                .map_err(|_| {
                                    Error::Unexpected(
                                        "Unable to deserialize NetworkType from integer"
                                            .to_string(),
                                    )
                                })?
                                .try_into()
                                .map_err(|_| Error::Unexpected(
                                    "Unable to convert storage NetworkType to \
                                     agglayer_types::NetworkType"
                                        .to_string(),
                                ))?,
                            "Wrong value type decode, was expecting NetworkType, decoded another \
                             type"
                        )?
                        .ok_or(Error::Unexpected(
                            "Unable to decode NetworkType".to_string(),
                        ))?;
                    }

                    network_info_value::ValueDiscriminants::LatestPendingCertificateInfo => {
                        state.latest_pending_height = expected_type_or_fail!(
                            maybe_value,
                            network_info::v0::network_info_value::Value::LatestPendingCertificateInfo(
                                PendingCertificateInfo{
                                    height: Some(network_info::v0::Height { height }),
                                    ..
                                },
                            ),
                            height.into(),
                            "Wrong value type decoded, was expecting LatestPendingCertificateInfo, decoded \
                             another type"
                        )?
                    }

                    network_info_value::ValueDiscriminants::LatestProvenCertificateInfo => {
                        // Currently not exposed as part of NetworkInfo.
                    }
                }

                Ok::<(), Error>(())
            })?;

        Ok(state)
    }

    fn get_latest_pending_height(&self, network_id: NetworkId) -> Result<Option<Height>, Error> {
        self.db
            .get::<NetworkInfoColumn>(&network_info::Key {
                network_id: network_id.to_u32(),
                kind: network_info_value::ValueDiscriminants::LatestPendingCertificateInfo,
            })
            .map_err(Into::into)
            .and_then(|value| {
                expected_type_or_fail!(
                    value,
                    network_info::v0::network_info_value::Value::LatestPendingCertificateInfo(
                        PendingCertificateInfo {
                            height: Some(network_info::v0::Height { height }),
                            ..
                        }
                    ),
                    height.into(),
                    "Wrong value type decoded, was expecting LatestPendingHeight, decoded another \
                     type"
                )
            })
    }

    fn get_latest_pending_certificate_info(
        &self,
        network_id: NetworkId,
    ) -> Result<Option<BasicPendingCertificateInfo>, Error> {
        self.db
            .get::<NetworkInfoColumn>(&network_info::Key {
                network_id: network_id.to_u32(),
                kind: network_info_value::ValueDiscriminants::LatestPendingCertificateInfo,
            })
            .map_err(Into::into)
            .and_then(|value| {
                expected_type_or_fail!(
                    value,
                    network_info::v0::network_info_value::Value::LatestPendingCertificateInfo(
                        network_info::v0::PendingCertificateInfo {
                            id: Some(network_info::v0::CertificateId { id }),
                            height: Some(network_info::v0::Height { height }),
                        }
                    ),
                    BasicPendingCertificateInfo {
                        certificate_id: try_digest!(&*id, "CertificateId")?.into(),
                        height: Height::new(height),
                    },
                    "Wrong value type decoded, was expecting CertificateId, decoded \
                    another type"
                )
            })
    }
}
