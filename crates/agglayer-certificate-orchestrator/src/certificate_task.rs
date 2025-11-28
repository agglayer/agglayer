use std::sync::Arc;

use agglayer_storage::{
    columns::latest_settled_certificate_per_network::SettledCertificate,
    stores::{PendingCertificateReader, PendingCertificateWriter, StateReader, StateWriter},
};
use agglayer_types::{
    Certificate, CertificateHeader, CertificateStatus, CertificateStatusError, Digest,
};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, trace, warn};

use crate::{
    network_task::{CertificateSettlementResult, NetworkTaskMessage},
    Certifier, Error, NonceInfo,
};

const MAX_TX_RETRY: usize = 5;

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
    new_pp_root: Option<Digest>,
    nonce_info: Option<NonceInfo>,
}

impl<StateStore, PendingStore, CertifierClient>
    CertificateTask<StateStore, PendingStore, CertifierClient>
where
    StateStore: StateReader + StateWriter,
    PendingStore: PendingCertificateReader + PendingCertificateWriter,
    CertifierClient: Certifier,
{
    pub fn new(
        certificate: Certificate,
        network_task: mpsc::Sender<NetworkTaskMessage>,
        state_store: Arc<StateStore>,
        pending_store: Arc<PendingStore>,
        certifier_client: Arc<CertifierClient>,
        cancellation_token: CancellationToken,
    ) -> Result<Self, Error> {
        let certificate_id = certificate.hash();
        let Some(header) = state_store.get_certificate_header(&certificate_id)? else {
            error!(%certificate_id, "Certificate header not found");

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
            nonce_info: None,
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
            warn!(%certificate_id,
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
        // `settlement_tx_hash_missing_on_l1` is `true` if the settlement tx hash in
        // certificate header is not found on L1.
        let previous_settlement_tx_hash = {
            let mut previous_settlement_tx_hash = None;
            let prev_settlement_tx_hashes = self
                .pending_store
                .get_settlement_tx_hashes_for_certificate(certificate_id)?;
            debug!(
                ?prev_settlement_tx_hashes,
                "About to check previous settlement transactions"
            );

            for previous_tx_hash in prev_settlement_tx_hashes.iter().rev() {
                debug!(
                    %previous_tx_hash,
                    "Checking previous settlement transaction"
                );

                let (request_is_settlement_tx_mined, response_is_settlement_tx_mined) =
                    oneshot::channel();
                self.send_to_network_task(NetworkTaskMessage::CheckSettlementTx {
                    settlement_tx_hash: *previous_tx_hash,
                    certificate_id,
                    tx_mined_notifier: request_is_settlement_tx_mined,
                })
                .await?;

                let result_is_settlement_tx_mined = response_is_settlement_tx_mined
                    .await
                    .map_err(recv_err)?
                    .inspect_err(|error| {
                        // Some error happened while checking the tx receipt on L1
                        warn!(
                            ?error,
                            %previous_tx_hash,
                            "Failed to check settlement tx prior existence on L1",
                        );
                    });
                let missing = match result_is_settlement_tx_mined {
                    Ok(crate::TxReceiptStatus::TxSuccessful)
                    | Ok(crate::TxReceiptStatus::TxFailed) => {
                        false // We have fetched the receipt, tx exists on L1
                    }
                    Ok(crate::TxReceiptStatus::NotFound) => true, // Tx not found on L1
                    Err(_error) => false,                         // On error we do nothing
                };
                if !missing {
                    debug!(%previous_tx_hash, "Found previous settlement tx hash");
                    previous_settlement_tx_hash = Some(*previous_tx_hash);
                    break;
                }
            }

            if previous_settlement_tx_hash.is_none() {
                warn!(
                    ?prev_settlement_tx_hashes,
                    "No previous settlement tx hash found L1"
                );
            }
            previous_settlement_tx_hash
        };

        if previous_settlement_tx_hash.is_none() {
            // If the settlement tx is not found on L1, we need to recover.
            // With the latest pp root from the contract, check maybe if this
            // certificate new pp root is the same as the latest pp root on the chain.
            let (request_latest_contract_pp_root, response_latest_contract_pp_root) =
                oneshot::channel();
            self.send_to_network_task(NetworkTaskMessage::FetchLatestContractPPRoot {
                contract_pp_root_notifier: request_latest_contract_pp_root,
            })
            .await?;
            let result_latest_contract_pp_root =
                response_latest_contract_pp_root.await.map_err(recv_err)?;
            let recomputed_from_contract: Option<Digest> = match result_latest_contract_pp_root {
                Ok(Some((contract_pp_root, contract_settlement_tx_hash))) => {
                    // Try to recompute the state with the latest tx from contract.
                    match self
                        .certifier_client
                        .witness_generation(
                            &self.certificate,
                            &mut state.clone(),
                            Some(contract_settlement_tx_hash.into()),
                        )
                        .await
                    {
                        Ok((_, _, recomputed_output)) => {
                            if contract_pp_root == recomputed_output.new_pessimistic_root {
                                info!(
                                    %contract_settlement_tx_hash,
                                    "Certificate new pp root matches the latest settled pp root \
                                     on L1, updating certificate settlement tx hash to the one in contracts"
                                );
                                let insert_result = self
                                    .pending_store
                                    .insert_settlement_tx_hash_for_certificate(
                                        &certificate_id,
                                        contract_settlement_tx_hash,
                                    );
                                if let Err(error) = insert_result {
                                    error!(
                                        ?error,
                                        "Failed to insert certificate settlement tx hash to DB",
                                    );
                                };
                                // TODO refactor this function to not calculate witness_generation
                                // twice in this function.
                                // As this would be very rare scenario, we can leave it like this
                                // for now.
                                Some(contract_settlement_tx_hash.into())
                            } else {
                                warn!(
                                    certificate_pp_root = %recomputed_output.new_pessimistic_root,
                                    %contract_settlement_tx_hash,
                                    %contract_pp_root,
                                    "Certificate pp root does not match the latest settled pp root \
                                     on L1 contract",
                                );
                                None
                            }
                        }
                        Err(error) => {
                            warn!(
                                %contract_settlement_tx_hash,
                                ?error,
                                "Failed to recompute the state with the latest contract tx"
                            );
                            None
                        }
                    }
                }
                Ok(None) => {
                    warn!("No pp root found on contract");
                    None
                }
                Err(error) => {
                    error!(?error, "Failed to fetch latest pp root from contract");
                    return Err(CertificateStatusError::SettlementError(format!(
                        "Cert settlement tx is missing from the l1, but failed to fetch latest pp \
                         root from contract: {error}"
                    )));
                }
            };

            if recomputed_from_contract.is_none() {
                // Tx not found on L1, and pp root from contract not matching,
                // Make the cert InError and wait for aggkit to resubmit it.
                return Err(CertificateStatusError::SettlementError(
                    "Settlement tx not found on L1 during recompute state".to_string(),
                ));
            }
        }

        // Execute the witness generation to retrieve the new local network state
        let (_, _, output) = self
            .certifier_client
            .witness_generation(
                &self.certificate,
                &mut state,
                previous_settlement_tx_hash.map(Digest::from),
            )
            .await
            .inspect_err(|error| {
                error!(
                    %certificate_id,
                    ?error,
                    "Failed recomputing the new state for already-proven certificate",
                );
            })?;
        debug!("Recomputing new state completed");

        if let Some(settlement_tx_hash) = previous_settlement_tx_hash {
            // The hash is technically already in the storage but re-inserting it will move
            // it to the front of the queue for subsequent processing.
            self.pending_store
                .insert_settlement_tx_hash_for_certificate(&certificate_id, settlement_tx_hash)?;
        }

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

        let height = self.header.height;
        let certificate_id = self.header.certificate_id;

        let mut previous_tx_hashes = self
            .pending_store
            .get_settlement_tx_hashes_for_certificate(certificate_id)?
            .into_vec();

        if previous_tx_hashes.len() > MAX_TX_RETRY {
            error!(
                ?previous_tx_hashes,
                max_retries = MAX_TX_RETRY,
                "Maximum number of settlement tx submission retries has been reached,
                 something is wrong"
            );
            return Err(CertificateStatusError::SettlementError(format!(
                "Too many different settlement transactions submitted for the same certificate: \
                 {previous_tx_hashes:?}",
            )));
        }

        debug!(
            "Submitting certificate for settlement, previous nonce is {:?}",
            self.nonce_info
        );
        let (settlement_submitted_notifier, settlement_submitted) = oneshot::channel();
        self.send_to_network_task(NetworkTaskMessage::CertificateReadyForSettlement {
            height,
            certificate_id,
            nonce_info: self.nonce_info.clone(),
            previous_tx_hashes: previous_tx_hashes.iter().copied().collect(),
            new_pp_root: self
                .new_pp_root
                .ok_or(CertificateStatusError::InternalError(
                    "CertificateTask::process_from_proven called without a pp_root".into(),
                ))?,
            settlement_submitted_notifier,
        })
        .await?;

        let (settlement_tx_hash, nonce_info) = settlement_submitted.await.map_err(recv_err)??;

        if !previous_tx_hashes.contains(&settlement_tx_hash) {
            let _ = self
                .pending_store
                .insert_settlement_tx_hash_for_certificate(&certificate_id, settlement_tx_hash)
                .inspect_err(|error| error!(?error, "Failed to insert settlement tx hash"));
            previous_tx_hashes.push(settlement_tx_hash);
        } else {
            warn!("Resubmitted the same settlement transaction hash {settlement_tx_hash}");
        }

        // Keep the nonce and previous fees for future use (e.g., retries)
        if let Some(nonce_info) = nonce_info {
            debug!("Settlement tx {settlement_tx_hash} submitted with nonce {nonce_info:?}");
            self.nonce_info = Some(nonce_info);
        }

        #[cfg(feature = "testutils")]
        fail::fail_point!("certificate_task::process_impl::about_to_record_candidate");

        self.set_status(CertificateStatus::Candidate)?;

        debug!(%settlement_tx_hash, "Submitted certificate for settlement");

        #[cfg(feature = "testutils")]
        testutils::inject_fail_points_after_proving(
            &certificate_id,
            &mut self.header,
            &*self.pending_store,
        );

        self.process_from_candidate().await
    }

    async fn process_from_candidate(&mut self) -> Result<(), CertificateStatusError> {
        if self.header.status != CertificateStatus::Candidate {
            return Err(CertificateStatusError::InternalError(format!(
                "CertificateTask::process_from_candidate called with cert status {}",
                self.header.status,
            )));
        }

        let height = self.header.height;
        let certificate_id = self.header.certificate_id;
        let new_pp_root = self
            .new_pp_root
            .ok_or(CertificateStatusError::InternalError(
                "CertificateTask::process_from_candidate called without a pp_root".into(),
            ))?;

        let mut prev_tx_hashes = self
            .pending_store
            .get_settlement_tx_hashes_for_certificate(certificate_id)
            .inspect_err(|error| error!(?error, "Failed to query prev tx hashes"))?
            .into_vec();
        let mut processed_tx_hashes = Vec::new();

        let (epoch_number, certificate_index, settlement_tx_hash) = loop {
            let settlement_tx_hash = prev_tx_hashes.pop().ok_or_else(|| {
                let err_message = String::from("No previous settlement tx matches");
                error!(?processed_tx_hashes, ?certificate_id, "{err_message}");
                CertificateStatusError::SettlementError(err_message)
            })?;

            debug!(
                "Pre check prev txhash {settlement_tx_hash}: {:?}",
                self.state_store
                    .get_certificate_header(&certificate_id)
                    .map(|h| h.map(|h| h.status))
            );

            if processed_tx_hashes.contains(&settlement_tx_hash) {
                continue;
            }
            processed_tx_hashes.push(settlement_tx_hash);

            debug!(
                %settlement_tx_hash,
                "Waiting for certificate settlement to complete"
            );

            match self
                .wait_for_settlement_of(settlement_tx_hash, new_pp_root)
                .await?
            {
                CertificateSettlementResult::Settled(epoch_number, certificate_index) => {
                    debug!(
                        %settlement_tx_hash,
                        %epoch_number, %certificate_index, "Previous tx hash settled"
                    );
                    break (epoch_number, certificate_index, settlement_tx_hash);
                }
                CertificateSettlementResult::Error(error) => {
                    return Err(error);
                }
                CertificateSettlementResult::TimeoutError => {
                    // Retry the settlement transaction
                    info!(
                        "Retrying the settlement transaction after a timeout for certificate \
                         {certificate_id}"
                    );
                    self.set_status(CertificateStatus::Proven)?;
                    return Box::pin(self.process_from_proven()).await;
                }
                CertificateSettlementResult::SettledThroughOtherTx(
                    alternative_settlement_tx_hash,
                ) => {
                    info!(
                        "Process alternative settlement transaction \
                         {alternative_settlement_tx_hash}"
                    );
                    self.pending_store
                        .insert_settlement_tx_hash_for_certificate(
                            &certificate_id,
                            alternative_settlement_tx_hash,
                        )?;
                    // Reprocess from Candidate, and not directly to Settled: we want to check
                    // the number of confirmations, and have not done that here yet.
                    self.set_status(CertificateStatus::Candidate)?;
                    return Box::pin(self.process_from_candidate()).await;
                }
            };
        };

        let settled_certificate =
            SettledCertificate(certificate_id, height, epoch_number, certificate_index);
        tracing::trace!(
            header = ?self
                .state_store
                .get_certificate_header(&certificate_id)
                .map(|h| h.map(|h| h.status)),
            "Header before settlement info write",
        );
        self.set_status(CertificateStatus::Settled)?;
        debug!(
            %settlement_tx_hash,
            ?settled_certificate,
            "Certificate settlement completed"
        );
        self.send_to_network_task(NetworkTaskMessage::CertificateSettled {
            height,
            certificate_id,
            settled_certificate,
        })
        .await?;

        if self.header.status != CertificateStatus::Settled {
            return Err(CertificateStatusError::InternalError(
                "CertificateTask completed with a non-settled certificate".into(),
            ));
        }

        Ok(())
    }

    async fn wait_for_settlement_of(
        &self,
        settlement_tx_hash: agglayer_types::SettlementTxHash,
        new_pp_root: Digest,
    ) -> Result<CertificateSettlementResult, CertificateStatusError> {
        let (settlement_complete_notifier, settlement_complete) = oneshot::channel();
        self.send_to_network_task(NetworkTaskMessage::CertificateWaitingForSettlement {
            height: self.header.height,
            certificate_id: self.header.certificate_id,
            settlement_tx_hash,
            settlement_complete_notifier,
            new_pp_root,
        })
        .await?;

        settlement_complete.await.map_err(recv_err)
    }

    fn set_status(&mut self, status: CertificateStatus) -> Result<(), CertificateStatusError> {
        self.state_store
            .update_certificate_header_status(&self.header.certificate_id, &status)?;
        self.header.status = status;
        Ok(())
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
    use agglayer_types::SettlementTxHash;

    use super::*;

    pub(crate) fn inject_fail_points_after_proving<PendingStore: PendingCertificateWriter>(
        certificate_id: &agglayer_types::CertificateId,
        header: &mut CertificateHeader,
        pending_store: &PendingStore,
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
                pending_store
                    .insert_settlement_tx_hash_for_certificate(certificate_id, unexistent_tx_hash)
                    .expect("Valid tx hash update");
                Some(())
            },
        );

        // Fail point to record candidate and potentially shutdown
        fail::fail_point!("certificate_task::process_impl::candidate_recorded");
    }
}
