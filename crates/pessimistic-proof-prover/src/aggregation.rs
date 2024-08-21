use sp1_sdk::{
    HashableKey, ProverClient, SP1Proof, SP1ProofWithPublicValues, SP1Stdin, SP1VerifyingKey,
};

const AGGREGATION_ELF: &[u8] =
    include_bytes!("../../pessimistic-proof-aggregation-program/elf/riscv32im-succinct-zkvm-elf");

/// Generate an aggregation proof of several pessimistic proofs.
pub fn prove_pessimistic_proofs_aggregation(
    proofs: Vec<SP1ProofWithPublicValues>,
    vk: SP1VerifyingKey,
) -> SP1ProofWithPublicValues {
    let client = ProverClient::new();
    let (aggregation_pk, _) = client.setup(AGGREGATION_ELF);
    let mut stdin = SP1Stdin::new();

    let vkey = vk.hash_u32();
    stdin.write::<[u32; 8]>(&vkey);

    let public_values = proofs.iter().map(|proof| proof.public_values.to_vec()).collect::<Vec<_>>();
    stdin.write::<Vec<Vec<u8>>>(&public_values);

    for proof in proofs {
        let SP1Proof::Compressed(proof) = proof.proof else {
            panic!()
        };
        stdin.write_proof(proof, vk.vk.clone());
    }

    client.prove(&aggregation_pk, stdin).run().expect("proving failed")
}

/// Verification key for the aggregation proof.
pub fn vk_pessimistic_proofs_aggregation() -> SP1VerifyingKey {
    let client = ProverClient::new();
    let (_aggregation_pk, aggregation_vk) = client.setup(AGGREGATION_ELF);
    aggregation_vk
}
