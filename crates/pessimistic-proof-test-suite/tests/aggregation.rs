use agglayer_types::{Certificate, Keccak256Hasher, LocalNetworkStateData, NetworkId};
use pessimistic_proof::{
    aggregation::wrap::ImportedLERWitness,
    bridge_exit::{BridgeExit, LeafType, TokenInfo},
    generate_pessimistic_proof,
    global_index::GlobalIndex,
    imported_bridge_exit::{ImportedBridgeExit, MerkleProof},
    keccak::Digest,
    local_exit_tree::data::{LETMerkleProof, LocalExitTreeData},
    utils::smt::Smt,
    LocalNetworkState, PessimisticProofOutput,
};
use pessimistic_proof_test_suite::{
    forest::{compute_signature_info, Forest},
    sample_data::{ETH, USDC},
    COMBINE_PESSIMISTIC_PROOF_ELF, PESSIMISTIC_PROOF_ELF, WRAP_PESSIMISTIC_PROOF_ELF,
};
use rand::random;
use reth_primitives::{revm_primitives::bitvec::vec, Address, U256};
use sp1_sdk::{utils, HashableKey, ProverClient, SP1Proof, SP1Stdin};

fn u(x: u64) -> U256 {
    x.try_into().unwrap()
}

#[test]
fn test_aggregation() {
    // Setup logging.
    utils::setup_logger();

    sp1_build::build_program("../pp-aggregation-wrap-program");
    dbg!("LOL");

    // Setup: We have three chains, A (mainnet), B, C.
    // B and C both start with 1000 USDC and 1000 ETH.
    // We send 100 USDC from chain B to chain C, and 10 ETH from chain C to chain B.
    // We generate the pessimistic proofs for chains B and C, wrap them, and combine
    // them.

    let network_b = NetworkId::from(1);
    let network_c = NetworkId::from(2);
    let mut state_b = LocalNetworkStateData::default();
    let mut state_c = LocalNetworkStateData::default();
    let mut let_b = LocalExitTreeData::new();
    let mut let_c = LocalExitTreeData::new();

    for state in [&mut state_b, &mut state_c] {
        state
            .balance_tree
            .insert(*USDC, u(1000).to_be_bytes())
            .unwrap();
        state
            .balance_tree
            .insert(*ETH, u(1000).to_be_bytes())
            .unwrap();
    }

    let b_to_c = BridgeExit::new(
        LeafType::Transfer,
        USDC.origin_network,
        USDC.origin_token_address,
        network_c,
        random(),
        u(100),
        vec![],
    );
    let c_to_b = BridgeExit::new(
        LeafType::Transfer,
        ETH.origin_network,
        ETH.origin_token_address,
        network_b,
        random(),
        u(100),
        vec![],
    );

    let prev_ler_b = let_b.get_root();
    let prev_ler_c = let_c.get_root();
    let mut aret = Smt::<Keccak256Hasher, 32>::default();
    aret.insert(*network_b, prev_ler_b).unwrap();
    aret.insert(*network_c, prev_ler_c).unwrap();

    let_b.add_leaf(b_to_c.hash());
    let_c.add_leaf(c_to_b.hash());

    let new_ler_b = let_b.get_root();
    let new_ler_c = let_c.get_root();
    let mut final_aret = Smt::<Keccak256Hasher, 32>::default();
    final_aret.insert(*network_b, new_ler_b).unwrap();
    final_aret.insert(*network_c, new_ler_c).unwrap();

    let imported_b_to_c = ImportedBridgeExit::new(
        b_to_c.clone(),
        MerkleProof {
            proof: let_b.get_proof(0),
            root: new_ler_b,
        },
        GlobalIndex {
            mainnet_flag: false,
            rollup_index: *network_b - 1,
            leaf_index: 0,
        },
    );
    let imported_c_to_b = ImportedBridgeExit::new(
        c_to_b.clone(),
        MerkleProof {
            proof: let_c.get_proof(0),
            root: new_ler_c,
        },
        GlobalIndex {
            mainnet_flag: false,
            rollup_index: *network_c - 1,
            leaf_index: 0,
        },
    );

    let (_combined_hash, signer_b, signature_b) =
        compute_signature_info(new_ler_b, &[imported_c_to_b.clone()]);
    let (_combined_hash, signer_c, signature_c) =
        compute_signature_info(new_ler_c, &[imported_b_to_c.clone()]);

    let certificate_b = Certificate {
        network_id: network_b,
        height: 0,
        prev_local_exit_root: prev_ler_b,
        new_local_exit_root: new_ler_b,
        bridge_exits: vec![b_to_c],
        imported_bridge_exits: vec![imported_c_to_b],
        signature: signature_b,
        metadata: Default::default(),
    };
    let certificate_c = Certificate {
        network_id: network_c,
        height: 0,
        prev_local_exit_root: prev_ler_c,
        new_local_exit_root: new_ler_c,
        bridge_exits: vec![c_to_b],
        imported_bridge_exits: vec![imported_b_to_c],
        signature: signature_c,
        metadata: Default::default(),
    };

    let mbh_b = state_b
        .make_multi_batch_header(&certificate_b, signer_b)
        .unwrap();
    let mbh_c = state_c
        .make_multi_batch_header(&certificate_c, signer_c)
        .unwrap();

    let state_b = LocalNetworkState::from(state_b);
    let state_c = LocalNetworkState::from(state_c);

    let mut stdin = SP1Stdin::new();
    stdin.write(&state_b);
    stdin.write(&mbh_b);
    // Generate the proof for the given program and input.
    let client = ProverClient::new();
    let (pk, vk) = client.setup(PESSIMISTIC_PROOF_ELF);
    let proof_b = client.prove(&pk, stdin).compressed().run().unwrap();

    // Verify proof and public values
    client.verify(&proof_b, &vk).expect("verification failed");
    // dbg!(bincode::deserialize::<PessimisticProofOutput>(
    //     proof_b.public_values.as_slice()
    // ));
    // dbg!(bincode::deserialize::<PessimisticProofOutput>(
    //     &bincode::deserialize::<Vec<u8>>(proof_b.public_values.as_slice()).
    // unwrap() ));

    let mut stdin = SP1Stdin::new();
    stdin.write(&state_c);
    stdin.write(&mbh_c);
    // Generate the proof for the given program and input.
    let client = ProverClient::new();
    let (pk, vk) = client.setup(PESSIMISTIC_PROOF_ELF);
    let proof_c = client.prove(&pk, stdin).compressed().run().unwrap();
    // Verify proof and public values
    client.verify(&proof_c, &vk).expect("verification failed");

    let mut stdin = SP1Stdin::new();
    let vkey = vk.hash_u32();
    let pv = proof_b.public_values.as_slice().to_vec();
    // dbg!(pv.len());
    let tmp_arer = aret.root;
    let selected_mer = [0; 32]; // TODO
    let selected_rer = final_aret.root;
    let tmp_arer_proof = aret.get_inclusion_proof(*network_b).unwrap();
    let imported_lers_witness = vec![ImportedLERWitness {
        old_ler: (&let_c).into(),
        new_ler: new_ler_c,
        next_leaf: Digest::default(),
        subtree_proof: LETMerkleProof::default(),
        new_ler_proof: Some(final_aret.get_inclusion_proof(*network_c).unwrap()),
    }];
    stdin.write::<[u32; 8]>(&vkey);
    stdin.write::<Vec<u8>>(&pv);
    let SP1Proof::Compressed(proof) = proof_b.proof else {
        panic!()
    };
    stdin.write_proof(proof, vk.vk.clone());
    stdin.write(&tmp_arer);
    stdin.write(&selected_mer);
    stdin.write(&selected_rer);
    stdin.write(&tmp_arer_proof);
    stdin.write(&imported_lers_witness);
    // Generate the proof for the given program and input.
    let client = ProverClient::new();
    let (pk, vk_wrap) = client.setup(WRAP_PESSIMISTIC_PROOF_ELF);
    let wrap_proof_b = client.prove(&pk, stdin).compressed().run().unwrap();
    // Verify proof and public values
    client
        .verify(&wrap_proof_b, &vk_wrap)
        .expect("verification failed");

    dbg!("DONE 1");

    let mut stdin = SP1Stdin::new();
    let vkey = vk.hash_u32();
    let pv = proof_c.public_values.as_slice().to_vec();
    // dbg!(pv.len());
    aret.update(*network_b, new_ler_b).unwrap();
    let tmp_arer = aret.root;
    let selected_mer = [0; 32]; // TODO
    let selected_rer = final_aret.root;
    let tmp_arer_proof = aret.get_inclusion_proof(*network_c).unwrap();
    let imported_lers_witness = vec![ImportedLERWitness {
        old_ler: (&let_b).into(),
        new_ler: new_ler_b,
        next_leaf: Digest::default(),
        subtree_proof: LETMerkleProof::default(),
        new_ler_proof: Some(final_aret.get_inclusion_proof(*network_b).unwrap()),
    }];
    stdin.write::<[u32; 8]>(&vkey);
    stdin.write::<Vec<u8>>(&pv);
    let SP1Proof::Compressed(pproof) = proof_c.proof else {
        panic!()
    };
    stdin.write_proof(pproof, vk.vk.clone());
    stdin.write(&tmp_arer);
    stdin.write(&selected_mer);
    stdin.write(&selected_rer);
    stdin.write(&tmp_arer_proof);
    stdin.write(&imported_lers_witness);
    // Generate the proof for the given program and input.
    let client = ProverClient::new();
    let (pk, vk_wrap) = client.setup(WRAP_PESSIMISTIC_PROOF_ELF);
    let wrap_proof_c = client.prove(&pk, stdin).compressed().run().unwrap();
    // Verify proof and public values
    client
        .verify(&wrap_proof_c, &vk_wrap)
        .expect("verification failed");

    dbg!("DONE 2");

    let mut stdin = SP1Stdin::new();
    let vkey = vk_wrap.hash_u32();
    let pv0 = wrap_proof_b.public_values.as_slice().to_vec();
    let pv1 = wrap_proof_c.public_values.as_slice().to_vec();
    stdin.write::<[u32; 8]>(&vkey);
    stdin.write::<Vec<u8>>(&pv0);
    stdin.write::<Vec<u8>>(&pv1);
    let SP1Proof::Compressed(proof) = wrap_proof_b.proof else {
        panic!()
    };
    stdin.write_proof(proof, vk_wrap.vk.clone());
    let SP1Proof::Compressed(proof) = wrap_proof_c.proof else {
        panic!()
    };
    stdin.write_proof(proof, vk_wrap.vk.clone());
    // Generate the proof for the given program and input.
    let client = ProverClient::new();
    let (pk, vk_combine) = client.setup(COMBINE_PESSIMISTIC_PROOF_ELF);
    let combined_proof = client.prove(&pk, stdin).run().unwrap();
    // Verify proof and public values
    client
        .verify(&combined_proof, &vk_combine)
        .expect("verification failed");
}
