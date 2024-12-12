use std::{sync::Arc, time::Duration};

use agglayer_certificate_orchestrator::{EpochPacker, Error};
use agglayer_config::outbound::OutboundRpcSettleConfig;
use agglayer_contracts::{RollupContract, Settler};
use agglayer_storage::{
    columns::latest_settled_certificate_per_network::SettledCertificate,
    stores::{PendingCertificateReader, PerEpochReader, PerEpochWriter, StateReader, StateWriter},
};
use agglayer_types::{
    CertificateHeader, CertificateId, CertificateStatus, ExecutionMode, NetworkId, Proof,
};
use arc_swap::ArcSwap;
use bincode::Options;
use ethers::{
    providers::PendingTransaction,
    types::{TransactionReceipt, U64},
};
use futures::future::BoxFuture;
use pessimistic_proof::PessimisticProofOutput;
use tracing::Instrument;
use tracing::{debug, error, info, instrument, warn};

#[cfg(test)]
mod tests;

const MAX_EPOCH_ASSIGNEMENT_RETRIES: usize = 5;

#[derive(Default, Clone)]
pub struct EpochPackerClient<StateStore, PendingStore, PerEpochStore, RollupManagerRpc> {
    state_store: Arc<StateStore>,
    pending_store: Arc<PendingStore>,
    config: Arc<OutboundRpcSettleConfig>,
    l1_rpc: Arc<RollupManagerRpc>,
    _phantom: std::marker::PhantomData<fn() -> PerEpochStore>,
}

impl<StateStore, PendingStore, PerEpochStore, RollupManagerRpc>
    EpochPackerClient<StateStore, PendingStore, PerEpochStore, RollupManagerRpc>
{
    /// Try to create a new notifier using the given configuration
    pub fn try_new(
        config: Arc<OutboundRpcSettleConfig>,
        state_store: Arc<StateStore>,
        pending_store: Arc<PendingStore>,
        l1_rpc: Arc<RollupManagerRpc>,
    ) -> Result<Self, Error> {
        Ok(Self {
            config,
            l1_rpc,
            state_store,
            pending_store,
            _phantom: std::marker::PhantomData,
        })
    }
}

type SettlementResult<'a> =
    Result<BoxFuture<'a, Result<(NetworkId, SettledCertificate), Error>>, Error>;

impl<StateStore, PendingStore, PerEpochStore, RollupManagerRpc> EpochPacker
    for EpochPackerClient<StateStore, PendingStore, PerEpochStore, RollupManagerRpc>
