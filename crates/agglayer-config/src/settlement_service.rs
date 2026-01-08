use serde::{Deserialize, Serialize};

/// The settlement service configuration.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct SettlementServiceConfig {}
