use std::{
    collections::{HashMap, HashSet},
    future::Future,
};

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
    configured_networks: HashSet<u32>,
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
                rpc_url = %endpoint,
                "Using configured Succinct RPC endpoint"
            );
        }

        Ok(Self {
            default,
            overrides,
            configured_networks: config.succinct_cluster.keys().copied().collect(),
        })
    }

    pub(crate) fn for_network(&self, network_id: NetworkId) -> eyre::Result<ProverService> {
        let network_id = network_id.to_u32();
        if self.configured_networks.contains(&network_id) {
            self.overrides.get(&network_id).cloned().ok_or_else(|| {
                eyre!("Missing configured Succinct RPC prover for network {network_id}")
            })
        } else {
            Ok(self.default.clone())
        }
    }

    pub(crate) fn from_default(default: ProverService) -> Self {
        Self {
            default,
            overrides: HashMap::new(),
            configured_networks: HashSet::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use prover_config::{CpuProverConfig, MockProverConfig};
    use sp1_sdk::SP1Stdin;
    use tokio::sync::Mutex;
    use tower::service_fn;

    use super::*;

    const DEFAULT_ENDPOINT: &str = "https://default.example/";
    const PRIVATE_ENDPOINT: &str = "https://private-a.example/";
    const OTHER_ENDPOINT: &str = "https://private-b.example/";

    fn config() -> eyre::Result<Config> {
        let mut config = Config {
            prover: ProverType::NetworkProver(NetworkProverConfig {
                sp1_cluster_endpoint: DEFAULT_ENDPOINT.parse()?,
                ..Default::default()
            }),
            ..Default::default()
        };
        for (network_id, endpoint) in [
            (48, PRIVATE_ENDPOINT),
            (49, OTHER_ENDPOINT),
            (50, PRIVATE_ENDPOINT),
            (51, DEFAULT_ENDPOINT),
        ] {
            config
                .succinct_cluster
                .insert(network_id, endpoint.parse()?);
        }
        Ok(config)
    }

    fn request() -> Request {
        Request {
            stdin: SP1Stdin::new(),
            proof_type: prover_executor::ProofType::Plonk,
        }
    }

    fn endpoint(prover: &ProverType) -> &str {
        match prover {
            ProverType::NetworkProver(config) => config.sp1_cluster_endpoint.as_str(),
            _ => "local",
        }
    }

    fn service(prover: ProverType, calls: Arc<Mutex<Vec<String>>>, timeout: bool) -> ProverService {
        let endpoint = endpoint(&prover).to_string();
        let service = service_fn(move |_: Request| {
            let calls = calls.clone();
            let endpoint = endpoint.clone();
            async move {
                calls.lock().await.push(endpoint.clone());
                if timeout {
                    std::future::pending::<()>().await;
                }
                Err::<Response, Error>(Error::ProverFailed(endpoint))
            }
        });
        Buffer::new(
            Executor::build_network_service(Duration::from_millis(10), service),
            1,
        )
    }

    async fn assert_route(
        router: &ProverRouter,
        network_id: u32,
        expected: &str,
    ) -> eyre::Result<()> {
        let error = router
            .for_network(network_id.into())?
            .oneshot(request())
            .await
            .expect_err("stub service must fail");
        assert_eq!(error.to_string(), format!("Prover failed: {expected}"));
        Ok(())
    }

    #[tokio::test]
    async fn routes_interleaved_requests_and_reuses_endpoints() -> eyre::Result<()> {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let mut created = Vec::new();
        let router = ProverRouter::with_factory(&config()?, |prover| {
            created.push(endpoint(&prover).to_string());
            std::future::ready(Ok(service(prover, calls.clone(), false)))
        })
        .await?;
        assert_eq!(
            created,
            [DEFAULT_ENDPOINT, PRIVATE_ENDPOINT, OTHER_ENDPOINT]
        );

        let requests = [
            (48, PRIVATE_ENDPOINT),
            (99, DEFAULT_ENDPOINT),
            (49, OTHER_ENDPOINT),
            (50, PRIVATE_ENDPOINT),
            (48, PRIVATE_ENDPOINT),
            (51, DEFAULT_ENDPOINT),
        ];
        let requests_in_flight = futures::future::try_join_all(
            requests.map(|(network_id, endpoint)| assert_route(&router, network_id, endpoint)),
        );
        tokio::time::timeout(Duration::from_secs(1), requests_in_flight).await??;
        let mut actual = calls.lock().await.clone();
        actual.sort();
        let mut expected = requests.map(|(_, endpoint)| endpoint.to_string());
        expected.sort();
        assert_eq!(actual, expected);
        Ok(())
    }

    #[tokio::test]
    async fn failures_never_switch_endpoints() -> eyre::Result<()> {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let mut router = ProverRouter::with_factory(&config()?, |prover| {
            std::future::ready(Ok(service(prover, calls.clone(), true)))
        })
        .await?;

        for network_id in [48, 99] {
            assert!(router
                .for_network(network_id.into())?
                .oneshot(request())
                .await
                .is_err());
        }
        router.overrides.remove(&48);
        assert!(router.for_network(48.into()).is_err());
        assert_eq!(*calls.lock().await, [PRIVATE_ENDPOINT, DEFAULT_ENDPOINT]);
        Ok(())
    }

    #[tokio::test]
    async fn initialization_failure_never_uses_another_endpoint() -> eyre::Result<()> {
        for (failed_endpoint, expected_created, context) in [
            (
                DEFAULT_ENDPOINT,
                vec![DEFAULT_ENDPOINT],
                "Failed initializing default prover",
            ),
            (
                PRIVATE_ENDPOINT,
                vec![DEFAULT_ENDPOINT, PRIVATE_ENDPOINT],
                "Failed initializing Succinct RPC override for network 48",
            ),
        ] {
            let calls = Arc::new(Mutex::new(Vec::new()));
            let mut created = Vec::new();
            let result = ProverRouter::with_factory(&config()?, |prover| {
                created.push(endpoint(&prover).to_string());
                std::future::ready(if endpoint(&prover) == failed_endpoint {
                    Err(eyre!("endpoint unavailable"))
                } else {
                    Ok(service(prover, calls.clone(), false))
                })
            })
            .await;
            let error = match result {
                Err(error) => error,
                Ok(_) => return Err(eyre!("initialization should have failed")),
            };
            assert_eq!(error.to_string(), context);
            assert_eq!(error.root_cause().to_string(), "endpoint unavailable");
            assert_eq!(created, expected_created);
        }
        Ok(())
    }

    #[tokio::test]
    async fn local_provers_do_not_create_network_overrides() -> eyre::Result<()> {
        for local in [
            ProverType::CpuProver(CpuProverConfig::default()),
            ProverType::MockProver(MockProverConfig::default()),
        ] {
            let mut config = config()?;
            config.prover = local.clone();
            let calls = Arc::new(Mutex::new(Vec::new()));
            let mut created = 0;
            let router = ProverRouter::with_factory(&config, |prover| {
                assert_eq!(prover, local);
                created += 1;
                std::future::ready(Ok(service(prover, calls.clone(), false)))
            })
            .await?;
            assert_eq!(created, 1);
            assert_route(&router, 48, "local").await?;
            assert_eq!(*calls.lock().await, ["local"]);
        }
        Ok(())
    }
}
