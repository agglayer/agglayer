//! Async adapters for complete logical store operations.
//!
//! The synchronous store traits remain the blocking core. These extension
//! traits move owned inputs and an [`Arc`] of the store onto Tokio's blocking
//! pool, and return only owned results.

use std::{future::Future, sync::Arc};

use agglayer_types::{
    network_info::DisabledBy, Address, Certificate, CertificateHeader, CertificateId,
    CertificateStatus, Height, NetworkId, Nonce, Proof, SettlementAttempt, SettlementAttemptResult,
    SettlementJob, SettlementJobId, SettlementJobResult, SettlementTxHash,
};

use super::{
    EditEvenIfCompleted, PendingCertificateReader, PendingCertificateWriter, SettlementReader,
    SettlementWriter, StateReader, StateWriter, UpdateEvenIfAlreadyPresent,
    UpdateStatusToCandidate,
};
use crate::{columns::latest_proven_certificate_per_network::ProvenCertificate, error::Error};

/// Async access to complete pending-store read operations.
///
/// Each future runs one synchronous logical store operation on Tokio's
/// blocking pool. RocksDB iterators and borrowed values never leave that job.
pub trait AsyncPendingCertificateReaderExt: PendingCertificateReader + 'static {
    fn get_certificate_async(
        self: &Arc<Self>,
        network_id: NetworkId,
        height: Height,
    ) -> impl Future<Output = Result<Option<Certificate>, Error>> + Send + 'static {
        let store = Arc::clone(self);
        async move {
            tokio::task::spawn_blocking(move || {
                PendingCertificateReader::get_certificate(store.as_ref(), network_id, height)
            })
            .await
            .expect("pending certificate read task panicked")
        }
    }

    fn get_proof_async(
        self: &Arc<Self>,
        certificate_id: CertificateId,
    ) -> impl Future<Output = Result<Option<Proof>, Error>> + Send + 'static {
        let store = Arc::clone(self);
        async move {
            tokio::task::spawn_blocking(move || {
                PendingCertificateReader::get_proof(store.as_ref(), certificate_id)
            })
            .await
            .expect("pending proof read task panicked")
        }
    }

    fn get_current_proven_height_async(
        self: &Arc<Self>,
    ) -> impl Future<Output = Result<Vec<ProvenCertificate>, Error>> + Send + 'static {
        let store = Arc::clone(self);
        async move {
            tokio::task::spawn_blocking(move || {
                PendingCertificateReader::get_current_proven_height(store.as_ref())
            })
            .await
            .expect("proven certificate scan task panicked")
        }
    }
}

impl<S> AsyncPendingCertificateReaderExt for S where S: PendingCertificateReader + ?Sized + 'static {}

/// Async access to complete pending-store write operations.
pub trait AsyncPendingCertificateWriterExt: PendingCertificateWriter + 'static {
    fn insert_generated_proof_async(
        self: &Arc<Self>,
        certificate_id: CertificateId,
        proof: Proof,
    ) -> impl Future<Output = Result<(), Error>> + Send + 'static {
        let store = Arc::clone(self);
        async move {
            tokio::task::spawn_blocking(move || {
                PendingCertificateWriter::insert_generated_proof(
                    store.as_ref(),
                    &certificate_id,
                    &proof,
                )
            })
            .await
            .expect("proof write task panicked")
        }
    }

    fn remove_generated_proof_async(
        self: &Arc<Self>,
        certificate_id: CertificateId,
    ) -> impl Future<Output = Result<(), Error>> + Send + 'static {
        let store = Arc::clone(self);
        async move {
            tokio::task::spawn_blocking(move || {
                PendingCertificateWriter::remove_generated_proof(store.as_ref(), &certificate_id)
            })
            .await
            .expect("proof removal task panicked")
        }
    }

    fn set_latest_proven_certificate_per_network_async(
        self: &Arc<Self>,
        network_id: NetworkId,
        height: Height,
        certificate_id: CertificateId,
    ) -> impl Future<Output = Result<(), Error>> + Send + 'static {
        let store = Arc::clone(self);
        async move {
            tokio::task::spawn_blocking(move || {
                PendingCertificateWriter::set_latest_proven_certificate_per_network(
                    store.as_ref(),
                    &network_id,
                    &height,
                    &certificate_id,
                )
            })
            .await
            .expect("latest proven certificate write task panicked")
        }
    }
}

