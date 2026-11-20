use std::sync::Arc;

use agglayer_config::Config;
use agglayer_contracts::{contracts::PolygonRollupManager, L1RpcClient};
use agglayer_storage::tests::mocks::{
    MockDebugStore, MockEpochsStore, MockPendingStore, MockStateStore,
};
use agglayer_types::{Address, Certificate, Height, SignerError};
use alloy::providers::{mock::Asserter, ProviderBuilder};

use crate::error::SignatureVerificationError;

/// Certificate signatures are checked against `proof_signers`; only a network
/// without a configured proof signer falls back to L1, which the empty mock
/// rejects.
#[tokio::test]
async fn verify_cert_signature() {
    let signer1: Address = Certificate::wallet_for_test(1.into()).address().into();
    let signer3: Address = Certificate::wallet_for_test(3.into()).address().into();
    let mut config = Config::default();
    // Proof signer for network 1 is ok
    config.proof_signers.insert(1, signer1);
    // Proof signer for network 2 is wrong
    config.proof_signers.insert(2, signer3);
    // No proof signer for network 3

    let provider = Arc::new(ProviderBuilder::new().connect_mocked_client(Asserter::new()));
    let l1_rpc = L1RpcClient::new(
        provider.clone(),
        PolygonRollupManager::PolygonRollupManagerInstance::new(
            Address::ZERO.into(),
            (*provider).clone(),
        ),
        Address::ZERO.into(),
        (0u32, [0u8; 32]),
    );

    let service = crate::AgglayerService::new(
        tokio::sync::mpsc::channel(1).0,
        Arc::new(MockPendingStore::new()),
        Arc::new(MockStateStore::new()),
        Arc::new(MockDebugStore::new()),
        Arc::new(MockEpochsStore::new()),
        Arc::new(config),
        Arc::new(l1_rpc),
    );

    {
        // valid signature
        let signed_cert = Certificate::new_for_test_v5(1.into(), Height::ZERO);
        assert!(service.verify_cert_signature(&signed_cert).await.is_ok());
    }

    {
        // valid (V5) signature with wrong signer
        let signed_cert = Certificate::new_for_test_v5(2.into(), Height::ZERO);
        assert!(matches!(
            service.verify_cert_signature(&signed_cert).await,
            Err(SignatureVerificationError::InvalidPessimisticProofSignature(
                SignerError::InvalidPessimisticProofSignature { expected_signer }
            ))
            if expected_signer == signer3
        ));
    }

    {
        // valid signature, but no proof signer configured: the trusted
        // sequencer must come from L1, and the empty mock has no answer
        let signed_cert = Certificate::new_for_test_v5(3.into(), Height::ZERO);
        assert!(matches!(
            service.verify_cert_signature(&signed_cert).await,
            Err(SignatureVerificationError::UnableToRetrieveTrustedSequencerAddress(network_id))
            if network_id.to_u32() == 3
        ));
    }

    {
        // wrong signature with valid signer: a V5 signature invalidated by
        // mutating the certificate after signing (deliberately not re-signed)
        let mut signed_cert = Certificate::new_for_test_v5(1.into(), Height::ZERO);
        signed_cert.new_local_exit_root.as_mut()[0] += 1;
        assert!(matches!(
            service.verify_cert_signature(&signed_cert).await,
            Err(SignatureVerificationError::InvalidPessimisticProofSignature(
                SignerError::InvalidPessimisticProofSignature { expected_signer }
            ))
            if expected_signer == signer1
        ));
    }
}
