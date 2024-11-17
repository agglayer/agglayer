use std::sync::Arc;

use agglayer_clock::ClockRef;
use agglayer_storage::{
    columns::{
        latest_proven_certificate_per_network::ProvenCertificate,
        latest_settled_certificate_per_network::SettledCertificate,
    },
    stores::{PendingCertificateReader, PendingCertificateWriter, StateReader, StateWriter},
};
use agglayer_types::{
    Certificate, CertificateId, CertificateStatus, CertificateStatusError, Hash, Height,
    LocalNetworkStateData, NetworkId,
};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

use crate::{error::PreCertificationError, CertificationError, Certifier, CertifierOutput, Error};

/// Message to notify the network task that a new certificate has been received.
#[derive(Debug)]
pub(crate) struct NewCertificate {
    pub(crate) certificate_id: CertificateId,
    pub(crate) height: Height,
}

/// Network task that is responsible to certify the certificates for a network.
pub(crate) struct NetworkTask<CertifierClient, PendingStore, StateStore> {
    /// The network id for the network task.
    network_id: NetworkId,
    /// The pending store to read and write the pending certificates.
    pending_store: Arc<PendingStore>,
    /// The state store to read and write the state of the network.
    state_store: Arc<StateStore>,
    /// The certifier client to certify the certificates.
    certifier_client: Arc<CertifierClient>,
    /// The local network state of the network task.
    local_state: LocalNetworkStateData,
    /// The sender to notify that a certificate has been proven.
    certification_notifier: mpsc::Sender<(oneshot::Sender<SettledCertificate>, ProvenCertificate)>,
    /// The clock reference to subscribe to the epoch events and check for
    /// current epoch.
    clock_ref: ClockRef,
    /// The pending local network state that should be applied on receiving
    /// settlement response.
    pending_state: Option<LocalNetworkStateData>,
    /// The stream of new certificates to certify.
    certificate_stream: mpsc::Receiver<NewCertificate>,
    /// Flag to indicate if the network is at capacity for the current epoch.
    at_capacity_for_epoch: bool,
}

impl<CertifierClient, PendingStore, StateStore>
    NetworkTask<CertifierClient, PendingStore, StateStore>