impl<S> AsyncPendingCertificateWriterExt for S where S: PendingCertificateWriter + ?Sized + 'static {}

/// Async access to complete state-store read operations.
pub trait AsyncStateReaderExt: StateReader + 'static {
    fn get_certificate_header_async(
        self: &Arc<Self>,
        certificate_id: CertificateId,
    ) -> impl Future<Output = Result<Option<CertificateHeader>, Error>> + Send + 'static {
        let store = Arc::clone(self);
        async move {
            tokio::task::spawn_blocking(move || {
                StateReader::get_certificate_header(store.as_ref(), &certificate_id)
            })
            .await
            .expect("certificate header read task panicked")
        }
    }

    fn get_certificate_settlement_job_id_async(
        self: &Arc<Self>,
        certificate_id: CertificateId,
    ) -> impl Future<Output = Result<Option<SettlementJobId>, Error>> + Send + 'static {
        let store = Arc::clone(self);
        async move {
            tokio::task::spawn_blocking(move || {
                StateReader::get_certificate_settlement_job_id(store.as_ref(), &certificate_id)
            })
            .await
            .expect("settlement job id read task panicked")
        }
    }

    fn get_disabled_networks_async(
        self: &Arc<Self>,
    ) -> impl Future<Output = Result<Vec<NetworkId>, Error>> + Send + 'static {
        let store = Arc::clone(self);
        async move {
            tokio::task::spawn_blocking(move || StateReader::get_disabled_networks(store.as_ref()))
                .await
                .expect("disabled network scan task panicked")
        }
    }
}

impl<S> AsyncStateReaderExt for S where S: StateReader + ?Sized + 'static {}

/// Async access to complete state-store write operations.
pub trait AsyncStateWriterExt: StateWriter + 'static {
    fn update_settlement_tx_hash_async(
        self: &Arc<Self>,
        certificate_id: CertificateId,
        tx_hash: SettlementTxHash,
        force: UpdateEvenIfAlreadyPresent,
        set_status: UpdateStatusToCandidate,
    ) -> impl Future<Output = Result<(), Error>> + Send + 'static {
        let store = Arc::clone(self);
        async move {
            tokio::task::spawn_blocking(move || {
                StateWriter::update_settlement_tx_hash(
                    store.as_ref(),
                    &certificate_id,
                    tx_hash,
                    force,
                    set_status,
                )
            })
            .await
            .expect("settlement transaction hash write task panicked")
        }
    }

    fn update_certificate_header_status_async(
        self: &Arc<Self>,
        certificate_id: CertificateId,
        status: CertificateStatus,
    ) -> impl Future<Output = Result<(), Error>> + Send + 'static {
        let store = Arc::clone(self);
        async move {
            tokio::task::spawn_blocking(move || {
                StateWriter::update_certificate_header_status(
                    store.as_ref(),
                    &certificate_id,
                    &status,
                )
            })
            .await
            .expect("certificate status write task panicked")
        }
    }

    fn disable_network_async(
        self: &Arc<Self>,
        network_id: NetworkId,
        disabled_by: DisabledBy,
    ) -> impl Future<Output = Result<(), Error>> + Send + 'static {
        let store = Arc::clone(self);
        async move {
            tokio::task::spawn_blocking(move || {
                StateWriter::disable_network(store.as_ref(), &network_id, disabled_by)
            })
            .await
            .expect("network disable task panicked")
        }
    }

    fn enable_network_async(
        self: &Arc<Self>,
        network_id: NetworkId,
    ) -> impl Future<Output = Result<(), Error>> + Send + 'static {
        let store = Arc::clone(self);
        async move {
            tokio::task::spawn_blocking(move || {
                StateWriter::enable_network(store.as_ref(), &network_id)
            })
            .await
            .expect("network enable task panicked")
        }
    }
}

