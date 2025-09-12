use std::sync::Arc;

use agglayer_config::Config;
use agglayer_storage::tests::mocks::{
    MockDebugStore, MockEpochsStore, MockPendingStore, MockStateStore,
};
use alloy::{
    providers::{
        mock::{Asserter, MockTransport},
        ProviderBuilder,
    },
    signers::k256::elliptic_curve::rand_core::le,
};

#[test]
fn transient_network_info() {
    let certificate_sender = tokio::sync::mpsc::channel(1).0;

    let pending_store = MockPendingStore::new();
    let state = MockStateStore::new();
    let debug_store = MockDebugStore::new();
    let epochs_store = MockEpochsStore::new();
    let config = Arc::new(Config::default());

    // Create a mock provider for the default case
    let asserter = Asserter::new();
    let _transport = MockTransport::new(asserter.clone());
    let l1_rpc_provider = Arc::new(ProviderBuilder::new().on_mocked_client(asserter));

    let service = crate::AgglayerService::new(
        certificate_sender,
        Arc::new(pending_store),
        Arc::new(state),
        Arc::new(debug_store),
        Arc::new(epochs_store),
        config,
        l1_rpc_provider,
    );
}