where
    StateStore: StateReader + StateWriter + 'static,
    PendingStore: PendingCertificateReader + 'static,
    RollupManagerRpc: RollupContract + Settler + Send + Sync + 'static,
    PerEpochStore: PerEpochWriter + PerEpochReader + 'static,
{
    type PerEpochStore = PerEpochStore;

    #[instrument(skip_all, fields(hash, network_id, calldata), level = "debug")]
    fn settle_certificate(
        &self,
        related_epoch: Arc<ArcSwap<Self::PerEpochStore>>,
        certificate_id: CertificateId,
    ) -> SettlementResult {
        let hash = certificate_id.to_string();
        tracing::Span::current().record("hash", &hash);
        let (network_id, height) = if let Some(CertificateHeader {
            status,
            network_id,
            height,
            ..
        }) =
            self.state_store.get_certificate_header(&certificate_id)?
        {
            if status == CertificateStatus::Settled {
                error!(
                    hash,
                    "The certificate {} is already settled", certificate_id
                );

                return Err(Error::InvalidCertificateStatus);
            }

            // TODO: Acquire lock for this certificate
            (network_id, height)
        } else {
            error!(
                hash,
                "The certificate header of {} is not found", certificate_id
            );

            return Err(Error::NotFoundCertificateHeader);
        };

        let certificate =
            if let Some(certificate) = self.pending_store.get_certificate(network_id, height)? {
                certificate
            } else {
                return Err(Error::InternalError(format!(
                    "Unable to find the certificate {} in pending store",
                    certificate_id
                )));
            };

        let network_id = certificate.network_id;
        tracing::Span::current().record("network_id", *network_id);

        let height = certificate.height;
        // let epoch_number = related_epoch.get_epoch_number();

        let l_1_info_tree_leaf_count = certificate
            .l1_info_tree_leaf_count()
            .unwrap_or_else(|| self.l1_rpc.default_l1_info_tree_entry().0);

        // Prepare the proof
        let (output, proof) =
            if let Some(Proof::SP1(proof)) = self.pending_store.get_proof(certificate_id)? {
                if let Ok(output) =
                    pessimistic_proof::PessimisticProofOutput::bincode_options()
                        .deserialize::<PessimisticProofOutput>(proof.public_values.as_slice())
                {
                    (output, proof.bytes())
                } else {
                    return Err(Error::InternalError(
                        "Unable to deserialize the proof output".to_string(),
                    ));
                }
            } else {
                return Err(Error::InternalError("Unable to find the proof".to_string()));
            };

        let contract_call = self
            .l1_rpc
            .build_verify_pessimistic_trusted_aggregator_call(
                *output.origin_network,
                l_1_info_tree_leaf_count,
                *output.new_local_exit_root,
                *output.new_pessimistic_root,
                proof.into(),
            );

        tracing::Span::current().record(
            "calldata",
            contract_call
                .tx
                .data()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "unable to serialize calldata".to_string()),
        );

        info!(
            "Initializing the settlement of the certificate {} on L1 with public inputs: {}",
            certificate_id,
            output.display_to_hex()
        );

        let state_store = self.state_store.clone();
        let config = self.config.clone();

        // Call the Provider
        let fut = Box::pin(
            async move {
                // Send the transaction
                let pending_tx = contract_call
                    .send()
                    .await
                    .inspect(|tx| info!(hash, "Inspect settle transaction: {:?}", tx))
                    .map_err(|e| {
                        println!("Error: {:?}", e);
                        let error_str =
                            RollupManagerRpc::decode_contract_revert(&e).unwrap_or(e.to_string());

                        error!(
                            error_code = %e,
                            error = error_str,
                            hash,
                            "Failed to settle the certificate {certificate_id}: {}", error_str
                        );

                        Error::SettlementError {
                            certificate_id,
                            error: error_str,
                        }
                    })?;
                if let Err(error) = state_store
                    .update_settlement_tx_hash(&certificate_id, pending_tx.tx_hash().0.into())
                {
                    error!(
                        hash,
                        "CRITICAL: Failed to update the settlement transaction hash of {} with {} \
                         the settlement transaction continues, this is due to: {}",
                        certificate_id,
                        pending_tx.tx_hash(),
                        error
                    );
                }

                // wait for the receipt
                let receipt =
                    handle_pending_tx::<RollupManagerRpc>(pending_tx, certificate_id, config)
                        .await?
                        // If the result is `None`, it means the transaction is no longer
                        // in the mempool.
                        .ok_or(Error::SettlementError {
                            certificate_id,
                            error: "No receipt hash returned, transaction still in mempool"
                                .to_string(),
                        })?;

                match receipt.status {
                    Some(n) if n == U64::zero() => {
                        warn!(
                            hash,
                            "The transaction failed to settle the certificate {}", certificate_id
                        );
                        Err(Error::SettlementError {
                            certificate_id,
                            error: "SettlementTransaction failed".to_string(),
                        })
                    }
                    None => {
                        error!(
                            hash,
                            "The transaction failed to settle the certificate {}", certificate_id
                        );

                        Err(Error::SettlementError {
                            certificate_id,
                            error: "SettlementTransaction failed due to no receipt status"
                                .to_string(),
                        })
                    }
                    Some(_) => {
                        info!(
                            "The transaction successfully settled the certificate {}",
                            certificate_id
                        );

                        let mut max_retries = MAX_EPOCH_ASSIGNEMENT_RETRIES;

                        let (epoch_number, certificate_index) = loop {
                            max_retries -= 1;

                            let related_epoch = related_epoch.load_full();
                            if related_epoch.is_epoch_packed() {
                                drop(related_epoch);

                                warn!(
                                    "The epoch is already packed, adding delay and retry the \
                                     assignment"
                                );

                                tokio::time::sleep(Duration::from_secs(1)).await;
                                continue;
                            }

                            match related_epoch.add_certificate(
                                network_id,
                                height,
                                ExecutionMode::Default,
                            ) {
                                Err(error) if max_retries == 0 => {
                                    let error_msg = format!(
                                        "CRITICAL: Failed to add the certificate {} to the epoch \
                                         after multiple retries: {}",
                                        certificate_id, error
                                    );
                                    error!(hash = certificate_id.to_string(), error_msg);

                                    return Err(Error::SettlementError {
                                        certificate_id,
                                        error: error_msg,
                                    });
                                }
                                Err(error) => warn!(
                                    "Failed to add the certificate {} to the epoch (retrying): {}",
                                    certificate_id, error
                                ),
                                Ok((epoch_number, certificate_index)) => {
                                    info!(
                                        "The certificate {} is added to the epoch {} with index {}",
                                        certificate_id, epoch_number, certificate_index
                                    );
                                    break (epoch_number, certificate_index);
                                }
                            }
                        };

                        if let Err(error) = state_store.update_certificate_header_status(
                            &certificate_id,
                            &CertificateStatus::Settled,
                        ) {
                            error!(
                                hash,
                                "Certificate settled but failed to update the certificate status \
                                 of {} due to: {}",
                                certificate_id,
                                error
                            );
                        }
                        if let Err(error) = state_store.set_latest_settled_certificate_for_network(
                            &network_id,
                            &height,
                            &certificate_id,
                            &epoch_number,
                            &certificate_index,
                        ) {
                            error!(
                                hash,
                                "Certificate settled but failed to update the latest settled \
                                 certificate for network {} with {} due to: {}",
                                network_id,
                                certificate_id,
                                error
                            );
                        }

                        Ok::<_, Error>((
                            network_id,
                            SettledCertificate(
                                certificate_id,
                                height,
                                epoch_number,
                                certificate_index,
                            ),
                        ))
                    }
                }
            }
            .instrument(tracing::Span::current()),
        );

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
            .map_err(|_| {
                Error::InternalError(format!(
                    "Unable to join the packing task for {}",
                    epoch_number
                ))
            })?;

            Ok(())
        }))
    }
}