impl<S> AsyncStateWriterExt for S where S: StateWriter + ?Sized + 'static {}

/// Async access to complete settlement-store read operations.
pub trait AsyncSettlementReaderExt: SettlementReader + 'static {
    fn list_settlement_job_ids_async(
        self: &Arc<Self>,
    ) -> impl Future<Output = Result<Vec<SettlementJobId>, Error>> + Send + 'static {
        let store = Arc::clone(self);
        async move {
            tokio::task::spawn_blocking(move || {
                SettlementReader::list_settlement_job_ids(store.as_ref())
            })
            .await
            .expect("settlement job scan task panicked")
        }
    }

    fn max_settlement_nonce_for_wallet_async(
        self: &Arc<Self>,
        wallet: Address,
    ) -> impl Future<Output = Result<Option<Nonce>, Error>> + Send + 'static {
        let store = Arc::clone(self);
        async move {
            tokio::task::spawn_blocking(move || {
                SettlementReader::max_settlement_nonce_for_wallet(store.as_ref(), wallet)
            })
            .await
            .expect("settlement nonce lookup task panicked")
        }
    }
}

impl<S> AsyncSettlementReaderExt for S where S: SettlementReader + ?Sized + 'static {}

/// Async access to complete settlement-store write operations.
///
/// The synchronous store retains every per-job lock, check/write sequence,
/// iterator, and RocksDB batch for the full duration of each blocking job.
pub trait AsyncSettlementWriterExt: SettlementWriter + 'static {
    fn insert_settlement_job_async(
        self: &Arc<Self>,
        settlement_job_id: SettlementJobId,
        settlement_job: SettlementJob,
        certificate_id: Option<CertificateId>,
    ) -> impl Future<Output = Result<SettlementJob, Error>> + Send + 'static {
        let store = Arc::clone(self);
        async move {
            tokio::task::spawn_blocking(move || {
                match certificate_id {
                    Some(certificate_id) => {
                        SettlementWriter::insert_settlement_job_with_certificate(
                            store.as_ref(),
                            &settlement_job_id,
                            &settlement_job,
                            &certificate_id,
                        )
                    }
                    None => SettlementWriter::insert_settlement_job(
                        store.as_ref(),
                        &settlement_job_id,
                        &settlement_job,
                    ),
                }
                .map(|()| settlement_job)
            })
            .await
            .expect("settlement job insert task panicked")
        }
    }

    fn insert_settlement_attempt_async(
        self: &Arc<Self>,
        settlement_job_id: SettlementJobId,
        attempt_sequence_number: u64,
        settlement_attempt: SettlementAttempt,
    ) -> impl Future<Output = Result<SettlementAttempt, Error>> + Send + 'static {
        let store = Arc::clone(self);
        async move {
            tokio::task::spawn_blocking(move || {
                SettlementWriter::insert_settlement_attempt(
                    store.as_ref(),
                    &settlement_job_id,
                    attempt_sequence_number,
                    &settlement_attempt,
                )
                .map(|()| settlement_attempt)
            })
            .await
            .expect("settlement attempt insert task panicked")
        }
    }

    fn insert_settlement_job_result_async(
        self: &Arc<Self>,
        settlement_job_id: SettlementJobId,
        result: SettlementJobResult,
    ) -> impl Future<Output = Result<SettlementJobResult, Error>> + Send + 'static {
        let store = Arc::clone(self);
        async move {
            tokio::task::spawn_blocking(move || {
                SettlementWriter::insert_settlement_job_result(
                    store.as_ref(),
                    &settlement_job_id,
                    &result,
                )
                .map(|()| result)
            })
            .await
            .expect("settlement job result insert task panicked")
        }
    }

    fn record_settlement_attempt_result_async(
        self: &Arc<Self>,
        settlement_job_id: SettlementJobId,
        attempt_sequence_number: u64,
        result: SettlementAttemptResult,
    ) -> impl Future<Output = Result<SettlementAttemptResult, Error>> + Send + 'static {
        let store = Arc::clone(self);
        async move {
            tokio::task::spawn_blocking(move || {
                SettlementWriter::record_settlement_attempt_result(
                    store.as_ref(),
                    &settlement_job_id,
                    attempt_sequence_number,
                    &result,
                )
                .map(|()| result)
            })
            .await
            .expect("settlement attempt result write task panicked")
        }
    }

    fn admin_insert_settlement_attempt_async(
        self: &Arc<Self>,
        settlement_job_id: SettlementJobId,
        settlement_attempt: SettlementAttempt,
        edit_even_if_completed: EditEvenIfCompleted,
    ) -> impl Future<Output = Result<u64, Error>> + Send + 'static {
        let store = Arc::clone(self);
        async move {
            tokio::task::spawn_blocking(move || {
                SettlementWriter::admin_insert_settlement_attempt(
                    store.as_ref(),
                    &settlement_job_id,
                    &settlement_attempt,
                    edit_even_if_completed,
                )
            })
            .await
            .expect("admin settlement attempt insert task panicked")
        }
    }

    fn admin_override_settlement_attempt_result_async(
        self: &Arc<Self>,
        settlement_job_id: SettlementJobId,
        attempt_number: u64,
        result: SettlementAttemptResult,
        edit_even_if_completed: EditEvenIfCompleted,
    ) -> impl Future<Output = Result<(), Error>> + Send + 'static {
        let store = Arc::clone(self);
        async move {
            tokio::task::spawn_blocking(move || {
                SettlementWriter::admin_override_settlement_attempt_result(
                    store.as_ref(),
                    &settlement_job_id,
                    attempt_number,
                    &result,
                    edit_even_if_completed,
                )
            })
            .await
            .expect("admin settlement attempt result override task panicked")
        }
    }

    fn admin_remove_settlement_attempt_result_async(
        self: &Arc<Self>,
        settlement_job_id: SettlementJobId,
        attempt_number: u64,
        edit_even_if_completed: EditEvenIfCompleted,
    ) -> impl Future<Output = Result<(), Error>> + Send + 'static {
        let store = Arc::clone(self);
        async move {
            tokio::task::spawn_blocking(move || {
                SettlementWriter::admin_remove_settlement_attempt_result(
                    store.as_ref(),
                    &settlement_job_id,
                    attempt_number,
                    edit_even_if_completed,
                )
            })
            .await
            .expect("admin settlement attempt result removal task panicked")
        }
    }

    fn admin_force_remove_settlement_job_result_async(
        self: &Arc<Self>,
        settlement_job_id: SettlementJobId,
    ) -> impl Future<Output = Result<(), Error>> + Send + 'static {
        let store = Arc::clone(self);
        async move {
            tokio::task::spawn_blocking(move || {
                SettlementWriter::admin_force_remove_settlement_job_result(
                    store.as_ref(),
                    &settlement_job_id,
                )
            })
            .await
            .expect("admin settlement job result removal task panicked")
        }
    }
}

impl<S> AsyncSettlementWriterExt for S where S: SettlementWriter + ?Sized + 'static {}

#[cfg(all(test, feature = "testutils"))]
mod tests {
    use std::sync::Arc;

    use agglayer_types::{Height, NetworkId};

    use super::AsyncPendingCertificateReaderExt as _;
    use crate::tests::mocks::MockPendingStore;

    #[tokio::test(flavor = "current_thread")]
    async fn async_extension_runs_the_store_call_off_the_runtime_worker() {
        let runtime_thread = std::thread::current().id();
        let mut store = MockPendingStore::new();
        store
            .expect_get_certificate()
            .once()
            .returning(move |_, _| {
                assert_ne!(std::thread::current().id(), runtime_thread);
                Ok(None)
            });

        Arc::new(store)
            .get_certificate_async(NetworkId::new(1), Height::ZERO)
            .await
            .expect("mocked certificate read should succeed");
    }
}
