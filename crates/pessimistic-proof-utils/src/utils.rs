use std::hash::Hash;

use pessimistic_proof::local_exit_tree::hasher::Hasher;

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
