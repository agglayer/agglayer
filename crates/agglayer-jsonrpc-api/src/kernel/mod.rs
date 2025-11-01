//! The core logic of the agglayer.
use std::sync::Arc;

use agglayer_config::{outbound::OutboundRpcSettleConfig, Config};
use agglayer_contracts::{
    adjust_gas_estimate,
    contracts::{
        PolygonRollupManager::{
            verifyBatchesTrustedAggregatorCall, PolygonRollupManagerInstance, RollupDataReturnV2,
        },
        PolygonZkEvm::PolygonZkEvmInstance,
    },
    GasPriceParams,
};
use agglayer_rate_limiting::RateLimiter;
use agglayer_rpc::error::SignatureVerificationError;
use agglayer_types::{primitives::alloy_primitives::TxHash, Address};
use alloy::{
    contract::Error as ContractError,
    primitives::{BlockNumber, B256},
    providers::{PendingTransactionError, Provider},
    rpc::types::TransactionReceipt,
    transports::{RpcError, TransportErrorKind},
};
use futures::TryFutureExt;
use thiserror::Error;
use tracing::{debug, error, info, instrument, warn};

use crate::{signed_tx::SignedTx, zkevm_node_client::ZkevmNodeClient};

#[cfg(test)]
pub(crate) mod tests;

/// The core logic of the agglayer.
///
/// Currently, it provides functionality for interacting with the various rollup
/// network components in a simplified manner.
///
/// In the future, it may provide functionality for proof aggregation,
/// batching, Epoch management, among other things.
#[derive(Debug)]
pub struct Kernel<RpcProvider> {
    rpc: Arc<RpcProvider>,
    rate_limiter: RateLimiter,
    config: Arc<Config>,
    gas_price_params: GasPriceParams,
    settlement_config: OutboundRpcSettleConfig,
}

/// Errors related to the ZkEVM node proof verification process.
#[derive(Error, Debug)]
pub enum ZkevmNodeVerificationError {
    /// The given rollup id is not specified in the configuration.
    #[error("invalid rollup id: {0}")]
    InvalidRollupId(u32),

    /// Generic error when communicating with the ZkEVM node.
    #[error("rpc error: {0}")]
    RpcError(#[from] jsonrpsee::core::client::error::Error),

    /// The state root in the proof does not match the ZkEVM node's local
    /// record.
    #[error("invalid state root. expected: {expected}, got: {got}")]
    InvalidStateRoot { expected: B256, got: B256 },

    /// The exit root in the proof does not match the ZkEVM node's local record.
    #[error("invalid exit root. expected: {expected}, got: {got}")]
    InvalidExitRoot { expected: B256, got: B256 },

    /// Unable to query the state and exit roots.
    #[error("Unable to query exit and state root for batch {batch_no} (network {network_id})")]
    RootsNotFound { batch_no: u64, network_id: u32 },
}

impl<RpcProvider> Kernel<RpcProvider> {
    pub fn new(rpc: Arc<RpcProvider>, config: Arc<Config>) -> Self {
        Self {
            rpc,
            rate_limiter: RateLimiter::new(config.rate_limiting.clone()),
            gas_price_params: {
                let gas_config = &config.outbound.rpc.settle.gas_price;
                agglayer_contracts::GasPriceParams {
                    multiplier_per_1000: gas_config.multiplier.as_u64_per_1000(),
                    floor: gas_config.floor,
                    ceiling: gas_config.ceiling,
                }
            },
            settlement_config: config.outbound.rpc.settle.clone(),
            config,
        }
    }

    pub(crate) fn rate_limiter(&self) -> &RateLimiter {
        &self.rate_limiter
    }

    /// Check if the given rollup id is registered in the configuration.
    pub(crate) fn check_rollup_registered(&self, rollup_id: u32) -> bool {
        self.config.full_node_rpcs.contains_key(&rollup_id)
    }

