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

async fn assert_route(router: &ProverRouter, network_id: u32, expected: &str) -> eyre::Result<()> {
    let error = router
        .for_network(network_id.into())
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
    let router = ProverRouter::with_factory(&config()?, |prover| {
        std::future::ready(Ok(service(prover, calls.clone(), true)))
    })
    .await?;

    for network_id in [48, 99] {
        assert!(router
            .for_network(network_id.into())
            .oneshot(request())
            .await
            .is_err());
    }
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
