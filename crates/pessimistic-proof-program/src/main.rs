#![no_main]

use pessimistic_proof_core::{
    generate_pessimistic_proof, keccak::Keccak256Hasher, multi_batch_header::{
        MultiBatchHeader, MultiBatchHeaderZeroCopy, BridgeExitZeroCopy, 
        ImportedBridgeExitZeroCopy, BalanceProofEntryZeroCopy,
        SmtMerkleProofZeroCopy, SmtNonInclusionProofZeroCopy
    },
    NetworkState, PessimisticProofOutput,
};
use agglayer_primitives::U256;
use bytemuck;
sp1_zkvm::entrypoint!(main);

/// Helper function to safely deserialize zero-copy data with proper alignment
fn deserialize_zero_copy<T: bytemuck::Pod>(data: &[u8]) -> Vec<T> {
    if data.is_empty() {
        return vec![];
    }
    // Copy to aligned buffer to fix alignment issue
    let mut aligned_buffer = vec![0u8; data.len()];
    aligned_buffer.copy_from_slice(data);
    bytemuck::cast_slice(&aligned_buffer).to_vec()
}

pub fn main() {
    // Read NetworkState (zero-copy)
    let network_state_bytes = sp1_zkvm::io::read_vec();
    let initial_state = NetworkState::from_bytes_zero_copy(&network_state_bytes)
        .expect("Failed to deserialize NetworkState");
    
    // Read MultiBatchHeader header (zero-copy)
    let header_bytes = sp1_zkvm::io::read_vec();
    // Copy to aligned buffer to fix alignment issue
    let mut aligned_header_buffer = [0u8; std::mem::size_of::<MultiBatchHeaderZeroCopy>()];
    aligned_header_buffer.copy_from_slice(&header_bytes);
    let header_zero_copy = bytemuck::from_bytes::<MultiBatchHeaderZeroCopy>(&aligned_header_buffer);
    
    // Read bridge_exits (zero-copy)
    let bridge_exits_bytes = sp1_zkvm::io::read_vec();
    let bridge_exits: Vec<BridgeExitZeroCopy> = deserialize_zero_copy(&bridge_exits_bytes);
    
    // Read imported_bridge_exits (zero-copy)
    let imported_bridge_exits_bytes = sp1_zkvm::io::read_vec();
    let imported_bridge_exits: Vec<ImportedBridgeExitZeroCopy> = deserialize_zero_copy(&imported_bridge_exits_bytes);
    
    // Read nullifier paths for imported bridge exits (zero-copy)
    let nullifier_paths_bytes = sp1_zkvm::io::read_vec();
    let nullifier_paths: Vec<SmtNonInclusionProofZeroCopy> = deserialize_zero_copy(&nullifier_paths_bytes);
    
    // Read balances_proofs entries (zero-copy)
    let balances_proofs_bytes = sp1_zkvm::io::read_vec();
    let balances_proofs: Vec<BalanceProofEntryZeroCopy> = deserialize_zero_copy(&balances_proofs_bytes);
    
    // Read balance Merkle paths (zero-copy)
    let balance_merkle_paths_bytes = sp1_zkvm::io::read_vec();
    let balance_merkle_paths: Vec<SmtMerkleProofZeroCopy> = deserialize_zero_copy(&balance_merkle_paths_bytes);
    
    // Read aggchain_proof separately using bincode (since zero-copy truncates it)
    let aggchain_proof = sp1_zkvm::io::read::<pessimistic_proof_core::aggchain_proof::AggchainData>();
    
    // Reconstruct the MultiBatchHeader from zero-copy components
    let mut batch_header = MultiBatchHeader::<Keccak256Hasher>::from_zero_copy(header_zero_copy)
        .expect("Failed to reconstruct MultiBatchHeader");
    
    // Set the aggchain_proof from the separately read data
    batch_header.aggchain_proof = aggchain_proof;
    
    // Convert bridge_exits back to original format
    batch_header.bridge_exits = bridge_exits.iter().map(|be| be.to_bridge_exit()).collect();
    
    // Convert imported_bridge_exits back to original format
    batch_header.imported_bridge_exits = imported_bridge_exits.iter().zip(nullifier_paths.iter()).map(|(ibe, path)| {
        let imported_bridge_exit = ibe.to_imported_bridge_exit();
        let nullifier_path = path.to_smt_non_inclusion_proof();
        (imported_bridge_exit, nullifier_path)
    }).collect();
    
    // Convert balances_proofs back to original format
    batch_header.balances_proofs = balances_proofs.iter().zip(balance_merkle_paths.iter()).map(|(bp, path)| {
        let token_info = bp.token_info.to_token_info();
        let balance = U256::from_be_bytes(bp.balance);
        let merkle_path = path.to_smt_merkle_proof();
        (token_info, (balance, merkle_path))
    }).collect();

    let (outputs, _targets) = generate_pessimistic_proof(initial_state, &batch_header)
        .expect("Failed to generate pessimistic proof");

    let pp_inputs = PessimisticProofOutput::bincode_codec()
        .serialize(&outputs)
        .expect("Failed to serialize proof outputs");

    sp1_zkvm::io::commit_slice(&pp_inputs);
}
