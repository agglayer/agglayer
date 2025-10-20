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

    pub fn insert(&mut self, hash: SettlementTxHash) -> bool {
        let do_insert = !self.contains(&hash);
        if do_insert {
            self.hashes.push(hash);
        }
        do_insert
    }

    pub fn retain(&mut self, f: impl FnMut(&SettlementTxHash) -> bool) {
        self.hashes.retain(f);
    }

    pub fn into_vec(self) -> Vec<SettlementTxHash> {
        self.hashes
    }
}
