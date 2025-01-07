use agglayer_primitives::{Address, U256};
use agglayer_types::{compute_signature_info, Certificate, LocalNetworkStateData};
use ethers_signers::{LocalWallet, Signer};
pub use pessimistic_proof::bridge_exit::LeafType;
use pessimistic_proof::{
    bridge_exit::{BridgeExit, TokenInfo},
    global_index::GlobalIndex,
    imported_bridge_exit::{
        Claim, ClaimFromMainnet, ImportedBridgeExit, L1InfoTreeLeaf, L1InfoTreeLeafInner,
        MerkleProof,
    },
    keccak::{digest::Digest, keccak256, keccak256_combine},
    local_exit_tree::{data::LocalExitTreeData, hasher::Keccak256Hasher, LocalExitTree},
    local_state::LocalNetworkState,
    utils::{smt::Smt, Hashable as _},
    PessimisticProofOutput,
};
use rand::{random, thread_rng};

type NetworkId = u32;

use super::sample_data::{NETWORK_A, NETWORK_B};

/// Trees for the network B, as well as the LET for network A.
#[derive(Clone)]
pub struct Forest {
    pub wallet: LocalWallet,
    pub l1_info_tree: LocalExitTreeData<Keccak256Hasher>,
    pub local_exit_tree_data_a: LocalExitTreeData<Keccak256Hasher>,
    pub state_b: LocalNetworkStateData,
}

impl Default for Forest {
    fn default() -> Self {
        let local_balance_tree = Smt::new();

        Self {
            wallet: LocalWallet::new(&mut thread_rng()),
            local_exit_tree_data_a: LocalExitTreeData::new(),
            l1_info_tree: Default::default(),
            state_b: LocalNetworkStateData {
                exit_tree: LocalExitTree::new(),
                balance_tree: local_balance_tree,
                nullifier_tree: Smt::new(),
            },
        }
    }
}

impl Forest {
    pub fn with_signer_seed(mut self, seed: u32) -> Self {
        let fake_priv_key = keccak256_combine([b"FAKEKEY:", seed.to_be_bytes().as_slice()]);
        self.wallet = LocalWallet::from_bytes(fake_priv_key.as_bytes()).unwrap();

        self
    }

    pub fn with_signer(mut self, signer: LocalWallet) -> Self {
        self.wallet = signer;

        self
    }

    /// Create a new forest based on given initial balances.
    pub fn new(initial_balances: impl IntoIterator<Item = (TokenInfo, U256)>) -> Self {
        Self::new_with_local_exit_tree(initial_balances, LocalExitTree::new())
    }

    /// Override the local exit tree for network B
    pub fn new_with_local_exit_tree(
        initial_balances: impl IntoIterator<Item = (TokenInfo, U256)>,
        local_exit_tree: LocalExitTree<Keccak256Hasher>,
    ) -> Self {
        let mut local_balance_tree = Smt::new();
        for (token, balance) in initial_balances {
            local_balance_tree
                .insert(token, balance.to_be_bytes().into())
                .unwrap();
        }

        Self {
            wallet: LocalWallet::new(&mut thread_rng()),
            local_exit_tree_data_a: LocalExitTreeData::new(),
            l1_info_tree: Default::default(),
            state_b: LocalNetworkStateData {
                exit_tree: local_exit_tree,
                balance_tree: local_balance_tree,
                nullifier_tree: Smt::new(),
            },
        }
    }