    /// Get a [`ZkevmNodeClient`] instance for the given rollup id.
    #[instrument(skip(self), level = "debug")]
    fn get_zkevm_node_client_for_rollup(
        &self,
        rollup_id: u32,
    ) -> Result<ZkevmNodeClient<jsonrpsee::http_client::HttpClient>, ZkevmNodeVerificationError>
    {
        let url = self
            .config
            .full_node_rpcs
            .get(&rollup_id)
            .ok_or(ZkevmNodeVerificationError::InvalidRollupId(rollup_id))?;

        let client = jsonrpsee::http_client::HttpClientBuilder::new()
            .request_timeout(self.config.l2.rpc_timeout)
            .build(url.as_str())?;

        Ok(ZkevmNodeClient::new(client))
    }

    /// Verify that the given [`SignedTx`] is valid according to the ZkEVM node.
    ///
    /// This involves an RPC call to the ZkEVM node to verify the state root and
    /// exit roots of the signed proof match that of the ZkEVM node's local
    /// record.
    #[instrument(skip(self), level = "debug")]
    pub(crate) async fn verify_proof_zkevm_node(
        &self,
        signed_tx: &SignedTx,
    ) -> Result<(), ZkevmNodeVerificationError> {
        let network_id = signed_tx.tx.rollup_id;
        let client = self.get_zkevm_node_client_for_rollup(network_id)?;
        let batch_no: u64 = signed_tx.tx.new_verified_batch.as_limbs()[0];
        let batch = client
            .batch_by_number(signed_tx.tx.new_verified_batch.as_limbs()[0])
            .await?
            .ok_or(ZkevmNodeVerificationError::RootsNotFound {
                batch_no,
                network_id,
            })?;

        if batch.state_root != signed_tx.tx.zkp.new_state_root {
            return Err(ZkevmNodeVerificationError::InvalidStateRoot {
                expected: signed_tx.tx.zkp.new_state_root,
                got: batch.state_root,
            });
        }

        if batch.local_exit_root != signed_tx.tx.zkp.new_local_exit_root {
            return Err(ZkevmNodeVerificationError::InvalidExitRoot {
                expected: signed_tx.tx.zkp.new_local_exit_root,
                got: batch.local_exit_root,
            });
        }

        Ok(())
    }
}

impl<RpcProvider> Kernel<RpcProvider>
where
    RpcProvider: Provider,
{
    /// Get a [`ContractInstance`] of the rollup manager contract,
    /// [`PolygonRollupManager`].
    ///
    /// The returned instance facilitates type-safe RPC interaction with the
    /// rollup manager contract.
    ///
    /// The rollup manager contract address is specified by the given
    /// configuration.
    fn get_rollup_manager_contract(&self) -> PolygonRollupManagerInstance<Arc<RpcProvider>> {
        PolygonRollupManagerInstance::new(
            self.config.l1.rollup_manager_contract.into(),
            self.rpc.clone(),
        )
    }
}

/// Errors related to settlement process.
#[derive(Error, Debug)]
pub enum SettlementError {
    /// The transaction receipt is missing.
    #[error("no receipt")]
    NoReceipt,
    #[error("provider error: {0}")]
    ProviderError(alloy::transports::RpcError<TransportErrorKind>),
    #[error("contract error: {0}")]
    ContractError(ContractError),
    #[error(transparent)]
    RateLimited(#[from] agglayer_rate_limiting::RateLimited),
    #[error("Settlement timed out after {}s", .0.as_secs())]
    Timeout(std::time::Duration),
    #[error("pending transaction error: {0}")]
    PendingTransactionError(PendingTransactionError),
    #[error("receipt without block number: {0}")]
    ReceiptWithoutBlockNumberError(TxHash),
}

#[derive(Error, Debug)]
pub enum CheckTxStatusError {
    #[error("provider error: {0}")]
    ProviderError(#[source] RpcError<TransportErrorKind>),
}

type VerifyBatchesMarker = std::marker::PhantomData<verifyBatchesTrustedAggregatorCall>;
type VerifyBatchesBuilder<Rpc> = alloy::contract::CallBuilder<Arc<Rpc>, VerifyBatchesMarker>;

impl<RpcProvider> Kernel<RpcProvider>
where
    RpcProvider: Provider + Clone + 'static,
{
    /// Get the rollup metadata for the given rollup id.
    ///
    /// This involves a contract read from the rollup manager contract. In
    /// particular, it calls `rollupIDToRollupData` (`0xf9c4c2ae`) on the rollup
    /// manager contract and returns the result.
    #[instrument(skip(self), level = "debug")]
    async fn get_rollup_metadata(
        &self,
        rollup_id: u32,
    ) -> Result<RollupDataReturnV2, ContractError> {
        self.get_rollup_manager_contract()
            .rollupIDToRollupDataV2(rollup_id)
            .call()
            .await
    }

    /// Get a [`ContractInstance`], [`PolygonZkEvm`], of the rollup contract at
    /// the given rollup id.
    #[instrument(skip(self), level = "debug")]
    async fn get_rollup_contract_instance(
        &self,
        rollup_id: u32,
    ) -> Result<PolygonZkEvmInstance<RpcProvider>, ContractError> {
        let rollup_metadata = self.get_rollup_metadata(rollup_id).await?;

        Ok(PolygonZkEvmInstance::new(
            rollup_metadata.rollupContract,
            (*self.rpc).clone(),
        ))
    }

    /// Get the address of the trusted sequencer for the given rollup id.
    ///
    /// This involves a contract read from the rollup contract. In particular,
    /// it calls `trustedSequencer` (`0xcfa8ed47`) on the rollup contract.
    #[instrument(skip(self), level = "debug")]
    async fn get_trusted_sequencer_address(
        &self,
        rollup_id: u32,
    ) -> Result<Address, ContractError> {
        if let Some(addr) = self.config.proof_signers.get(&rollup_id) {
            Ok(*addr)
        } else {
            self.get_rollup_contract_instance(rollup_id)
                .await?
                .trustedSequencer()
                .call()
                .await
                .map(Into::into)
        }
    }

    /// Execute a call to the `verifyBatchesTrustedAggregator` (`0x1489ed10`)
    /// method on the rollup manager contract for a given [`SignedTx`].
    ///
    /// Note that this does not actually invoke the function, but rather
    /// constructs a [`FunctionCall`] that can be used to create a dry-run
    /// or send a transaction.
    #[instrument(skip(self), level = "debug")]
    pub(crate) async fn verify_batches_trusted_aggregator(
        &self,
        signed_tx: &SignedTx,
    ) -> Result<VerifyBatchesBuilder<RpcProvider>, ContractError> {
        let sequencer_address = self
            .get_trusted_sequencer_address(signed_tx.tx.rollup_id)
            .await?;

        // TODO: pending state num is not yet supported
        const PENDING_STATE_NUM: u64 = 0;

        let rollup_manager_contract = self.get_rollup_manager_contract();
        let proof_bytes = signed_tx.tx.zkp.proof.to_fixed_bytes().map(Into::into);
        let call = rollup_manager_contract
            .verifyBatchesTrustedAggregator(
                signed_tx.tx.rollup_id,
                PENDING_STATE_NUM,
                signed_tx.tx.last_verified_batch.as_limbs()[0],
                signed_tx.tx.new_verified_batch.as_limbs()[0],
                signed_tx.tx.zkp.new_local_exit_root,
                signed_tx.tx.zkp.new_state_root,
                sequencer_address.into(),
                proof_bytes,
            )
            .with_cloned_provider();

        Ok(call)
    }

    /// Verify that the signer of the given [`SignedTx`] is the trusted
    /// sequencer for the rollup id specified in the proof.
    #[instrument(skip(self), level = "debug")]
    pub(crate) async fn verify_tx_signature(
        &self,
        signed_tx: &SignedTx,
    ) -> Result<(), SignatureVerificationError> {
        let sequencer_address = self
            .get_trusted_sequencer_address(signed_tx.tx.rollup_id)
            .await?;

        let signer = signed_tx
            .signer()
            .map_err(SignatureVerificationError::CouldNotRecoverTxSigner)?;

        // ECDSA-k256 signature verification works by recovering the public key from the
        // signature, and then checking that it is the expected one.
        if signer != sequencer_address {
            return Err(SignatureVerificationError::InvalidSigner {
                signer,
                trusted_sequencer: sequencer_address,
            });
        }

        Ok(())
    }

    /// Settle the given [`SignedTx`] to the rollup manager.
    #[instrument(skip(self, signed_tx, rate_guard), level = "debug")]
    pub(crate) async fn settle(
        &self,
        signed_tx: &SignedTx,
        rate_guard: agglayer_rate_limiting::SendTxSlotGuard,
    ) -> Result<TransactionReceipt, SettlementError> {
        let signed_tx_hash = format!("{}", signed_tx.hash());
        let pending_tx = self
            .verify_batches_trusted_aggregator(signed_tx)
            .and_then(|call| async move {
                let estimate = self.rpc.estimate_eip1559_fees().await?;
                let adjusted = adjust_gas_estimate(&estimate, &self.gas_price_params);

                debug!(
                    gas_price_params=?self.gas_price_params,
                    estimate=?estimate,
                    adjusted=?adjusted,
                    "Applying fee adjustments with gas price params"
                );

                call.max_priority_fee_per_gas(adjusted.max_priority_fee_per_gas)
                    .max_fee_per_gas(adjusted.max_fee_per_gas)
                    .send()
                    .await
            })
            .await
            .map_err(SettlementError::ContractError)?;

        debug!(l1_tx_hash = %pending_tx.tx_hash(), "For signed tx {signed_tx_hash} l1 transaction {} sent", pending_tx.tx_hash());

        // We submit the transaction in a separate task so we can observe the
        // settlement process even if the client drops the transaction
        // submission request. This is needed to correctly record the settlement
        // rate limiting event in case the client drops the request.
        let rpc = self.rpc.clone();
        let settlement_config = self.settlement_config.clone();
        let tx_hash = *pending_tx.tx_hash();
        
        let receipt = tokio::spawn(async move {
            Self::wait_for_transaction_receipt_static(rpc, settlement_config, &tx_hash)
                .await
                .inspect(|tx_receipt| {
                    rate_guard.record(tokio::time::Instant::now());
                    info!(
                        block_hash = ?tx_receipt.block_hash,
                        block_number = ?tx_receipt.block_number,
                        l1_tx_hash = %tx_receipt.transaction_hash,
                        "Inspect settle l1 transaction"
                    )
                })
        });
        receipt.await.map_err(|_| SettlementError::NoReceipt)?
    }

    /// Wait for transaction receipt with configurable retries and intervals (static version)
    async fn wait_for_transaction_receipt_static(
        rpc: Arc<RpcProvider>,
        settlement_config: OutboundRpcSettleConfig,
        tx_hash: &TxHash,
    ) -> Result<TransactionReceipt, SettlementError> {
        let timeout = settlement_config
            .retry_interval
            .mul_f64(settlement_config.max_retries as f64); // only used for logs
        let max_retries = settlement_config.max_retries;
        let retry_interval = settlement_config.retry_interval;
        let required_confirmations = settlement_config.confirmations;

        debug!(
            max_retries,
            timeout=?timeout,
            retry_interval=?retry_interval,
            "Waiting for signed transaction receipt with timeout of {timeout:?}, max_retries: \
             {max_retries} and retry_interval: {retry_interval:?}",
        );

        for attempt in 0..=max_retries {
            match rpc.get_transaction_receipt(*tx_hash).await {
                Ok(Some(receipt)) => {
                    info!(attempt, "Successfully fetched transaction receipt");

                    // Wait for the required number of confirmations
                    if required_confirmations > 0 {
                        let receipt_block = receipt.block_number.ok_or_else(|| {
                            error!(%tx_hash, "Transaction receipt has no block number");
                            SettlementError::ReceiptWithoutBlockNumberError(
                                receipt.transaction_hash,
                            )
                        })?;

                        debug!(
                            receipt_block,
                            required_confirmations, "Waiting for L1 block confirmations"
                        );

                        // Wait until we have the required number of confirmations
                        for confirmation_attempt in attempt..=max_retries {
                            match rpc.get_block_number().await {
                                Ok(current_block) => {
                                    let current_confirmations = current_block
                                        .saturating_sub(receipt_block)
                                        .saturating_add(1);
                                    if current_confirmations >= required_confirmations as u64 {
                                        info!(
                                            current_confirmations,
                                            required_confirmations,
                                            current_block,
                                            "L1 transaction confirmed with required confirmations"
                                        );
                                        return Ok(receipt);
                                    } else {
                                        debug!(
                                            current_confirmations,
                                            required_confirmations,
                                            "Waiting for more confirmations, sleeping"
                                        );
                                        tokio::time::sleep(retry_interval).await;
                                    }
                                }
                                Err(error) => {
                                    if confirmation_attempt < max_retries {
                                        warn!(
                                            ?error,
                                            "Failed to get current block number, retrying"
                                        );
                                        tokio::time::sleep(retry_interval).await;
                                        continue;
                                    } else {
                                        error!(
                                            ?error,
                                            "Failed to get current block number after maximum \
                                             retries"
                                        );
                                        return Err(SettlementError::ProviderError(error));
                                    }
                                }
                            }
                        }

                        // Timeout waiting for confirmations
                        error!(
                            ?timeout,
                            "Timeout while waiting for transaction confirmations"
                        );
                        return Err(SettlementError::Timeout(timeout));
                    } else {
                        // No confirmations required, return immediately
                        return Ok(receipt);
                    }
                }
                Ok(None) => {
                    // Transaction not yet included in a block, continue retrying
                    if attempt < max_retries {
                        debug!(
                            %tx_hash,
                            next_attempt = attempt + 1,
                            max_retries,
                            retry_interval = ?retry_interval,
                            "L1 transaction receipt not found yet, retrying",
                        );
                        tokio::time::sleep(retry_interval).await;
                        continue;
                    }
                }
                Err(error) => {
                    // Other error (e.g., network issue, RPC error)
                    error!(
                        ?error,
                        "Error watching the pending signed transaction settlement"
                    );
                    return Err(SettlementError::ProviderError(error));
                }
            }
        }

        error!(
            ?timeout,
            "Timeout while watching the pending signed transaction settlement"
        );
        Err(SettlementError::Timeout(timeout))
    }

    /// Check the status of the given hash.
    #[instrument(skip(self), level = "debug")]
    pub(crate) async fn check_tx_status(
        &self,
        hash: B256,
    ) -> Result<Option<TransactionReceipt>, CheckTxStatusError> {
        self.rpc
            .get_transaction_receipt(hash)
            .await
            .map_err(CheckTxStatusError::ProviderError)
    }

    /// Get the current L1 block height.
    #[instrument(skip(self), level = "debug")]
    pub(crate) async fn current_l1_block_height(&self) -> Result<BlockNumber, CheckTxStatusError> {
        self.rpc
            .get_block_number()
            .await
            .map_err(CheckTxStatusError::ProviderError)
    }
}

#[cfg(test)]
impl<RpcProvider> Kernel<RpcProvider>
where
    RpcProvider: Provider + Clone + 'static,
{
    /// Verify that the signer of the given [`Certificate`] is the trusted
    /// sequencer for the rollup id it specified.
    #[instrument(skip(self), level = "debug")]
    pub(crate) async fn verify_cert_signature(
        &self,
        cert: &agglayer_types::Certificate,
    ) -> Result<(), SignatureVerificationError> {
        use agglayer_types::{aggchain_data::MultisigCtx, aggchain_proof::AggchainData};

        let sequencer_address = self
            .get_trusted_sequencer_address(u32::from(cert.network_id))
            .await?;

        let multisig_ctx = MultisigCtx {
            signers: Default::default(), // TODO: to fetch from L1
            threshold: 1,                // TODO: to fetch from L1
            prehash: cert.signature_commitment_values().multisig_commitment(),
        };

        match &cert.aggchain_data {
            AggchainData::ECDSA { signature } => {
                cert.verify_legacy_ecdsa(sequencer_address, signature)
            }
            AggchainData::Generic { signature, .. } => {
                cert.verify_aggchain_proof_signature(sequencer_address, signature)
            }
            AggchainData::MultisigOnly { multisig } => {
                cert.verify_multisig(multisig.into(), multisig_ctx)
            }
            AggchainData::MultisigAndAggchainProof { multisig, .. } => {
                cert.verify_multisig(multisig.into(), multisig_ctx)
            }
        }
        .map_err(SignatureVerificationError::from_signer_error)
    }
}