where
    CertifierClient: Certifier,
    PendingStore: PendingCertificateReader + PendingCertificateWriter,
    StateStore: StateReader + StateWriter,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pending_store: Arc<PendingStore>,
        state_store: Arc<StateStore>,
        certifier_client: Arc<CertifierClient>,
        certification_notifier: mpsc::Sender<(
            oneshot::Sender<SettledCertificate>,
            ProvenCertificate,
        )>,
        clock_ref: ClockRef,
        network_id: NetworkId,
        certificate_stream: mpsc::Receiver<NewCertificate>,
    ) -> Result<Self, Error> {
        info!("Creating a new network task for network {}", network_id);

        let local_state = state_store
            .read_local_network_state(network_id)?
            .unwrap_or_default();

        Ok(Self {
            network_id,
            pending_store,
            state_store,
            certifier_client,
            local_state,
            certification_notifier,
            clock_ref,
            pending_state: None,
            certificate_stream,
            at_capacity_for_epoch: false,
        })
    }

    pub(crate) async fn run(
        mut self,
        cancellation_token: CancellationToken,
    ) -> Result<NetworkId, Error> {
        info!("Starting the network task for network {}", self.network_id);

        let mut stream_epoch = self.clock_ref.subscribe()?;

        let current_epoch = self.clock_ref.current_epoch();

        // Start from the latest settled certificate to define the next expected height
        let latest_settled = self
            .state_store
            .get_latest_settled_certificate_per_network(&self.network_id)?
            .map(|(_network_id, settled)| settled);

        let mut next_expected_height =
            if let Some(SettledCertificate(_, current_height, epoch, _)) = latest_settled {
                debug!("Current network height is {}", current_height);
                if epoch == current_epoch {
                    debug!("Already settled for the epoch {current_epoch}");
                    self.at_capacity_for_epoch = true;
                }

                current_height + 1
            } else {
                debug!("Network never settled any certificate");
                0
            };

        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    debug!("Network task for network {} has been cancelled", self.network_id);
                    return Ok(self.network_id);
                }

                result = self.make_progress(&mut stream_epoch, &mut next_expected_height) => {
                    if let Err(error)= result {
                        error!("Error during the certification process: {}", error);

                        return Err(error)
                    }
                }
            }
        }
    }

    async fn make_progress(
        &mut self,
        stream_epoch: &mut tokio::sync::broadcast::Receiver<agglayer_clock::Event>,
        next_expected_height: &mut u64,
    ) -> Result<(), Error> {
        let height = tokio::select! {
            Ok(agglayer_clock::Event::EpochEnded(epoch)) = stream_epoch.recv() => {
                info!("Received an epoch event: {}", epoch);

                let current_epoch = self.clock_ref.current_epoch();
                if epoch != 0 && epoch < (current_epoch - 1) {
                    debug!("Received an epoch event for epoch {epoch} which is outdated, current epoch is {current_epoch}");

                    return Ok(());
                }

                self.at_capacity_for_epoch = false;
                *next_expected_height
            }
            Some(NewCertificate { certificate_id, height, .. }) = self.certificate_stream.recv(), if !self.at_capacity_for_epoch => {
                info!(
                    hash = certificate_id.to_string(),
                    "Received a certificate event for {certificate_id} at height {height}"
                );

                if *next_expected_height != height {
                    warn!(
                        hash = certificate_id.to_string(),
                        "Received a certificate event for the wrong height");

                    return Ok(());
                }

                *next_expected_height
            }
            // Need to implement the cancellation token
            // _ = cancellation_token.cancelled() => {
            //     break;
            // }
        };
        // Get the certificate the pending certificate for the network at the height
        let certificate = if let Some(certificate) = self
            .pending_store
            .get_certificate(self.network_id, height)?
        {
            certificate
        } else {
            // There is no certificate to certify at this height for now
            return Ok(());
        };

        let certificate_id = certificate.hash();
        let header =
            if let Some(header) = self.state_store.get_certificate_header(&certificate_id)? {
                header
            } else {
                error!(
                    hash = certificate_id.to_string(),
                    "Certificate header not found for {certificate_id}"
                );

                return Ok(());
            };

        match header.status {
            CertificateStatus::Pending => {}

            // If the certificate is already proven or candidate, it means that the
            // certification process has already been initiated but not completed.
            // It also means that the proof exists and thus we should redo the native
            // execution to update the local state.
            CertificateStatus::Proven | CertificateStatus::Candidate => {
                // Redo native execution to get the new_state

                error!(
                    hash = certificate_id.to_string(),
                    "CRITICAL: Certificate {certificate_id} is already proven or candidate but we \
                     do not have the new_state anymore...",
                );

                return Ok(());
            }
            CertificateStatus::InError { error } => {
                warn!(
                    hash = certificate_id.to_string(),
                    "Certificate {certificate_id} is in error: {}", error
                );

                return Ok(());
            }
            CertificateStatus::Settled => {
                warn!(
                    hash = certificate_id.to_string(),
                    "Certificate {certificate_id} is already settled while trying to certify the \
                     certificate for network {} at height {}",
                    self.network_id,
                    height - 1
                );

                return Ok(());
            }
        }

        info!(
            hash = certificate_id.to_string(),
            "Certifying the certificate {certificate_id} for network {} at height {}",
            self.network_id,
            height
        );

        let result =
            match self
                .certifier_client
                .certify(self.local_state.clone(), self.network_id, height)
            {
                Ok(certifier_task) => certifier_task.await,

                // If we received a `CertificateNotFound` error, it means that the certificate was
                // not found in the pending store. This can happen if we try to
                // certify a certificate that has not been received yet. When
                // received, the certificate will be stored in the pending store and
                // the certifier task will be spawned again.
                Err(PreCertificationError::CertificateNotFound(_network_id, _height)) => {
                    return Ok(());
                }

                Err(PreCertificationError::ProofAlreadyExists(
                    network_id,
                    height,
                    certificate_id,
                )) => {
                    warn!(
                        hash = certificate_id.to_string(),
                        "Received a proof certification error for a proof that already exists for \
                         network {} at height {}",
                        network_id,
                        height
                    );

                    return Ok(());
                }
                Err(PreCertificationError::Storage(error)) => {
                    warn!(
                        hash = certificate_id.to_string(),
                        "Received a storage error while trying to certify the certificate for \
                         network {} at height {}: {:?}",
                        self.network_id,
                        height,
                        error
                    );

                    return Ok(());
                }
            };

        match result {
            Ok(CertifierOutput {
                height,
                certificate,
                new_state,
                ..
            }) => {
                debug!(
                    hash = certificate_id.to_string(),
                    "Proof certification completed for {certificate_id} for network {}",
                    self.network_id
                );
                if let Err(error) = self
                    .on_proven_certificate(height, certificate, new_state)
                    .await
                {
                    error!(
                        hash = certificate_id.to_string(),
                        "Error during the certification process of {certificate_id} for network \
                         {}: {:?}",
                        self.network_id,
                        error
                    );
                }

                *next_expected_height += 1;

                self.at_capacity_for_epoch = true;
                debug!(
                    hash = certificate_id.to_string(),
                    "Certification process completed for {certificate_id} for network {}",
                    self.network_id
                );

                Ok(())
            }

            Err(error) => {
                warn!(
                    hash = certificate_id.to_string(),
                    "Error during certification process of {certificate_id}: {}", error
                );
                let certificate_error: Option<(CertificateId, CertificateStatusError)> = match error
                {
                    CertificationError::TrustedSequencerNotFound(certificate_id, network) => {
                        Some((
                            certificate_id,
                            CertificateStatusError::TrustedSequencerNotFound(network),
                        ))
                    }
                    CertificationError::ProofVerificationFailed {
                        source,
                        certificate_id,
                    } => Some((certificate_id, source.into())),

                    CertificationError::ProverExecutionFailed {
                        source,
                        certificate_id,
                    } => Some((
                        certificate_id,
                        CertificateStatusError::ProofGenerationError {
                            generation_type: agglayer_types::GenerationType::Prover,
                            source,
                        },
                    )),

                    CertificationError::NativeExecutionFailed {
                        source,
                        certificate_id,
                    } => Some((
                        certificate_id,
                        CertificateStatusError::ProofGenerationError {
                            generation_type: agglayer_types::GenerationType::Native,
                            source,
                        },
                    )),

                    CertificationError::Types {
                        source,
                        certificate_id,
                    } => Some((certificate_id, source.into())),

                    CertificationError::Storage(error) => {
                        warn!(
                            hash = certificate_id.to_string(),
                            "Storage error happened in the certification process of \
                             {certificate_id}: {:?}",
                            error
                        );
                        None
                    }
                    CertificationError::Serialize {
                        certificate_id,
                        source,
                    } => {
                        warn!(
                            hash = certificate_id.to_string(),
                            "Serialization error happened in the certification process of \
                             {certificate_id}: {:?}",
                            source
                        );
                        None
                    }
                    CertificationError::Deserialize {
                        certificate_id,
                        source,
                    } => {
                        warn!(
                            hash = certificate_id.to_string(),
                            "Deserialization error happened in the certification process of \
                             {certificate_id}: {:?}",
                            source
                        );
                        None
                    }
                    CertificationError::InternalError => {
                        warn!(
                            hash = certificate_id.to_string(),
                            "Internal error happened in the certification process of \
                             {certificate_id}"
                        );

                        return Err(Error::InternalError);
                    }
                };

                if let Some((certificate_id, error)) = certificate_error {
                    if self
                        .state_store
                        .update_certificate_header_status(
                            &certificate_id,
                            &CertificateStatus::InError { error },
                        )
                        .is_err()
                    {
                        error!(
                            hash = certificate_id.to_string(),
                            "Certificate {certificate_id} in error and failed to update the \
                             certificate header status"
                        );
                    }
                }
                Ok(())
            }
        }
    }
}

