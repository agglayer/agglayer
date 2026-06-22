use std::sync::Arc;

use agglayer_config::settlement_service::SettlementTransactionConfig;
use agglayer_settlement_service::SettlementServiceTrait;
use agglayer_storage::stores::{
    PendingCertificateReader, PendingCertificateWriter, StateReader, StateWriter,
    UpdateEvenIfAlreadyPresent, UpdateStatusToCandidate,
};
#[cfg(feature = "testutils")]
use agglayer_types::SettlementTxHash;
use agglayer_types::{
    Address, Certificate, CertificateHeader, CertificateStatus, CertificateStatusError,
    ContractCallOutcome, Digest, SettlementJob, SettlementJobResult, U256,
};
use alloy::primitives::Bytes;
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
pub struct CertificateTask<StateStore, PendingStore, CertifierClient, SettlementSvc> {
    certificate: Certificate,
    header: CertificateHeader,

    network_task: mpsc::Sender<NetworkTaskMessage>,
    state_store: Arc<StateStore>,
    pending_store: Arc<PendingStore>,
    certifier_client: Arc<CertifierClient>,
    cancellation_token: CancellationToken,
    settlement_service: Arc<SettlementSvc>,
    settlement_config: Arc<SettlementTransactionConfig>,
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
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        certificate: Certificate,
        network_task: mpsc::Sender<NetworkTaskMessage>,
        state_store: Arc<StateStore>,
        pending_store: Arc<PendingStore>,
        certifier_client: Arc<CertifierClient>,
        settlement_service: Arc<SettlementSvc>,
        settlement_config: Arc<SettlementTransactionConfig>,
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
            settlement_config,
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
        let (_, _, _output) = self
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

        // Record the certification success
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

        // Hand settlement off to the settlement service and remember the job id.
        //
        // The settlement service is agnostic of certificates, so the
        // certificate <-> settlement-job link lives here. We persist it and move
        // to Candidate *before* waiting for the result, so that a reboot resumes
        // from `process_from_candidate` using the stored job id.
        let job = self.build_settlement_job();
        let job_id = self
            .settlement_service
            .submit_settlement_job(job)
            .await
            .map_err(|error| {
                CertificateStatusError::SettlementError(format!(
                    "Failed to submit settlement job: {error}"
                ))
            })?;
        info!(%job_id, "Settlement job submitted");

        self.state_store
            .insert_certificate_settlement_job_id(&certificate_id, &job_id)?;
        self.set_status(CertificateStatus::Candidate)?;

        #[cfg(feature = "testutils")]
        testutils::inject_fail_points_after_proving(
            &certificate_id,
            &mut self.header,
            &self.state_store,
        );

        self.process_from_candidate().await
    }

    /// Build the settlement job handed to the settlement service.
    ///
    /// TODO: build the real `verifyPessimisticTrustedAggregator` calldata from
    /// the certificate proof. For now the job carries placeholder call data;
    /// the node wires an inert settlement service so this is never
    /// submitted to L1. Real call data (and constructing the production
    /// settlement service) is a follow-up.
    fn build_settlement_job(&self) -> SettlementJob {
        SettlementJob {
            contract_address: Address::ZERO,
            calldata: Bytes::new(),
            eth_value: U256::ZERO,
            gas_limit: self
                .settlement_config
                .gas_limit_ceiling
                .saturating_to::<u128>(),
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
