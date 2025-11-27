use std::{sync::Arc, time::Duration};

use agglayer_certificate_orchestrator::{Error, NonceInfo, SettlementClient, TxReceiptStatus};
use agglayer_config::outbound::OutboundRpcSettleConfig;
use agglayer_contracts::{rollup::VerifierType, L1TransactionFetcher, RollupContract, Settler};
use agglayer_storage::stores::{
    PendingCertificateReader, PerEpochReader, PerEpochWriter, StateReader, StateWriter,
};
use agglayer_types::{
    CertificateHeader, CertificateId, CertificateIndex, CertificateStatus, Digest, EpochNumber,
    ExecutionMode, Proof, SettlementTxHash, U256,
};
use alloy::{
    eips::BlockNumberOrTag, providers::Provider, rpc::types::TransactionReceipt,
    signers::k256::elliptic_curve::ff::derive::bitvec::macros::internal::funty::Fundamental,
};
use arc_swap::ArcSwap;
use eyre::eyre;
use pessimistic_proof::{proof::DisplayToHex, PessimisticProofOutput};
use tracing::{debug, error, info, instrument, warn};

const MAX_EPOCH_ASSIGNMENT_RETRIES: usize = 5;

/// Rpc-based settlement client for L1 certificate settlement.
/// Using alloy client to interact with the L1 rollup manager contract.
#[derive(Default, Clone)]
pub struct RpcSettlementClient<StateStore, PendingStore, PerEpochStore, RollupManagerRpc> {
    state_store: Arc<StateStore>,
    pending_store: Arc<PendingStore>,
    config: Arc<OutboundRpcSettleConfig>,
    l1_rpc: Arc<RollupManagerRpc>,
    current_epoch: Arc<ArcSwap<PerEpochStore>>,
}

impl<StateStore, PendingStore, PerEpochStore, RollupManagerRpc>
    RpcSettlementClient<StateStore, PendingStore, PerEpochStore, RollupManagerRpc>
{
    /// Try to create a new rpc-based settlement client
    pub fn new(
        config: Arc<OutboundRpcSettleConfig>,
        state_store: Arc<StateStore>,
        pending_store: Arc<PendingStore>,
        l1_rpc: Arc<RollupManagerRpc>,
        current_epoch: Arc<ArcSwap<PerEpochStore>>,
    ) -> Self {
        Self {
            config,
            l1_rpc,
            state_store,
            pending_store,
            current_epoch,
        }
    }
}

impl<StateStore, PendingStore, PerEpochStore, RollupManagerRpc>
    RpcSettlementClient<StateStore, PendingStore, PerEpochStore, RollupManagerRpc>
