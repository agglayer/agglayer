use std::sync::Arc;

use agglayer_settlement_service::{
    ContractCallOutcome, RetrievedSettlementResult, SettlementJobResult, SettlementServiceTrait,
};
use agglayer_storage::stores::{
    PendingCertificateReader, PendingCertificateWriter, StateReader, StateWriter,
    UpdateEvenIfAlreadyPresent, UpdateStatusToCandidate,
};
#[cfg(feature = "testutils")]
use agglayer_types::SettlementTxHash;
use agglayer_types::{
    Certificate, CertificateHeader, CertificateStatus, CertificateStatusError, Digest,
};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, instrument, trace, warn};

use crate::{network_task::NetworkTaskMessage, Certifier, Error};

/// A task that processes a certificate, including certifying it and settling
/// it.
///
/// Once the `process` function is called, this task will handle everything
/// related to the certificate until it gets finalized, including exchanging the
/// required messages with the network task to both get required information
/// from it and notify it of certificate progress.
pub struct CertificateTask<StateStore, PendingStore, CertifierClient, SettlementSvc>
where
    SettlementSvc: SettlementServiceTrait,
{
    certificate: Certificate,
    header: CertificateHeader,

    network_task: mpsc::Sender<NetworkTaskMessage>,
    state_store: Arc<StateStore>,
    pending_store: Arc<PendingStore>,
    certifier_client: Arc<CertifierClient>,
    cancellation_token: CancellationToken,
    new_pp_root: Option<Digest>,
    settlement_service: Arc<SettlementSvc>,
}

impl<StateStore, PendingStore, CertifierClient, SettlementSvc>
    CertificateTask<StateStore, PendingStore, CertifierClient, SettlementSvc>
