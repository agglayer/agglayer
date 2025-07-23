#![no_main]

use pessimistic_proof_core::{
    generate_pessimistic_proof, keccak::Keccak256Hasher, multi_batch_header::{
        MultiBatchHeader, MultiBatchHeaderZeroCopy, BridgeExitZeroCopy, 
        ImportedBridgeExitZeroCopy, BalanceProofEntryZeroCopy,
        SmtMerkleProofZeroCopy, SmtNonInclusionProofZeroCopy
    },
    NetworkState, PessimisticProofOutput,
};
use agglayer_primitives;
use bytemuck;
sp1_zkvm::entrypoint!(main);

pub fn main() {
    // Read NetworkState (zero-copy)
    let network_state_bytes = sp1_zkvm::io::read_vec();
    let initial_state = NetworkState::from_bytes_zero_copy(&network_state_bytes).unwrap();
    
    // Read MultiBatchHeader header (zero-copy)
    let header_bytes = sp1_zkvm::io::read_vec();
    // Copy to aligned buffer to fix alignment issue
    let mut aligned_header_buffer = [0u8; std::mem::size_of::<MultiBatchHeaderZeroCopy>()];
    aligned_header_buffer.copy_from_slice(&header_bytes);
    let header_zero_copy = bytemuck::from_bytes::<MultiBatchHeaderZeroCopy>(&aligned_header_buffer);
    
    // Read bridge_exits (zero-copy)
    let bridge_exits_bytes = sp1_zkvm::io::read_vec();
    let bridge_exits: Vec<BridgeExitZeroCopy> = if bridge_exits_bytes.is_empty() {
        vec![]
    } else {
        // Copy to aligned buffer
        let mut aligned_buffer = vec![0u8; bridge_exits_bytes.len()];
        aligned_buffer.copy_from_slice(&bridge_exits_bytes);
        bytemuck::cast_slice(&aligned_buffer).to_vec()
    };
    
    // Read imported_bridge_exits (zero-copy)
    let imported_bridge_exits_bytes = sp1_zkvm::io::read_vec();
    let imported_bridge_exits: Vec<ImportedBridgeExitZeroCopy> = if imported_bridge_exits_bytes.is_empty() {
        vec![]
    } else {
        // Copy to aligned buffer
        let mut aligned_buffer = vec![0u8; imported_bridge_exits_bytes.len()];
        aligned_buffer.copy_from_slice(&imported_bridge_exits_bytes);
        bytemuck::cast_slice(&aligned_buffer).to_vec()
    };
    
    // Read nullifier paths for imported bridge exits (zero-copy)
    let nullifier_paths_bytes = sp1_zkvm::io::read_vec();
    let nullifier_paths: Vec<SmtNonInclusionProofZeroCopy> = if nullifier_paths_bytes.is_empty() {
        vec![]
    } else {
        // Copy to aligned buffer
        let mut aligned_buffer = vec![0u8; nullifier_paths_bytes.len()];
        aligned_buffer.copy_from_slice(&nullifier_paths_bytes);
        bytemuck::cast_slice(&aligned_buffer).to_vec()
    };
    
    // Read balances_proofs entries (zero-copy)
    let balances_proofs_bytes = sp1_zkvm::io::read_vec();
    let balances_proofs: Vec<BalanceProofEntryZeroCopy> = if balances_proofs_bytes.is_empty() {
        vec![]
    } else {
        // Copy to aligned buffer
        let mut aligned_buffer = vec![0u8; balances_proofs_bytes.len()];
        aligned_buffer.copy_from_slice(&balances_proofs_bytes);
        bytemuck::cast_slice(&aligned_buffer).to_vec()
    };
    
    // Read balance Merkle paths (zero-copy)
    let balance_merkle_paths_bytes = sp1_zkvm::io::read_vec();
    let balance_merkle_paths: Vec<SmtMerkleProofZeroCopy> = if balance_merkle_paths_bytes.is_empty() {
        vec![]
    } else {
        // Copy to aligned buffer
        let mut aligned_buffer = vec![0u8; balance_merkle_paths_bytes.len()];
        aligned_buffer.copy_from_slice(&balance_merkle_paths_bytes);
        bytemuck::cast_slice(&aligned_buffer).to_vec()
    };
    
    // Read aggchain_proof separately using bincode (since zero-copy truncates it)
    let aggchain_proof = sp1_zkvm::io::read::<pessimistic_proof_core::aggchain_proof::AggchainData>();
    
    // Reconstruct the MultiBatchHeader from zero-copy components
    let mut batch_header = MultiBatchHeader::<Keccak256Hasher>::from_zero_copy(header_zero_copy).unwrap();
    
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
        let balance = agglayer_primitives::U256::from_be_bytes(bp.balance);
        let merkle_path = path.to_smt_merkle_proof();
        (token_info, (balance, merkle_path))
    }).collect();

    let (outputs, _targets) = generate_pessimistic_proof(initial_state, &batch_header).unwrap();

    let pp_inputs = PessimisticProofOutput::bincode_codec()
        .serialize(&outputs)
        .unwrap();

    sp1_zkvm::io::commit_slice(&pp_inputs);
}