where
    StateStore: StateReader,
    PendingStore: PendingCertificateReader,
    RollupManagerRpc: RollupContract + Settler,
    PerEpochStore: PerEpochWriter,
{
    #[instrument(skip(self), fields(network_id, settlement_params), level = "debug")]
    async fn submit_certificate_settlement(
        &self,
        certificate_id: CertificateId,
        nonce_info: Option<NonceInfo>,
    ) -> Result<SettlementTxHash, Error> {
        // Step 1: Get certificate header and validate
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
            (network_id, height)
        } else {
            error!("Certificate header not found");
            return Err(Error::NotFoundCertificateHeader);
        };

        // Step 2: Validate epoch assignment
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

        // Step 3: Get certificate from pending store
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

        // Step 4: Deserialize and prepare the proof
        let (output, proof) =
            if let Some(Proof::SP1(proof)) = self.pending_store.get_proof(certificate_id)? {
                if let Ok(output) = PessimisticProofOutput::bincode_codec()
                    .deserialize::<PessimisticProofOutput>(proof.public_values.as_slice())
                {
                    (output, proof.bytes())
                } else {
                    return Err(Error::InternalError(
                        "Unable to deserialize the proof output".to_string(),
                    ));
                }
            } else {
                return Err(Error::InternalError(
                    "Unable to find the proof in the pending store".to_string(),
                ));
            };

        // Step 5: Get verifier type and prepare proof
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

        info!(
            "Initializing the settlement on L1 with public inputs: {}",
            output.display_to_hex()
        );

        // Record the settlement parameters for tracing
        let settlement_params = format!(
            "origin_network: {}, l1_info_tree_leaf_count: {}, new_local_exit_root: 0x{}, \
             new_pessimistic_root: 0x{}, proof_length: {}, custom_chain_data_length: {}",
            output.origin_network.to_u32(),
            l1_info_tree_leaf_count,
            hex::encode(output.new_local_exit_root),
            hex::encode(output.new_pessimistic_root),
            proof_with_selector.len(),
            certificate.custom_chain_data.len()
        );
        tracing::Span::current().record("settlement_params", &settlement_params);

        // Step 6: Call the contract settlement function and get the pending transaction
        let pending_tx = match self
            .l1_rpc
            .verify_pessimistic_trusted_aggregator(
                output.origin_network.to_u32(),
                l1_info_tree_leaf_count,
                *output.new_local_exit_root.as_ref(),
                *output.new_pessimistic_root,
                proof_with_selector.into(),
                certificate.custom_chain_data.into(),
                nonce_info.map(|n| {
                    (
                        n.nonce,
                        n.previous_max_fee_per_gas,
                        n.previous_max_priority_fee_per_gas,
                    )
                }),
            )
            .await
        {
            Ok(pending_tx) => {
                info!("Certificate settlement transaction submitted");
                pending_tx
            }
            Err(error) => {
                // TODO: Differentiate between different error types, check if decoding works
                // properly for custom errors as well.
                let error_decoded = RollupManagerRpc::decode_contract_revert(&error)
                    .unwrap_or_else(|| error.to_string());
                let error_message = error.to_string();

                error!(error_message, error_decoded, "Failed to settle certificate");

                return Err(Error::SettlementError {
                    certificate_id,
                    error: format!("Message: {error_message}\nDecoded error: {error_decoded}"),
                });
            }
        };

        // Get the transaction hash from the pending transaction
        let tx_hash = *pending_tx.tx_hash();
        info!("Settlement transaction hash: {}", tx_hash);

        Ok(SettlementTxHash::from(tx_hash))
    }
}

impl<StateStore, PendingStore, PerEpochStore, RollupManagerRpc>
    RpcSettlementClient<StateStore, PendingStore, PerEpochStore, RollupManagerRpc>
where
    RollupManagerRpc: L1TransactionFetcher,
    PerEpochStore: PerEpochWriter + PerEpochReader,
{
    #[tracing::instrument(skip(self), fields(%settlement_tx_hash, %certificate_id))]
    async fn wait_for_settlement(
        &self,
        settlement_tx_hash: SettlementTxHash,
        certificate_id: CertificateId,
    ) -> Result<(EpochNumber, CertificateIndex), Error> {
        info!(%settlement_tx_hash, "Waiting for settlement of tx {settlement_tx_hash}");

        // Apply timeout fail point if they are active for integration testing
        #[cfg(feature = "testutils")]
        testutils::inject_settle_certificate_timeout_fail_points(
            certificate_id,
            settlement_tx_hash,
        )?;

        // Step 1: Wait for transaction receipt with retries
        let receipt = self
            .wait_for_transaction_receipt(settlement_tx_hash, certificate_id)
            .await?;

        if !receipt.inner.tx_type().is_eip1559() {
            warn!(tx = %settlement_tx_hash, "Settlement tx is not eip1559.");
        }

        // Apply fail points if they are active for integration testing
        #[cfg(feature = "testutils")]
        testutils::inject_settle_certificate_fail_points(certificate_id)?;

        // Step 2: Check transaction status
        if !receipt.status() {
            warn!(%settlement_tx_hash, "Certificate settlement transaction failed to settle");
            return Err(Error::SettlementError {
                certificate_id,
                error: "Settlement transaction failed".to_string(),
            });
        }

        info!(%settlement_tx_hash, "Certificate settlement transaction successfully settled on l1");

        // Step 3: Add certificate to epoch with retries
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
                        "CRITICAL: Failed to add the certificate to the epoch after multiple \
                         retries: {error}"
                    );
                    error!(%error, error_msg);

                    return Err(Error::SettlementError {
                        certificate_id,
                        error: error_msg,
                    });
                }
                Err(error) => {
                    warn!(%error, "Failed to add the certificate to the epoch (retrying)");
                }
                Ok((epoch_number, certificate_index)) => {
                    info!(
                        "Certificate added to epoch {epoch_number} with index {certificate_index}"
                    );
                    break (epoch_number, certificate_index);
                }
            }
        };

        Ok((epoch_number, certificate_index))
    }
}

