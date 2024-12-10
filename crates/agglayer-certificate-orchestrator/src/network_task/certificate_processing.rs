use std::{future::Future, pin::pin, task::Poll};

use agglayer_storage::columns::latest_proven_certificate_per_network::ProvenCertificate;
use agglayer_types::Certificate;
use tokio::{sync::oneshot, task::JoinHandle};

use super::{
    CertificateId, CertificationError, CertificationNotifier, CertifierResult, SettlementResult,
};

/// Manages the jobs associated with certificate processing.
///
/// The manager takes care that only one proving or settling job is active at
/// any given time. Attempting to spawn a job while one is running will fail.
pub enum Manager {
    Idle,
    Proving(ProverJob),
    Settling(SettlementJob),
}

impl Manager {
    /// New certifier task manager.
    pub fn new() -> Self {
        Self::Idle
    }

    /// Check if a certification job is running.
    pub fn is_running(&self) -> bool {
        match self {
            Manager::Idle => false,
            Manager::Proving(_) | Manager::Settling(_) => true,
        }
    }

    /// Start the proving/certification process.
    pub fn start_proving(
        &mut self,
        certificate_id: CertificateId,
        task: impl Future<Output = CertifierResult> + Send + 'static,
    ) -> Result<(), CertificationError> {
        self.start_with(move || {
            Ok(Self::Proving(ProverJob {
                task_handle: tokio::spawn(task),
                certificate_id,
            }))
        })
    }

    /// Start the settlement process.
    pub fn start_settlement(
        &mut self,
        certificate: Certificate,
        sender: &CertificationNotifier,
    ) -> Result<(), CertificationError> {
        self.start_with(move || {
            let proven_certificate = ProvenCertificate(
                certificate.hash(),
                certificate.network_id,
                certificate.height,
            );

            let (result_tx, result_rx) = oneshot::channel();
            let err_f = |_| CertificationError::InternalError("Settlement channel full".into());
            sender
                .try_send((result_tx, proven_certificate))
                .map_err(err_f)?;

            Ok(Self::Settling(SettlementJob {
                certificate,
                result_rx,
            }))
        })
    }

    fn start_with(
        &mut self,
        start_fn: impl FnOnce() -> Result<Self, CertificationError>,
    ) -> Result<(), CertificationError> {
        match self {
            Self::Idle => {
                *self = start_fn()?;
                Ok(())
            }
            Self::Proving(_) | Self::Settling(_) => Err(CertificationError::InternalError(
                "Certificate processing in progress".into(),
            )),
        }
    }

    /// Wait for a job to finish (pending if no job is running).
    pub fn join(&mut self) -> impl Future<Output = JobResult> + '_ {
        std::future::poll_fn(|cx| self.poll_join(cx))
    }

    /// Implementation details for [Self::join].
    fn poll_join(&mut self, cx: &mut std::task::Context) -> Poll<JobResult> {
        let was_running = self.is_running();
        let this = std::mem::replace(self, Self::Idle);

        let poll = match this {
            Self::Idle => Poll::Pending,

            Self::Proving(mut job) => match pin!(&mut job.task_handle).poll(cx) {
                Poll::Ready(result) => Poll::Ready(JobResult::Certification {
                    certificate_id: job.certificate_id,
                    result: result.unwrap_or_else(|join_error| {
                        let err = format!("Certifier ended abnormally: {join_error}");
                        Err(CertificationError::InternalError(err))
                    }),
                }),
                Poll::Pending => {
                    *self = Self::Proving(job);
                    Poll::Pending
                }
            },

            Self::Settling(mut job) => match pin!(&mut job.result_rx).poll(cx) {
                Poll::Ready(result) => Poll::Ready(JobResult::Settlement {
                    certificate: job.certificate,
                    result: result.unwrap_or_else(|recv_err| {
                        Err(format!("Settlement ended abnormally: {recv_err}"))
                    }),
                }),
                Poll::Pending => {
                    *self = Self::Settling(job);
                    Poll::Pending
                }
            },
        };

        // Sanity checks
        match poll {
            Poll::Ready(_) => debug_assert!(!self.is_running()),
            Poll::Pending => debug_assert_eq!(
                was_running,
                self.is_running(),
                "Pending poll should not change the running state"
            ),
        }

        poll
    }
}

/// Context for a running prover job.
pub struct ProverJob {
    certificate_id: CertificateId,
    task_handle: JoinHandle<CertifierResult>,
}

/// Context for a running settlement job.
pub struct SettlementJob {
    certificate: Certificate,
    result_rx: oneshot::Receiver<SettlementResult>,
}

/// Result of a prover or settlement job.
#[allow(clippy::large_enum_variant)]
pub enum JobResult {
    Certification {
        certificate_id: CertificateId,
        result: CertifierResult,
    },

    Settlement {
        certificate: Certificate,
        result: SettlementResult,
    },
}
