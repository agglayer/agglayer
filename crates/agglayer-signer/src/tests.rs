use alloy::{
    consensus::{SignableTransaction, TxEip1559, TypedTransaction},
    signers::Signer,
};
use alloy_primitives::{Address, B256, U256};

use super::*;

fn testing_local_wallet() -> ConfiguredSigner {
    let private_key = [0x55; 32];
    let signer = PrivateKeySigner::from_slice(&private_key).unwrap();
    ConfiguredSigner::Local(signer)
}

#[rstest::rstest]
#[case(testing_local_wallet())]
#[tokio::test]
async fn signer_works(#[case] signer: ConfiguredSigner) {
    let mut tx = TxEip1559 {
        chain_id: 1337,
        nonce: 123,
        gas_limit: 21000,
        max_fee_per_gas: 20_000_000_000,
        max_priority_fee_per_gas: 1_000_000_000,
        to: alloy_primitives::TxKind::Call(Address::from([0x11; 20])),
        value: U256::from(1_000_000_000_u64),
        access_list: Default::default(),
        input: Default::default(),
    };
    let typed_tx = TypedTransaction::Eip1559(tx.clone());

    // Check the signature returned by the signer successfully verifies
    let signature = signer.sign_transaction_typed(&typed_tx).await.unwrap();

    // Verify the signature by checking the recovered address matches the signer
    // address
    let recovered_address = signature
        .recover_address_from_prehash(&typed_tx.signature_hash())
        .unwrap();
    assert_eq!(recovered_address, Signer::address(&signer));

    // Change the tx to and verify that the signature is invalid
    tx.to = alloy_primitives::TxKind::Call(Address::from([0x22; 20]));
    let changed_typed_tx = TypedTransaction::Eip1559(tx);
    let recovered_address = signature
        .recover_address_from_prehash(&changed_typed_tx.signature_hash())
        .unwrap();
    assert_ne!(recovered_address, Signer::address(&signer));

    // Test message signing as well
    let message = b"test message";
    let message_signature = signer.sign_message(message).await.unwrap();
    let recovered_from_message = message_signature.recover_address_from_msg(message).unwrap();
    assert_eq!(recovered_from_message, Signer::address(&signer));
}

#[rstest::rstest]
#[case(testing_local_wallet())]
#[tokio::test]
async fn sign_hash_works(#[case] signer: ConfiguredSigner) {
    let test_hash = B256::from([0x42; 32]);

    // Test the sign hash functionality.
    let signature = signer.sign_hash(&test_hash).await.unwrap();

    // Verify the signature is correct.
    let recovered_address = signature.recover_address_from_prehash(&test_hash).unwrap();
    assert_eq!(recovered_address, Signer::address(&signer));
}

#[tokio::test]
async fn local_signer_type_detection() {
    // Test local signer type detection
    let signer = testing_local_wallet();
    assert!(signer.is_local());
    assert!(!signer.is_kms());
}

#[test]
fn constructor_methods_work() {
    let private_key = [0x55; 32];
    let local_signer = PrivateKeySigner::from_slice(&private_key).unwrap();

    // Test from_local constructor
    let configured_signer = ConfiguredSigner::from_local(local_signer);
    assert!(configured_signer.is_local());
    assert!(!configured_signer.is_kms());
}
