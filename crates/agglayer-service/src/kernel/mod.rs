//! The core logic of the agglayer.
use std::sync::Arc;

use agglayer_config::Config;
use agglayer_contracts::{
    polygon_rollup_manager::{PolygonRollupManager, RollupIDToRollupDataReturn},
    polygon_zk_evm::PolygonZkEvm,
};
use agglayer_types::Certificate;
use ethers::{
    contract::{ContractCall, ContractError},
    providers::{Middleware, ProviderError},
    types::{Address, TransactionReceipt, H160, H256, U64},
};
use thiserror::Error;
use tracing::{info, instrument, warn};

use crate::{
    rate_limiting::{self, RateLimiter},
    signed_tx::SignedTx,
    zkevm_node_client::ZkevmNodeClient,
};

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
pub(crate) struct Kernel<RpcProvider> {
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
    InvalidStateRoot { expected: H256, got: H256 },

    /// The exit root in the proof does not match the ZkEVM node's local record.
    #[error("invalid exit root. expected: {expected}, got: {got}")]
    InvalidExitRoot { expected: H256, got: H256 },

    /// Unable to query the state and exit roots.
    #[error("Unable to query exit and state root for batch {batch_no} (network {network_id})")]
    RootsNotFound { batch_no: u64, network_id: u32 },
}

