//! The core logic of the agglayer.
use std::sync::Arc;

use ethers::prelude::*;
use thiserror::Error;
use tracing::instrument;

use crate::{
    config::Config,
    contracts::{
        polygon_rollup_manager::{PolygonRollupManager, RollupIDToRollupDataReturn},
        polygon_zk_evm::PolygonZkEvm,
    },
    signed_tx::SignedTx,
    zkevm_node_client::ZkevmNodeClient,
};

/// The core logic of the agglayer.
///
/// Currently, it provides functionality for interacting with the various rollup
/// network components in a simplified manner.
///
/// In the future, it may provide functionality for proof aggregation,
/// batching, epoch management, among other things.
#[derive(Debug)]
pub(crate) struct Kernel<RpcProvider> {
    rpc: Arc<RpcProvider>,
    config: Config,
}

/// Kernel constructor arguments.
pub(crate) struct KernelArgs<RpcProvider> {
    pub(crate) rpc: RpcProvider,
    pub(crate) config: Config,
}

/// Errors related to the ZkEVM node proof verification process.
#[derive(Error, Debug)]
pub(crate) enum ZkevmNodeVerificationError {
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
}

impl<RpcProvider> Kernel<RpcProvider> {
    pub(crate) fn new(KernelArgs { rpc, config }: KernelArgs<RpcProvider>) -> Self {
        Self {
            rpc: Arc::new(rpc),
            config,
        }
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

        Ok(ZkevmNodeClient::new(
            jsonrpsee::http_client::HttpClientBuilder::new().build(url.as_str())?,
        ))
    }

    /// Verify that the given [`SignedProof`] is valid according to the ZkEVM
    /// node.
    ///
    /// This involves an RPC call to the ZkEVM node to verify the state root and
    /// exit roots of the signed proof match that of the ZkEVM node's local
    /// record.
    #[instrument(skip(self), level = "debug")]
    pub(crate) async fn verify_proof_zkevm_node(
        &self,
        signed_tx: &SignedTx,
    ) -> Result<(), ZkevmNodeVerificationError> {
        let client = self.get_zkevm_node_client_for_rollup(signed_tx.tx.rollup_id)?;
        let batch = client
            .batch_by_number(signed_tx.tx.rollup_id as u64)
            .await?;

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
        PolygonRollupManager::new(self.config.l1.rollup_manager_contract, self.rpc.clone())
    }
}

/// Errors related to signature verification process.
#[derive(Error, Debug)]
pub(crate) enum SignatureVerificationError<RpcProvider>
where
    RpcProvider: Middleware,
{
    /// The signer could not be recovered from the signature.
    #[error("could not recover signer: {0}")]
    CouldNotRecoverSigner(SignatureError),
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
pub(crate) enum SettlementError<RpcProvider>
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
        let tuple = self
            .get_rollup_manager_contract()
            .rollup_id_to_rollup_data(rollup_id)
            .await?;

        Ok(RollupIDToRollupDataReturn {
            rollup_contract: tuple.0,
            chain_id: tuple.1,
            verifier: tuple.2,
            fork_id: tuple.3,
            last_local_exit_root: tuple.4,
            last_batch_sequenced: tuple.5,
            last_verified_batch: tuple.6,
            last_pending_state: tuple.7,
            last_pending_state_consolidated: tuple.8,
            last_verified_batch_before_upgrade: tuple.9,
            rollup_type_id: tuple.10,
            rollup_compatibility_id: tuple.11,
        })
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
            rollup_metadata.rollup_contract,
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
        self.get_rollup_contract(rollup_id)
            .await?
            .trusted_sequencer()
            .await
    }

    /// Construct a call to the `verifyBatchesTrustedAggregator` (`0x1489ed10`)
    /// method on the rollup manager contract for a given [`SignedProof`].
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

    /// Verify that the signer of the given [`SignedProof`] is the trusted
    /// sequencer for the rollup id specified in the proof.
    #[instrument(skip(self), level = "debug")]
    pub(crate) async fn verify_signature(
        &self,
        signed_tx: &SignedTx,
    ) -> Result<(), SignatureVerificationError<RpcProvider>> {
        let sequencer_address = self
            .get_trusted_sequencer_address(signed_tx.tx.rollup_id)
            .await?;
        let signer = signed_tx
            .signer()
            .map_err(|e| SignatureVerificationError::CouldNotRecoverSigner(e))?;

        if signer != sequencer_address {
            return Err(SignatureVerificationError::InvalidSigner {
                signer,
                trusted_sequencer: sequencer_address,
            });
        }

        Ok(())
    }

    /// Verify that the given [`SignedProof`] does not error during eth_call
    /// dry run.
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

    /// Settle the given [`SignedProof`] to the rollup manager.
    #[instrument(skip(self), level = "debug")]
    pub(crate) async fn settle(
        &self,
        signed_tx: &SignedTx,
    ) -> Result<TransactionReceipt, SettlementError<RpcProvider>> {
        let f = self
            .build_verify_batches_trusted_aggregator_call(signed_tx)
            .await
            .map_err(SettlementError::ContractError)?;

        let tx = f
            .send()
            .await
            .map_err(SettlementError::ContractError)?
            .await
            .map_err(SettlementError::ProviderError)?
            // If the result is `None`, it means the transaction is no longer in the mempool.
            .ok_or(SettlementError::NoReceipt)?;

        Ok(tx)
    }
}