async fn handle_pending_tx<RollupManagerRpc>(
    pending_tx: PendingTransaction<
        '_,
        <<RollupManagerRpc as Settler>::M as ethers::providers::Middleware>::Provider,
    >,
    certificate_id: CertificateId,
    config: Arc<OutboundRpcSettleConfig>,
) -> Result<Option<TransactionReceipt>, Error>
where
    RollupManagerRpc: RollupContract + Settler + Send + Sync + 'static,
{
    let receipt = pending_tx
        .interval(config.retry_interval)
        .retries(config.max_retries)
        .confirmations(config.confirmations)
        .await;

    fail::fail_point!(
        "notifier::packer::settle_certificate::receipt_future_ended::status_0",
        |_| {
            Ok(Some(TransactionReceipt {
                transaction_hash: ethers::types::H256::random(),
                transaction_index: U64([1; 1]),
                block_hash: None,
                block_number: None,
                from: ethers::types::H160::random(),
                to: None,
                cumulative_gas_used: ethers::types::U256::zero(),
                gas_used: None,
                contract_address: None,
                logs: vec![],
                status: Some(U64([0; 1])),
                root: None,
                logs_bloom: ethers::types::Bloom::zero(),
                transaction_type: None,
                effective_gas_price: None,
                other: ethers::types::OtherFields::default(),
            }))
        }
    );

    fail::fail_point!(
        "notifier::packer::settle_certificate::receipt_future_ended::no_receipt",
        |_| { Ok(None) }
    );

    receipt.map_err(|error| Error::SettlementError {
        certificate_id,
        error: error.to_string(),
    })
}
