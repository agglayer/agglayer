use std::{collections::BTreeMap, sync::Arc};

use agglayer_config::Config;
use agglayer_storage::{
    backup::BackupClient,
    stores::{
        epochs::EpochsStore, pending::PendingStore, state::StateStore, EpochStoreWriter as _,
        PendingCertificateWriter as _, PerEpochWriter as _, StateReader as _, StateWriter as _,
    },
};
use agglayer_types::{
    primitives::Hashable as _, Address, Certificate, CertificateId, CertificateInfo,
    CertificateStatus, Digest, EpochNumber, ExecutionMode, Height, NetworkId, Proof, U256,
};
use insta::assert_snapshot;
use jsonrpsee::{
    core::{client::ClientT, ClientError},
    rpc_params,
};
use pessimistic_proof::unified_bridge::TokenInfo;
use pessimistic_proof_test_suite::forest::Forest;
use rstest::*;
use serde_json::json;

use crate::{
    testutils::{context, raw_rpc, RawRpcContext, TestContext},
    AgglayerServer,
};

/// [`Forest::default`] builds certificates for this network, signed by the
/// test wallet the default config registers as its proof signer.
const NETWORK: u32 = 1;

/// Bridge events for [`Forest::apply_events`], distinct per (height, index).
fn events(height: u64, count: usize) -> Vec<(TokenInfo, U256)> {
    (0..count as u64)
        .map(|index| {
            (
                TokenInfo {
                    origin_network: NetworkId::new(NETWORK),
                    origin_token_address: Address::from([index as u8 + 1; 20]),
                },
                U256::from(1 + height * 100 + index),
            )
        })
        .collect()
}

/// Next certificate in the forest's chain, with `num_exits` bridge exits and
/// `num_claims` imported bridge exits (claims).
fn next_certificate(
    forest: &mut Forest,
    height: u64,
    num_exits: usize,
    num_claims: usize,
) -> Certificate {
    let mut certificate =
        forest.apply_events(&events(height, num_claims), &events(height, num_exits));
    certificate.height = Height::new(height);
    certificate
}

/// Drive a certificate through the same store writes the settlement flow
/// performs: pending insert, epoch packing, cursor + latest-settled pointer,
/// and the local exit tree move.
fn insert_pending(state: &StateStore, pending: &PendingStore, certificate: &Certificate) {
    pending
        .insert_pending_certificate(certificate.network_id, certificate.height, certificate)
        .unwrap();
    state
        .insert_certificate_header(certificate, CertificateStatus::Pending)
        .unwrap();
}

fn settle(
    config: &Arc<Config>,
    state: &Arc<StateStore>,
    pending: &Arc<PendingStore>,
    certificate: &Certificate,
    epoch: EpochNumber,
    start_checkpoint: Option<Height>,
) {
    let certificate_id = certificate.hash();
    insert_pending(state, pending, certificate);
    pending
        .insert_generated_proof(&certificate_id, &Proof::dummy())
        .unwrap();

    let epochs_store = EpochsStore::new(
        config.clone(),
        pending.clone(),
        state.clone(),
        BackupClient::noop(),
    )
    .unwrap();
    let per_epoch = match start_checkpoint {
        None => epochs_store.open(epoch).unwrap(),
        // A fresh epoch opened mid-stream carries the last settled height per
        // network as its start checkpoint.
        Some(last_settled_height) => epochs_store
            .open_with_start_checkpoint(
                epoch,
                BTreeMap::from([(certificate.network_id, last_settled_height)]),
            )
            .unwrap(),
    };
    let (epoch_number, certificate_index) = per_epoch
        .add_certificate(certificate_id, ExecutionMode::Default)
        .unwrap();

    state
        .update_certificate_header_status(&certificate_id, &CertificateStatus::Settled)
        .unwrap();
    state
        .set_latest_settled_certificate_for_network(
            &certificate.network_id,
            &certificate.height,
            &certificate_id,
            &epoch_number,
            &certificate_index,
        )
        .unwrap();

    let mut local_state = state
        .read_local_network_state(certificate.network_id)
        .unwrap()
        .unwrap_or_default();
    let leaves: Vec<Digest> = certificate
        .bridge_exits
        .iter()
        .map(|exit| exit.hash())
        .collect();
    for leaf in &leaves {
        local_state.exit_tree.add_leaf(*leaf).unwrap();
    }
    state
        .write_local_network_state(&certificate.network_id, &local_state, &leaves)
        .unwrap();
}

async fn get_info(context: &TestContext, height: u64) -> Result<CertificateInfo, ClientError> {
    context
        .api_client
        .request(
            "interop_getCertificateInfo",
            rpc_params![NetworkId::new(NETWORK), Height::new(height)],
        )
        .await
}