where
    StateStore: StateReader + StateWriter,
    PendingStore: PendingCertificateReader + PendingCertificateWriter,
    CertifierClient: Certifier,
    SettlementSvc: SettlementServiceTrait,
{
    #[instrument(skip_all, fields(certificate_id))]
    pub fn new(
        certificate: Certificate,
        network_task: mpsc::Sender<NetworkTaskMessage>,
        state_store: Arc<StateStore>,
        pending_store: Arc<PendingStore>,
        certifier_client: Arc<CertifierClient>,
        cancellation_token: CancellationToken,
        settlement_service: Arc<SettlementSvc>,
    ) -> Result<Self, Error> {
        let certificate_id = certificate.hash();
        tracing::Span::current().record("certificate_id", certificate_id.to_string());
        let Some(header) = state_store.get_certificate_header(&certificate_id)? else {
            error!("Certificate header not found");

            return Err(Error::InternalError(format!(
                "Certificate header not found for {certificate_id}"
            )));
        };

        Ok(Self {
            certificate,
            header,
            network_task,
            state_store,
            pending_store,
            certifier_client,
            cancellation_token,
            new_pp_root: None,
            settlement_service,
        })
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
                    let error = eyre::Error::from(error.clone());
                    debug!(?error, "Error in certificate processing");
                }
            }

            // Then record it to the database
            if let Err(error) = self.state_store.update_certificate_header_status(
                &self.header.certificate_id,
                &CertificateStatus::error(error.clone()),
            ) {
                error!(?error, "Failed to update certificate status in database");
            };

            self.send_to_network_task(NetworkTaskMessage::CertificateErrored {
                height: self.header.height,
                certificate_id: self.header.certificate_id,
                error,
            })
            .await
            .unwrap_or_else(|error| {
                error!(?error, "Failed to send certificate error message");
            });
        }
    }

    /// Process a certificate, not doing any specific error handling except for
    /// returning it
    async fn process_impl(&mut self) -> Result<(), CertificateStatusError> {
        let certificate_id = self.header.certificate_id;

        // TODO: when all the storage related to this cert is only ever handled from the
        // certificate task, the certificate task should be the one to start
        // with storing the certificate if needed.

        debug!(initial_status = ?self.header.status, "Processing certificate");

        // TODO: Hack to deal with Proven certificates in case the PP changed.
        // See https://github.com/agglayer/agglayer/pull/819#discussion_r2152193517 for the details
        // Note that we still have the problem, this is here only to mitigate a bit the
        // issue. When we finally do the storage refactoring, we should remove
        // this.
        if self.header.status == CertificateStatus::Proven {
            warn!(
                "Certificate is already proven but we do not have the new_state anymore... \
                 reproving"
            );

            self.state_store
                .update_certificate_header_status(&certificate_id, &CertificateStatus::Pending)?;
            self.header.status = CertificateStatus::Pending;
            self.pending_store.remove_generated_proof(&certificate_id)?;
        }

        match &self.header.status {
            CertificateStatus::Pending => self.process_from_pending().await,
            CertificateStatus::Proven => {
                self.recompute_state().await?;
                self.process_from_proven().await
            }
            CertificateStatus::Candidate => {
                self.recompute_state().await?;
                self.process_from_candidate().await
            }
            CertificateStatus::Settled => {
                warn!("Built a CertificateTask for a certificate that is already settled");
                Ok(())
            }
            CertificateStatus::InError { error } => {
                warn!(error = ?eyre::Error::from(error.clone()), "Certificate is already in error");
                Err(*error.clone())
            }
        }
    }

    async fn recompute_state(&mut self) -> Result<(), CertificateStatusError> {
        // TODO: once we store network_id -> height -> state and not just network_id ->
        // state, we should not need this any longer, because the state will
        // already be recorded.

        let height = self.header.height;
        let certificate_id = self.header.certificate_id;

        // Retrieve local network state
        trace!("Retrieving local network state");
        let (response, state) = oneshot::channel();
        self.send_to_network_task(NetworkTaskMessage::GetLocalNetworkStateBeforeHeight {
            height,
            response,
        })
        .await?;
        let mut state = state.await.map_err(recv_err)??;

        debug!("Recomputing new state for already-proven certificate");

        // Execute the witness generation to retrieve the new local network state
        let (_, _, output) = self
            .certifier_client
            .witness_generation(
                &self.certificate,
                &mut state,
                self.header.settlement_tx_hash.map(Digest::from),
            )
            .await
            .map_err(|error| {
                error!(
                    ?error,
                    "Failed recomputing the new state for already-proven certificate"
                );
                error
            })?;
        debug!("Recomputing new state completed");

        self.new_pp_root = Some(output.new_pessimistic_root);
        // Send the new state to the network task
        // TODO: Once we update the storage we'll have to remove this! It wouldn't be
        // valid if we had multiple certificates inflight. Thankfully, until
        // we update the storage we cannot have multiple certificates
        // inflight, so we should be fine until then.
        self.send_to_network_task(NetworkTaskMessage::CertificateExecuted {
            height,
            certificate_id,
            new_state: state,
        })
        .await?;

        Ok(())
    }

    async fn process_from_pending(&mut self) -> Result<(), CertificateStatusError> {
        if self.header.status != CertificateStatus::Pending {
            return Err(CertificateStatusError::InternalError(format!(
                "CertificateTask::process_from_pending called with cert status {}",
                self.header.status,
            )));
        }

        let height = self.header.height;
        let network_id = self.header.network_id;
        let certificate_id = self.header.certificate_id;

        // Retrieve local network state
        trace!("Retrieving local network state");
        let (response, state) = oneshot::channel();
        self.send_to_network_task(NetworkTaskMessage::GetLocalNetworkStateBeforeHeight {
            height,
            response,
        })
        .await?;
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
        self.new_pp_root = Some(certifier_output.new_pp_root);
        self.send_to_network_task(NetworkTaskMessage::CertificateExecuted {
            height,
            certificate_id,
            new_state: Box::new(certifier_output.new_state),
        })
        .await?;
        self.send_to_network_task(NetworkTaskMessage::CertificateProven {
            height,
            certificate_id,
        })
        .await?;

        self.process_from_proven().await
    }

    async fn process_from_proven(&mut self) -> Result<(), CertificateStatusError> {
        if self.header.status != CertificateStatus::Proven {
            return Err(CertificateStatusError::InternalError(format!(
                "CertificateTask::process_from_proven called with cert status {}",
                self.header.status,
            )));
        }

        let certificate_id = self.header.certificate_id;

        // Submit settlement request
        let mut watcher = self
            .settlement_service
            .request_new_settlement(certificate_id)
            .await
            .map_err(|e| {
                CertificateStatusError::SettlementError(format!("Failed to submit: {e}"))
            })?;

        let job_id = watcher.job_id();
        info!("Settlement job {} submitted", job_id);

        // Wait for result — the settlement service owns its own timeouts
        let result = watcher
            .wait_for_result()
            .await
            .map_err(|e| CertificateStatusError::SettlementError(e.to_string()))?;

        match result {
            SettlementJobResult::ContractCall(contract_result)
                if contract_result.outcome == ContractCallOutcome::Revert =>
            {
                Err(CertificateStatusError::SettlementError(format!(
                    "Settlement tx {} reverted",
                    contract_result.tx_hash
                )))
            }
            SettlementJobResult::ContractCall(contract_result) => {
                let tx_hash = contract_result.tx_hash;
                info!("Settlement successful: {}", tx_hash);

                self.header.settlement_tx_hash = Some(tx_hash);
                self.state_store.update_settlement_tx_hash(
                    &certificate_id,
                    tx_hash,
                    UpdateEvenIfAlreadyPresent::Yes,
                    UpdateStatusToCandidate::Yes,
                )?;

                self.header.settlement_job_id = Some(job_id.into());
                self.state_store
                    .update_settlement_job_id(&certificate_id, job_id.into())?;

                self.header.status = CertificateStatus::Candidate;

                #[cfg(feature = "testutils")]
                testutils::inject_fail_points_after_proving(
                    &certificate_id,
                    &mut self.header,
                    &self.state_store,
                );

                self.process_from_candidate().await
            }
            SettlementJobResult::ClientError(error) => {
                error!("Settlement failed: {}", error.message);
                Err(CertificateStatusError::SettlementError(error.message))
            }
        }
    }

    async fn process_from_candidate(&mut self) -> Result<(), CertificateStatusError> {
        if self.header.status != CertificateStatus::Candidate {
            return Err(CertificateStatusError::InternalError(format!(
                "CertificateTask::process_from_candidate called with cert status {}",
                self.header.status,
            )));
        }

        let certificate_id = self.header.certificate_id;
        let job_id = self
            .header
            .settlement_job_id
            .ok_or_else(|| CertificateStatusError::SettlementError("No settlement job id".into()))?
            .as_ulid();

        match self
            .settlement_service
            .retrieve_settlement_result(job_id)
            .await
            .map_err(|e| CertificateStatusError::SettlementError(format!("{e}")))?
        {
            RetrievedSettlementResult::Pending(mut watcher) => {
                let result = watcher
                    .wait_for_result()
                    .await
                    .map_err(|e| CertificateStatusError::SettlementError(e.to_string()))?;
                validate_settlement_result(&result)?;
                info!(
                    "Certificate {certificate_id} settlement job {job_id} completed \
                     (pending→completed)",
                );
            }
            RetrievedSettlementResult::Completed(result) => {
                validate_settlement_result(&result)?;
                info!("Certificate {certificate_id} settlement job {job_id} completed");
            }
            RetrievedSettlementResult::NotFound => {
                return Err(CertificateStatusError::SettlementError(format!(
                    "Certificate {certificate_id} settlement job {job_id} not found, unable to \
                     recover",
                )));
            }
        }

        self.finalize_settlement().await
    }

    fn set_status(&mut self, status: CertificateStatus) -> Result<(), CertificateStatusError> {
        self.state_store
            .update_certificate_header_status(&self.header.certificate_id, &status)?;
        self.header.status = status;
        Ok(())
    }

    /// Common finalization logic for all settlement completion paths.
    async fn finalize_settlement(&mut self) -> Result<(), CertificateStatusError> {
        self.set_status(CertificateStatus::Settled)?;
        self.send_to_network_task(NetworkTaskMessage::CertificateSettled {
            height: self.header.height,
            certificate_id: self.header.certificate_id,
        })
        .await
    }

    async fn send_to_network_task(
        &self,
        message: NetworkTaskMessage,
    ) -> Result<(), CertificateStatusError> {
        trace!(?message, "Sending message to network task");
        self.network_task.send(message).await.map_err(send_err)
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

fn validate_settlement_result(result: &SettlementJobResult) -> Result<(), CertificateStatusError> {
    match result {
        SettlementJobResult::ClientError(error) => Err(CertificateStatusError::SettlementError(
            error.message.clone(),
        )),
        SettlementJobResult::ContractCall(call) if call.outcome == ContractCallOutcome::Revert => {
            Err(CertificateStatusError::SettlementError(format!(
                "Settlement tx {} reverted",
                call.tx_hash
            )))
        }
        SettlementJobResult::ContractCall(_) => Ok(()),
    }
}

#[cfg(feature = "testutils")]
mod testutils {
    use agglayer_storage::stores::{UpdateEvenIfAlreadyPresent, UpdateStatusToCandidate};

    use super::*;

    pub(crate) fn inject_fail_points_after_proving<StateStore: StateWriter>(
        certificate_id: &agglayer_types::CertificateId,
        header: &mut CertificateHeader,
        state_store: &Arc<StateStore>,
    ) {
        // Fail point to inject invalid settlement tx hash
        fail::eval(
            "certificate_task::process_impl::invalid_settlement_tx_hash",
            |_| {
                // Write an unexistent tx hash to simulate the settlement tx not being found on
                // L1
                warn!("FAIL POINT ACTIVE: Injecting invalid settlement tx hash");
                let unexistent_tx_hash = SettlementTxHash::new(Digest::from([21u8; 32]));
                header.settlement_tx_hash = Some(unexistent_tx_hash);
                state_store
                    .update_settlement_tx_hash(
                        certificate_id,
                        unexistent_tx_hash,
                        UpdateEvenIfAlreadyPresent::Yes,
                        UpdateStatusToCandidate::Yes,
                    )
                    .expect("Valid tx hash update");
                Some(())
            },
        );

        // Fail point to record candidate and potentially shutdown
        fail::fail_point!("certificate_task::process_impl::candidate_recorded");
    }
}
