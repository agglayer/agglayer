use std::{future::IntoFuture, sync::Arc};

use agglayer_contracts::{
    contracts::PolygonRollupManager::VerifyPessimisticStateTransition, rollup::RollupContract,
    L1TransactionFetcher,
};
use agglayer_storage::stores::{MetadataReader, MetadataWriter, StateReader, StateWriter};
use agglayer_types::Digest;
use alloy::{
    eips::BlockNumberOrTag,
    providers::Provider,
    rpc::types::{Filter, Log},
    sol_types::SolEvent,
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

use crate::Error;

pub struct ListenerTask<StateStore, RollupManagerRpc> {
    state_store: Arc<StateStore>,
    l1_rpc: Arc<RollupManagerRpc>,
}

impl<StateStore, RollupManagerRpc> ListenerTask<StateStore, RollupManagerRpc>
where
    StateStore: StateWriter + StateReader + MetadataReader + MetadataWriter,
    RollupManagerRpc: RollupContract + L1TransactionFetcher,
{
    pub fn new(state_store: Arc<StateStore>, l1_rpc: Arc<RollupManagerRpc>) -> Self {
        info!("Creating a new listener task");

        Self {
            state_store,
            l1_rpc,
        }
    }

    pub(crate) async fn run(&self, cancellation_token: CancellationToken) -> Result<(), Error> {
        info!("Starting the listener task");

        let start_block = self
            .state_store
            .get_latest_certificate_settling_block()?
            .unwrap_or(0); // start from genesis if the column doesn't exist

        let rollup_address = self.l1_rpc.get_rollup_manager_address();
        let filter = Filter::new()
            .address(rollup_address.into_alloy())
            .event_signature(VerifyPessimisticStateTransition::SIGNATURE_HASH)
            .from_block(BlockNumberOrTag::Number(start_block));
        let mut subscription = self
            .l1_rpc
            .get_provider()
            .subscribe_logs(&filter)
            .into_future()
            .await
            .map_err(|e| Error::InternalError(format!("Failed to subscribe to logs: {e}")))?;

        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    debug!("Listener task has been cancelled");
                    return Ok(());
                }

                log = subscription.recv() => {
                    let log = log.map_err(|e| Error::InternalError(format!("Failed to receive log: {e}")))?;
                    if let Err(err) = self.handle_event(log) {
                        error!("Error handling settlement event: {err}");
                    }
                }
            }
        }
    }

    fn handle_event(&self, log: Log) -> Result<(), Error> {
        let event = VerifyPessimisticStateTransition::decode_log(&log.clone().into()).ok();
        let latest_pp_root = event
            .as_ref()
            .map(|val| Digest::from(val.newPessimisticRoot));
        let tx_hash = log.transaction_hash.map(Digest::from);
        // rollupID is the same as topic1, which is the network ID
        let network_id = event.as_ref().map(|val| val.rollupID);

        if let (Some(pp_root), Some(tx_hash), Some(network_id), Some(block_number)) =
            (latest_pp_root, tx_hash, network_id, log.block_number)
        {
            debug!(
                "Retrieved latest VerifyPessimisticStateTransition event for network {}, latest \
                pp_root: {}, tx_hash: {tx_hash}",
                network_id, pp_root
            );

            let certificate_ids = self
                .state_store
                .get_certificate_ids_for_pp_root(&pp_root)?;

            let Some(settled_certificate_id) = certificate_ids.last() else {
                return Err(Error::InternalError(format!(
                    "No settled certificate found for pp root: {pp_root}"
                )));
            };

            self.state_store
                .update_settlement_tx_hash(settled_certificate_id, tx_hash.into(), false)?;

            self.state_store
                .set_latest_certificate_settling_block(block_number)?;
        }
        Ok(())
    }
}