    /// Imported bridge exits from network A to network B.
    pub fn imported_bridge_exits(
        &mut self,
        events: &[(TokenInfo, U256)],
    ) -> Vec<ImportedBridgeExit> {
        let mut res = Vec::new();

        let exits: Vec<BridgeExit> = events
            .iter()
            .map(|(token, amount)| exit_to_b(*token, *amount))
            .collect();

        // Append all the leafs in LET A (mainnet)
        for exit in &exits {
            self.local_exit_tree_data_a.add_leaf(exit.hash()).unwrap();
        }

        let l1_leaf = L1InfoTreeLeaf {
            l1_info_tree_index: 0,
            rer: Digest::default(),
            mer: self.local_exit_tree_data_a.get_root(),
            inner: L1InfoTreeLeafInner {
                block_hash: Digest::default(),
                timestamp: 0,
                global_exit_root: Digest::default(),
            },
        };

        self.l1_info_tree.add_leaf(l1_leaf.hash()).unwrap();

        let proof_ger_l1root = MerkleProof {
            proof: self.l1_info_tree.get_proof(0).unwrap(),
            root: self.l1_info_tree.get_root(),
        };

        // Generate them as imported bridge exits
        for (idx, exit) in exits.into_iter().enumerate() {
            let index = idx as u32;
            let imported_exit = ImportedBridgeExit {
                bridge_exit: exit,
                global_index: GlobalIndex {
                    mainnet_flag: true,
                    rollup_index: **NETWORK_A,
                    leaf_index: index,
                },
                claim_data: Claim::Mainnet(Box::new(ClaimFromMainnet {
                    proof_leaf_mer: MerkleProof {
                        proof: self.local_exit_tree_data_a.get_proof(index).unwrap(),
                        root: self.local_exit_tree_data_a.get_root(),
                    },
                    proof_ger_l1root: proof_ger_l1root.clone(),
                    l1_leaf: l1_leaf.clone(),
                })),
            };
            res.push(imported_exit);
        }

        res
    }

    /// Bridge exits from network B to network A.
    pub fn bridge_exits(&mut self, events: &[(TokenInfo, U256)]) -> Vec<BridgeExit> {
        let mut res = Vec::new();
        for (token, amount) in events {
            let exit = exit_to_a(*token, *amount);
            self.state_b.exit_tree.add_leaf(exit.hash()).unwrap();
            res.push(exit);
        }

        res
    }

    /// Local state associated with this forest.
    pub fn local_state(&self) -> LocalNetworkState {
        LocalNetworkState::from(self.state_b.clone())
    }

    /// Apply a sequence of events and return the corresponding [`Certificate`].
    pub fn apply_events(
        &mut self,
        imported_bridge_events: &[(TokenInfo, U256)],
        bridge_events: &[(TokenInfo, U256)],
    ) -> Certificate {
        let prev_local_exit_root = self.state_b.exit_tree.get_root();
        let imported_bridge_exits = self.imported_bridge_exits(imported_bridge_events);
        let bridge_exits = self.bridge_exits(bridge_events);
        let new_local_exit_root = self.state_b.exit_tree.get_root();
        let (_combined_hash, signature) =
            compute_signature_info(new_local_exit_root, &imported_bridge_exits, &self.wallet);

        Certificate {
            network_id: (*NETWORK_B),
            height: 0,
            prev_local_exit_root,
            new_local_exit_root,
            bridge_exits,
            imported_bridge_exits,
            signature,
            metadata: Default::default(),
        }
    }

    pub fn get_signer(&self) -> Address {
        self.wallet.address().0.into()
    }

    /// Check the current state corresponds to given proof output.
    pub fn assert_output_matches(&self, output: &PessimisticProofOutput) {
        assert_eq!(
            output.new_local_exit_root,
            self.state_b.exit_tree.get_root()
        );
        assert_eq!(
            output.new_pessimistic_root,
            keccak256_combine([
                self.state_b.balance_tree.root.as_slice(),
                self.state_b.nullifier_tree.root.as_slice(),
                self.state_b.exit_tree.leaf_count().to_le_bytes().as_slice(),
            ])
        );
    }
}

fn exit(token_info: TokenInfo, dest_network: NetworkId, amount: U256) -> BridgeExit {
    BridgeExit {
        leaf_type: LeafType::Transfer,
        token_info,
        dest_network: dest_network.into(),
        dest_address: random::<[u8; 20]>().into(),
        amount,
        metadata: Some(keccak256(&[])),
    }
}

fn exit_to_a(token_info: TokenInfo, amount: U256) -> BridgeExit {
    exit(token_info, **NETWORK_A, amount)
}

fn exit_to_b(token_info: TokenInfo, amount: U256) -> BridgeExit {
    exit(token_info, **NETWORK_B, amount)
}