#[rstest]
#[awt]
#[test_log::test(tokio::test)]
async fn certificate_info_lifecycle(#[future] mut context: TestContext) {
    let mut forest = Forest::default();

    // Unknown height.
    let error = get_info(&context, 0).await.unwrap_err();
    let expected = format!("Resource not found: Certificate({NETWORK}, height 0)");
    assert!(matches!(error, ClientError::Call(obj) if obj.message() == expected));

    // In flight: submitted through the real API, provisional range.
    let certificate_0 = next_certificate(&mut forest, 0, 3, 0);
    let id: CertificateId = context
        .api_client
        .request(
            "interop_sendCertificate",
            rpc_params![certificate_0.clone()],
        )
        .await
        .unwrap();
    assert!(context.certificate_receiver.try_recv().is_ok());
    let info = get_info(&context, 0).await.unwrap();
    assert_eq!(info.certificate_id, id);
    assert_eq!(info.status, CertificateStatus::Pending);
    assert_eq!(info.exit_count, 3);
    assert_eq!(info.leaf_range, 0..3);
    assert_eq!(info.claims, None);

    // Settled: the same height now resolves through the epoch store.
    settle(
        &context.config,
        &context.state_store,
        &context.pending_store,
        &certificate_0,
        EpochNumber::ZERO,
        None,
    );
    let info = get_info(&context, 0).await.unwrap();
    assert_eq!(info.status, CertificateStatus::Settled);
    assert_eq!(info.leaf_range, 0..3);

    // Second settlement in the same epoch: exact range, and the previous
    // height stays served through the walk-back.
    let certificate_1 = next_certificate(&mut forest, 1, 2, 0);
    settle(
        &context.config,
        &context.state_store,
        &context.pending_store,
        &certificate_1,
        EpochNumber::ZERO,
        None,
    );
    let info = get_info(&context, 1).await.unwrap();
    assert_eq!(info.leaf_range, 3..5);
    let info = get_info(&context, 0).await.unwrap();
    assert_eq!(info.leaf_range, 0..3);

    // Queued beyond the in-flight height: provisional ranges from the
    // forward walk over the pending certificates.
    let certificate_2 = next_certificate(&mut forest, 2, 2, 0);
    let certificate_3 = next_certificate(&mut forest, 3, 1, 0);
    insert_pending(&context.state_store, &context.pending_store, &certificate_2);
    insert_pending(&context.state_store, &context.pending_store, &certificate_3);
    let info = get_info(&context, 2).await.unwrap();
    assert_eq!(info.status, CertificateStatus::Pending);
    assert_eq!(info.leaf_range, 5..7);
    let info = get_info(&context, 3).await.unwrap();
    assert_eq!(info.leaf_range, 7..8);
}

#[rstest]
#[awt]
#[test_log::test(tokio::test)]
async fn past_epoch_certificate_is_rejected(#[future] context: TestContext) {
    let mut forest = Forest::default();
    let certificate_0 = next_certificate(&mut forest, 0, 1, 0);
    let certificate_1 = next_certificate(&mut forest, 1, 1, 0);
    settle(
        &context.config,
        &context.state_store,
        &context.pending_store,
        &certificate_0,
        EpochNumber::ZERO,
        None,
    );
    settle(
        &context.config,
        &context.state_store,
        &context.pending_store,
        &certificate_1,
        EpochNumber::new(1),
        Some(Height::ZERO),
    );

    // Settled in a past epoch: rejected before the per-epoch storage is read.
    let error = get_info(&context, 0).await.unwrap_err();
    let expected = format!(
        "Invalid argument: Certificate at height 0 for network {NETWORK} was settled in a past \
         epoch (0), latest epoch with a settlement is 1"
    );
    assert!(matches!(error, ClientError::Call(obj) if obj.message() == expected));

    // The latest epoch with a settlement is still served.
    let info = get_info(&context, 1).await.unwrap();
    assert_eq!(info.status, CertificateStatus::Settled);
    assert_eq!(info.leaf_range, 1..2);
}

#[rstest]
#[awt]
#[test_log::test(tokio::test)]
async fn certificate_info(#[future] raw_rpc: RawRpcContext) {
    let mut forest = Forest::default();
    let certificate = next_certificate(&mut forest, 0, 2, 1);
    settle(
        &raw_rpc.config,
        &raw_rpc.state_store,
        &raw_rpc.pending_store,
        &certificate,
        EpochNumber::ZERO,
        None,
    );
    let rpc = raw_rpc.rpc.into_rpc();

    let payload = json!({
        "jsonrpc": "2.0",
        "method": "interop_getCertificateInfo",
        "params": [NETWORK, 0, true],
        "id": 0
    });
    let (response, _) = rpc
        .raw_json_request(&serde_json::to_string(&payload).unwrap(), 1)
        .await
        .unwrap();
    let json = serde_json::from_str::<serde_json::Value>(&response).unwrap();
    let json = serde_json::to_string_pretty(&json).unwrap();

    // The forest generates random bridge exits, making the certificate id the
    // only run-dependent field.
    insta::with_settings!({
        filters => vec![(r#""certificate_id": "0x[0-9a-f]{64}""#, r#""certificate_id": "[certificate_id]""#)]
    }, {
        assert_snapshot!(
            "get_certificate_info__found",
            json,
            &serde_json::to_string_pretty(&payload).unwrap()
        );
    });
}
