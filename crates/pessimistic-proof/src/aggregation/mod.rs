use serde::{Deserialize, Serialize};

use crate::{
    bridge_exit::NetworkId,
    keccak::{keccak256, keccak256_combine, Digest},
    local_exit_tree::{data::LETMerkleProof, hasher::Keccak256Hasher, LocalExitTree},
    utils::smt::SmtMerkleProof,
    PessimisticProofOutput,
};

/// Outputs of the pessimistic proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stage1AggregationProofOutput {
    tmp_rer: Digest,
    tmp_rer_next: Digest,
    new_ger: Digest,
    ler_hash: Digest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedLERWitness {
    old_ler: LocalExitTree<Keccak256Hasher>,
    new_ler: Digest,
    next_leaf: Digest,
    subtree_proof: LETMerkleProof<Keccak256Hasher>,
    new_ler_proof: Option<SmtMerkleProof<Keccak256Hasher, 32>>,
}

pub fn wrap_stage1_aggregation_proof_output(
    pp_output: PessimisticProofOutput,
    tmp_rer: Digest,
    new_mer: Digest,
    new_rer: Digest,
    tmp_rer_proof: SmtMerkleProof<Keccak256Hasher, 32>, // TODO: use constant for depth
    imported_lers_witness: Vec<ImportedLERWitness>,
) -> Stage1AggregationProofOutput {
    let PessimisticProofOutput {
        prev_local_exit_root,
        prev_pessimistic_root: _prev_pessimistic_root, // TODO: use this,
        imported_local_exit_roots,
        origin_network,
        consensus_hash: _consensus_hash, // TODO: use this,
        new_local_exit_root,
        new_pessimistic_root: _new_pessimistic_root, // TODO: use this,
    } = pp_output;

    // Compute `ler_hash`
    let mut data = [0; 4 + 32];
    data[..4].copy_from_slice(&origin_network.to_le_bytes());
    data[4..].copy_from_slice(&new_local_exit_root);
    let ler_hash = keccak256(&data);

    // Update `tmp_rer`
    let tmp_rer_next = tmp_rer_proof
        .verify_and_update(
            *origin_network,
            prev_local_exit_root,
            new_local_exit_root,
            tmp_rer,
        )
        .expect("Invalid Merkle proof");

    // Verify imported LERs
    for (i, (network_id, ler)) in imported_local_exit_roots.into_iter().enumerate() {
        let ImportedLERWitness {
            old_ler,
            new_ler,
            next_leaf,
            subtree_proof,
            new_ler_proof,
        } = &imported_lers_witness[i];
        // Check that the LER frontier is consistent with the LER root.
        assert_eq!(old_ler.get_root(), ler);
        // Check that the LER used is a subtree of the new LER.
        assert!(old_ler.is_subtree(*new_ler, *next_leaf, subtree_proof.clone()));
        if network_id == NetworkId::new(0) {
            // For mainnet, only need to check that the new LER is the MER.
            assert_eq!(*new_ler, new_mer);
            assert!(new_ler_proof.is_none());
        } else {
            // Check that the new LER is in the new RER.
            let new_ler_proof = new_ler_proof.as_ref().expect("Missing proof for LER");
            new_ler_proof.verify(*network_id, *new_ler, new_rer);
        }
    }

    Stage1AggregationProofOutput {
        tmp_rer,
        tmp_rer_next,
        new_ger: keccak256_combine([new_mer, new_rer]),
        ler_hash,
    }
}
