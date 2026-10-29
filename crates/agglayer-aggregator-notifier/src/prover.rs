use std::{collections::HashMap, ffi::OsString, future::Future};

use agglayer_config::Config;
use agglayer_types::NetworkId;
use eyre::{bail, ensure, eyre, Context as _};
use prover_config::{NetworkProverConfig, ProverType};
use prover_executor::{Error, Executor, Request, Response};
use tower::{buffer::Buffer, util::BoxCloneService, ServiceExt as _};
use tracing::info;
use url::Url;

const NETWORK_MTLS_ENV_VARS: [&str; 2] = ["NETWORK_MTLS_CERT_PATH", "NETWORK_MTLS_KEY_PATH"];

type ProverService = Buffer<BoxCloneService<Request, Response, Error>, Request>;

#[derive(Clone)]
pub struct ProverRouter {
    default: ProverService,
    overrides: HashMap<u32, ProverService>,
}

impl ProverRouter {
    pub async fn new(config: &Config) -> eyre::Result<Self> {
        reject_mtls_env_vars(std::env::var_os)?;
        Self::with_factory(config, |prover| async move {
            let (_verifying_key, service) = Executor::create_prover(prover, crate::ELF).await?;
            Ok(Buffer::new(service, config.prover_buffer_size))
        })
        .await
    }

    async fn with_factory<F, Fut>(config: &Config, mut factory: F) -> eyre::Result<Self>
    where
        F: FnMut(ProverType) -> Fut,
        Fut: Future<Output = eyre::Result<ProverService>>,
    {
        validate_mtls(config)?;

        let default_prover = match &config.prover {
            ProverType::NetworkProver(network_config) => {
                network_prover(config, network_config, &network_config.sp1_cluster_endpoint)
            }
            prover => prover.clone(),
        };
        let default = factory(default_prover)
            .await
            .context("Failed initializing default prover")?;
        // Readiness reserves buffer capacity; probing a clone releases that
        // permit.
        default
            .clone()
            .ready()
            .await
            .map_err(|error| eyre!("Default prover is not ready: {error}"))?;

        let ProverType::NetworkProver(network_config) = &config.prover else {
            return Ok(Self::from_default(default));
        };

        let mut services_by_endpoint =
            HashMap::from([(network_config.sp1_cluster_endpoint.clone(), default.clone())]);
        let mut overrides = HashMap::new();
        for (network_id, endpoint) in &config.succinct_cluster {
            let service = if let Some(service) = services_by_endpoint.get(endpoint) {
                service.clone()
            } else {
                let service = factory(network_prover(config, network_config, endpoint))
                    .await
                    .with_context(|| {
                        format!(
                            "Failed initializing Succinct RPC override for network {network_id}"
                        )
                    })?;
                service.clone().ready().await.map_err(|error| {
                    eyre!("Succinct RPC override for network {network_id} is not ready: {error}")
                })?;
                services_by_endpoint.insert(endpoint.clone(), service.clone());
                service
            };
            overrides.insert(*network_id, service);
        }

        for (network_id, endpoint) in &config.succinct_cluster {
            info!(
                network_id,
                rpc_origin = %endpoint.origin().ascii_serialization(),
                mtls = config.succinct_cluster_mtls.contains_key(endpoint),
                "Using configured Succinct RPC endpoint"
            );
        }

        Ok(Self { default, overrides })
    }

    pub(crate) fn for_network(&self, network_id: NetworkId) -> ProverService {
        self.overrides
            .get(&network_id.to_u32())
            .unwrap_or(&self.default)
            .clone()
    }

    pub(crate) fn from_default(default: ProverService) -> Self {
        Self {
            default,
            overrides: HashMap::new(),
        }
    }
}

fn network_prover(
    config: &Config,
    network_config: &NetworkProverConfig,
    endpoint: &Url,
) -> ProverType {
    ProverType::NetworkProver(NetworkProverConfig {
        sp1_cluster_endpoint: endpoint.clone(),
        mtls: config.succinct_cluster_mtls.get(endpoint).cloned(),
        ..network_config.clone()
    })
}

fn validate_mtls(config: &Config) -> eyre::Result<()> {
    let ProverType::NetworkProver(network_config) = &config.prover else {
        ensure!(
            config.succinct_cluster_mtls.is_empty(),
            "succinct-cluster-mtls requires the network prover"
        );
        return Ok(());
    };
    ensure!(
        network_config.mtls.is_none(),
        "prover.network-prover.mtls is not supported, use succinct-cluster-mtls"
    );
    for endpoint in config.succinct_cluster_mtls.keys() {
        ensure!(
            *endpoint == network_config.sp1_cluster_endpoint
                || config
                    .succinct_cluster
                    .values()
                    .any(|used| used == endpoint),
            "succinct-cluster-mtls endpoint {endpoint} is not used by any prover"
        );
    }
    Ok(())
}

fn reject_mtls_env_vars(var_os: impl Fn(&'static str) -> Option<OsString>) -> eyre::Result<()> {
    if let Some(name) = NETWORK_MTLS_ENV_VARS
        .into_iter()
        .find(|name| var_os(name).is_some())
    {
        bail!("{name} is not supported, use succinct-cluster-mtls");
    }
    Ok(())
}

#[cfg(test)]
mod tests;
