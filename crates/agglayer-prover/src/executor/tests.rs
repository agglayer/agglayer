use std::time::Duration;

use agglayer_types::{Certificate, LocalNetworkStateData, Proof};
use pessimistic_proof::LocalNetworkState;
use tower::timeout::TimeoutLayer;
use tower::{service_fn, Service, ServiceBuilder, ServiceExt};

use crate::executor::{Executor, Request, Response};

#[tokio::test]
async fn executor_normal_behavior() {
    let network = Executor::build_network_service(
        Duration::from_secs(1),
        service_fn(|_: Request| async {
            let Proof::SP1(mut proof) = Proof::new_for_test();
            proof.sp1_version = "from_network".to_string();

            Ok(Response { proof })
        }),
    );

    let local = Executor::build_local_service(
        Duration::from_secs(1),
        1,
        service_fn(|_: Request| async { panic!("Shouldn't be called") }),
    );

    let mut executor = Executor::new_with_services(Some(network), Some(local));

    let signer = pessimistic_proof::Address::new([0; 20]);
    let mut state = LocalNetworkStateData::default();
    let certificate = Certificate {
        new_local_exit_root: state.exit_tree.get_root(),
        ..Default::default()
    };
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
        service_fn(|_: Request| async {
            let Proof::SP1(mut proof) = Proof::new_for_test();
            proof.sp1_version = "from_network".to_string();

            Ok(Response { proof })
        }),
    );

    let mut executor = Executor::new_with_services(Some(network), None);

    let signer = pessimistic_proof::Address::new([0; 20]);
    let mut state = LocalNetworkStateData::default();
    let certificate = Certificate {
        new_local_exit_root: state.exit_tree.get_root(),
        ..Default::default()
    };
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
        service_fn(|_: Request| async {
            let Proof::SP1(mut proof) = Proof::new_for_test();
            proof.sp1_version = "from_local".to_string();

            Ok(Response { proof })
        }),
    );

    let mut executor = Executor::new_with_services(Some(network), Some(local));

    let signer = pessimistic_proof::Address::new([0; 20]);
    let mut state = LocalNetworkStateData::default();
    let certificate = Certificate {
        new_local_exit_root: state.exit_tree.get_root(),
        ..Default::default()
    };
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
            let Proof::SP1(mut proof) = Proof::new_for_test();
            proof.sp1_version = "from_network".to_string();

            Ok(Response { proof })
        }),
    );

    let local = Executor::build_local_service(
        Duration::from_secs(1),
        1,
        service_fn(|_: Request| async {
            let Proof::SP1(mut proof) = Proof::new_for_test();
            proof.sp1_version = "from_local".to_string();

            Ok(Response { proof })
        }),
    );

    let mut executor = Executor::new_with_services(Some(network), Some(local));

    let signer = pessimistic_proof::Address::new([0; 20]);
    let mut state = LocalNetworkStateData::default();
    let certificate = Certificate {
        new_local_exit_root: state.exit_tree.get_root(),
        ..Default::default()
    };
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
        service_fn(|_: Request| async {
            tokio::time::sleep(Duration::from_secs(20)).await;
            let Proof::SP1(mut proof) = Proof::new_for_test();
            proof.sp1_version = "from_network".to_string();

            Ok(Response { proof })
        }),
    );

    let local = Executor::build_local_service(
        Duration::from_millis(100),
        1,
        service_fn(|_: Request| async {
            tokio::time::sleep(Duration::from_secs(20)).await;
            let Proof::SP1(mut proof) = Proof::new_for_test();
            proof.sp1_version = "from_local".to_string();

            Ok(Response { proof })
        }),
    );

    let mut executor = ServiceBuilder::new()
        .layer(TimeoutLayer::new(Duration::from_millis(100)))
        .service(Executor::new_with_services(Some(network), Some(local)));

    let signer = pessimistic_proof::Address::new([0; 20]);
    let mut state = LocalNetworkStateData::default();
    let certificate = Certificate {
        new_local_exit_root: state.exit_tree.get_root(),
        ..Default::default()
    };
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
        service_fn(|_: Request| async {
            tokio::time::sleep(Duration::from_secs(20)).await;
            let Proof::SP1(mut proof) = Proof::new_for_test();
            proof.sp1_version = "from_network".to_string();

            Ok(Response { proof })
        }),
    );

    let local = Executor::build_local_service(
        Duration::from_secs(20),
        1,
        service_fn(|_: Request| async {
            tokio::time::sleep(Duration::from_secs(20)).await;
            let Proof::SP1(mut proof) = Proof::new_for_test();
            proof.sp1_version = "from_local".to_string();

            Ok(Response { proof })
        }),
    );

    let mut executor = ServiceBuilder::new()
        .layer(TimeoutLayer::new(Duration::from_secs(1)))
        .service(Executor::new_with_services(Some(network), Some(local)));

    let signer = pessimistic_proof::Address::new([0; 20]);
    let mut state = LocalNetworkStateData::default();
    let certificate = Certificate {
        new_local_exit_root: state.exit_tree.get_root(),
        ..Default::default()
    };
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