impl<CertifierClient, PendingStore, StateStore>
    NetworkTask<CertifierClient, PendingStore, StateStore>
where
    CertifierClient: Certifier,
    PendingStore: PendingCertificateReader + PendingCertificateWriter,
    StateStore: StateWriter,
{
    /// Context:
    ///
    /// At one point in time, there is at most one certifier task per network
    /// running. The certifier task try to generate a proof based on
    /// a certificate. The certifier task doesn't know about other tasks nor if
    /// the certificate will be included in an epoch. The Orchestrator is
    /// the one that is responsible to decide if a proof is valid and should
    /// be included in an epoch.
    ///
    /// Based on the current context of the orchestrator, we can
    /// determine the following:
    ///
    /// 1. If the state doesn't know the network and the height is 0, we update
    ///    the state. This is the first certificate for this network.
    /// 2. If the state knows the network and the height is the next one, we
    ///    update the state. This is the next certificate for this network.
    /// 3. If the state doesn't know the network and the height is not 0, we
    ///    ignore the proof.
    /// 4. If the state knows the network and the height is not the next
    ///    expected one, we ignore the proof.
    ///
    /// When a generated proof is accepted:
    /// - We update the cursor for the network.
    /// - We update the latest proven certificate for the network.
    /// - We do not remove the pending certificate. (as it needs to be included
    ///   in an epoch)
    /// - We spawn the next certificate for the network.
    async fn on_proven_certificate(
        &mut self,
        height: Height,
        certificate: Certificate,
        new_state: LocalNetworkStateData,
    ) -> Result<(), Error> {
        let certificate_id = certificate.hash();
        if let Err(error) = self
            .pending_store
            .set_latest_proven_certificate_per_network(&self.network_id, &height, &certificate_id)
        {
            error!(
                hash = certificate_id.to_string(),
                "Failed to set the latest proven certificate per network: {:?}", error
            );
        }

        if let Err(error) = self
            .state_store
            .update_certificate_header_status(&certificate_id, &CertificateStatus::Proven)
        {
            error!(
                hash = certificate_id.to_string(),
                "Failed to update the certificate header status: {:?}", error
            );
        }

        self.pending_state = Some(new_state);

        let (sender, receiver) = oneshot::channel();

        if self
            .certification_notifier
            .send((
                sender,
                ProvenCertificate(certificate_id, self.network_id, height),
            ))
            .await
            .is_err()
        {
            error!("Failed to send the proven certificate notification");
        }

        if let Ok(SettledCertificate(certificate_id, _height, _epoch, _index)) = receiver.await {
            info!(
                hash = certificate_id.to_string(),
                "Received a certificate settlement notification"
            );
            if let Some(new) = self.pending_state.take() {
                debug!(
                    "Updated the state for network {} with the new state {} > {}",
                    self.network_id,
                    self.local_state.get_roots().display_to_hex(),
                    new.get_roots().display_to_hex()
                );

                self.local_state = new;
            } else {
                debug!(
                    "Updated the state for network {} with the new state [] > {}",
                    self.network_id,
                    self.local_state.get_roots().display_to_hex()
                );
            }

            // Store the current state
            let new_leaves = certificate
                .bridge_exits
                .iter()
                .map(|exit| exit.hash().into())
                .collect::<Vec<Hash>>();

            _ = self.state_store.write_local_network_state(
                &certificate.network_id,
                &self.local_state,
                new_leaves.as_slice(),
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use agglayer_storage::tests::mocks::{
         MockPendingStore, MockStateStore,
    };
    use mockall::predicate::{always, eq};
    use rstest::rstest;

    use super::*;
    use crate::tests::{clock, mocks::MockCertifier};

    #[rstest]
    #[tokio::test]
    #[timeout(Duration::from_secs(1))]
    async fn start_from_zero() {
        let mut pending = MockPendingStore::new();
        let mut state = MockStateStore::new();
        let mut certifier = MockCertifier::new();
        let (certification_notifier, mut receiver) = mpsc::channel(1);
        let clock_ref = clock();
        let network_id = 1.into();
        let (sender, certificate_stream) = mpsc::channel(1);

        let certificate = Certificate::new_for_test(network_id, 0);
        let certificate_id = certificate.hash();
        pending
            .expect_get_certificate()
            .once()
            .with(eq(network_id), eq(0))
            .returning(|network_id, height| {
                Ok(Some(Certificate::new_for_test(network_id, height)))
            });

        state
            .expect_get_certificate_header()
            .once()
            .with(eq(certificate_id))
            .returning(|certificate_id| {
                Ok(Some(agglayer_types::CertificateHeader {
                    network_id: 1.into(),
                    height: 0,
                    epoch_number: None,
                    certificate_index: None,
                    certificate_id: *certificate_id,
                    new_local_exit_root: [0; 32].into(),
                    metadata: [0; 32].into(),
                    status: CertificateStatus::Pending,
                }))
            });

        certifier
            .expect_certify()
            .once()
            .with(always(), eq(network_id), eq(0))
            .return_once(move |new_state, network_id, _height| {
                Ok(Box::pin(async move {
                    let result = crate::CertifierOutput {
                        certificate,
                        height: 0,
                        new_state,
                        network: network_id,
                    };

                    Ok(result)
                }))
            });

        state
            .expect_read_local_network_state()
            .returning(|_| Ok(Default::default()));

        state
            .expect_write_local_network_state()
            .returning(|_, _, _| Ok(()));

        pending
            .expect_set_latest_proven_certificate_per_network()
            .once()
            .with(eq(network_id), eq(0), eq(certificate_id))
            .returning(|_, _, _| Ok(()));
        state
            .expect_update_certificate_header_status()
            .once()
            .with(eq(certificate_id), eq(CertificateStatus::Proven))
            .returning(|_, _| Ok(()));

        let mut task = NetworkTask::new(
            Arc::new(pending),
            Arc::new(state),
            Arc::new(certifier),
            certification_notifier,
            clock_ref,
            network_id,
            certificate_stream,
        )
        .expect("Failed to create a new network task");

        let mut epochs = task.clock_ref.subscribe().unwrap();
        let mut next_expected_height = 0;

        let _ = sender
            .send(NewCertificate {
                certificate_id,
                height: 0,
            })
            .await;

        tokio::spawn(async move {
            let (sender, cert) = receiver.recv().await.unwrap();

            _ = sender.send(SettledCertificate(cert.0, cert.2, 0, 0));
        });

        task.make_progress(&mut epochs, &mut next_expected_height)
            .await
            .unwrap();

        assert_eq!(next_expected_height, 1);
    }

    #[rstest]
    #[tokio::test]
    #[timeout(Duration::from_secs(1))]
    async fn one_per_epoch() {
        let mut pending = MockPendingStore::new();
        let mut state = MockStateStore::new();
        let mut certifier = MockCertifier::new();
        let (certification_notifier, mut receiver) = mpsc::channel(1);
        let clock_ref = clock();
        let network_id = 1.into();
        let (sender, certificate_stream) = mpsc::channel(100);

        let certificate = Certificate::new_for_test(network_id, 0);
        let certificate2 = Certificate::new_for_test(network_id, 1);
        let certificate_id = certificate.hash();
        let certificate_id2 = certificate2.hash();

        pending
            .expect_get_certificate()
            .once()
            .with(eq(network_id), eq(0))
            .returning(|network_id, height| {
                Ok(Some(Certificate::new_for_test(network_id, height)))
            });

        pending
            .expect_get_certificate()
            .never()
            .with(eq(network_id), eq(1))
            .returning(|network_id, height| {
                Ok(Some(Certificate::new_for_test(network_id, height)))
            });
        state
            .expect_get_certificate_header()
            .once()
            .with(eq(certificate_id))
            .returning(|certificate_id| {
                Ok(Some(agglayer_types::CertificateHeader {
                    network_id: 1.into(),
                    height: 0,
                    epoch_number: None,
                    certificate_index: None,
                    certificate_id: *certificate_id,
                    new_local_exit_root: [0; 32].into(),
                    metadata: [0; 32].into(),
                    status: CertificateStatus::Pending,
                }))
            });

        state
            .expect_get_certificate_header()
            .never()
            .with(eq(certificate_id2))
            .returning(|certificate_id| {
                Ok(Some(agglayer_types::CertificateHeader {
                    network_id: 1.into(),
                    height: 1,
                    epoch_number: None,
                    certificate_index: None,
                    certificate_id: *certificate_id,
                    new_local_exit_root: [0; 32].into(),
                    metadata: [0; 32].into(),
                    status: CertificateStatus::Pending,
                }))
            });
        certifier
            .expect_certify()
            .once()
            .with(always(), eq(network_id), eq(0))
            .return_once(move |new_state, network_id, _height| {
                Ok(Box::pin(async move {
                    let result = crate::CertifierOutput {
                        certificate,
                        height: 0,
                        new_state,
                        network: network_id,
                    };

                    Ok(result)
                }))
            });

        state
            .expect_read_local_network_state()
            .returning(|_| Ok(Default::default()));

        state
            .expect_write_local_network_state()
            .returning(|_, _, _| Ok(()));

        certifier
            .expect_certify()
            .never()
            .with(always(), eq(network_id), eq(1))
            .return_once(move |new_state, network_id, _height| {
                Ok(Box::pin(async move {
                    let result = crate::CertifierOutput {
                        certificate: certificate2,
                        height: 1,
                        new_state,
                        network: network_id,
                    };

                    Ok(result)
                }))
            });
        pending
            .expect_set_latest_proven_certificate_per_network()
            .once()
            .with(eq(network_id), eq(0), eq(certificate_id))
            .returning(|_, _, _| Ok(()));
        state
            .expect_update_certificate_header_status()
            .once()
            .with(eq(certificate_id), eq(CertificateStatus::Proven))
            .returning(|_, _| Ok(()));

        let mut task = NetworkTask::new(
            Arc::new(pending),
            Arc::new(state),
            Arc::new(certifier),
            certification_notifier,
            clock_ref,
            network_id,
            certificate_stream,
        )
        .expect("Failed to create a new network task");

        let mut epochs = task.clock_ref.subscribe().unwrap();
        let mut next_expected_height = 0;

        sender
            .send(NewCertificate {
                certificate_id,
                height: 0,
            })
            .await
            .expect("Failed to send the certificate");

        sender
            .send(NewCertificate {
                certificate_id: certificate_id2,
                height: 1,
            })
            .await
            .expect("Failed to send the certificate");

        tokio::spawn(async move {
            let (sender, cert) = receiver.recv().await.unwrap();

            sender
                .send(SettledCertificate(cert.0, cert.2, 0, 0))
                .expect("Failed to send");
        });

        task.make_progress(&mut epochs, &mut next_expected_height)
            .await
            .unwrap();

        assert_eq!(next_expected_height, 1);

        tokio::time::timeout(
            Duration::from_millis(100),
            task.make_progress(&mut epochs, &mut next_expected_height),
        )
        .await
        .expect_err("Should have timed out");

        assert_eq!(next_expected_height, 1);
    }

    #[rstest]
    #[tokio::test]
    #[timeout(Duration::from_secs(1))]
    async fn changing_epoch_triggers_certify() {
        let mut pending = MockPendingStore::new();
        let mut state = MockStateStore::new();
        let mut certifier = MockCertifier::new();
        let (certification_notifier, mut receiver) = mpsc::channel(1);
        let clock_ref = clock();
        let network_id = 1.into();
        let (sender, certificate_stream) = mpsc::channel(100);

        let certificate = Certificate::new_for_test(network_id, 0);
        let certificate2 = Certificate::new_for_test(network_id, 1);
        let certificate_id = certificate.hash();
        let certificate_id2 = certificate2.hash();

        pending
            .expect_get_certificate()
            .once()
            .with(eq(network_id), eq(0))
            .returning(|network_id, height| {
                Ok(Some(Certificate::new_for_test(network_id, height)))
            });

        pending
            .expect_get_certificate()
            .once()
            .with(eq(network_id), eq(1))
            .returning(|network_id, height| {
                Ok(Some(Certificate::new_for_test(network_id, height)))
            });

        state
            .expect_read_local_network_state()
            .returning(|_| Ok(Default::default()));

        state
            .expect_write_local_network_state()
            .returning(|_, _, _| Ok(()));

        state
            .expect_get_certificate_header()
            .once()
            .with(eq(certificate_id))
            .returning(|certificate_id| {
                Ok(Some(agglayer_types::CertificateHeader {
                    network_id: 1.into(),
                    height: 0,
                    epoch_number: None,
                    certificate_index: None,
                    certificate_id: *certificate_id,
                    new_local_exit_root: [0; 32].into(),
                    metadata: [0; 32].into(),
                    status: CertificateStatus::Pending,
                }))
            });

        state
            .expect_get_certificate_header()
            .once()
            .with(eq(certificate_id2))
            .returning(|certificate_id| {
                Ok(Some(agglayer_types::CertificateHeader {
                    network_id: 1.into(),
                    height: 1,
                    epoch_number: None,
                    certificate_index: None,
                    certificate_id: *certificate_id,
                    new_local_exit_root: [0; 32].into(),
                    metadata: [0; 32].into(),
                    status: CertificateStatus::Pending,
                }))
            });
        certifier
            .expect_certify()
            .once()
            .with(always(), eq(network_id), eq(0))
            .return_once(move |new_state, network_id, _height| {
                Ok(Box::pin(async move {
                    let result = crate::CertifierOutput {
                        certificate,
                        height: 0,
                        new_state,
                        network: network_id,
                    };

                    Ok(result)
                }))
            });

        certifier
            .expect_certify()
            .once()
            .with(always(), eq(network_id), eq(1))
            .return_once(move |new_state, network_id, _height| {
                Ok(Box::pin(async move {
                    let result = crate::CertifierOutput {
                        certificate: certificate2,
                        height: 1,
                        new_state,
                        network: network_id,
                    };

                    Ok(result)
                }))
            });

        pending
            .expect_set_latest_proven_certificate_per_network()
            .once()
            .with(eq(network_id), eq(0), eq(certificate_id))
            .returning(|_, _, _| Ok(()));
        pending
            .expect_set_latest_proven_certificate_per_network()
            .once()
            .with(eq(network_id), eq(1), eq(certificate_id2))
            .returning(|_, _, _| Ok(()));

        state
            .expect_update_certificate_header_status()
            .once()
            .with(eq(certificate_id), eq(CertificateStatus::Proven))
            .returning(|_, _| Ok(()));

        state
            .expect_update_certificate_header_status()
            .once()
            .with(eq(certificate_id2), eq(CertificateStatus::Proven))
            .returning(|_, _| Ok(()));

        let mut task = NetworkTask::new(
            Arc::new(pending),
            Arc::new(state),
            Arc::new(certifier),
            certification_notifier,
            clock_ref.clone(),
            network_id,
            certificate_stream,
        )
        .expect("Failed to create a new network task");

        let mut epochs = task.clock_ref.subscribe().unwrap();
        let mut next_expected_height = 0;

        sender
            .send(NewCertificate {
                certificate_id,
                height: 0,
            })
            .await
            .expect("Failed to send the certificate");

        sender
            .send(NewCertificate {
                certificate_id: certificate_id2,
                height: 1,
            })
            .await
            .expect("Failed to send the certificate");

        tokio::spawn(async move {
            let (sender, cert) = receiver.recv().await.unwrap();

            sender
                .send(SettledCertificate(cert.0, cert.2, 0, 0))
                .expect("Failed to send");

            let (sender, cert) = receiver.recv().await.unwrap();

            sender
                .send(SettledCertificate(cert.0, cert.2, 1, 0))
                .expect("Failed to send");
        });

        task.make_progress(&mut epochs, &mut next_expected_height)
            .await
            .unwrap();

        assert_eq!(next_expected_height, 1);

        tokio::time::timeout(
            Duration::from_millis(100),
            task.make_progress(&mut epochs, &mut next_expected_height),
        )
        .await
        .expect_err("Should have timed out");

        assert_eq!(next_expected_height, 1);

        clock_ref
            .get_sender()
            .send(agglayer_clock::Event::EpochEnded(0))
            .expect("Failed to send");

        task.make_progress(&mut epochs, &mut next_expected_height)
            .await
            .unwrap();

        assert_eq!(next_expected_height, 2);
    }
}
