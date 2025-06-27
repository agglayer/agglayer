use alloy::{
    consensus::{SignableTransaction, TxEip1559, TypedTransaction},
    signers::Signer,
};
use alloy_primitives::{Address, U256};

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
    let tx = TxEip1559 {
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
    let typed_tx = TypedTransaction::Eip1559(tx);

    // Check the signature returned by the signer successfully verifies
    let signature = signer.sign_transaction_typed(&typed_tx).await.unwrap();

    // Verify the signature by checking the recovered address matches the signer
    // address
    let recovered_address = signature
        .recover_address_from_prehash(&typed_tx.signature_hash())
        .unwrap();
    assert_eq!(recovered_address, Signer::address(&signer));

    // Test message signing as well
    let message = b"test message";
    let message_signature = signer.sign_message(message).await.unwrap();
    let recovered_from_message = message_signature.recover_address_from_msg(message).unwrap();
    assert_eq!(recovered_from_message, Signer::address(&signer));
}
