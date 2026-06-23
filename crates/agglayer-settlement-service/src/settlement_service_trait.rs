//! The settlement capability the certificate orchestrator depends on.
//!
//! The trait hands the orchestrator plain values: a job id to persist, and the
//! terminal settlement result to act on. The [`SettlementJobWatcher`] and the
//! in-memory/storage lookup stay an implementation detail of the service, so
//! the orchestrator never has to reason about watchers.
//!
//! [`SettlementJobWatcher`]: crate::settlement_service::SettlementJobWatcher

use agglayer_storage::stores::{SettlementReader, SettlementWriter};
use agglayer_types::{SettlementJob, SettlementJobId, SettlementJobResult};
use alloy::providers::{Provider, WalletProvider};

use crate::settlement_service::{RetrievedSettlementResult, SettlementService};

#[cfg_attr(feature = "testutils", mockall::automock)]
#[async_trait::async_trait]
pub trait SettlementServiceTrait: Send + Sync {
    /// Submit a settlement job and return its id once the service has accepted
    /// and persisted it. The caller persists the id to link it to its
    /// certificate, then waits for the result separately.
    async fn submit_settlement_job(&self, job: SettlementJob) -> eyre::Result<SettlementJobId>;

    /// Wait for a previously submitted job to reach a terminal result.
    ///
    /// Resolves from the running task when the job is still in flight, or from
    /// storage when it already completed (e.g. after a reboot).
    async fn wait_for_settlement(
        &self,
        job_id: SettlementJobId,
    ) -> eyre::Result<SettlementJobResult>;
}

#[async_trait::async_trait]
impl<L1Provider, SettlementStore> SettlementServiceTrait
    for SettlementService<L1Provider, SettlementStore>
where
    L1Provider: Provider + WalletProvider + 'static,
    SettlementStore: SettlementReader + SettlementWriter + Send + Sync + 'static,
{
    async fn submit_settlement_job(&self, job: SettlementJob) -> eyre::Result<SettlementJobId> {
        Ok(self.request_new_settlement(job).await?.job_id())
    }

    async fn wait_for_settlement(
        &self,
        job_id: SettlementJobId,
    ) -> eyre::Result<SettlementJobResult> {
        match self.retrieve_settlement_result(job_id).await? {
            RetrievedSettlementResult::Completed(result) => Ok(result),
            RetrievedSettlementResult::Pending(mut watcher) => watcher.wait_for_result().await,
        }
    }
}
