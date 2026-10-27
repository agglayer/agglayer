use std::{collections::BTreeMap, sync::Arc};

use agglayer_config::Config;
use agglayer_rpc::AgglayerService;
use agglayer_storage::tests::mocks::{
    MockDebugStore, MockEpochsStore, MockPendingStore, MockStateStore,
};
use prost::Message as _;
use prost_types::FileDescriptorSet;

use super::certificate_submission::L1Rpc;
use crate::Server;

#[test]
fn reflection_descriptors_include_all_dependencies() {
    let config = Arc::new(Config::default());
    let (sender, _receiver) = tokio::sync::mpsc::channel(1);
    let service = Arc::new(AgglayerService::new(
        sender,
        Arc::new(MockPendingStore::new()),
        Arc::new(MockStateStore::new()),
        Arc::new(MockDebugStore::new()),
        Arc::new(MockEpochsStore::new()),
        config.clone(),
        Arc::new(L1Rpc),
    ));
    let builder = Server::with_config(config, service);
    let files: BTreeMap<_, _> = builder
        .reflection_fds
        .iter()
        .flat_map(|encoded| FileDescriptorSet::decode(*encoded).unwrap().file)
        .map(|file| (file.name().to_owned(), file))
        .collect();

    for service in [
        "CertificateSubmissionService",
        "ConfigurationService",
        "NodeStateService",
    ] {
        assert!(
            files.values().any(|file| {
                file.package() == "agglayer.node.v1"
                    && file.service.iter().any(|s| s.name() == service)
            }),
            "missing descriptor for {service}"
        );
    }

    for (name, file) in &files {
        for dependency in &file.dependency {
            assert!(
                files.contains_key(dependency),
                "{name} imports missing reflection descriptor {dependency}"
            );
        }
    }
}
