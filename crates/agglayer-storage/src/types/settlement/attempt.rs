use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Key {
    pub(crate) settlement_job_id: ulid::Ulid,
    pub(crate) attempt_sequence_number: u64,
}

pub type Value = crate::types::generated::agglayer::storage::v0::SettlementAttempt;

crate::schema::impl_codec_using_protobuf_for!(Value);
