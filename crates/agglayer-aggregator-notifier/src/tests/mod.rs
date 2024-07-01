use agglayer_config::certificate_orchestrator::prover::ProverConfig;

use crate::AggregatorNotifier;

mod sp1 {
    use agglayer_certificate_orchestrator::EpochPacker;

    use super::*;

    #[tokio::test]
    #[cfg_attr(not(feature = "sp1-network"), ignore)]
    async fn aggregator_notifier_for_network() {
        let config = ProverConfig::SP1Network {};
        let notifier = AggregatorNotifier::try_from(config);
        assert!(notifier.is_ok());

        let notifier = notifier.unwrap();

        let _result = notifier.pack(1, vec![]).unwrap().await;
    }

    #[tokio::test]
    async fn aggregator_notifier_for_local() {
        let config = ProverConfig::SP1Local {};
        let notifier = AggregatorNotifier::try_from(config);
        assert!(notifier.is_ok());

        let notifier = notifier.unwrap();

        let _result = notifier.pack(1, vec![]).unwrap().await;
    }

    #[tokio::test]
    async fn aggregator_notifier_for_mock() {
        let config = ProverConfig::SP1Mock {};
        let notifier = AggregatorNotifier::try_from(config);
        assert!(notifier.is_ok());

        let notifier = notifier.unwrap();

        let _result = notifier.pack(1, vec![]).unwrap().await;
    }
}
