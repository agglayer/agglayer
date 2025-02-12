#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_primitives::B256;
use ecdsa_proof_lib::AggchainECDSA;
use tiny_keccak::{Hasher, Keccak};

type Digest = [u8; 32];

pub fn keccak256_combine<I, T>(items: I) -> Digest
where
    I: IntoIterator<Item = T>,
    T: AsRef<[u8]>,
{
    let mut hasher = Keccak::v256();
    for data in items {
        hasher.update(data.as_ref());
    }

    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    output
}

pub fn main() {
    let aggchain_ecdsa: AggchainECDSA = sp1_zkvm::io::read::<AggchainECDSA>();

    let combined_hash = keccak256_combine([
        aggchain_ecdsa.new_local_exit_root,
        aggchain_ecdsa.commit_imported_bridge_exits,
    ]);

    let recovered_signer = aggchain_ecdsa
        .signature
        .recover_address_from_prehash(&B256::new(combined_hash.try_into().unwrap()))
        .expect("Invalid signature");

    assert_eq!(recovered_signer.as_slice(), aggchain_ecdsa.signer);

    sp1_zkvm::io::commit(&aggchain_ecdsa.public_values());
}
