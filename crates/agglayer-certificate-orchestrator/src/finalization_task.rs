use agglayer_storage::stores::{StateReader, StateWriter};
use agglayer_types::CertificateStatus;
use alloy::{eips::BlockId, providers::Provider};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

use tracing::error;

use crate::SettlementClient;

pub async fn certificate_finalization_update_task<StateStore, Sc>(
    store: Arc<StateStore>,
    settlement_client: Arc<Sc>,
    cancellation_token: CancellationToken,
) -> eyre::Result<()>
where
    StateStore: StateReader + StateWriter + 'static,
    Sc: SettlementClient,
{
    let provider = settlement_client.get_provider();
    let mut new_block_subscription = provider.subscribe_blocks().await?;
    let mut latest_finalized_block = 0;
    loop {
        tokio::select! {
            _ = new_block_subscription.recv() => {
                let current_finalized_block = match provider.get_block(BlockId::finalized()).await {
                    Ok(Some(current_finalized_block)) => current_finalized_block,
                    Ok(None) => { continue }
                    Err(e) => {
                        error!(error = ?e, "Failed to receive the latest finalized block");
                        agglayer_telemetry::mainnet_rpc::record_connection_error();
                        continue;
                    }
                };

                if latest_finalized_block != current_finalized_block.number() {
                    latest_finalized_block = current_finalized_block.number();
                    update_non_finalized_certificates(&store, latest_finalized_block)?;
                }
            }
            _ = cancellation_token.cancelled() => break,
        }
    }
    Ok(())
}

fn update_non_finalized_certificates<StateStore>(
    store: &Arc<StateStore>,
    finalized_block: u64,
) -> eyre::Result<()>
where
    StateStore: StateReader + StateWriter + 'static,
{
    let active_networks = store.get_active_networks()?;
    for network_id in active_networks {
        let Some((_network_id, cert)) =
            store.get_latest_settled_certificate_per_network(&network_id)?
        else {
            continue;
        };
        let settled_block_number = cert.4;
        if finalized_block >= settled_block_number.into() {
            store.update_certificate_header_status(&cert.0, &CertificateStatus::Finalized)?;
        }
    }

    Ok(())
}
