use pessimistic_proof::local_exit_tree::hasher::Hasher;

/// Returns an array whose `i`th element is the root of an empty Merkle tree of
/// depth `i`.
pub(crate) fn empty_hash_at_height<H, const DEPTH: usize>() -> [H::Digest; DEPTH]
where
    H: Hasher,
    H::Digest: Default + Copy,
{
    let mut empty_hash_at_height = [H::Digest::default(); DEPTH];
    for height in 1..DEPTH {
        empty_hash_at_height[height] = H::merge(
            &empty_hash_at_height[height - 1],
            &empty_hash_at_height[height - 1],
        );
    }
    empty_hash_at_height
}