impl<StateStore, PendingStore, PerEpochStore, RollupManagerRpc>
    RpcSettlementClient<StateStore, PendingStore, PerEpochStore, RollupManagerRpc>
where
    RollupManagerRpc: L1TransactionFetcher,
{
    /// Wait for transaction receipt with configurable retries and intervals
    async fn wait_for_transaction_receipt(
        &self,
        settlement_tx_hash: SettlementTxHash,
        certificate_id: CertificateId,
    ) -> Result<TransactionReceipt, Error> {
        let tx_hash = settlement_tx_hash.into();
        let timeout = self
            .config
            .retry_interval
            .mul_f64(self.config.max_retries as f64);

        debug!(
            ?timeout,
            max_retries = self.config.max_retries,
            retry_interval = ?self.config.retry_interval,
            "Waiting for transaction receipt",
        );

        for attempt in 0..=self.config.max_retries {
            match self.l1_rpc.fetch_transaction_receipt(tx_hash).await {
                Ok(Some(receipt)) => {
                    info!(attempt, "Successfully fetched transaction receipt");

                    // Wait for the required number of confirmations
                    if self.config.confirmations > 0 {
                        let receipt_block = receipt.block_number.ok_or_else(|| {
                            error!(%settlement_tx_hash, "Transaction receipt has no block number");
                            Error::SettlementError {
                                certificate_id,
                                error: "Transaction receipt has no block number".to_string(),
                            }
                        })?;

                        debug!(
                            receipt_block,
                            required_confirmations = self.config.confirmations,
                            "Waiting for block confirmations"
                        );

                        // Wait until we have the required number of confirmations
                        for confirmation_attempt in attempt..=self.config.max_retries {
                            match self.l1_rpc.get_provider().get_block_number().await {
                                Ok(current_block) => {
                                    let confirmations = current_block
                                        .saturating_sub(receipt_block)
                                        .saturating_add(1);
                                    if confirmations >= self.config.confirmations as u64 {
                                        info!(
                                            confirmations,
                                            required_confirmations = self.config.confirmations,
                                            current_block,
                                            "Transaction confirmed with required confirmations"
                                        );
                                        return Ok(receipt);
                                    } else {
                                        debug!(
                                            confirmations,
                                            required_confirmations = self.config.confirmations,
                                            "Waiting for more confirmations, sleeping"
                                        );
                                        tokio::time::sleep(self.config.retry_interval).await;
                                    }
                                }
                                Err(error) => {
                                    if confirmation_attempt <= self.config.max_retries {
                                        warn!(
                                            ?error,
                                            "Failed to get current block number, retrying"
                                        );
                                        tokio::time::sleep(self.config.retry_interval).await;
                                        continue;
                                    } else {
                                        error!(
                                            ?error,
                                            "Failed to get current block number after maximum \
                                             retries"
                                        );
                                        return Err(Error::SettlementError {
                                            certificate_id,
                                            error: format!(
                                                "Failed to get current block number while waiting \
                                                 for confirmations of tx {tx_hash}: {error}"
                                            ),
                                        });
                                    }
                                }
                            }
                        }

                        // Timeout waiting for confirmations
                        error!(
                            ?timeout,
                            "Timeout while waiting for transaction confirmations"
                        );
                        return Err(Error::PendingTransactionTimeout {
                            certificate_id,
                            settlement_tx_hash,
                            source: eyre!(
                                "Timeout while waiting for transaction confirmations for tx \
                                 {tx_hash} after {timeout:?}"
                            ),
                        });
                    } else {
                        // No confirmations required, return immediately
                        return Ok(receipt);
                    }
                }
                Ok(None) => {
                    // Transaction not yet included in a block, continue retrying
                    if attempt <= self.config.max_retries {
                        debug!(
                            %settlement_tx_hash,
                            next_attempt = attempt + 1,
                            max_retries = self.config.max_retries,
                            retry_interval = ?self.config.retry_interval,
                            "Transaction receipt not found yet, retrying after {:?}",
                            self.config.retry_interval
                        );
                        tokio::time::sleep(self.config.retry_interval).await;
                        continue;
                    } else {
                        // Max retries reached
                        error!(
                            %settlement_tx_hash,
                            ?timeout,
                            "Timeout while waiting the pending settlement transaction"
                        );
                        return Err(Error::PendingTransactionTimeout {
                            certificate_id,
                            settlement_tx_hash,
                            source: eyre!(
                                "Timeout while waiting for the pending settlement transaction \
                                 {timeout:?}"
                            ),
                        });
                    }
                }
                Err(error) => {
                    // Other error (e.g., network issue, RPC error)
                    error!(?error, "Error watching the pending settlement transaction");
                    return Err(Error::SettlementError {
                        certificate_id,
                        error: format!(
                            "Error while waiting for the pending settlement transaction tx \
                             {tx_hash}: {error}"
                        ),
                    });
                }
            }
        }

        // This should not be reached, but added for completeness
        error!(
            ?timeout,
            "Unexpected timeout while watching the pending settlement transaction"
        );
        Err(Error::PendingTransactionTimeout {
            certificate_id,
            settlement_tx_hash,
            source: eyre!(
                "Unexpected timeout while watching the pending settlement transaction after \
                 {timeout:?}"
            ),
        })
    }
}

