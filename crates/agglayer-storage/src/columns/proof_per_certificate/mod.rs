use bincode::Options;

use super::{default_bincode_options, Codec, ColumnSchema, PROOF_PER_CERTIFICATE_CF};
use crate::types::{CertificateId, Proof};

#[cfg(test)]
mod tests;

/// Column family that returns the generated proof for one certificate.
///
/// | --- key ------  |    | --- value ---- |
/// | CertificateId   | => | Proof bytes    |
pub struct ProofPerCertificateColumn;

impl Codec for Vec<u8> {
    fn encode(&self) -> Result<Vec<u8>, crate::error::Error> {
        Ok(default_bincode_options().serialize(&self)?)
    }

    fn decode(buf: &[u8]) -> Result<Self, crate::error::Error> {
        Ok(default_bincode_options().deserialize(buf)?)
    }
}

impl ColumnSchema for ProofPerCertificateColumn {
    type Key = CertificateId;
    type Value = Proof;

    const COLUMN_FAMILY_NAME: &'static str = PROOF_PER_CERTIFICATE_CF;
}
