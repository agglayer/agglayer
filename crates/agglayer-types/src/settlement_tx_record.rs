use crate::SettlementTxHash;

/// Contains historical data about settlement attempts for one certificate.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct SettlementTxRecord {
    // Hash data, uniqued and in the order of insertion
    hashes: Vec<SettlementTxHash>,
}

impl SettlementTxRecord {
    pub const fn new() -> Self {
        Self::from_vec(Vec::new())
    }

    pub const fn from_vec(hashes: Vec<SettlementTxHash>) -> Self {
        Self { hashes }
    }

    pub const fn len(&self) -> usize {
        self.hashes.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.hashes.is_empty()
    }

    pub const fn hashes(&self) -> &[SettlementTxHash] {
        self.hashes.as_slice()
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

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &SettlementTxHash> + '_ {
        self.hashes.iter()
    }

    pub fn into_vec(self) -> Vec<SettlementTxHash> {
        self.hashes
    }
}