#[async_trait::async_trait]
impl<StateStore, PendingStore, PerEpochStore, RollupManagerRpc> SettlementClient
    for RpcSettlementClient<StateStore, PendingStore, PerEpochStore, RollupManagerRpc>
where
    StateStore: StateReader + StateWriter + 'static,
    PendingStore: PendingCertificateReader + 'static,
    RollupManagerRpc: RollupContract + Settler + L1TransactionFetcher + Send + Sync + 'static,
    PerEpochStore: PerEpochWriter + PerEpochReader + 'static,
{
    type Provider = <RollupManagerRpc as L1TransactionFetcher>::Provider;

    async fn submit_certificate_settlement(
        &self,
        certificate_id: CertificateId,
        nonce_info: Option<NonceInfo>,
    ) -> Result<SettlementTxHash, Error> {
        self.submit_certificate_settlement(certificate_id, nonce_info)
            .await
    }

    async fn wait_for_settlement(
        &self,
        settlement_tx_hash: SettlementTxHash,
        certificate_id: CertificateId,
    ) -> Result<(EpochNumber, CertificateIndex), Error> {
        self.wait_for_settlement(settlement_tx_hash, certificate_id)
            .await
    }

    fn get_provider(&self) -> &Self::Provider {
        self.l1_rpc.get_provider()
    }

    /// Queries the L1 for the latest `VerifyPessimisticStateTransition` event
    /// for the given `network_id` and returns its `newPessimisticRoot`
    /// along with the transaction receipt of the transaction that has
    /// caused it.
    async fn fetch_last_settled_pp_root(
        &self,
        network_id: agglayer_types::NetworkId,
    ) -> Result<Option<([u8; 32], SettlementTxHash)>, Error> {
        use agglayer_contracts::contracts::PolygonRollupManager::VerifyPessimisticStateTransition;
        use alloy::{providers::Provider, sol_types::SolEvent};

        // Create a filter for the latest VerifyPessimisticStateTransition event for
        // this `network_id`. Using from_block Latest ensures we only get recent
        // events.
        let rollup_address = self.l1_rpc.get_rollup_manager_address();

        let latest_network_block = self
            .l1_rpc
            .get_provider()
            .get_block_number()
            .await
            .map_err(|error| {
                error!(?error, "Failed to get the latest block number");
                Error::L1CommunicationError(Box::new(
                    agglayer_contracts::L1RpcError::FailedToQueryEvents(error.into()),
                ))
            })?
            .as_u64();
        let mut events = Vec::new();
        let mut end_block = latest_network_block;
        while events.is_empty() && end_block > 0 {
            // start_block, end_block are inclusive
            let start_block =
                end_block.saturating_sub(self.l1_rpc.get_event_filter_block_range() - 1);
            let filter = alloy::rpc::types::Filter::new()
                .address(rollup_address.into_alloy())
                .event_signature(VerifyPessimisticStateTransition::SIGNATURE_HASH)
                .topic1(U256::from(network_id.to_u32()))
                .from_block(BlockNumberOrTag::Number(start_block))
                .to_block(BlockNumberOrTag::Number(end_block));

            // Fetch the logs through from the network.
            events = self
                .l1_rpc
                .get_provider()
                .get_logs(&filter)
                .await
                .map_err(|error| {
                    error!(
                        ?error,
                        "Failed to fetch VerifyPessimisticStateTransition logs"
                    );
                    Error::L1CommunicationError(Box::new(
                        agglayer_contracts::L1RpcError::FailedToQueryEvents(error.into()),
                    ))
                })?;

            end_block = end_block.saturating_sub(self.l1_rpc.get_event_filter_block_range());
        }

        // Get the most recent event (last in the list) and extract its new pessimistic
        // root.
        let result = events.iter().rev().find_map(|log| {
            let latest_pp_root =
                VerifyPessimisticStateTransition::decode_log(&log.clone().into()).ok();
            let tx_hash = log.transaction_hash.map(Digest::from);
            match (
                latest_pp_root.map(|val| <[u8; 32]>::from(val.newPessimisticRoot)),
                tx_hash,
            ) {
                (Some(pp_root), Some(tx_hash)) => Some((pp_root, SettlementTxHash::new(tx_hash))),
                _ => None,
            }
        });

        Ok(match result {
            Some((pp_root, tx_hash)) => {
                debug!(
                    %network_id,
                    latest_pp_root = %Digest(pp_root),
                    %tx_hash,
                    "Retrieved latest VerifyPessimisticStateTransition event",
                );
                Some((pp_root, tx_hash))
            }
            None => {
                debug!(
                    %network_id,
                    "No VerifyPessimisticStateTransition events found for network",
                );
                None
            }
        })
    }

    async fn fetch_settlement_receipt_status(
        &self,
        settlement_tx_hash: SettlementTxHash,
    ) -> Result<TxReceiptStatus, Error> {
        match self
            .l1_rpc
            .fetch_transaction_receipt(settlement_tx_hash.into())
            .await
        {
            Ok(Some(receipt)) => {
                debug!(
                    %settlement_tx_hash,
                    ?receipt,
                    "Fetched receipt for settlement tx",
                );
                if receipt.status() {
                    Ok(TxReceiptStatus::TxSuccessful)
                } else {
                    Ok(TxReceiptStatus::TxFailed)
                }
            }
            Ok(None) => {
                warn!(%settlement_tx_hash, "No receipt found for settlement tx");
                Ok(TxReceiptStatus::NotFound)
            }
            Err(error) => Err(Error::SettlementTransactionFetchReceiptError {
                tx_hash: settlement_tx_hash,
                error: Box::new(error),
            }),
        }
    }

    /// Returns the nonce for a settlement tx.
    async fn fetch_settlement_nonce(
        &self,
        settlement_tx_hash: SettlementTxHash,
    ) -> Result<Option<NonceInfo>, Error> {
        // First, get the transaction to extract the nonce.
        let nonce_info = match self
            .l1_rpc
            .get_provider()
            .get_transaction_by_hash(settlement_tx_hash.into())
            .await
            .map_err(|e| {
                Error::L1CommunicationError(Box::new(
                    agglayer_contracts::L1RpcError::UnableToGetTransaction {
                        tx_hash: settlement_tx_hash,
                        source: e.into(),
                    },
                ))
            })? {
            Some(tx) => {
                // Extract nonce from the inner transaction envelope.
                // The inner field derefs to the transaction type which implements the
                // Transaction trait.
                use alloy::consensus::Transaction as _;
                NonceInfo {
                    nonce: tx.inner.nonce(),
                    previous_max_fee_per_gas: tx.inner.max_fee_per_gas(),
                    previous_max_priority_fee_per_gas: tx.inner.max_priority_fee_per_gas(),
                }
            }
            None => {
                warn!(%settlement_tx_hash, "Settlement tx not found on L1");
                return Err(Error::L1CommunicationError(Box::new(
                    agglayer_contracts::L1RpcError::UnableToGetTransaction {
                        tx_hash: settlement_tx_hash,
                        source: eyre::eyre!(
                            "Settlement tx not found on L1 for tx: {settlement_tx_hash}"
                        ),
                    },
                )));
            }
        };

        Ok(Some(nonce_info))
    }
}

