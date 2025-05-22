use std::sync::Arc;

use agglayer_storage::stores::{StateReader, StateWriter};
use agglayer_types::{Certificate, CertificateHeader, CertificateStatus, CertificateStatusError};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, trace, warn};

use crate::{network_task::NetworkTaskMessage, Certifier, EpochPacker};

pub struct CertificateTask<StateStore, CertifierClient, SettlementClient> {
    certificate: Certificate,
    header: CertificateHeader,

    network_task: mpsc::Sender<NetworkTaskMessage>,
    state_store: Arc<StateStore>,
    certifier_client: Arc<CertifierClient>,
    settlement_client: Arc<SettlementClient>,
}

impl<StateStore, CertifierClient, SettlementClient>
    CertificateTask<StateStore, CertifierClient, SettlementClient>
where
    StateStore: StateReader + StateWriter,
    CertifierClient: Certifier,
    SettlementClient: EpochPacker,
{
    #[tracing::instrument(
        name = "CertificateTask::process",
        skip_all,
        fields(
            network_id = %self.header.network_id,
            height = self.header.height,
            certificate_id = %self.header.certificate_id,
        )
    )]
    pub async fn process(mut self) -> Result<(), CertificateStatusError> {
        match self.process_impl().await {
            Ok(()) => Ok(()),
            Err(error) => {
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

                Err(error)
            }
        }
    }

    /// Process a certificate, not doing any specific error handling except for returning it
    async fn process_impl(&mut self) -> Result<(), CertificateStatusError> {
        let network_id = self.header.network_id;
        let height = self.header.height;
        let certificate_id = self.header.certificate_id;

        // Skip if there's nothing to do
        if matches!(
            self.header.status,
            CertificateStatus::Settled | CertificateStatus::InError { error: _ }
        ) {
            warn!("Built a CertificateTask for a certificate that has no processing left to do");
            return Ok(());
        }

        // First, prove the certificate
        if self.header.status < CertificateStatus::Proven {
            // Retrieve local network state
            trace!("Retrieving local network state");
            let (response, state) = oneshot::channel();
            self.network_task
                .send(NetworkTaskMessage::GetLocalNetworkStateAtHeight { height, response })
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
        }

        // Second, submit settlement to L1
        if self.header.status < CertificateStatus::Candidate {
            debug!("Starting certificate settlement");
            self.settlement_client
                .settle_certificate(certificate_id)
                .await?;
            debug!("Certificate settlement completed");
            // Note: settle_certificate currently updates the certificate status itself.
            // It will go to Candidate upon submission, and then up to Settled when the settlement is confirmed.
            // However, if agglayer shuts down before settlement confirmation, we can still have a certificate with
            // a Candidate status.
            // TODO: all the storage related to certificate should only be touched from this struct, we should refactor
        }
        // If we're not in the shortcut path of `settle_certificate`, wait for settlement
        // TODO: once the storage is only ever touched from here, we can make this a regular `if`
        // For now, the only way to reach this stage is to boot with an already-candidate certificate.
        // So we know that `self.header.settlement_tx_hash` is set.
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
            self.settlement_client
                .recover_settlement(tx_hash.0.into(), certificate_id, network_id, height)
                .await?;
            debug!("Resumed certificate settlement completed");
        }

        // We just finished settling a new certificate, record that
        // TODO: once the storage is only ever touched from here, we can have that part of the last `if`
        self.network_task
            .send(NetworkTaskMessage::CertificateSettled {
                height,
                certificate_id,
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
