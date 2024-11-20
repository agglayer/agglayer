#![allow(clippy::needless_range_loop)]
use std::{collections::HashMap, hash::Hash};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_with::serde_as;
use thiserror::Error;

use crate::{local_exit_tree::hasher::Hasher, utils::empty_hash::empty_hash_at_height};

pub trait ToBits<const NUM_BITS: usize> {
    fn to_bits(&self) -> [bool; NUM_BITS];
}

#[derive(Error, Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub enum SmtError {
    #[error("trying to insert a key already in the SMT")]
    KeyAlreadyPresent,
    #[error("trying to generate a Merkle proof for a key not in the SMT")]
    KeyNotPresent,
    #[error("trying to generate a non-inclusion proof for a key present in the SMT")]
    KeyPresent,
    #[error("depth out of bounds")]
    DepthOutOfBounds,
}

/// A node in an SMT
#[serde_as]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Node<H>
where
    H: Hasher,
    H::Digest: Serialize + DeserializeOwned,
{
    #[serde_as(as = "_")]
    pub left: H::Digest,
    #[serde_as(as = "_")]
    pub right: H::Digest,
}

impl<H> Clone for Node<H>
where
    H: Hasher,
    H::Digest: Copy + Serialize + DeserializeOwned,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<H> Copy for Node<H>
where
    H: Hasher,
    H::Digest: Copy + Serialize + DeserializeOwned,
{
}

impl<H> Node<H>
where
    H: Hasher,
    H::Digest: Serialize + DeserializeOwned,
{
    pub fn hash(&self) -> H::Digest {
        H::merge(&self.left, &self.right)
    }
}

