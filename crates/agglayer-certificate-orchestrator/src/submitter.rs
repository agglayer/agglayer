use tokio::sync::{mpsc, oneshot};

use crate::{CertResponseSender, Certificate, InitialCheckError as Error};

/// An entry point to submit certificates to the orchestrator.
#[derive(Debug, Clone)]
pub struct Submitter(mpsc::Sender<(Certificate, CertResponseSender)>);

impl Submitter {
    pub fn new(cert_sender: mpsc::Sender<(Certificate, CertResponseSender)>) -> Self {
        Self(cert_sender)
    }
}

impl Submitter {
    pub async fn submit(&self, certificate: Certificate) -> Result<(), Error> {
        let (resp_send, resp_recv) = oneshot::channel();

        self.0
            .send((certificate, resp_send))
            .await
            .map_err(|_| Error::CertificateSubmission)?;

        resp_recv.await.map_err(|_| Error::Internal)?
    }
}
