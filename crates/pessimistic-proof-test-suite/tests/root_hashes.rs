use pessimistic_proof::{local_balance_tree::LocalBalanceTree, nullifier_tree::NullifierTree};

#[rstest::rstest]
fn root_hashes_sanity_check() {
    insta::assert_debug_snapshot!(LocalBalanceTree::new().root);
    insta::assert_debug_snapshot!(NullifierTree::new().root);
}
