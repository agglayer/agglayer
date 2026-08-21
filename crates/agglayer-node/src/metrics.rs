//! Registration of metrics backed by in-memory node state.

use std::sync::Arc;

use agglayer_settlement_service::SettlementService;

/// Register the gauge counting live settlement jobs, backed by the running
/// settlement service.
///
/// The closure holds a weak reference because the process-global meter
/// provider outlives in-process node shutdowns. Once the service drops, the
/// gauge reports zero jobs.
pub(crate) fn register_settlement_job_metrics<L1Provider, SettlementStore>(
    settlement_service: &Arc<SettlementService<L1Provider, SettlementStore>>,
) where
    L1Provider: Send + Sync + 'static,
    SettlementStore: Send + Sync + 'static,
{
    let settlement_service = Arc::downgrade(settlement_service);

    agglayer_telemetry::settlement::register_settlement_job_metrics(Box::new(move || {
        match settlement_service.upgrade() {
            Some(service) => service.live_job_count(),
            None => 0,
        }
    }));
}
