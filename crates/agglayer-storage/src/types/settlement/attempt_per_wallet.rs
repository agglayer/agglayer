use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Key {
    pub(crate) address: [u8; 20],
    pub(crate) nonce: u64,
}

pub type Value = super::attempt::Key;
