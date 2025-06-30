use std::sync::Arc;

use agglayer_storage::{
    columns::latest_settled_certificate_per_network::SettledCertificate,
    stores::{PendingCertificateReader, PendingCertificateWriter, StateReader, StateWriter},
};
use agglayer_types::{Certificate, CertificateHeader, CertificateStatus, CertificateStatusError};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, trace, warn};

use crate::{network_task::NetworkTaskMessage, Certifier};

/// A task that processes a certificate, including certifying it and settling
/// it.
///
/// Once the `process` function is called, this task will handle everything
/// related to the certificate until it gets finalized, including exchanging the
/// required messages with the network task to both get required information
/// from it and notify it of certificate progress.
pub struct CertificateTask<StateStore, PendingStore, CertifierClient> {
    certificate: Certificate,
    header: CertificateHeader,

    network_task: mpsc::Sender<NetworkTaskMessage>,
    state_store: Arc<StateStore>,
    pending_store: Arc<PendingStore>,
    certifier_client: Arc<CertifierClient>,
    cancellation_token: CancellationToken,
}

impl<StateStore, PendingStore, CertifierClient>
    CertificateTask<StateStore, PendingStore, CertifierClient>
where
    StateStore: StateReader + StateWriter,
    PendingStore: PendingCertificateReader + PendingCertificateWriter,
    CertifierClient: Certifier,
{
    #[allow(clippy::too_many_arguments)] // TODO: should go away with the next few PRs
    pub fn new(
        certificate: Certificate,
        header: CertificateHeader,
        network_task: mpsc::Sender<NetworkTaskMessage>,
        state_store: Arc<StateStore>,
        pending_store: Arc<PendingStore>,
        certifier_client: Arc<CertifierClient>,
        cancellation_token: CancellationToken,
    ) -> Self {
        Self {
            certificate,
            header,
            network_task,
            state_store,
            pending_store,
            certifier_client,
            cancellation_token,
        }
    }

    #[tracing::instrument(
        name = "CertificateTask::process",
        skip_all,
        fields(
            network_id = %self.header.network_id,
            height = self.header.height.as_u64(),
            certificate_id = %self.header.certificate_id,
        )
    )]
    pub async fn process(mut self) {
        if let Err(error) = self.process_impl().await {
            // If requested to cancel, don't do anything — the error could have arisen from
            // a partially-shutdown process.
            if self.cancellation_token.is_cancelled() {
                return;
            }

            // First, log the error
            match &error {
                CertificateStatusError::InternalError(error) => {
                    error!(?error, "Internal error in certificate processing");
                }
                _ => {
                    let error = anyhow::Error::from(error.clone());
                    debug!(?error, "Error in certificate processing");
                }
            }

            // Then record it to the database
            if let Err(error) = self.state_store.update_certificate_header_status(
                &self.header.certificate_id,
                &CertificateStatus::InError {
                    error: error.clone(),
                },
            ) {
                error!(?error, "Failed to update certificate status in database");
            };

            self.network_task
                .send(NetworkTaskMessage::CertificateErrored {
                    height: self.header.height,
                    certificate_id: self.header.certificate_id,
                    error,
                })
                .await
                .map_err(send_err)
                .unwrap_or_else(|error| {
                    error!(?error, "Failed to send certificate error message");
                });
        }
    }

    /// Process a certificate, not doing any specific error handling except for
    /// returning it
    async fn process_impl(&mut self) -> Result<(), CertificateStatusError> {
        let network_id = self.header.network_id;
        let height = self.header.height;
        let certificate_id = self.header.certificate_id;

        // TODO: when all the storage related to this cert is only ever handled from the
        // certificate task, the certificate task should be the one to start
        // with storing the certificate if needed.

        debug!(initial_status = ?self.header.status, "Processing certificate");

        // Skip if there's nothing to do
        if self.header.status == CertificateStatus::Settled {
            warn!("Built a CertificateTask for a certificate that is already settled");
            return Ok(());
        }
        if let CertificateStatus::InError { error } = &self.header.status {
            warn!(error = ?anyhow::Error::from(error.clone()), "Certificate is already in error");
            return Err(error.clone());
        }

        // TODO: Hack to deal with Proven certificates in case the PP changed.
        // See https://github.com/agglayer/agglayer/pull/819#discussion_r2152193517 for the details
        // Note that we still have the problem, this is here only to mitigate a bit the
        // issue When we finally make the storage refactoring, we should remove
        // this
        if self.header.status == CertificateStatus::Proven {
            warn!(
                "Certificate is already proven but we do not have the  new_state anymore... \
                 reproving"
            );

            self.state_store
                .update_certificate_header_status(&certificate_id, &CertificateStatus::Pending)?;
            self.header.status = CertificateStatus::Pending;
            self.pending_store.remove_generated_proof(&certificate_id)?;
        }

        // First, prove the certificate (or recompute just the new state if it's already
        // proven)
        if self.header.status < CertificateStatus::Proven {
            // Retrieve local network state
            trace!("Retrieving local network state");
            let (response, state) = oneshot::channel();
            self.network_task
                .send(NetworkTaskMessage::GetLocalNetworkStateBeforeHeight { height, response })
                .await
                .map_err(send_err)?;
            let state = state.await.map_err(recv_err)??;

            // Actually certify
            debug!("Starting certification");
            let certifier_output = self
                .certifier_client
                .certify(*state, network_id, height)
                .await?;
            debug!("Proof certification completed");

            // Record the certification success
            self.set_status(CertificateStatus::Proven)?;
            self.network_task
                .send(NetworkTaskMessage::CertificateExecuted {
                    height,
                    certificate_id,
                    new_state: Box::new(certifier_output.new_state),
                })
                .await
                .map_err(send_err)?;
            self.network_task
                .send(NetworkTaskMessage::CertificateProven {
                    height,
                    certificate_id,
                })
                .await
                .map_err(send_err)?;
        } else {
            // TODO: once we store network_id -> height -> state and not just network_id ->
            // state, we should not need this any longer, because the state will
            // already be recorded.

            // Retrieve local network state
            trace!("Retrieving local network state");
            let (response, state) = oneshot::channel();
            self.network_task
                .send(NetworkTaskMessage::GetLocalNetworkStateBeforeHeight { height, response })
                .await
                .map_err(send_err)?;
            let mut state = state.await.map_err(recv_err)??;

            // Execute the witness generation to retrieve the new local network state
            debug!("Recomputing new state for already-proven certificate");
            let _ = self
                .certifier_client
                .witness_generation(&self.certificate, &mut state)
                .await
                .map_err(|error| {
                    error!(%certificate_id, ?error, "Failed recomputing the new state for already-proven certificate");
                    error
                })?;
            debug!("Recomputing new state completed");

            // Send the new state to the network task
            // TODO: Once we update the storage we'll have to remove this! It wouldn't be
            // valid if we had multiple certificates inflight. Thankfully, until
            // we update the storage we cannot have multiple certificates
            // inflight, so we should be fine until then.
            self.network_task
                .send(NetworkTaskMessage::CertificateExecuted {
                    height,
                    certificate_id,
                    new_state: state,
                })
                .await
                .map_err(send_err)?;
        }

        // Second, submit settlement to L1
        if self.header.status < CertificateStatus::Proven {
            return Err(CertificateStatusError::InternalError(
                "Trying to settle a non-proven certificate".into(),
            ));
        }
        if self.header.status < CertificateStatus::Candidate {
            debug!("Submitting certificate for settlement");
            let (settlement_submitted_notifier, settlement_submitted) = oneshot::channel();
            self.network_task
                .send(NetworkTaskMessage::CertificateReadyForSettlement {
                    height,
                    certificate_id,
                    settlement_submitted_notifier,
                })
                .await
                .map_err(send_err)?;

            let settlement_tx_hash = settlement_submitted.await.map_err(recv_err)??;
            fail::fail_point!("certificate_task::process_impl::about_to_record_candidate");
            self.header.settlement_tx_hash = Some(settlement_tx_hash);
            self.state_store
                .update_settlement_tx_hash(&certificate_id, settlement_tx_hash)?;
            self.header.status = CertificateStatus::Candidate; // No set_status: update_settlement_tx_hash already updates the status in
                                                               // database
            debug!(settlement_tx_hash = ?self.header.settlement_tx_hash, "Submitted certificate for settlement");
        }

        // Third, wait for settlement to complete
        if self.header.status < CertificateStatus::Candidate {
            return Err(CertificateStatusError::InternalError(
                "Trying to wait on a non-settling certificate".into(),
            ));
        }
        if self.header.status < CertificateStatus::Settled {
            debug!(settlement_tx_hash = ?self.header.settlement_tx_hash, "Waiting for certificate settlement to complete");
            let settlement_tx_hash = self.header.settlement_tx_hash.ok_or_else(|| {
                CertificateStatusError::SettlementError(
                    "Candidate certificate header has no settlement tx hash".into(),
                )
            })?;
            let (settlement_complete_notifier, settlement_complete) = oneshot::channel();
            self.network_task
                .send(NetworkTaskMessage::CertificateWaitingForSettlement {
                    height,
                    certificate_id,
                    settlement_tx_hash,
                    settlement_complete_notifier,
                })
                .await
                .map_err(send_err)?;

            let (epoch_number, certificate_index) =
                settlement_complete.await.map_err(recv_err)??;
            let settled_certificate =
                SettledCertificate(certificate_id, height, epoch_number, certificate_index);
            self.set_status(CertificateStatus::Settled)?;
            debug!(
                ?settlement_tx_hash,
                ?settled_certificate,
                "Certificate settlement completed"
            );
            self.network_task
                .send(NetworkTaskMessage::CertificateSettled {
                    height,
                    certificate_id,
                    settled_certificate,
                })
                .await
                .map_err(send_err)?;
        }

        if self.header.status != CertificateStatus::Settled {
            return Err(CertificateStatusError::InternalError(
                "CertificateTask completed with a non-settled certificate".into(),
            ));
        }

        Ok(())
    }

    fn set_status(&mut self, status: CertificateStatus) -> Result<(), CertificateStatusError> {
        self.state_store
            .update_certificate_header_status(&self.header.certificate_id, &status)?;
        self.header.status = status;
        Ok(())
    }
}

fn send_err<T>(_: mpsc::error::SendError<T>) -> CertificateStatusError {
    CertificateStatusError::InternalError("Failed to send network task message: no listener".into())
}

fn recv_err(_: oneshot::error::RecvError) -> CertificateStatusError {
    CertificateStatusError::InternalError(
        "Failed to receive network task answer: sender dropped".into(),
    )
}
