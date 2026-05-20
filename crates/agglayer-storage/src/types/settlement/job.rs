use agglayer_types::SettlementJobId;

pub type Key = SettlementJobId;
pub type Value = crate::types::generated::agglayer::storage::v0::SettlementJob;

crate::schema::impl_codec_using_protobuf_for!(Value);
