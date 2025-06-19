use std::sync::Arc;

use agglayer_storage::stores::{
    PendingCertificateReader, PendingCertificateWriter, StateReader, StateWriter,
};
use agglayer_types::{Certificate, CertificateHeader, CertificateStatus, CertificateStatusError};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, trace, warn};

use crate::{network_task::NetworkTaskMessage, Certifier, EpochPacker};

/// A task that processes a certificate, including certifying it and settling
/// it.
///
/// Once the `process` function is called, this task will handle everything
/// related to the certificate until it gets finalized, including exchanging the
/// required messages with the network task to both get required information
/// from it and notify it of certificate progress.
pub struct CertificateTask<StateStore, PendingStore, CertifierClient, SettlementClient> {
    certificate: Certificate,
    header: CertificateHeader,

    network_task: mpsc::Sender<NetworkTaskMessage>,
    state_store: Arc<StateStore>,
    pending_store: Arc<PendingStore>,
    certifier_client: Arc<CertifierClient>,
    settlement_client: Arc<SettlementClient>,
}

impl<StateStore, PendingStore, CertifierClient, SettlementClient>
    CertificateTask<StateStore, PendingStore, CertifierClient, SettlementClient>
where
    StateStore: StateReader + StateWriter,
    PendingStore: PendingCertificateReader + PendingCertificateWriter,
    CertifierClient: Certifier,
    SettlementClient: EpochPacker,
{
    pub fn new(
        certificate: Certificate,
        header: CertificateHeader,
        network_task: mpsc::Sender<NetworkTaskMessage>,
        state_store: Arc<StateStore>,
        pending_store: Arc<PendingStore>,
        certifier_client: Arc<CertifierClient>,
        settlement_client: Arc<SettlementClient>,
    ) -> Self {
        Self {
            certificate,
            header,
            network_task,
            state_store,
            pending_store,
            certifier_client,
            settlement_client,
        }
    }

    #[tracing::instrument(
        name = "CertificateTask::process",
        skip_all,
        fields(
            network_id = %self.header.network_id,
            height = self.header.height,
            certificate_id = %self.header.certificate_id,
        )
    )]
    pub async fn process(mut self) {
        if let Err(error) = self.process_impl().await {
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
            let state = state.await.map_err(recv_err)?;

            // Actually certify
            debug!("Starting certification");
            let certifier_output = self
                .certifier_client
                .certify(*state, network_id, height)
                .await?;
            debug!("Proof certification completed");

            // Record the certification success
            self.header.status = CertificateStatus::Proven;
            self.state_store
                .update_certificate_header_status(&certificate_id, &CertificateStatus::Proven)?;
            self.network_task
                .send(NetworkTaskMessage::CertificateProven {
                    height,
                    certificate_id,
                    new_state: Box::new(certifier_output.new_state),
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
            let mut state = state.await.map_err(recv_err)?;

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
                .send(NetworkTaskMessage::CertificateProven {
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
        let settled_certificate = if self.header.status < CertificateStatus::Candidate {
            debug!("Starting certificate settlement");
            let result = self
                .settlement_client
                .settle_certificate(certificate_id)
                .await?;
            self.header.status = CertificateStatus::Settled;
            debug!("Certificate settlement completed");
            // Note: settle_certificate currently updates the certificate status itself.
            // It will go to Candidate upon submission, and then up to Settled when the
            // settlement is confirmed. However, if agglayer shuts down before
            // settlement confirmation, we can still have a certificate with
            // a Candidate status.
            // TODO: all the storage related to certificate should only be touched from this
            // struct, we should refactor
            result.1
        }
        // If we're not in the shortcut path of `settle_certificate`, wait for settlement
        // TODO: once the storage is only ever touched from here, we can make this a regular `if`
        // For now, the only way to reach this stage is to boot with an already-candidate
        // certificate. So we know that `self.header.settlement_tx_hash` is set.
        else if self.header.status < CertificateStatus::Settled {
            debug!("Resuming certificate settlement");
            let tx_hash = self.header.settlement_tx_hash.ok_or_else(|| {
                CertificateStatusError::SettlementError(
                    "Settled certificate header has no settlement tx hash".into(),
                )
            })?;
            if !self
                .settlement_client
                .transaction_exists(tx_hash.0.into())
                .await?
            {
                return Err(CertificateStatusError::SettlementError(
                    "Settlement transaction not found".into(),
                ));
            }
            debug!("Found settlement transaction");
            let result = self
                .settlement_client
                .recover_settlement(tx_hash.0.into(), certificate_id, network_id, height)
                .await?;
            self.header.status = CertificateStatus::Settled;
            debug!("Resumed certificate settlement completed");
            result.1
        } else {
            // The Settled and InError statuses are handled above
            return Err(CertificateStatusError::InternalError(
                "Certificate task reached code expected to be unreachable".into(),
            ));
        };

        if self.header.status != CertificateStatus::Settled {
            return Err(CertificateStatusError::InternalError(
                "CertificateTask completed with a non-settled certificate".into(),
            ));
        }

        // We just finished settling a new certificate, record that
        // TODO: once the storage is only ever touched from here, we can have that part
        // of the last `if`
        self.network_task
            .send(NetworkTaskMessage::CertificateSettled {
                height,
                certificate_id,
                settled_certificate,
            })
            .await
            .map_err(send_err)?;

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
