use agglayer_types::SettlementTxHash;

use super::VersionTag;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct SettlementTxHashRecord {
    // Version tag for future storage evolution
    version: VersionTag<0>,

    // Hash data, uniqued and in the order of insertion
    hashes: Vec<SettlementTxHash>,
}

impl SettlementTxHashRecord {
    pub const fn new() -> Self {
        Self {
            version: VersionTag,
            hashes: Vec::new(),
        }
    }

    pub const fn len(&self) -> usize {
        self.hashes.len()
    }

    pub fn contains(&self, hash: &SettlementTxHash) -> bool {
        self.hashes.contains(hash)
    }

    pub fn insert(&mut self, hash: SettlementTxHash) {
        // If we already have this hash, put it last.
        if let Some(orig_idx) = self.hashes.iter().position(|h| h == &hash) {
            self.hashes.remove(orig_idx);
        }
        self.hashes.push(hash);
    }

    pub fn retain(&mut self, f: impl FnMut(&SettlementTxHash) -> bool) {
        self.hashes.retain(f);
    }

    pub fn into_vec(self) -> Vec<SettlementTxHash> {
        self.hashes
    }
}
