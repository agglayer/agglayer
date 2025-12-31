use serde::{Deserialize, Serialize};

use crate::with::HumanSize;

pub const DEFAULT_GRPC_MESSAGE_SIZE: usize = 64 * 1024 * 1024; // 64MiB

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct GrpcConfig {
    #[serde(
        with = "HumanSize",
        skip_serializing_if = "same_as_default_max_decoding_message_size",
        default = "default_max_decoding_message_size"
    )]
    pub max_decoding_message_size: usize,
    #[serde(
        with = "HumanSize",
        skip_serializing_if = "same_as_default_max_encoding_message_size",
        default = "default_max_encoding_message_size"
    )]
    pub max_encoding_message_size: usize,
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            max_decoding_message_size: default_max_decoding_message_size(),
            max_encoding_message_size: default_max_encoding_message_size(),
        }
    }
}

const fn default_max_decoding_message_size() -> usize {
    DEFAULT_GRPC_MESSAGE_SIZE
}
fn same_as_default_max_decoding_message_size(value: &usize) -> bool {
    *value == default_max_decoding_message_size()
}
const fn default_max_encoding_message_size() -> usize {
    DEFAULT_GRPC_MESSAGE_SIZE
}
fn same_as_default_max_encoding_message_size(value: &usize) -> bool {
    *value == default_max_encoding_message_size()
}
