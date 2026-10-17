use std::{collections::HashMap, future::Future};

use agglayer_config::Config;
use agglayer_types::NetworkId;
use eyre::{eyre, Context as _};
use prover_config::{NetworkProverConfig, ProverType};
use prover_executor::{Error, Executor, Request, Response};
use tower::{buffer::Buffer, util::BoxCloneService, ServiceExt as _};
use tracing::info;

type ProverService = Buffer<BoxCloneService<Request, Response, Error>, Request>;

#[derive(Clone)]
pub struct ProverRouter {
    default: ProverService,
    overrides: HashMap<u32, ProverService>,
}

impl ProverRouter {
    pub async fn new(config: &Config) -> eyre::Result<Self> {
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
        let default = factory(config.prover.clone())
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
                let service = factory(ProverType::NetworkProver(NetworkProverConfig {
                    sp1_cluster_endpoint: endpoint.clone(),
                    ..network_config.clone()
                }))
                .await
                .with_context(|| {
                    format!("Failed initializing Succinct RPC override for network {network_id}")
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

#[cfg(test)]
mod tests;
