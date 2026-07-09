use std::sync::Arc;

use agglayer_contracts::{
    rollup::VerifierType, settler::verify_pessimistic_trusted_aggregator_calldata,
};
use agglayer_settlement_service::SettlementServiceTrait;
use agglayer_storage::stores::{
    PendingCertificateReader, PendingCertificateWriter, StateReader, StateWriter,
    UpdateEvenIfAlreadyPresent, UpdateStatusToCandidate,
};
use agglayer_telemetry::certificate::{self, CertificateStage};
#[cfg(feature = "testutils")]
use agglayer_types::SettlementTxHash;
use agglayer_types::{
    Certificate, CertificateHeader, CertificateStatus, CertificateStatusError, ContractCallOutcome,
    Digest, Proof, SettlementJob, SettlementJobResult, U256,
};
use pessimistic_proof::{core::PESSIMISTIC_PROOF_PROGRAM_SELECTOR, PessimisticProofOutput};
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
pub struct CertificateTask<StateStore, PendingStore, CertifierClient, SettlementService> {
    certificate: Certificate,
    header: CertificateHeader,

    network_task: mpsc::Sender<NetworkTaskMessage>,
    state_store: Arc<StateStore>,
    pending_store: Arc<PendingStore>,
    certifier_client: Arc<CertifierClient>,
    cancellation_token: CancellationToken,
    settlement_service: Arc<SettlementService>,

    /// Bridging-time timer; `None` for resumed certificates (only fresh
    /// `Pending` -> `Settled` lifecycles are timed).
    bridging_timer: Option<certificate::CertificateTimer>,
}

impl<StateStore, PendingStore, CertifierClient, SettlementService>
    CertificateTask<StateStore, PendingStore, CertifierClient, SettlementService>
