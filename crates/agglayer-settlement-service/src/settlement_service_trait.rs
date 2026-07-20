//! The settlement capability the certificate orchestrator depends on.
//!
//! The trait hands the orchestrator plain values: a job id to persist, and the
//! terminal settlement result to act on. The [`SettlementJobWatcher`] and the
//! in-memory/storage lookup stay an implementation detail of the service, so
//! the orchestrator never has to reason about watchers.
//!
//! [`SettlementJobWatcher`]: crate::settlement_service::SettlementJobWatcher

use agglayer_storage::stores::{SettlementReader, SettlementWriter, StateReader, StateWriter};
use agglayer_types::{CertificateId, SettlementJob, SettlementJobId, SettlementJobResult};
use alloy::providers::{Provider, WalletProvider};

use crate::settlement_service::{RetrievedSettlementResult, SettlementService};

#[cfg_attr(feature = "testutils", mockall::automock)]
#[async_trait::async_trait]
pub trait SettlementServiceTrait: Send + Sync {
    /// Submit a settlement job for `certificate_id` and return its id once the
    /// service has accepted and persisted it.
    ///
    /// The service records the certificate <-> job-id link atomically when it
    /// creates the job and rejects a second submission for the same
    /// certificate; that guard is what keeps settlement at-most-once across
    /// a crash/restart. The caller waits for the result separately.
    async fn submit_settlement_job(
        &self,
        certificate_id: CertificateId,
        job: SettlementJob,
    ) -> eyre::Result<SettlementJobId>;

    /// Wait for a previously submitted job to reach a terminal result.
    ///
    /// Resolves from the running task while the job is in flight (the service
    /// re-spawns a task + watcher for every pending job on startup), or from
    /// storage when the job already reached a terminal result.
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
    SettlementStore:
        SettlementReader + SettlementWriter + StateReader + StateWriter + Send + Sync + 'static,
{
    async fn submit_settlement_job(
        &self,
        certificate_id: CertificateId,
        job: SettlementJob,
    ) -> eyre::Result<SettlementJobId> {
        Ok(self
            .request_new_settlement(Some(certificate_id), job)
            .await?
            .job_id())
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
