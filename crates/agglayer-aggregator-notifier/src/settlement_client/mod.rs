use std::{sync::Arc, time::Duration};

use agglayer_certificate_orchestrator::{Error, SettlementClient};
use agglayer_config::outbound::OutboundRpcSettleConfig;
use agglayer_contracts::{rollup::VerifierType, RollupContract, Settler};
use agglayer_storage::stores::{
    PendingCertificateReader, PerEpochReader, PerEpochWriter, StateReader, StateWriter,
};
use agglayer_types::{
    CertificateHeader, CertificateId, CertificateIndex, CertificateStatus, EpochNumber,
    ExecutionMode, Proof, SettlementTxHash,
};
use arc_swap::ArcSwap;
use bincode::Options;
use ethers::{
    providers::PendingTransaction,
    types::{TransactionReceipt, U256, U64},
};
use pessimistic_proof::{proof::DisplayToHex, PessimisticProofOutput};
use tracing::{debug, error, info, instrument, warn};

#[cfg(test)]
mod tests;

const MAX_EPOCH_ASSIGNMENT_RETRIES: usize = 5;

#[derive(Default, Clone)]
pub struct EthersSettlementClient<StateStore, PendingStore, PerEpochStore, RollupManagerRpc> {
    state_store: Arc<StateStore>,
    pending_store: Arc<PendingStore>,
    config: Arc<OutboundRpcSettleConfig>,
    l1_rpc: Arc<RollupManagerRpc>,
    current_epoch: Arc<ArcSwap<PerEpochStore>>,
}

impl<StateStore, PendingStore, PerEpochStore, RollupManagerRpc>
    EthersSettlementClient<StateStore, PendingStore, PerEpochStore, RollupManagerRpc>
{
    /// Try to create a new notifier using the given configuration
    pub fn try_new(
        config: Arc<OutboundRpcSettleConfig>,
        state_store: Arc<StateStore>,
        pending_store: Arc<PendingStore>,
        l1_rpc: Arc<RollupManagerRpc>,
        current_epoch: Arc<ArcSwap<PerEpochStore>>,
    ) -> Result<Self, Error> {
        Ok(Self {
            config,
            l1_rpc,
            state_store,
            pending_store,
            current_epoch,
        })
    }
}

#[async_trait::async_trait]
impl<StateStore, PendingStore, PerEpochStore, RollupManagerRpc> SettlementClient
    for EthersSettlementClient<StateStore, PendingStore, PerEpochStore, RollupManagerRpc>