impl<RpcProvider> Kernel<RpcProvider> {
    pub(crate) fn new(rpc: Arc<RpcProvider>, config: Arc<Config>) -> Self {
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
        let batch_no = signed_tx.tx.new_verified_batch.as_u64();
        let batch = client
            .batch_by_number(signed_tx.tx.new_verified_batch.as_u64())
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
    RpcProvider: Middleware,
{
    /// Get a [`ContractInstance`] of the rollup manager contract,
    /// [`PolygonRollupManager`].
    ///
    /// The returned instance facilitates type-safe RPC interaction with the
    /// rollup manager contract.
    ///
    /// The rollup manager contract address is specified by the given
    /// configuration.
    fn get_rollup_manager_contract(&self) -> PolygonRollupManager<RpcProvider> {
        PolygonRollupManager::new(self.config.l1.rollup_manager_contract, self.rpc.clone()).clone()
    }
}

/// Errors related to signature verification process.
#[derive(Error, Debug)]
pub enum SignatureVerificationError<RpcProvider>
where
    RpcProvider: Middleware,
{
    /// FEP (0.1): The signer could not be recovered from the [`SignedTx`].
    #[error("could not recover transaction signer: {0}")]
    CouldNotRecoverTxSigner(#[source] ethers::types::SignatureError),

    /// The signer could not be recovered from the certificate signature.
    #[error("could not recover certificate signer: {0}")]
    CouldNotRecoverCertSigner(#[source] alloy::primitives::SignatureError),

    /// The signer of the proof is not the trusted sequencer for the given
    /// rollup id.
    #[error("invalid signer: expected {trusted_sequencer}, got {signer}")]
    InvalidSigner {
        /// The recovered signer address.
        signer: Address,
        /// The trusted sequencer address.
        trusted_sequencer: Address,
    },

    /// Generic network error when attempting to retrieve the trusted sequencer
    /// address from the rollup contract.
    #[error("contract error: {0}")]
    ContractError(#[from] ContractError<RpcProvider>),
}

/// Errors related to settlement process.
#[derive(Error, Debug)]
pub enum SettlementError<RpcProvider>
where
    RpcProvider: Middleware,
{
    /// The transaction receipt is missing.
    #[error("no receipt")]
    NoReceipt,
    #[error("provider error: {0}")]
    ProviderError(ProviderError),
    #[error("contract error: {0}")]
    ContractError(ContractError<RpcProvider>),
    #[error(transparent)]
    RateLimited(#[from] crate::rate_limiting::RateLimited),
    #[error("Settlement timed out after {}s", .0.as_secs())]
    Timeout(std::time::Duration),
}

#[derive(Error, Debug)]
pub enum CheckTxStatusError<RpcProvider: Middleware> {
    #[error("middleware error: {0}")]
    ProviderError(RpcProvider::Error),
}

impl<RpcProvider> Kernel<RpcProvider>
where
    RpcProvider: Middleware + 'static,
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
    ) -> Result<RollupIDToRollupDataReturn, ContractError<RpcProvider>> {
        let rollup_data = self
            .get_rollup_manager_contract()
            .rollup_id_to_rollup_data(rollup_id)
            .await?;

        Ok(RollupIDToRollupDataReturn { rollup_data })
    }

    /// Get a [`ContractInstance`], [`PolygonZkEvm`], of the rollup contract at
    /// the given rollup id.
    #[instrument(skip(self), level = "debug")]
    async fn get_rollup_contract(
        &self,
        rollup_id: u32,
    ) -> Result<PolygonZkEvm<RpcProvider>, ContractError<RpcProvider>> {
        let rollup_metadata = self.get_rollup_metadata(rollup_id).await?;

        Ok(PolygonZkEvm::new(
            rollup_metadata.rollup_data.rollup_contract,
            self.rpc.clone(),
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
    ) -> Result<Address, ContractError<RpcProvider>> {
        if let Some(addr) = self.config.proof_signers.get(&rollup_id) {
            Ok(*addr)
        } else {
            self.get_rollup_contract(rollup_id)
                .await?
                .trusted_sequencer()
                .await
        }
    }

    /// Construct a call to the `verifyBatchesTrustedAggregator` (`0x1489ed10`)
    /// method on the rollup manager contract for a given [`SignedTx`].
    ///
    /// Note that this does not actually invoke the function, but rather
    /// constructs a [`FunctionCall`] that can be used to create a dry-run
    /// or send a transaction.
    #[instrument(skip(self), level = "debug")]
    pub(crate) async fn build_verify_batches_trusted_aggregator_call(
        &self,
        signed_tx: &SignedTx,
    ) -> Result<ContractCall<RpcProvider, ()>, ContractError<RpcProvider>> {
        let sequencer_address = self
            .get_trusted_sequencer_address(signed_tx.tx.rollup_id)
            .await?;

        // TODO: pending state num is not yet supported
        const PENDING_STATE_NUM: u64 = 0;

        let call = self
            .get_rollup_manager_contract()
            .verify_batches_trusted_aggregator(
                signed_tx.tx.rollup_id,
                PENDING_STATE_NUM,
                signed_tx.tx.last_verified_batch.as_u64(),
                signed_tx.tx.new_verified_batch.as_u64(),
                signed_tx.tx.zkp.new_local_exit_root.to_fixed_bytes(),
                signed_tx.tx.zkp.new_state_root.to_fixed_bytes(),
                sequencer_address,
                signed_tx.tx.zkp.proof.to_fixed_bytes(),
            );

        Ok(call)
    }

    /// Verify that the signer of the given [`SignedTx`] is the trusted
    /// sequencer for the rollup id specified in the proof.
    #[instrument(skip(self), level = "debug")]
    pub(crate) async fn verify_tx_signature(
        &self,
        signed_tx: &SignedTx,
    ) -> Result<(), SignatureVerificationError<RpcProvider>> {
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

    /// Verify that the signer of the given [`Certificate`] is the trusted
    /// sequencer for the rollup id it specified.
    #[instrument(skip(self), level = "debug")]
    pub(crate) async fn verify_cert_signature(
        &self,
        cert: &Certificate,
    ) -> Result<(), SignatureVerificationError<RpcProvider>> {
        let sequencer_address = self
            .get_trusted_sequencer_address(u32::from(cert.network_id))
            .await?;

        let signer: H160 = cert
            .signer()
            .map_err(SignatureVerificationError::CouldNotRecoverCertSigner)
            .map(|signer| signer.into_array().into())?;

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

    /// Verify that the given [`SignedTx`] does not error during eth_call dry
    /// run.
    ///
    /// This involves a contract call to the rollup manager contract. In
    /// particular, it calls `verifyBatchesTrustedAggregator` (`0x1489ed10`) on
    /// the rollup manager contract to assert validitiy of the proof.
    #[instrument(skip(self), level = "debug")]
    pub(crate) async fn verify_proof_eth_call(
        &self,
        signed_tx: &SignedTx,
    ) -> Result<(), ContractError<RpcProvider>> {
        let f = self
            .build_verify_batches_trusted_aggregator_call(signed_tx)
            .await?;
        f.call().await?;

        Ok(())
    }

    /// Settle the given [`SignedTx`] to the rollup manager.
    #[instrument(skip(self, rate_guard), level = "debug")]
    pub(crate) async fn settle(
        &self,
        signed_tx: &SignedTx,
        rate_guard: rate_limiting::SendTxSlotGuard,
    ) -> Result<TransactionReceipt, SettlementError<RpcProvider>> {
        let hex_hash = signed_tx.hash();
        let hash = format!("{:?}", hex_hash);

        let f = self
            .build_verify_batches_trusted_aggregator_call(signed_tx)
            .await
            .map_err(SettlementError::ContractError)?;

        if let Ok(Some(tx)) = self.check_tx_status(hex_hash).await {
            warn!(hash, "Transaction already settled: {:?}", tx);
        }

        // We submit the transaction in a separate task so we can observe the
        // settlement process even if the client drops the transaction
        // submission request. This is needed to correctly record the settlement
        // rate limiting event in case the client drops the request.
        let receipt = tokio::spawn({
            let config = Arc::clone(&self.config);
            async move {
                let config = &*config;
                let hash = &hash;

                let settlement = async move {
                    f.send()
                        .await
                        .inspect(|tx| info!(hash, "Inspect settle transaction: {:?}", tx))
                        .map_err(SettlementError::ContractError)?
                        .interval(config.outbound.rpc.settle.retry_interval)
                        .retries(config.outbound.rpc.settle.max_retries)
                        .confirmations(config.outbound.rpc.settle.confirmations)
                        .await
                        .map_err(SettlementError::ProviderError)?
                        // The result of `None` means the transaction is no longer in the mempool.
                        .ok_or(SettlementError::NoReceipt)
                };

                let settlement_timeout = config.outbound.rpc.settle.settlement_timeout;
                let receipt = tokio::time::timeout(settlement_timeout, settlement)
                    .await
                    .map_err(|_| {
                        warn!(hash, "Settlement of {hash} timed out");
                        SettlementError::Timeout(settlement_timeout)
                    })??;

                rate_guard.record(tokio::time::Instant::now());
                Ok(receipt)
            }
        });

        receipt.await.map_err(|_| SettlementError::NoReceipt)?
    }
}

impl<RpcProvider> Kernel<RpcProvider>
where
    RpcProvider: Middleware + 'static,
{
    /// Check the status of the given hash.
    #[instrument(skip(self), level = "debug")]
    pub(crate) async fn check_tx_status(
        &self,
        hash: H256,
    ) -> Result<Option<TransactionReceipt>, CheckTxStatusError<RpcProvider>> {
        self.rpc
            .get_transaction_receipt(hash)
            .await
            .map_err(CheckTxStatusError::ProviderError)
    }

    /// Get the current L1 block height.
    #[instrument(skip(self), level = "debug")]
    pub(crate) async fn current_l1_block_height(
        &self,
    ) -> Result<U64, CheckTxStatusError<RpcProvider>> {
        self.rpc
            .get_block_number()
            .await
            .map_err(CheckTxStatusError::ProviderError)
    }
}
