use alloy::primitives::Bytes;

use crate::contracts::PolygonRollupManager;

/// ABI-encode a `verifyPessimisticTrustedAggregator` call into raw settlement
/// calldata, without a provider. The settlement task owns gas/nonce/fees and
/// only needs the encoded input to submit to L1.
pub fn verify_pessimistic_trusted_aggregator_calldata(
    rollup_id: u32,
    l1_info_tree_leaf_count: u32,
    new_local_exit_root: [u8; 32],
    new_pessimistic_root: [u8; 32],
    proof: Bytes,
    aggchain_data: Bytes,
) -> Bytes {
    use alloy::sol_types::SolCall as _;

    PolygonRollupManager::verifyPessimisticTrustedAggregatorCall {
        rollupID: rollup_id,
        l1InfoTreeLeafCount: l1_info_tree_leaf_count,
        newLocalExitRoot: new_local_exit_root.into(),
        newPessimisticRoot: new_pessimistic_root.into(),
        proof,
        aggchainData: aggchain_data,
    }
    .abi_encode()
    .into()
}
