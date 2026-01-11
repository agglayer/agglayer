use serde::{Deserialize, Deserializer, Serialize, Serializer};
use ulid::Ulid;

/// Settlement job ID used to track settlement jobs until they are marked as Settled.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct SettlementJobId(Ulid);

impl SettlementJobId {
    /// Create a new settlement job ID from a ULID.
    pub fn new(ulid: Ulid) -> Self {
        Self(ulid)
    }

    /// Get the underlying ULID.
    pub fn ulid(&self) -> Ulid {
        self.0
    }

    /// Create a zero settlement job ID for testing.
    #[cfg(any(test, feature = "testutils"))]
    pub fn for_tests() -> Self {
        Self(Ulid::from_bytes([0u8; 16]))
    }
}

impl From<Ulid> for SettlementJobId {
    fn from(ulid: Ulid) -> Self {
        Self(ulid)
    }
}

impl From<SettlementJobId> for Ulid {
    fn from(job_id: SettlementJobId) -> Self {
        job_id.0
    }
}

impl std::fmt::Display for SettlementJobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for SettlementJobId {
    type Err = ulid::DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Ulid::from_string(s)?))
    }
}

impl Serialize for SettlementJobId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize ULID as bytes (16 bytes)
        serializer.serialize_bytes(&self.0.to_bytes())
    }
}

impl<'de> Deserialize<'de> for SettlementJobId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let bytes: Vec<u8> = Vec::deserialize(deserializer)?;
        if bytes.len() != 16 {
            return Err(D::Error::custom(format!(
                "Expected 16 bytes for ULID, got {}",
                bytes.len()
            )));
        }
        let mut ulid_bytes = [0u8; 16];
        ulid_bytes.copy_from_slice(&bytes);
        Ok(Self(Ulid::from_bytes(ulid_bytes)))
    }
}

