use std::{
    collections::{BTreeMap, HashMap},
    ops::{Deref, DerefMut},
};

use reth_primitives::U256;
use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};

use crate::{
    keccak::Digest,
    withdrawal::{NetworkId, TokenInfo},
    Withdrawal,
};

/// Records all the deposits and withdrawals for each network.
///
/// Specifically, this records a map `network => (token_id => (deposit, withdraw))`: for each
/// network, the amounts withdrawn and deposited for every token are recorded.
///
/// Note: a "deposit" is the counterpart of a [`Withdrawal`]; a "withdrawal" from the source
/// network is a "deposit" in the destination network.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BalanceTreeByNetwork(BTreeMap<NetworkId, BalanceTree>);

impl BalanceTreeByNetwork {
    /// Creates a new empty [`BalanceTreeByNetwork`].
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// Updates the origin and destination network in the aggregate from a [`Withdrawal`].
    pub fn insert(&mut self, origin_network: NetworkId, withdrawal: Withdrawal) {
        // Withdraw the origin network
        self.0
            .entry(origin_network)
            .or_default()
            .withdraw(withdrawal.token_info.clone(), withdrawal.amount);

        // Deposit the destination network
        self.0
            .entry(withdrawal.dest_network)
            .or_default()
            .deposit(withdrawal.token_info, withdrawal.amount);
    }

    /// Merge two [`BalanceTreeByNetwork`].
    pub fn merge(&mut self, other: &BalanceTreeByNetwork) {
        for (network, balance_tree) in other.0.iter() {
            self.0
                .entry(*network)
                .and_modify(|bt| bt.merge(balance_tree))
                .or_insert(balance_tree.clone());
        }
    }
}

/// Merge a set of [`BalanceTreeByNetwork`].
pub fn merge_balance_trees(
    balance_trees: &HashMap<NetworkId, BalanceTreeByNetwork>,
) -> BalanceTreeByNetwork {
    let mut merged_balance_trees = BalanceTreeByNetwork::new();

    for balance_tree in balance_trees.values() {
        merged_balance_trees.merge(balance_tree);
    }

    merged_balance_trees
}

impl From<BTreeMap<NetworkId, BalanceTree>> for BalanceTreeByNetwork {
    fn from(value: BTreeMap<NetworkId, BalanceTree>) -> Self {
        Self(value)
    }
}

impl Deref for BalanceTreeByNetwork {
    type Target = BTreeMap<NetworkId, BalanceTree>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BalanceTreeByNetwork {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Record the balance as total deposit and total withdraw.
#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct Balance {
    deposit: U256,
    withdraw: U256,
}

pub struct Deposit(pub U256);
pub struct Withdraw(pub U256);

impl From<Deposit> for Balance {
    fn from(v: Deposit) -> Self {
        Self {
            deposit: v.0,
            withdraw: U256::ZERO,
        }
    }
}

impl From<Withdraw> for Balance {
    fn from(v: Withdraw) -> Self {
        Self {
            deposit: U256::ZERO,
            withdraw: v.0,
        }
    }
}

impl Balance {
    pub fn is_negative(&self) -> bool {
        self.withdraw > self.deposit
    }

    pub fn deposit(&mut self, amount: U256) {
        self.deposit += amount;
    }

    pub fn withdraw(&mut self, amount: U256) {
        self.withdraw += amount;
    }

    pub fn hash(&self) -> Digest {
        let mut hasher = Keccak::v256();

        hasher.update(&self.deposit.to_be_bytes::<32>());
        hasher.update(&self.withdraw.to_be_bytes::<32>());

        let mut output = [0u8; 32];
        hasher.finalize(&mut output);
        output
    }
}

/// Records the balances for each [`TokenInfo`].
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct BalanceTree(BTreeMap<TokenInfo, Balance>);

impl From<Vec<(TokenInfo, Balance)>> for BalanceTree {
    fn from(initial_balance: Vec<(TokenInfo, Balance)>) -> Self {
        Self(initial_balance.into_iter().collect())
    }
}

impl BalanceTree {
    /// Apply deposit to the given [`TokenInfo`].
    pub fn deposit(&mut self, token: TokenInfo, amount: U256) {
        self.0.entry(token).or_default().deposit(amount);
    }

    /// Apply withdraw to the given [`TokenInfo`].
    pub fn withdraw(&mut self, token: TokenInfo, amount: U256) {
        self.0.entry(token).or_default().withdraw(amount);
    }

    /// Merge with another [`BalanceTree`].
    pub fn merge(&mut self, other: &BalanceTree) {
        for (token, balance) in other.0.iter() {
            self.deposit(token.clone(), balance.deposit);
            self.withdraw(token.clone(), balance.withdraw)
        }
    }

    /// Returns whether any token has debt.
    /// TODO: We may want to return the debtor (token, debt)
    pub fn has_debt(&self) -> bool {
        self.0.iter().any(|(_, balance)| balance.is_negative())
    }

    /// Returns the hash of [`BalanceTree`].
    pub fn hash(&self) -> Digest {
        let mut hasher = Keccak::v256();

        for (token_info, balance) in self.0.iter() {
            hasher.update(&token_info.hash());
            hasher.update(&balance.hash());
        }

        let mut output = [0u8; 32];
        hasher.finalize(&mut output);
        output
    }
}
