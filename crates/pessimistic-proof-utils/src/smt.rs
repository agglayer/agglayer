use std::collections::HashMap;
use std::hash::Hash;

use pessimistic_proof::local_exit_tree::hasher::Hasher;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use thiserror::Error;

use crate::utils::empty_hash_at_height;

pub trait ToBits<const NUM_BITS: usize> {
    fn to_bits(&self) -> [bool; NUM_BITS];
}

#[derive(Error, Debug)]
pub(crate) enum SmtError {
    #[error("trying to insert a key already in the smt")]
    KeyAlreadyPresent,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct Node<H>
where
    H: Hasher,
    H::Digest: Serialize + for<'a> Deserialize<'a>,
{
    #[serde_as(as = "_")]
    left: H::Digest,
    #[serde_as(as = "_")]
    right: H::Digest,
}

impl<H> Clone for Node<H>
where
    H: Hasher,
    H::Digest: Copy + Serialize + for<'a> Deserialize<'a>,
{
    fn clone(&self) -> Self {
        Node {
            left: self.left,
            right: self.right,
        }
    }
}

impl<H> Copy for Node<H>
where
    H: Hasher,
    H::Digest: Copy + Serialize + for<'a> Deserialize<'a>,
{
}

impl<H> Node<H>
where
    H: Hasher,
    H::Digest: Serialize + for<'a> Deserialize<'a>,
{
    pub fn hash(&self) -> H::Digest {
        H::merge(&self.left, &self.right)
    }
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Smt<H, const DEPTH: usize>
where
    H: Hasher,
    H::Digest: Copy + Eq + Hash + Serialize + for<'a> Deserialize<'a>,
{
    #[serde_as(as = "_")]
    root: H::Digest,
    #[serde_as(as = "HashMap<_, _>")]
    tree: HashMap<H::Digest, Node<H>>,
    #[serde_as(as = "[_; DEPTH]")]
    empty_hash_at_height: [H::Digest; DEPTH],
}

impl<H, const DEPTH: usize> Default for Smt<H, DEPTH>
where
    H: Hasher,
    H::Digest: Copy + Eq + Hash + Serialize + for<'a> Deserialize<'a> + Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<H, const DEPTH: usize> Smt<H, DEPTH>
where
    H: Hasher,
    H::Digest: Copy + Eq + Hash + Serialize + for<'a> Deserialize<'a>,
{
    pub fn new() -> Self
    where
        H::Digest: Default,
    {
        let empty_hash_at_height = empty_hash_at_height::<H, DEPTH>();
        let root = H::merge(
            &empty_hash_at_height[DEPTH - 1],
            &empty_hash_at_height[DEPTH - 1],
        );
        let tree = HashMap::new();
        Smt {
            root,
            tree,
            empty_hash_at_height,
        }
    }

    pub fn get<K>(&self, key: K) -> Option<H::Digest>
    where
        K: ToBits<DEPTH>,
    {
        let mut hash = self.root;
        for b in key.to_bits() {
            hash = if b {
                self.tree.get(&hash)?.right
            } else {
                self.tree.get(&hash)?.left
            };
        }

        Some(hash)
    }

    fn insert_helper(
        &mut self,
        hash: H::Digest,
        depth: usize,
        bits: &[bool; DEPTH],
        value: H::Digest,
    ) -> Result<H::Digest, SmtError> {
        if depth == DEPTH {
            return if hash != self.empty_hash_at_height[0] {
                Err(SmtError::KeyAlreadyPresent)
            } else {
                Ok(value)
            };
        }
        let node = self.tree.get(&hash);
        let mut node = node.copied().unwrap_or(Node {
            left: self.empty_hash_at_height[DEPTH - depth - 1],
            right: self.empty_hash_at_height[DEPTH - depth - 1],
        });
        let child_hash = if bits[depth] {
            self.insert_helper(node.right, depth + 1, bits, value)
        } else {
            self.insert_helper(node.left, depth + 1, bits, value)
        }?;
        if bits[depth] {
            node.right = child_hash;
        } else {
            node.left = child_hash;
        }

        let new_hash = node.hash();
        self.tree.insert(new_hash, node);

        Ok(new_hash)
    }

    pub fn insert<K>(&mut self, key: K, value: H::Digest) -> Result<(), SmtError>
    where
        K: ToBits<DEPTH>,
    {
        let new_root = self.insert_helper(self.root, 0, &key.to_bits(), value)?;
        self.root = new_root;

        Ok(())
    }
}

impl ToBits<32> for u32 {
    fn to_bits(&self) -> [bool; 32] {
        std::array::from_fn(|i| (self >> i) & 1 == 1)
    }
}

#[cfg(test)]
mod tests {

    use pessimistic_proof::local_exit_tree::hasher::Keccak256Hasher;
    use pessimistic_proof::local_exit_tree::LocalExitTree;
    use rand::prelude::SliceRandom;
    use rand::{random, thread_rng, Rng};

    use crate::smt::Smt;

    const TREE_DEPTH: usize = 32;
    type H = Keccak256Hasher;

    #[test]
    fn test_order_consistency() {
        let mut rng = thread_rng();
        let num_keys = rng.gen_range(0..100);
        let mut smt = Smt::<H, TREE_DEPTH>::new();
        let mut kvs: Vec<(u32, _)> = (0..num_keys).map(|_| (random(), random())).collect();
        for (key, value) in kvs.iter() {
            smt.insert(*key, *value).unwrap();
        }
        let mut shuffled_smt = Smt::<H, TREE_DEPTH>::new();
        kvs.shuffle(&mut rng);
        for (key, value) in kvs.iter() {
            shuffled_smt.insert(*key, *value).unwrap();
        }

        assert_eq!(smt.root, shuffled_smt.root);
    }
}
