//! The core logic of the agglayer.
use std::sync::Arc;

use agglayer_config::Config;
use agglayer_contracts::contracts::{
    PolygonRollupManager::{PolygonRollupManagerInstance, RollupDataReturnV2},
    PolygonZkEvm::PolygonZkEvmInstance,
};
use agglayer_rate_limiting::RateLimiter;
use agglayer_rpc::error::SignatureVerificationError;
use agglayer_types::Address;
use alloy::{
    contract::Error as ContractError,
    network::Ethereum,
    primitives::{BlockNumber, B256},
    providers::{PendingTransactionBuilder, PendingTransactionError, Provider},
    rpc::types::TransactionReceipt,
    transports::{RpcError, TransportErrorKind},
};
use thiserror::Error;
use tracing::{info, instrument, warn};

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
}

#[derive(Error, Debug)]
pub enum CheckTxStatusError {
    #[error("provider error: {0}")]
    ProviderError(#[source] RpcError<TransportErrorKind>),
}

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
    ) -> Result<PendingTransactionBuilder<Ethereum>, ContractError> {
        let sequencer_address = self
            .get_trusted_sequencer_address(signed_tx.tx.rollup_id)
            .await?;

        // TODO: pending state num is not yet supported
        const PENDING_STATE_NUM: u64 = 0;

        self.get_rollup_manager_contract()
            .verifyBatchesTrustedAggregator(
                signed_tx.tx.rollup_id,
                PENDING_STATE_NUM,
                signed_tx.tx.last_verified_batch.as_limbs()[0],
                signed_tx.tx.new_verified_batch.as_limbs()[0],
                signed_tx.tx.zkp.new_local_exit_root,
                signed_tx.tx.zkp.new_state_root,
                sequencer_address.into(),
                signed_tx
                    .tx
                    .zkp
                    .proof
                    .to_fixed_bytes()
                    .map(|value| value.into()),
            )
            .send()
            .await
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
    #[instrument(skip(self, rate_guard), level = "debug")]
    pub(crate) async fn settle(
        &self,
        signed_tx: &SignedTx,
        rate_guard: agglayer_rate_limiting::SendTxSlotGuard,
    ) -> Result<TransactionReceipt, SettlementError> {
        let hex_hash = signed_tx.hash();
        let hash = format!("{hex_hash:?}");

        let pending_tx = self
            .verify_batches_trusted_aggregator(signed_tx)
            .await
            .map_err(SettlementError::ContractError)?;

        if let Ok(Some(tx)) = self.check_tx_status(*pending_tx.tx_hash()).await {
            warn!(hash, "Transaction already settled: {tx:?}");
        }

        pending_tx
            .get_receipt()
            .await
            .inspect(|tx_receipt| {
                rate_guard.record(tokio::time::Instant::now());
                info!(
                    block_hash = ?tx_receipt.block_hash,
                    block_number = ?tx_receipt.block_number,
                    "Inspect settle transaction: {}", tx_receipt.transaction_hash
                )
            })
            .map_err(SettlementError::PendingTransactionError)
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
        let sequencer_address = self
            .get_trusted_sequencer_address(u32::from(cert.network_id))
            .await?;

        let signer: Address = cert
            .signer()
            .map_err(SignatureVerificationError::CouldNotRecoverCertSigner)?;

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
}