where
    StateStore: StateReader + StateWriter,
    PendingStore: PendingCertificateReader + PendingCertificateWriter,
    CertifierClient: Certifier,
    SettlementService: SettlementServiceTrait,
{
    #[instrument(skip_all, fields(certificate_id))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        certificate: Certificate,
        network_task: mpsc::Sender<NetworkTaskMessage>,
        state_store: Arc<StateStore>,
        pending_store: Arc<PendingStore>,
        certifier_client: Arc<CertifierClient>,
        settlement_service: Arc<SettlementService>,
        cancellation_token: CancellationToken,
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
            settlement_service,
            bridging_timer: None,
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

        // Only time fresh certificates: a full Pending -> Settled lifecycle in
        // one process. Resumed certificates leave the timer `None`.
        if self.header.status == CertificateStatus::Pending {
            self.bridging_timer = Some(certificate::CertificateTimer::start(
                self.header.network_id.to_u32(),
            ));
        }

        // TODO: Hack to deal with Proven certificates in case the PP changed.
        // See https://github.com/agglayer/agglayer/pull/819#discussion_r2152193517 for the details
        // Note that we still have the problem, this is here only to mitigate a bit the
        // issue. When we finally do the storage refactoring, we should remove
        // this.
        if self.header.status == CertificateStatus::Proven {
            // A settlement job may already exist for this certificate if a previous
            // run crashed after submitting it but before recording `Candidate`.
            // Resume that job rather than re-proving and re-submitting (which the
            // at-most-once guard rejects), so it recovers instead of erroring.
            if let Some(job_id) = self
                .state_store
                .get_certificate_settlement_job_id(&certificate_id)?
            {
                info!(%job_id, "Proven certificate already has a settlement job; resuming");
                self.set_status(CertificateStatus::Candidate)?;
                return self.process_from_candidate(true).await;
            }

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
                self.recompute_state(self.header.settlement_tx_hash.map(Digest::from))
                    .await?;
                self.process_from_proven().await
            }
            CertificateStatus::Candidate => self.process_from_candidate(true).await,
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

    async fn recompute_state(
        &mut self,
        before_tx: Option<Digest>,
    ) -> Result<(), CertificateStatusError> {
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

        // Recompute the local network state in place; the proof output is unused here.
        self.certifier_client
            .witness_generation(&self.certificate, &mut state, before_tx)
            .await
            .map_err(|error| {
                error!(
                    ?error,
                    "Failed recomputing the new state for already-proven certificate"
                );
                error
            })?;
        debug!("Recomputing new state completed");

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
        self.certify_pending().await?;

        self.process_from_proven().await
    }

    async fn certify_pending(&mut self) -> Result<(), CertificateStatusError> {
        if self.header.status != CertificateStatus::Pending {
            return Err(CertificateStatusError::InternalError(format!(
                "CertificateTask::certify_pending called with cert status {}",
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

        // Certification succeeded: close out the `pending` (proving) stage, then
        // record the new status.
        self.record_stage();
        self.set_status(CertificateStatus::Proven)?;
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

        Ok(())
    }

    async fn process_from_proven(&mut self) -> Result<(), CertificateStatusError> {
        if self.header.status != CertificateStatus::Proven {
            return Err(CertificateStatusError::InternalError(format!(
                "CertificateTask::process_from_proven called with cert status {}",
                self.header.status,
            )));
        }

        let certificate_id = self.header.certificate_id;

        // The settlement service records the certificate -> job-id link
        // atomically when creating the job and rejects duplicates, so the
        // orchestrator does not persist the link itself.
        let job = self.build_settlement_job().await?;
        let job_id = self
            .settlement_service
            .submit_settlement_job(certificate_id, job)
            .await
            .map_err(|error| {
                CertificateStatusError::SettlementError(format!(
                    "Failed to submit settlement job: {error}"
                ))
            })?;
        info!(%job_id, "Settlement job submitted");

        // Test hook: crash after the job + cert->job link are persisted but before
        // `Candidate` is recorded -- the recovery window the resume path above handles.
        #[cfg(feature = "testutils")]
        fail::fail_point!("certificate_task::process_impl::about_to_record_candidate");

        // Close out the `proven` (submission) stage, then record the new status.
        self.record_stage();
        self.set_status(CertificateStatus::Candidate)?;

        #[cfg(feature = "testutils")]
        testutils::inject_fail_points_after_proving(
            &certificate_id,
            &mut self.header,
            &self.state_store,
        );

        self.process_from_candidate(false).await
    }

    /// Build the real `verifyPessimisticTrustedAggregator` settlement calldata
    /// from the certificate's proof, to hand to the settlement service.
    async fn build_settlement_job(&self) -> Result<SettlementJob, CertificateStatusError> {
        let certificate_id = self.header.certificate_id;

        let proof = match self.pending_store.get_proof(certificate_id)? {
            Some(Proof::SP1(proof)) => proof,
            _ => {
                return Err(CertificateStatusError::SettlementError(format!(
                    "No SP1 proof found for certificate {certificate_id}"
                )))
            }
        };
        let output = PessimisticProofOutput::bincode_codec()
            .deserialize::<PessimisticProofOutput>(proof.public_values.as_slice())
            .map_err(|error| {
                CertificateStatusError::SettlementError(format!(
                    "Failed to deserialize proof output: {error}"
                ))
            })?;

        let rollup_id = output.origin_network.to_u32();
        let verifier_type = self
            .certifier_client
            .verifier_type(rollup_id)
            .await
            .map_err(|error| {
                CertificateStatusError::SettlementError(format!(
                    "Failed to resolve verifier type: {error}"
                ))
            })?;

        let proof_bytes = proof.bytes();
        let proof_with_selector: Vec<u8> = match verifier_type {
            VerifierType::Pessimistic => proof_bytes,
            VerifierType::ALGateway => {
                let mut prefixed = PESSIMISTIC_PROOF_PROGRAM_SELECTOR.to_vec();
                prefixed.extend(&proof_bytes);
                prefixed
            }
            VerifierType::StateTransition => {
                return Err(CertificateStatusError::SettlementError(
                    "Unsupported verifier type for settlement".to_string(),
                ))
            }
        };

        let l1_info_tree_leaf_count = self
            .certificate
            .l1_info_tree_leaf_count()
            .unwrap_or_else(|| self.certifier_client.default_l1_info_tree_leaf_count());

        let calldata = verify_pessimistic_trusted_aggregator_calldata(
            rollup_id,
            l1_info_tree_leaf_count,
            *output.new_local_exit_root.as_ref(),
            *output.new_pessimistic_root,
            proof_with_selector.into(),
            self.certificate.custom_chain_data.clone().into(),
        );

        Ok(SettlementJob {
            contract_address: self.certifier_client.rollup_manager_address(),
            calldata,
            eth_value: U256::ZERO,
            // Resolved and capped at the ceiling by the settlement service.
            gas_limit: 0,
        })
    }

    async fn process_from_candidate(
        &mut self,
        recompute_local_state: bool,
    ) -> Result<(), CertificateStatusError> {
        if self.header.status != CertificateStatus::Candidate {
            return Err(CertificateStatusError::InternalError(format!(
                "CertificateTask::process_from_candidate called with cert status {}",
                self.header.status,
            )));
        }

        let certificate_id = self.header.certificate_id;

        // The settlement job id is the only link the orchestrator keeps to the
        // in-flight settlement, and it survives reboots.
        let job_id = self
            .state_store
            .get_certificate_settlement_job_id(&certificate_id)?
            .ok_or_else(|| {
                CertificateStatusError::SettlementError(
                    "Candidate certificate has no settlement job id".into(),
                )
            })?;

        // Wait for the settlement to reach a terminal result. The service owns
        // its own timeouts and reorg handling; here we only react to the outcome.
        let result: SettlementJobResult = self
            .settlement_service
            .wait_for_settlement(job_id)
            .await
            .map_err(|error| CertificateStatusError::SettlementError(error.to_string()))?;

        // Success is the only happy path; revert (and any future outcome) is an
        // error, so we never accidentally settle on a non-success result.
        let contract_call = result.contract_call_result;
        if contract_call.outcome != ContractCallOutcome::Success {
            return Err(CertificateStatusError::SettlementError(format!(
                "Settlement tx {} did not succeed: {:?}",
                contract_call.tx_hash, contract_call.outcome
            )));
        }

        let tx_hash = contract_call.tx_hash;
        info!(%tx_hash, "Settlement successful");

        // On reboot the executed state is lost (only persisted on settlement), so
        // re-derive it from the hash the service actually settled -- not the header's
        // recorded hash, which can be stale after a reboot. Skipped on the live path.
        if recompute_local_state {
            self.recompute_state(Some(Digest::from(tx_hash))).await?;
        }

        // Record the settlement tx hash before marking Settled: the storage
        // layer rejects tx-hash updates on already-settled certificates.
        self.header.settlement_tx_hash = Some(tx_hash);
        self.state_store.update_settlement_tx_hash(
            &certificate_id,
            tx_hash,
            UpdateEvenIfAlreadyPresent::Yes,
            UpdateStatusToCandidate::No,
        )?;

        self.finalize_settlement().await
    }

    fn set_status(&mut self, status: CertificateStatus) -> Result<(), CertificateStatusError> {
        self.state_store
            .update_certificate_header_status(&self.header.certificate_id, &status)?;
        self.header.status = status;
        Ok(())
    }

    /// Records the current stage's duration. Call just before the status
    /// changes.
    fn record_stage(&mut self) {
        if let (Some(timer), Some(stage)) = (
            self.bridging_timer.as_mut(),
            Self::stage_label(&self.header.status),
        ) {
            timer.complete_stage(stage);
        }
    }

    /// Metric `stage` label for a non-terminal status; `None` for terminal
    /// ones.
    fn stage_label(status: &CertificateStatus) -> Option<CertificateStage> {
        match status {
            CertificateStatus::Pending => Some(CertificateStage::Pending),
            CertificateStatus::Proven => Some(CertificateStage::Proven),
            CertificateStatus::Candidate => Some(CertificateStage::Candidate),
            CertificateStatus::Settled | CertificateStatus::InError { .. } => None,
        }
    }

    /// Common finalization logic for all settlement completion paths.
    async fn finalize_settlement(&mut self) -> Result<(), CertificateStatusError> {
        // Close out the `candidate` stage while the status still reflects it.
        self.record_stage();

        // The network task persists `Settled` once the epoch is assigned, so a
        // failed assignment leaves the certificate `Candidate` (recoverable) rather
        // than durably `Settled` with no epoch. Reflect the status in memory only.
        self.header.status = CertificateStatus::Settled;

        // For fresh certificates, record the end-to-end bridging duration.
        if let Some(timer) = &self.bridging_timer {
            timer.complete();
        }

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
