use std::path::PathBuf;

use serde::Deserialize;
use serde_with::serde_as;

/// Local configuration.
///
/// It includes private keys for a local wallet if no kms configuration is
/// provided.
#[serde_as]
#[derive(Deserialize, Debug)]
#[cfg_attr(any(test, feature = "testutils"), derive(Default))]
#[serde(rename_all = "PascalCase")]
pub struct Local {
    pub private_keys: Vec<PrivateKey>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct PrivateKey {
    pub path: PathBuf,
    pub password: String,
}
