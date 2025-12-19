use std::sync::Arc;

use agglayer_storage::{
    columns::latest_settled_certificate_per_network::SettledCertificate,
    stores::{StateReader, StateWriter},
};
use agglayer_types::CertificateStatus;
use alloy::{eips::BlockId, providers::Provider};
use tokio_util::sync::CancellationToken;
use tracing::{error, warn};

use crate::SettlementClient;

/// Monitors L1 finalized blocks and updates certificate statuses
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
    let mut subscription = provider.subscribe_blocks().await?;
    let mut latest_finalized_block = 0;
    loop {
        tokio::select! {
            _ = subscription.recv() => {
                let current_finalized_block = match provider.get_block(BlockId::finalized()).await {
                    Ok(Some(current_finalized_block)) => current_finalized_block,
                    Ok(None) => {
                        warn!("Failed to receive the latest finalized block (1)");
                        continue;
                    }
                    Err(e) => {
                        error!(error = ?e, "Failed to receive the latest finalized block (2)");
                        agglayer_telemetry::mainnet_rpc::record_connection_error();
                        continue;
                    }
                };
                if latest_finalized_block != current_finalized_block.number() {
                    latest_finalized_block = current_finalized_block.number();
                    if let Err(e) = finalize_settled_certificates(&store, latest_finalized_block) {
                        error!(error=?e, "Failed finalize_settled_certificates call");
                    }
                }
            }
            _ = cancellation_token.cancelled() => break,
        }
    }
    Ok(())
}

/// Updates certificate statuses from `Settled` to `Finalized` based on L1
/// finalized block height.
pub(crate) fn finalize_settled_certificates<StateStore>(
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
        let SettledCertificate(certificate_id, _, _, _, settled_block_number) = cert;

        let Some(header) = store.get_certificate_header(&certificate_id)? else {
            continue;
        };
        if header.status == CertificateStatus::Settled
            && finalized_block >= settled_block_number.into()
        {
            store.update_certificate_header_status(&cert.0, &CertificateStatus::Finalized)?;
        }
    }

    Ok(())
}
