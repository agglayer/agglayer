use agglayer_config::certificate_orchestrator::prover::ProverConfig;

use crate::AggregatorNotifier;

mod sp1 {
    #[derive(Debug, Clone, PartialEq)]
    struct DummyStore {}
    impl StateReader for DummyStore {
        fn get_active_networks(
            &self,
        ) -> Result<Vec<agglayer_types::NetworkId>, agglayer_storage::error::Error> {
            todo!()
        }

        fn get_certificate_header_by_cursor(
            &self,
            _network_id: agglayer_types::NetworkId,
            _height: agglayer_types::Height,
        ) -> Result<Option<agglayer_types::CertificateHeader>, agglayer_storage::error::Error>
        {
            todo!()
        }

        fn get_current_settled_height(
            &self,
        ) -> Result<
            Vec<(
                agglayer_types::NetworkId,
                agglayer_types::Height,
                agglayer_types::CertificateId,
            )>,
            agglayer_storage::error::Error,
        > {
            todo!()
        }
    }
    impl PendingCertificateReader for DummyStore {
        fn get_certificate(
            &self,
            _network_id: agglayer_types::NetworkId,
            _height: agglayer_types::Height,
        ) -> Result<Option<agglayer_types::Certificate>, agglayer_storage::error::Error> {
            todo!()
        }

        fn multi_get_certificate(
            &self,
            _keys: &[(agglayer_types::NetworkId, agglayer_types::Height)],
        ) -> Result<Vec<Option<agglayer_types::Certificate>>, agglayer_storage::error::Error>
        {
            todo!()
        }

        fn multi_get_proof(
            &self,
            _keys: &[agglayer_types::CertificateId],
        ) -> Result<Vec<Option<agglayer_types::Proof>>, agglayer_storage::error::Error> {
            todo!()
        }

        fn get_proof(
            &self,
            _certificate_id: agglayer_types::CertificateId,
        ) -> Result<Option<agglayer_types::Proof>, agglayer_storage::error::Error> {
            todo!()
        }
    }
    use agglayer_certificate_orchestrator::EpochPacker;
    use agglayer_storage::stores::{PendingCertificateReader, StateReader};

    use super::*;

    #[tokio::test]
    #[rstest::rstest]
    #[case(ProverConfig::SP1Local {})]
    #[case(ProverConfig::SP1Mock {})]
    async fn aggregator_notifier_can_be_implemented(#[case] config: ProverConfig) {
        use std::sync::Arc;

        let store = Arc::new(DummyStore {});
        let notifier = AggregatorNotifier::<(), DummyStore, DummyStore>::try_new(
            &config,
            store.clone(),
            store,
        );

        assert!(notifier.is_ok());

        let notifier = notifier.unwrap();

        let _result = notifier.pack(1, vec![]).unwrap().await;
    }
}
