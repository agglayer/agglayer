use crate::{error::ProofError, version::Sp1ProofVersion};

#[derive(Debug, Clone)]
pub struct AcceptancePolicy {
    pub readable: &'static [Sp1ProofVersion],
    pub executable: &'static [Sp1ProofVersion],
    pub writable: &'static [Sp1ProofVersion],
}

impl AcceptancePolicy {
    /// Default policy for this milestone: read v5 + v6 rows from storage;
    /// execute and write only the versions the active SDK supports (v5).
    pub const DEFAULT: Self = Self {
        readable: &[Sp1ProofVersion::V5, Sp1ProofVersion::V6],
        executable: &[Sp1ProofVersion::V5],
        writable: &[Sp1ProofVersion::V5],
    };

    pub fn ensure_readable(&self, version: Sp1ProofVersion, raw: &str) -> Result<(), ProofError> {
        if self.readable.contains(&version) {
            Ok(())
        } else {
            Err(ProofError::UnsupportedReadableSp1Version {
                version: raw.to_owned(),
            })
        }
    }

    pub fn ensure_executable(&self, version: Sp1ProofVersion, raw: &str) -> Result<(), ProofError> {
        if self.executable.contains(&version) {
            Ok(())
        } else {
            Err(ProofError::UnsupportedExecutableSp1Version {
                version: raw.to_owned(),
            })
        }
    }

    pub fn ensure_writable(&self, version: Sp1ProofVersion, raw: &str) -> Result<(), ProofError> {
        if self.writable.contains(&version) {
            Ok(())
        } else {
            Err(ProofError::UnsupportedWritableSp1Version {
                version: raw.to_owned(),
            })
        }
    }
}