where
    StateStore: StateReader + StateWriter + 'static,
    PendingStore: PendingCertificateReader + 'static,
    RollupManagerRpc: RollupContract + Settler + Send + Sync + 'static,
    PerEpochStore: PerEpochWriter + PerEpochReader + 'static,
{
    type Provider = <<RollupManagerRpc as Settler>::M as ethers::providers::Middleware>::Provider;

    #[instrument(skip(self), fields(network_id, calldata), level = "debug")]
    async fn submit_certificate_settlement(
        &self,
        certificate_id: CertificateId,
    ) -> Result<SettlementTxHash, Error> {
        let (network_id, height) = if let Some(CertificateHeader {
            status,
            network_id,
            height,
            ..
        }) =
            self.state_store.get_certificate_header(&certificate_id)?
        {
            if status == CertificateStatus::Settled {
                error!("Certificate is already settled");

                return Err(Error::InvalidCertificateStatus);
            }

            // TODO: Acquire lock for this certificate
            (network_id, height)
        } else {
            error!("Certificate header not found");

            return Err(Error::NotFoundCertificateHeader);
        };

        let dry_current_epoch = self.current_epoch.load();
        match dry_current_epoch.add_certificate(certificate_id, ExecutionMode::DryRun) {
            Err(error) => {
                drop(dry_current_epoch);
                error!(
                    %error,
                    "{}Failed to add the certificate to the current epoch",
                    ExecutionMode::DryRun.prefix(),
                );

                return Err(Error::Storage(error));
            }
            Ok((_epoch_number, _certificate_index)) => {
                drop(dry_current_epoch);
                info!("Certificate passes the epoch dry run");
            }
        }

        let certificate =
            if let Some(certificate) = self.pending_store.get_certificate(network_id, height)? {
                certificate
            } else {
                return Err(Error::InternalError(format!(
                    "Unable to find the certificate {certificate_id} in pending store"
                )));
            };

        let network_id = certificate.network_id;
        tracing::Span::current().record("network_id", network_id.to_u32());

        let l1_info_tree_leaf_count = certificate
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

        let verifier_type = self
            .l1_rpc
            .get_verifier_type(network_id.to_u32())
            .await
            .map_err(|_| Error::UnableToGetVerifierType {
                certificate_id,
                network_id,
            })?;

        debug!("Network {network_id} has {verifier_type:?}");

        let proof_with_selector: Vec<u8> = match verifier_type {
            VerifierType::StateTransition => {
                return Err(Error::InternalError(
                    "Unsupported verifier type".to_string(),
                ));
            }
            VerifierType::Pessimistic => proof,
            VerifierType::ALGateway => {
                let mut proof_with_selector =
                    pessimistic_proof::core::PESSIMISTIC_PROOF_PROGRAM_SELECTOR.to_vec();
                proof_with_selector.extend(&proof);
                proof_with_selector
            }
        };

        let contract_call = self
            .l1_rpc
            .build_verify_pessimistic_trusted_aggregator_call(
                output.origin_network.to_u32(),
                l1_info_tree_leaf_count,
                *output.new_local_exit_root,
                *output.new_pessimistic_root,
                proof_with_selector.into(),
                certificate.custom_chain_data.into(),
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
            "Initializing the settlement on L1 with public inputs: {}",
            output.display_to_hex()
        );

        let gas_estimate = contract_call.estimate_gas().await.map_err(|error| {
            let error_str =
                RollupManagerRpc::decode_contract_revert(&error).unwrap_or(error.to_string());

            error!(
                error_code = %error,
                error = error_str,
                "Failed to settle certificate"
            );

            Error::SettlementError {
                certificate_id,
                error: error_str,
            }
        })?;

        let gas = calculate_gas(&gas_estimate, &self.config);
        debug!("Gas estimate: {gas_estimate}, Gas calculated: {gas}");

        let contract_call = contract_call.gas(gas);

        // Send the transaction
        let pending_tx = contract_call
            .send()
            .await
            .inspect(|tx| info!("Inspect settle transaction: {tx:?}"))
            .map_err(|e| {
                let error_str =
                    RollupManagerRpc::decode_contract_revert(&e).unwrap_or(e.to_string());

                error!(
                    error_code = %e,
                    error = error_str,
                    "Failed to settle certificate"
                );

                Error::SettlementError {
                    certificate_id,
                    error: error_str,
                }
            })?;

        fail::fail_point!("notifier::packer::settle_certificate::transaction_sent::kill_node");

        Ok(SettlementTxHash::from(pending_tx.tx_hash()))
    }

    #[tracing::instrument(skip(self))]
    async fn wait_for_settlement(
        &self,
        settlement_tx_hash: SettlementTxHash,
        certificate_id: CertificateId,
    ) -> Result<(EpochNumber, CertificateIndex), Error> {
        let pending_tx = self
            .l1_rpc
            .build_pending_transaction(settlement_tx_hash.into());
        // wait for the receipt
        let receipt =
            handle_pending_tx::<RollupManagerRpc>(pending_tx, certificate_id, &self.config)
                .await?
                // If the result is `None`, it means the transaction is no longer
                // in the mempool.
                .ok_or(Error::SettlementError {
                    certificate_id,
                    error: "No receipt hash returned, transaction still in mempool".to_string(),
                })?;

        match receipt.status {
            Some(n) if n == U64::zero() => {
                warn!("Transaction failed to settle");
                Err(Error::SettlementError {
                    certificate_id,
                    error: "SettlementTransaction failed".to_string(),
                })
            }
            None => {
                error!("Transaction failed to settle");

                Err(Error::SettlementError {
                    certificate_id,
                    error: "SettlementTransaction failed due to no receipt status".to_string(),
                })
            }
            Some(_) => {
                info!("Transaction successfully settled",);

                let mut max_retries = MAX_EPOCH_ASSIGNMENT_RETRIES;

                let (epoch_number, certificate_index) = loop {
                    max_retries -= 1;

                    let related_epoch = self.current_epoch.load_full();
                    if related_epoch.is_epoch_packed() {
                        drop(related_epoch);

                        warn!("The epoch is already packed, adding delay and retry the assignment");

                        tokio::time::sleep(Duration::from_secs(1)).await;
                        continue;
                    }

                    match related_epoch.add_certificate(certificate_id, ExecutionMode::Default) {
                        Err(error) if max_retries == 0 => {
                            let error_msg = format!(
                                "CRITICAL: Failed to add the certificate to the epoch after \
                                 multiple retries: {error}"
                            );
                            error!(%error, error_msg);

                            return Err(Error::SettlementError {
                                certificate_id,
                                error: error_msg,
                            });
                        }
                        Err(error) => warn!(
                            %error, "Failed to add the certificate to the epoch (retrying)"
                        ),
                        Ok((epoch_number, certificate_index)) => {
                            info!(
                                "Certificate added to epoch {epoch_number} with index \
                                 {certificate_index}"
                            );
                            break (epoch_number, certificate_index);
                        }
                    }
                };

                Ok::<_, Error>((epoch_number, certificate_index))
            }
        }
    }
}

fn calculate_gas(gas_estimate: &U256, config: &OutboundRpcSettleConfig) -> U256 {
    fail::fail_point!(
        "notifier::packer::settle_certificate::gas_estimate::low_gas",
        |_| { gas_estimate * 50 / 100 }
    );
    let gas_multiplier = config.gas_multiplier_factor;
    gas_estimate * gas_multiplier / 100
}

async fn handle_pending_tx<RollupManagerRpc>(
    pending_tx: PendingTransaction<
        '_,
        <<RollupManagerRpc as Settler>::M as ethers::providers::Middleware>::Provider,
    >,
    certificate_id: CertificateId,
    config: &OutboundRpcSettleConfig,
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
