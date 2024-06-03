use agglayer_config::certificate_orchestrator::prover::ProverConfig;

use crate::AggregatorNotifier;

mod sp1 {
    use agglayer_certificate_orchestrator::EpochPacker;

    use super::*;

    #[tokio::test]
    #[rstest::rstest]
    #[case(ProverConfig::SP1Local {})]
    #[case(ProverConfig::SP1Mock {})]
    async fn aggregator_notifier_can_be_implemented(#[case] config: ProverConfig) {
        let notifier = AggregatorNotifier::<()>::try_new(&config);

        assert!(notifier.is_ok());

        let notifier = notifier.unwrap();

        let _result = notifier.pack(1, vec![]).unwrap().await;
    }
}
