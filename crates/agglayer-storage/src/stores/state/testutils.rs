use super::*;

impl StateStore {
    /// Insert a header directly for storage fixtures.
    pub fn insert_certificate_header(
        &self,
        certificate: &Certificate,
        status: CertificateStatus,
    ) -> Result<(), Error> {
        let certificate_id = certificate.hash();
        let certificate_header = Self::certificate_header(certificate, status.clone());
        let mut metrics = self.network_metrics.mutation();
        self.db
            .put::<CertificateHeaderColumn>(&certificate_id, &certificate_header)?;
        metrics.header_written(
            certificate.network_id,
            certificate_id,
            &certificate_header.status,
        );

        if let CertificateStatus::Settled = status {
            self.db.put::<CertificatePerNetworkColumn>(
                &certificate_per_network::Key {
                    network_id: certificate.network_id.to_u32(),
                    height: certificate.height,
                },
                &certificate_id,
            )?;
        }

        if let CertificateStatus::Pending = status {
            self.request_backup();
        }

        Ok(())
    }
}
