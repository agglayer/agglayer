use std::time::Duration;

use agglayer_types::{Address, Certificate, LocalNetworkStateData, Proof};
use pessimistic_proof::LocalNetworkState;
use tower::timeout::TimeoutLayer;
use tower::{service_fn, Service, ServiceBuilder, ServiceExt};
use tower::limit::ConcurrencyLimitLayer;
use agglayer_config::prover::{AgglayerProverType, MockProverConfig, ProverConfig};
use crate::executor::{Executor, Request, Response};

#[tokio::test]
async fn executor_normal_behavior() {
    let network = Executor::build_network_service(
        Duration::from_secs(1),
        service_fn(|r: Request| async move {
            let Proof::SP1(mut proof) =
                Proof::new_for_test(&r.initial_state.into(), &r.batch_header);
            proof.sp1_version = "from_network".to_string();

            Ok(Response { proof })
        }),
    );

    let local = Executor::build_local_service(
        Duration::from_secs(1),
        1,
        service_fn(|_: Request| async { panic!("Shouldn't be called") }),
    );

    let mut executor = Executor::new_with_services(network, Some(local));

    let mut state = LocalNetworkStateData::default();
    let certificate = Certificate::new_for_test(0.into(), 0);
    let signer = certificate.get_signer();
    let batch_header = state
        .apply_certificate(
            &certificate,
            signer,
            certificate.l1_info_root().unwrap().unwrap_or_default(),
        )
        .unwrap();

    let result = executor
        .call(Request {
            initial_state: LocalNetworkState::default(),
            batch_header,
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().proof.sp1_version, "from_network");
}

#[tokio::test]
async fn executor_normal_behavior_only_network() {
    let network = Executor::build_network_service(
        Duration::from_secs(1),
        service_fn(|r: Request| async move {
            let Proof::SP1(mut proof) =
                Proof::new_for_test(&r.initial_state.into(), &r.batch_header);
            proof.sp1_version = "from_network".to_string();

            Ok(Response { proof })
        }),
    );

    let mut executor = Executor::new_with_services(network, None);

    let mut state = LocalNetworkStateData::default();
    let certificate = Certificate::new_for_test(0.into(), 0);
    let signer = certificate.get_signer();

    let batch_header = state
        .apply_certificate(
            &certificate,
            signer,
            certificate.l1_info_root().unwrap().unwrap_or_default(),
        )
        .unwrap();

    let result = executor
        .call(Request {
            initial_state: LocalNetworkState::default(),
            batch_header,
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().proof.sp1_version, "from_network");
}

#[tokio::test]
async fn executor_fallback_behavior_cpu() {
    let network = Executor::build_network_service(
        Duration::from_secs(1),
        service_fn(|_: Request| async {
            Err(crate::executor::Error::ProverFailed("failure".to_string()))
        }),
    );

    let local = Executor::build_local_service(
        Duration::from_secs(1),
        1,
        service_fn(|r: Request| async move {
            let Proof::SP1(mut proof) =
                Proof::new_for_test(&r.initial_state.into(), &r.batch_header);
            proof.sp1_version = "from_local".to_string();

            Ok(Response { proof })
        }),
    );

    let mut executor = Executor::new_with_services(network, Some(local));

    let mut state = LocalNetworkStateData::default();
    let certificate = Certificate::new_for_test(0.into(), 0);
    let signer = certificate.get_signer();
    let batch_header = state
        .apply_certificate(
            &certificate,
            signer,
            certificate.l1_info_root().unwrap().unwrap_or_default(),
        )
        .unwrap();

    let result = executor
        .call(Request {
            initial_state: LocalNetworkState::default(),
            batch_header,
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().proof.sp1_version, "from_local");
}

#[tokio::test]
async fn executor_fallback_because_of_timeout_cpu() {
    let network = Executor::build_network_service(
        Duration::from_millis(100),
        service_fn(|_: Request| async {
            tokio::time::sleep(Duration::from_secs(20)).await;
            let Proof::SP1(mut proof) = Proof::dummy();
            proof.sp1_version = "from_network".to_string();

            Ok(Response { proof })
        }),
    );

    let local = Executor::build_local_service(
        Duration::from_secs(1),
        1,
        service_fn(|_: Request| async {
            let Proof::SP1(mut proof) = Proof::dummy();
            proof.sp1_version = "from_local".to_string();

            Ok(Response { proof })
        }),
    );

    let mut executor = Executor::new_with_services(network, Some(local));

    let signer = Address::new([0; 20]);
    let mut state = LocalNetworkStateData::default();
    let certificate =
        Certificate::new_for_test(0.into(), 0).with_new_local_exit_root(state.exit_tree.get_root());
    let batch_header = state
        .apply_certificate(
            &certificate,
            signer,
            certificate.l1_info_root().unwrap().unwrap_or_default(),
        )
        .unwrap();

    let result = executor
        .call(Request {
            initial_state: LocalNetworkState::default(),
            batch_header,
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().proof.sp1_version, "from_local");
}

#[tokio::test]
async fn executor_fails_because_of_timeout_cpu() {
    let network = Executor::build_network_service(
        Duration::from_millis(100),
        service_fn(|r: Request| async move {
            tokio::time::sleep(Duration::from_secs(20)).await;
            let Proof::SP1(mut proof) =
                Proof::new_for_test(&r.initial_state.into(), &r.batch_header);
            proof.sp1_version = "from_network".to_string();

            Ok(Response { proof })
        }),
    );

    let local = Executor::build_local_service(
        Duration::from_millis(100),
        1,
        service_fn(|r: Request| async move {
            tokio::time::sleep(Duration::from_secs(20)).await;
            let Proof::SP1(mut proof) =
                Proof::new_for_test(&r.initial_state.into(), &r.batch_header);
            proof.sp1_version = "from_local".to_string();

            Ok(Response { proof })
        }),
    );

    let mut executor = ServiceBuilder::new()
        .layer(TimeoutLayer::new(Duration::from_millis(100)))
        .service(Executor::new_with_services(network, Some(local)));

    let signer = Address::new([0; 20]);
    let mut state = LocalNetworkStateData::default();
    let certificate =
        Certificate::new_for_test(0.into(), 0).with_new_local_exit_root(state.exit_tree.get_root());
    let batch_header = state
        .apply_certificate(
            &certificate,
            signer,
            certificate.l1_info_root().unwrap().unwrap_or_default(),
        )
        .unwrap();

    let result = executor
        .call(Request {
            initial_state: LocalNetworkState::default(),
            batch_header,
        })
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn executor_fails_because_of_concurrency_cpu() {
    let network = Executor::build_network_service(
        Duration::from_millis(100),
        service_fn(|r: Request| async move {
            tokio::time::sleep(Duration::from_secs(20)).await;
            let Proof::SP1(mut proof) =
                Proof::new_for_test(&r.initial_state.into(), &r.batch_header);
            proof.sp1_version = "from_network".to_string();

            Ok(Response { proof })
        }),
    );

    let local = Executor::build_local_service(
        Duration::from_secs(20),
        1,
        service_fn(|r: Request| async move {
            tokio::time::sleep(Duration::from_secs(20)).await;
            let Proof::SP1(mut proof) =
                Proof::new_for_test(&r.initial_state.into(), &r.batch_header);
            proof.sp1_version = "from_local".to_string();

            Ok(Response { proof })
        }),
    );

    let mut executor = ServiceBuilder::new()
        .layer(TimeoutLayer::new(Duration::from_secs(1)))
        .service(Executor::new_with_services(network, Some(local)));

    let signer = Address::new([0; 20]);
    let mut state = LocalNetworkStateData::default();
    let certificate =
        Certificate::new_for_test(0.into(), 0).with_new_local_exit_root(state.exit_tree.get_root());
    let batch_header = state
        .apply_certificate(
            &certificate,
            signer,
            certificate.l1_info_root().unwrap().unwrap_or_default(),
        )
        .unwrap();

    let mut executor2 = executor.clone();
    let batch_header2 = batch_header.clone();

    tokio::spawn(async move {
        executor
            .ready()
            .await
            .unwrap()
            .call(Request {
                initial_state: LocalNetworkState::default(),
                batch_header,
            })
            .await
    });

    let result = executor2
        .ready()
        .await
        .unwrap()
        .call(Request {
            initial_state: LocalNetworkState::default(),
            batch_header: batch_header2,
        })
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn executor_normal_behavior_mock_prover() {
    let mock_prover_config = ProverConfig {
        primary_prover: AgglayerProverType::MockProver(MockProverConfig::default()),
            ..Default::default()
    };
    let mut executor = tower::ServiceBuilder::new()
        .timeout(Duration::from_secs(100))
        .layer(ConcurrencyLimitLayer::new(10))
        .service(Executor::new(&mock_prover_config))
        .into_inner()
        .boxed();

    let mut state = LocalNetworkStateData::default();
    let certificate = Certificate::new_for_test(0.into(), 0);
    let signer = certificate.get_signer();

    let batch_header = state
        .apply_certificate(
            &certificate,
            signer,
            certificate.l1_info_root().unwrap().unwrap_or_default(),
        )
        .unwrap();

    let executor = executor
        .ready()
        .await
        .expect("valid executor");

    println!(">>>>>>>>>>>>>>>>> CHECKPOINT 1");


    let result = executor
        .call(Request {
            initial_state: LocalNetworkState::default(),
            batch_header,
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().proof.sp1_version, "from_network");
}
