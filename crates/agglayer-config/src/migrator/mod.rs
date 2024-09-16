use std::path::Path;

use serde::Deserialize;

use crate::certificate_orchestrator::CertificateOrchestrator;

mod v0_1;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ConfigMigrator {
    V0_2(crate::Config),
    V0_1(v0_1::Config),
}

impl ConfigMigrator {
    pub fn migrate(self, config_path: &Path) -> crate::Config {
        match self {
            ConfigMigrator::V0_1(v0_1::Config {
                full_node_rpcs,
                proof_signers,
                log,
                rpc,
                outbound,
                l1,
                auth,
                telemetry,
                epoch,
                shutdown,
            }) => {
                tracing::warn!("Migration from v0.1 to v0.2");

                crate::Config {
                    full_node_rpcs,
                    proof_signers,
                    log,
                    rpc,
                    outbound,
                    l1,
                    l2: Default::default(),
                    auth,
                    telemetry,
                    epoch,
                    shutdown,
                    certificate_orchestrator: CertificateOrchestrator::default(),
                    storage: crate::storage::StorageConfig::new_from_path(config_path),
                }
            }
            ConfigMigrator::V0_2(config) => config,
        }
        .path_contextualized(config_path)
    }
}
