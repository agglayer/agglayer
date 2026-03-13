use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Key {
    pub(crate) address: [u8; 20],
    pub(crate) nonce: u64,
    pub(crate) settlement_job_id: ulid::Ulid,
    pub(crate) attempt_sequence_number: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Value;
