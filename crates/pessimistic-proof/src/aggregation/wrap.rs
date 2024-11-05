use serde::{Deserialize, Serialize};

use crate::{
    bridge_exit::NetworkId,
    keccak::{keccak256, keccak256_combine, Digest},
    local_exit_tree::{data::LETMerkleProof, hasher::Keccak256Hasher, LocalExitTree},
    utils::smt::SmtMerkleProof,
    PessimisticProofOutput,
};

/// Outputs of the aggregated pessimistic proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationProofOutput {
    pub(crate) tmp_arer: Digest,
    pub(crate) tmp_arer_next: Digest,
    pub(crate) selected_ger: Digest,
    pub(crate) chain_info_tree_node: Digest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedLERWitness {
    pub old_ler: LocalExitTree<Keccak256Hasher>,
    pub new_ler: Digest,
    pub next_leaf: Digest,
    pub subtree_proof: LETMerkleProof<Keccak256Hasher>,
    pub new_ler_proof: Option<SmtMerkleProof<Keccak256Hasher, 32>>,
}

pub fn wrap_proof(
    pp_output: PessimisticProofOutput,
    tmp_arer: Digest,
    selected_mer: Digest,
    selected_rer: Digest,
    tmp_arer_proof: SmtMerkleProof<Keccak256Hasher, 32>, // TODO: use constant for depth
    imported_lers_witness: Vec<ImportedLERWitness>,
) -> AggregationProofOutput {
    let PessimisticProofOutput {
        prev_local_exit_root,
        prev_pessimistic_root,
        imported_local_exit_roots,
        origin_network,
        consensus_hash,
        new_local_exit_root,
        new_pessimistic_root,
    } = pp_output;

    // Compute leaf `H(chainID || nLER || oPPR || nPPR || consensusHash)` of
    // `chainInfoTree`
    let mut data = [0; 4 + 32 + 32 + 32 + 32];
    data[..4].copy_from_slice(&origin_network.to_le_bytes());
    data[4..4 + 32].copy_from_slice(&new_local_exit_root);
    data[4 + 32..4 + 32 + 32].copy_from_slice(&prev_pessimistic_root);
    data[4 + 32 + 32..4 + 32 + 32 + 32].copy_from_slice(&new_pessimistic_root);
    data[4 + 32 + 32 + 32..].copy_from_slice(&consensus_hash);
    let chain_info_leaf = keccak256(&data);

    // Update `tmp_arer`
    let tmp_arer_next = tmp_arer_proof
        .verify_and_update(
            *origin_network,
            prev_local_exit_root,
            new_local_exit_root,
            tmp_arer,
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
        if ler != *new_ler {
            // Check that the LER used is a subtree of the new LER.
            assert!(old_ler.is_subtree(*new_ler, *next_leaf, subtree_proof.clone()));
        }
        if network_id == NetworkId::new(0) {
            // For mainnet, only need to check that the new LER is the MER.
            assert_eq!(*new_ler, selected_mer);
            assert!(new_ler_proof.is_none());
        } else {
            // Check that the new LER is in the new RER.
            let new_ler_proof = new_ler_proof.as_ref().expect("Missing proof for LER");
            new_ler_proof.verify(*network_id, *new_ler, selected_rer);
        }
    }

    AggregationProofOutput {
        tmp_arer,
        tmp_arer_next,
        selected_ger: keccak256_combine([selected_mer, selected_rer]),
        chain_info_tree_node: chain_info_leaf,
    }
}
