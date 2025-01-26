#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_primitives::{Signature, B256};

pub fn main() {
    let signature_bytes = sp1_zkvm::io::read::<Vec<u8>>();
    let signature = Signature::try_from(signature_bytes.as_slice()).unwrap();

    let public_values = sp1_zkvm::io::read::<Vec<u8>>();
    // `public_values = combined_hash (32 bytes) || signer_address (20 bytes padded to 32)`
    assert_eq!(public_values.len(), 64);
    let combined_hash = &public_values[0..32];
    assert!(public_values[32..44].iter().all(|&b| b==0));
    let signer = &public_values[44..];

    let recovered_signer = signature.recover_address_from_prehash(&B256::new(combined_hash.try_into().unwrap())).expect("Invalid signature");
    assert_eq!(recovered_signer.as_slice(), signer);

    sp1_zkvm::io::commit_slice(&public_values);
}