#[cfg(feature = "testutils")]
mod testutils {
    use agglayer_types::{CertificateId, SettlementTxHash};
    use tracing::warn;

    use super::Error;

    pub(crate) fn inject_settle_certificate_fail_points(
        certificate_id: CertificateId,
    ) -> Result<(), Error> {
        // Check if fail points are active and log warnings
        if fail::eval(
            "notifier::packer::settle_certificate::receipt_future_ended::status_0",
            |_| true,
        )
        .unwrap_or(false)
        {
            warn!(
                "FAIL POINT ACTIVE: Simulating transaction receipt with status 0 (failed \
                 transaction)"
            );
            return Err(Error::SettlementError {
                certificate_id,
                error: "Settlement transaction failed (simulated via fail point)".to_string(),
            });
        }

        if fail::eval(
            "notifier::packer::settle_certificate::receipt_future_ended::no_receipt",
            |_| true,
        )
        .unwrap_or(false)
        {
            warn!("FAIL POINT ACTIVE: Simulating no receipt found");
            return Err(Error::SettlementError {
                certificate_id,
                error: "No transaction receipt found (simulated via fail point)".to_string(),
            });
        }

        fail::fail_point!("notifier::packer::settle_certificate::receipt_future_ended");

        Ok(())
    }

    #[cfg(feature = "testutils")]
    pub(crate) fn inject_settle_certificate_timeout_fail_points(
        certificate_id: CertificateId,
        settlement_tx_hash: SettlementTxHash,
    ) -> Result<(), Error> {
        // Check if fail points are active and log warnings
        if fail::eval(
            "notifier::packer::settle_certificate::receipt_future_ended::timeout",
            |_| true,
        )
        .unwrap_or(false)
        {
            warn!("FAIL POINT ACTIVE: Simulating pending transaction timeout");
            return Err(Error::PendingTransactionTimeout {
                certificate_id,
                settlement_tx_hash,
                source: eyre::eyre!("Pending transaction timeout (simulated via fail point)"),
            });
        }

        Ok(())
    }
}
