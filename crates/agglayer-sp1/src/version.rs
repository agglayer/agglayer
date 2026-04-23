use crate::error::ProofError;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Sp1ProofVersion {
    V5,
    V6,
}

pub fn version_kind(version: &str) -> Result<Sp1ProofVersion, ProofError> {
    let major = version
        .trim_start_matches('v')
        .split('.')
        .next()
        .filter(|segment| !segment.is_empty())
        .ok_or_else(|| ProofError::InvalidSp1Version {
            version: version.to_owned(),
        })?;

    match major {
        "4" | "5" => Ok(Sp1ProofVersion::V5),
        "6" => Ok(Sp1ProofVersion::V6),
        other if other.chars().all(|c| c.is_ascii_digit()) => {
            Err(ProofError::UnsupportedSp1VersionMajor {
                version: version.to_owned(),
            })
        }
        _ => Err(ProofError::InvalidSp1Version {
            version: version.to_owned(),
        }),
    }
}
