use super::wrap::AggregationProofOutput;
use crate::keccak::keccak256_combine;

pub fn combine_proofs(
    wrapped_proof0: AggregationProofOutput,
    wrapped_proof1: AggregationProofOutput,
) -> AggregationProofOutput {
    let AggregationProofOutput {
        tmp_arer: tmp_arer0,
        tmp_arer_next: tmp_arer_next0,
        selected_ger: selected_ger0,
        chain_info_tree_node: chain_info_tree_node0,
    } = wrapped_proof0;
    let AggregationProofOutput {
        tmp_arer: tmp_arer1,
        tmp_arer_next: tmp_arer_next1,
        selected_ger: selected_ger1,
        chain_info_tree_node: chain_info_tree_node1,
    } = wrapped_proof1;

    assert_eq!(tmp_arer_next0, tmp_arer1);
    assert_eq!(selected_ger0, selected_ger1);
    let chain_info_tree_node = keccak256_combine([chain_info_tree_node0, chain_info_tree_node1]);

    AggregationProofOutput {
        tmp_arer: tmp_arer0,
        tmp_arer_next: tmp_arer_next1,
        selected_ger: selected_ger0,
        chain_info_tree_node,
    }
}
