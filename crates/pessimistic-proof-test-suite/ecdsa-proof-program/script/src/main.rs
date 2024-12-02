use sp1_sdk::{ProverClient, SP1Stdin, utils};
use alloy::signers::{local::PrivateKeySigner, SignerSync};
use rand::random;

const ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");

fn main() {

    utils::setup_logger();

    let signer = PrivateKeySigner::random();

    let combined_hash: [u8; 32] = random();
    let signature = signer.sign_hash_sync(&combined_hash.into()).unwrap();

    let mut public_values = vec![0; 64];
    public_values[..32].copy_from_slice(&combined_hash);
    public_values[44..].copy_from_slice(signer.address().as_slice());

    dbg!(&public_values);

    // Generate proof.
    let mut stdin = SP1Stdin::new();
    
    stdin.write(&signature.as_bytes().to_vec());
    stdin.write(&public_values);

    let client = ProverClient::new();
    let (pk, vk) = client.setup(ELF);
    let proof = client.prove(&pk, stdin).compressed().run().expect("proving failed");
    
    dbg!("done proof");

    // Verify proof.
    client.verify(&proof, &vk).expect("verification failed");
}
