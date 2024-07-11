use ethers::types::{Eip1559TransactionRequest, H160};

use super::*;

fn testing_local_wallet() -> ConfiguredSigner {
    ConfiguredSigner::Local(LocalWallet::from_bytes([0x55; 32].as_slice()).unwrap())
}

#[rstest::rstest]
#[case(testing_local_wallet())]
#[tokio::test]
async fn signer_works(#[case] signer: ConfiguredSigner) {
    let txn = Eip1559TransactionRequest {
        from: Some(signer.address()),
        to: Some(H160([0x11; 20]).into()),
        gas: None,
        value: Some(1_000_000_000_u64.into()),
        data: None,
        nonce: Some(123_u64.into()),
        access_list: Default::default(),
        max_priority_fee_per_gas: None,
        max_fee_per_gas: None,
        chain_id: Some(1337.into()),
    };
    let txn = TypedTransaction::Eip1559(txn);

    // Check the signature returned by the signer successfully verifies
    let signature = signer.sign_transaction(&txn).await.unwrap();
    assert!(signature.verify(txn.sighash(), signer.address()).is_ok());

    // Change the amount, check the signature no longer verifies
    let txn = {
        let mut txn = txn;
        match &mut txn {
            TypedTransaction::Eip1559(txn) => {
                txn.value = Some(2_000_000_000_u64.into());
            }
            _ => panic!("Must be Eip1559, constructed just a few lines above"),
        };
        txn
    };

    assert!(signature.verify(txn.sighash(), signer.address()).is_err());
}
