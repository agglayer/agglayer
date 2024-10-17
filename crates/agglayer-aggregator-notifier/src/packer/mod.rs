use std::sync::Arc;

use agglayer_certificate_orchestrator::{EpochPacker, Error};
use agglayer_config::outbound::OutboundRpcSettleConfig;
use agglayer_contracts::Settler;
use agglayer_storage::stores::{PerEpochReader, PerEpochWriter, StateReader, StateWriter};
use agglayer_types::{
    CertificateHeader, CertificateId, CertificateIndex, CertificateStatus, Proof,
};
use bincode::Options;
use futures::future::BoxFuture;
use pessimistic_proof::PessimisticProofOutput;
use tracing::{debug, error, info};

#[cfg(test)]
mod tests;

#[derive(Default, Clone)]
pub struct EpochPackerClient<StateStore, PerEpochStore, RollupManagerRpc> {
    state_store: Arc<StateStore>,
    config: Arc<OutboundRpcSettleConfig>,
    l1_rpc: Arc<RollupManagerRpc>,
    _phantom: std::marker::PhantomData<fn() -> PerEpochStore>,
}

impl<StateStore, PerEpochStore, RollupManagerRpc>
    EpochPackerClient<StateStore, PerEpochStore, RollupManagerRpc>
{
    /// Try to create a new notifier using the given configuration
    pub fn try_new(
        config: Arc<OutboundRpcSettleConfig>,
        state_store: Arc<StateStore>,
        l1_rpc: Arc<RollupManagerRpc>,
    ) -> Result<Self, Error> {
        Ok(Self {
            config,
            l1_rpc,
            state_store,
            _phantom: std::marker::PhantomData,
        })
    }
}

impl<StateStore, PerEpochStore, RollupManagerRpc> EpochPacker
    for EpochPackerClient<StateStore, PerEpochStore, RollupManagerRpc>
where
    StateStore: StateReader + StateWriter + 'static,
    RollupManagerRpc: Settler + Send + Sync + 'static,
    PerEpochStore: PerEpochWriter + PerEpochReader + 'static,
{
    type PerEpochStore = PerEpochStore;

    fn settle_certificate(
        &self,
        related_epoch: Arc<Self::PerEpochStore>,
        certificate_index: CertificateIndex,
        certificate_id: CertificateId,
    ) -> Result<BoxFuture<Result<(), Error>>, Error> {
        let hash = certificate_id.to_string();
        if let Some(CertificateHeader {
            status: CertificateStatus::Candidate,
            ..
        }) = self.state_store.get_certificate_header(&certificate_id)?
        {
            // TODO: Acquire lock for this certificate
        } else {
            error!(
                hash,
                "The certificate {} is not in the candidate status, can't settle", certificate_id
            );

            return Err(Error::InvalidCertificateStatus);
        }

        let certificate =
            if let Some(certificate) = related_epoch.get_certificate_at_index(certificate_index)? {
                certificate
            } else {
                return Err(Error::InternalError);
            };

        let height = certificate.height;
        let network_id = certificate.network_id;
        let epoch_number = related_epoch.get_epoch_number();

        let l_1_info_tree_leaf_count = certificate.l1_info_tree_leaf_count();

        // Prepare the proof
        let (output, proof) =
            if let Some(Proof::SP1(proof)) = related_epoch.get_proof_at_index(certificate_index)? {
                if let Ok(output) =
                    pessimistic_proof::PessimisticProofOutput::bincode_options()
                        .deserialize::<PessimisticProofOutput>(proof.public_values.as_slice())
                {
                    (output, proof.bytes())
                } else {
                    return Err(Error::InternalError);
                }
            } else {
                return Err(Error::InternalError);
            };

        let contract_call = self
            .l1_rpc
            .build_verify_pessimistic_trusted_aggregator_call(
                *output.origin_network,
                l_1_info_tree_leaf_count,
                output.new_local_exit_root,
                output.new_pessimistic_root,
                proof.into(),
            );

        let state_store = self.state_store.clone();
        let config = self.config.clone();
        // Call the Provider
        let fut = Box::pin(async move {
            let _tx = contract_call
                .send()
                .await
                .inspect(|tx| info!(hash, "Inspect settle transaction: {:?}", tx))
                // .map_err(SettlementError::ContractError)?
                .map_err(|e| {
                    let error_str =
                        RollupManagerRpc::decode_contract_revert(&e).unwrap_or(e.to_string());

                    error!(
                        error_code = %e,
                        error = error_str,
                        hash,
                            "Failed to settle the certificate {certificate_id}: {}", error_str);

                    Error::SettlementError {
                        certificate_id,
                        error: error_str,
                    }
                })?
                .interval(config.retry_interval)
                .retries(config.max_retries)
                .confirmations(config.confirmations)
                .await
                .map_err(|error| Error::SettlementError {
                    certificate_id,
                    error: error.to_string(),
                })?
                // If the result is `None`, it means the transaction is no longer
                // in the mempool.
                .ok_or(Error::SettlementError {
                    certificate_id,
                    error: "No receipt hash returned, transaction still in mempool".to_string(),
                })?;

            if let Err(error) = state_store
                .update_certificate_header_status(&certificate_id, &CertificateStatus::Settled)
            {
                error!(
                    hash,
                    "Certificate settled but failed to update the certificate status of {} due \
                     to: {}",
                    certificate_id,
                    error
                );
            }
            if let Err(error) = state_store.set_latest_settled_certificate_for_network(
                &network_id,
                &certificate_id,
                &epoch_number,
                &height,
            ) {
                error!(
                    hash,
                    "Certificate settled but failed to update the latest settled certificate for \
                     network {} with {} due to: {}",
                    network_id,
                    certificate_id,
                    error
                );
            }

            Ok::<_, Error>(())
        });

        Ok(fut)
    }

    fn pack(
        &self,
        closing_epoch: Arc<Self::PerEpochStore>,
    ) -> Result<BoxFuture<Result<(), Error>>, Error> {
        let epoch_number = closing_epoch.get_epoch_number();
        debug!("Start the settlement of the epoch {}", epoch_number);

        Ok(Box::pin(async move {
            // No aggregation for now, we settle each PP individually
            let _result: Result<(), Error> = tokio::task::spawn_blocking(move || {
                closing_epoch.start_packing()?;

                Ok(())
            })
            .await
            // TODO: Handle error in a better way
            .map_err(|_| Error::InternalError)?;

            Ok(())
        }))
    }
}