/// An SMT consistent with a zero-initialized Merkle tree
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Smt<H, const DEPTH: usize>
where
    H: Hasher,
    H::Digest: Copy + Eq + Hash + Serialize + DeserializeOwned,
{
    /// The SMT root
    #[serde_as(as = "_")]
    pub root: H::Digest,
    /// A map from node hash to node
    #[serde_as(as = "HashMap<_, _>")]
    pub tree: HashMap<H::Digest, Node<H>>,
    /// `empty_hash_at_height[i]` is the root of an empty Merkle tree of depth
    /// `i`.
    #[serde_as(as = "[_; DEPTH]")]
    empty_hash_at_height: [H::Digest; DEPTH],
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SmtMerkleProof<H, const DEPTH: usize>
where
    H: Hasher,
    H::Digest: Copy + Eq + Hash + Serialize + DeserializeOwned,
{
    #[serde_as(as = "[_; DEPTH]")]
    siblings: [H::Digest; DEPTH],
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SmtNonInclusionProof<H, const DEPTH: usize>
where
    H: Hasher,
    H::Digest: Copy + Eq + Serialize + DeserializeOwned,
{
    #[serde_as(as = "Vec<_>")]
    siblings: Vec<H::Digest>,
}

impl<H, const DEPTH: usize> Default for Smt<H, DEPTH>
where
    H: Hasher,
    H::Digest: Copy + Eq + Hash + Serialize + DeserializeOwned + Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<H, const DEPTH: usize> Smt<H, DEPTH>
where
    H: Hasher,
    H::Digest: Copy + Eq + Hash + Serialize + DeserializeOwned,
{
    pub fn new() -> Self
    where
        H::Digest: Default,
    {
        let empty_hash_at_height = empty_hash_at_height::<H, DEPTH>();
        let root = Node {
            left: empty_hash_at_height[DEPTH - 1],
            right: empty_hash_at_height[DEPTH - 1],
        };
        Self::new_with_nodes(root.hash(), &[root])
    }

    pub fn new_with_nodes(root: H::Digest, nodes: &[Node<H>]) -> Self
    where
        H::Digest: Default,
    {
        let empty_hash_at_height = empty_hash_at_height::<H, DEPTH>();
        Smt {
            root,
            tree: nodes.iter().map(|n| (n.hash(), *n)).collect(),
            empty_hash_at_height,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.root
            == H::merge(
                &self.empty_hash_at_height[DEPTH - 1],
                &self.empty_hash_at_height[DEPTH - 1],
            )
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
        // If true, update the value at the key.
        // If false, insert the value at the key and error if the key is present.
        update: bool,
    ) -> Result<H::Digest, SmtError> {
        if depth > DEPTH {
            return Err(SmtError::DepthOutOfBounds);
        }
        if depth == DEPTH {
            return if !update && hash != self.empty_hash_at_height[0] {
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
            self.insert_helper(node.right, depth + 1, bits, value, update)
        } else {
            self.insert_helper(node.left, depth + 1, bits, value, update)
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
        let new_root = self.insert_helper(self.root, 0, &key.to_bits(), value, false)?;
        self.root = new_root;

        Ok(())
    }

    pub fn update<K>(&mut self, key: K, value: H::Digest) -> Result<(), SmtError>
    where
        K: ToBits<DEPTH>,
    {
        let new_root = self.insert_helper(self.root, 0, &key.to_bits(), value, true)?;
        self.root = new_root;

        Ok(())
    }

    fn get_inclusion_proof_helper<K>(
        &self,
        key: K,
        zero_allowed: bool,
    ) -> Result<SmtMerkleProof<H, DEPTH>, SmtError>
    where
        K: ToBits<DEPTH>,
    {
        let mut siblings = [self.empty_hash_at_height[0]; DEPTH];
        let mut hash = self.root;
        let bits = key.to_bits();
        for i in 0..DEPTH {
            let node = self.tree.get(&hash).ok_or(SmtError::KeyNotPresent)?;
            siblings[DEPTH - i - 1] = if bits[i] { node.left } else { node.right };
            hash = if bits[i] { node.right } else { node.left };
        }
        if !zero_allowed && hash == self.empty_hash_at_height[0] {
            return Err(SmtError::KeyNotPresent);
        }

        Ok(SmtMerkleProof { siblings })
    }
    pub fn get_inclusion_proof<K>(&self, key: K) -> Result<SmtMerkleProof<H, DEPTH>, SmtError>
    where
        K: ToBits<DEPTH>,
    {
        self.get_inclusion_proof_helper(key, false)
    }

    /// Returns an inclusion proof that the key is not in the SMT.
    /// This has the same purpose as a non-inclusion proof, but with the same
    /// format as an inclusion proof. Use case: In the balance tree, we use
    /// inclusion proofs to verify the balance of a token in the tree and
    /// update it. If the token is not already in the tree, we still want an
    /// inclusion proof, so we use this function.
    pub fn get_inclusion_proof_zero<K>(
        &mut self,
        key: K,
    ) -> Result<SmtMerkleProof<H, DEPTH>, SmtError>
    where
        K: Copy + ToBits<DEPTH>,
    {
        // Hack: We use `insert` to insert all the necessary nodes in the SMT.
        // This will return an error if the key is in the SMT.
        self.insert(key, self.empty_hash_at_height[0])?;
        self.get_inclusion_proof_helper(key, true)
    }

    pub fn get_non_inclusion_proof<K>(
        &self,
        key: K,
    ) -> Result<SmtNonInclusionProof<H, DEPTH>, SmtError>
    where
        K: ToBits<DEPTH>,
    {
        let mut siblings = vec![];
        let mut hash = self.root;
        let bits = key.to_bits();
        for i in 0..DEPTH {
            if self.empty_hash_at_height.contains(&hash) {
                return Ok(SmtNonInclusionProof { siblings });
            }
            let node = self.tree.get(&hash);
            let node = match node {
                Some(node) => node,
                None => {
                    return Ok(SmtNonInclusionProof { siblings });
                }
            };
            siblings.push(if bits[i] { node.left } else { node.right });
            hash = if bits[i] { node.right } else { node.left };
        }
        if hash != self.empty_hash_at_height[0] {
            return Err(SmtError::KeyPresent);
        }

        Ok(SmtNonInclusionProof { siblings })
    }
}

impl<H, const DEPTH: usize> SmtMerkleProof<H, DEPTH>
where
    H: Hasher,
    H::Digest: Copy + Eq + Hash + Serialize + DeserializeOwned,
{
    pub fn verify<K>(&self, key: K, value: H::Digest, root: H::Digest) -> bool
    where
        K: ToBits<DEPTH>,
    {
        let bits = key.to_bits();
        let mut hash = value;
        for i in 0..DEPTH {
            hash = if bits[DEPTH - i - 1] {
                H::merge(&self.siblings[i], &hash)
            } else {
                H::merge(&hash, &self.siblings[i])
            };
        }

        hash == root
    }

    /// Verify the inclusion proof (i.e. that `(key, old_value)` is in the SMT)
    /// and return the updated root of the SMT with `(key, new_value)`
    /// inserted, or `None` if the inclusion proof is invalid.
    pub fn verify_and_update<K>(
        &self,
        key: K,
        old_value: H::Digest,
        new_value: H::Digest,
        root: H::Digest,
    ) -> Option<H::Digest>
    where
        K: ToBits<DEPTH> + Copy,
    {
        if !self.verify(key, old_value, root) {
            return None;
        }
        let bits = key.to_bits();
        let mut hash = new_value;
        for i in 0..DEPTH {
            hash = if bits[DEPTH - i - 1] {
                H::merge(&self.siblings[i], &hash)
            } else {
                H::merge(&hash, &self.siblings[i])
            };
        }

        Some(hash)
    }
}

impl<H, const DEPTH: usize> SmtNonInclusionProof<H, DEPTH>
where
    H: Hasher,
    H::Digest: Copy + Eq + Serialize + DeserializeOwned,
{
    pub fn verify<K>(
        &self,
        key: K,
        root: H::Digest,
        empty_hash_at_height: &[H::Digest; DEPTH],
    ) -> bool
    where
        K: ToBits<DEPTH>,
    {
        if self.siblings.len() > DEPTH {
            return false;
        }
        if self.siblings.is_empty() {
            let empty_root = H::merge(
                &empty_hash_at_height[DEPTH - 1],
                &empty_hash_at_height[DEPTH - 1],
            );
            return root == empty_root;
        }
        let bits = key.to_bits();
        let mut entry = empty_hash_at_height[DEPTH - self.siblings.len()];
        for i in (0..self.siblings.len()).rev() {
            let sibling = self.siblings[i];
            entry = if bits[i] {
                H::merge(&sibling, &entry)
            } else {
                H::merge(&entry, &sibling)
            };
        }

        entry == root
    }

    /// Verify the non-inclusion proof (i.e. that `key` is not in the SMT) and
    /// return the updated root of the SMT with `(key, value)` inserted, or
    /// `None` if the inclusion proof is invalid.
    pub fn verify_and_update<K>(
        &self,
        key: K,
        new_value: H::Digest,
        root: H::Digest,
        empty_hash_at_height: &[H::Digest; DEPTH],
    ) -> Option<H::Digest>
    where
        K: Copy + ToBits<DEPTH>,
    {
        if !self.verify(key, root, empty_hash_at_height) {
            return None;
        }

        let mut entry = new_value;
        let bits = key.to_bits();
        for i in (self.siblings.len()..DEPTH).rev() {
            let sibling = empty_hash_at_height[DEPTH - i - 1];
            entry = if bits[i] {
                H::merge(&sibling, &entry)
            } else {
                H::merge(&entry, &sibling)
            };
        }
        for i in (0..self.siblings.len()).rev() {
            let sibling = self.siblings[i];
            entry = if bits[i] {
                H::merge(&sibling, &entry)
            } else {
                H::merge(&entry, &sibling)
            };
        }

        Some(entry)
    }
}

impl ToBits<32> for u32 {
    fn to_bits(&self) -> [bool; 32] {
        std::array::from_fn(|i| (self >> i) & 1 == 1)
    }
}

#[cfg(test)]
mod tests {
    use std::hash::Hash;

    use rand::{prelude::SliceRandom, random, thread_rng, Rng};
    use rs_merkle::{Hasher as MerkleHasher, MerkleTree};
    use tiny_keccak::{Hasher as _, Keccak};

    use crate::{
        local_exit_tree::hasher::Keccak256Hasher,
        utils::smt::{Smt, SmtError, ToBits},
    };

    const DEPTH: usize = 32;
    type H = Keccak256Hasher;

    impl ToBits<8> for u8 {
        fn to_bits(&self) -> [bool; 8] {
            std::array::from_fn(|i| (self >> i) & 1 == 1)
        }
    }

    #[derive(Clone, Debug)]
    pub struct TestKeccak256;

    impl MerkleHasher for TestKeccak256 {
        type Hash = [u8; 32];

        fn hash(data: &[u8]) -> [u8; 32] {
            let mut keccak256 = Keccak::v256();
            keccak256.update(data);
            let mut output = [0u8; 32];
            keccak256.finalize(&mut output);
            output
        }
    }

    fn check_no_duplicates<A: Eq + Hash, B>(v: &[(A, B)]) {
        let mut seen = std::collections::HashSet::new();
        for (a, _) in v {
            assert!(seen.insert(a), "Duplicate key. Check your rng.");
        }
    }

    #[test]
    fn test_compare_with_other_impl() {
        const DEPTH: usize = 8;
        let mut rng = thread_rng();
        let num_keys = rng.gen_range(0..=1 << DEPTH);
        let mut smt = Smt::<H, DEPTH>::new();
        let mut kvs: Vec<_> = (0..=u8::MAX).map(|i| (i, random())).collect();
        kvs.shuffle(&mut rng);
        for (key, value) in &kvs[..num_keys] {
            smt.insert(*key, *value).unwrap();
        }

        let mut leaves = vec![[0_u8; 32]; 1 << DEPTH];
        for (key, value) in &kvs[..num_keys] {
            leaves[key.reverse_bits() as usize] = *value;
        }
        let mt: MerkleTree<TestKeccak256> = MerkleTree::from_leaves(&leaves);

        assert_eq!(smt.root, mt.root().unwrap());
    }

    #[test]
    fn test_order_consistency() {
        let mut rng = thread_rng();
        let num_keys = rng.gen_range(0..100);
        let mut smt = Smt::<H, DEPTH>::new();
        let mut kvs: Vec<(u32, _)> = (0..num_keys).map(|_| (random(), random())).collect();
        check_no_duplicates(&kvs);
        for (key, value) in kvs.iter() {
            smt.insert(*key, *value).unwrap();
        }
        let mut shuffled_smt = Smt::<H, DEPTH>::new();
        kvs.shuffle(&mut rng);
        for (key, value) in kvs.iter() {
            shuffled_smt.insert(*key, *value).unwrap();
        }

        assert_eq!(smt.root, shuffled_smt.root);
    }

    #[test]
    fn test_inclusion_proof() {
        let mut rng = thread_rng();
        let num_keys = rng.gen_range(1..100);
        let mut smt = Smt::<H, DEPTH>::new();
        let kvs: Vec<(u32, _)> = (0..num_keys).map(|_| (random(), random())).collect();
        check_no_duplicates(&kvs);
        for (key, value) in kvs.iter() {
            smt.insert(*key, *value).unwrap();
        }
        let (key, value) = *kvs.choose(&mut rng).unwrap();
        let proof = smt.get_inclusion_proof(key).unwrap();
        assert!(proof.verify(key, value, smt.root));
    }

    #[test]
    fn test_inclusion_proof_wrong_value() {
        let mut rng = thread_rng();
        let num_keys = rng.gen_range(1..100);
        let mut smt = Smt::<H, DEPTH>::new();
        let kvs: Vec<(u32, _)> = (0..num_keys).map(|_| (random(), random())).collect();
        check_no_duplicates(&kvs);
        for (key, value) in kvs.iter() {
            smt.insert(*key, *value).unwrap();
        }
        let (key, real_value) = *kvs.choose(&mut rng).unwrap();
        let proof = smt.get_inclusion_proof(key).unwrap();
        let fake_value = random();
        assert_ne!(real_value, fake_value, "Check your rng");
        assert!(!proof.verify(key, fake_value, smt.root));
    }

    #[test]
    fn test_non_inclusion_proof() {
        let mut rng = thread_rng();
        let num_keys = rng.gen_range(0..100);
        let mut smt = Smt::<H, DEPTH>::new();
        let kvs: Vec<(u32, _)> = (0..num_keys).map(|_| (random(), random())).collect();
        check_no_duplicates(&kvs);
        for (key, value) in kvs.iter() {
            smt.insert(*key, *value).unwrap();
        }
        let key: u32 = random();
        assert!(!kvs.iter().any(|(k, _)| k == &key), "Check your rng");
        let proof = smt.get_non_inclusion_proof(key).unwrap();
        assert!(proof.verify(key, smt.root, &smt.empty_hash_at_height));
    }

    #[test]
    fn test_non_inclusion_proof_failing() {
        let mut rng = thread_rng();
        let num_keys = rng.gen_range(1..100);
        let mut smt = Smt::<H, DEPTH>::new();
        let kvs: Vec<(u32, _)> = (0..num_keys).map(|_| (random(), random())).collect();
        check_no_duplicates(&kvs);
        for (key, value) in kvs.iter() {
            smt.insert(*key, *value).unwrap();
        }
        let (key, _) = *kvs.choose(&mut rng).unwrap();
        let error = smt.get_non_inclusion_proof(key).unwrap_err();
        assert_eq!(error, SmtError::KeyPresent);
    }

    fn test_non_inclusion_proof_and_update(num_keys: usize) {
        let mut smt = Smt::<H, DEPTH>::new();
        let kvs: Vec<(u32, _)> = (0..num_keys).map(|_| (random(), random())).collect();
        check_no_duplicates(&kvs);
        for (key, value) in kvs.iter() {
            smt.insert(*key, *value).unwrap();
        }
        let key: u32 = random();
        assert!(!kvs.iter().any(|(k, _)| k == &key), "Check your rng");
        let proof = smt.get_non_inclusion_proof(key).unwrap();
        assert!(proof.verify(key, smt.root, &smt.empty_hash_at_height));
        let value = random();
        let new_root = proof
            .verify_and_update(key, value, smt.root, &smt.empty_hash_at_height)
            .unwrap();
        smt.insert(key, value).unwrap();
        assert_eq!(smt.root, new_root);
    }

    #[test]
    fn test_non_inclusion_proof_and_update_empty() {
        test_non_inclusion_proof_and_update(0)
    }

    #[test]
    fn test_non_inclusion_proof_and_update_nonempty() {
        let num_keys = thread_rng().gen_range(1..100);
        test_non_inclusion_proof_and_update(num_keys)
    }

    #[test]
    fn test_inclusion_proof_and_update() {
        let num_keys = thread_rng().gen_range(1..100);
        let mut smt = Smt::<H, DEPTH>::new();
        let kvs: Vec<(u32, _)> = (0..num_keys).map(|_| (random(), random())).collect();
        check_no_duplicates(&kvs);
        for (key, value) in kvs.iter() {
            smt.insert(*key, *value).unwrap();
        }
        let (key, value) = kvs[thread_rng().gen_range(0..num_keys)];
        let proof = smt.get_inclusion_proof(key).unwrap();
        assert!(proof.verify(key, value, smt.root));
        let new_value = random();
        let new_root = proof
            .verify_and_update(key, value, new_value, smt.root)
            .unwrap();
        smt.update(key, new_value).unwrap();
        assert_eq!(smt.root, new_root);
    }

    #[test]
    fn test_inclusion_proof_zero_doesnt_update() {
        let mut smt = Smt::<H, DEPTH>::new();
        let num_keys = thread_rng().gen_range(1..100);
        let kvs: Vec<(u32, _)> = (0..num_keys).map(|_| (random(), random())).collect();
        check_no_duplicates(&kvs);
        for (key, value) in kvs.iter() {
            smt.insert(*key, *value).unwrap();
        }
        let (key, value) = kvs[thread_rng().gen_range(0..num_keys)];
        assert_ne!(value, smt.empty_hash_at_height[0], "Check your rng");
        let root = smt.root;
        let proof = smt.get_inclusion_proof_zero(key);
        assert!(proof.is_err(), "The key is in the SMT");
        assert_eq!(root, smt.root, "The SMT should not be updated");
    }
}
