use agglayer_types::{compute_signature_info, Address, Certificate, LocalNetworkStateData, U256};
use ecdsa_proof_lib::AggchainECDSA;
use ethers_signers::{LocalWallet, Signer};
pub use pessimistic_proof::bridge_exit::LeafType;
use pessimistic_proof::{
    bridge_exit::{BridgeExit, TokenInfo},
    global_index::GlobalIndex,
    imported_bridge_exit::{
        commit_imported_bridge_exits, Claim, ClaimFromMainnet, ImportedBridgeExit, L1InfoTreeLeaf,
        L1InfoTreeLeafInner, MerkleProof,
    },
    keccak::{digest::Digest, keccak256, keccak256_combine},
    local_exit_tree::{data::LocalExitTreeData, hasher::Keccak256Hasher, LocalExitTree},
    local_state::LocalNetworkState,
    utils::{smt::Smt, Hashable as _},
    PessimisticProofOutput,
};
use rand::random;
use sp1_sdk::{ProverClient, SP1Proof, SP1Stdin, SP1VerifyingKey};

type NetworkId = u32;

use super::sample_data::{NETWORK_A, NETWORK_B};
use crate::AGGCHAIN_PROOF_ECDSA_ELF;

pub fn compute_aggchain_proof(
    aggchain_ecdsa_witness: AggchainECDSA,
) -> (SP1Proof, SP1VerifyingKey, [u8; 32]) {
    let mut stdin = SP1Stdin::new();
    stdin.write(&aggchain_ecdsa_witness);

    let client = ProverClient::from_env();
    let (pk, vk) = client.setup(AGGCHAIN_PROOF_ECDSA_ELF);
    let proof = client
        .prove(&pk, &stdin)
        .compressed()
        .run()
        .expect("proving failed");

    (proof.proof, vk, aggchain_ecdsa_witness.aggchain_params())
}

/// Trees for the network B, as well as the LET for network A.
#[derive(Clone, Debug)]
pub struct Forest {
    pub wallet: LocalWallet,
    pub l1_info_tree: LocalExitTreeData<Keccak256Hasher>,
    pub local_exit_tree_data_a: LocalExitTreeData<Keccak256Hasher>,
    pub state_b: LocalNetworkStateData,
}

impl Default for Forest {
    fn default() -> Self {
        Self::new([])
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
            wallet: Certificate::wallet_for_test(NETWORK_B),
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
        events: impl IntoIterator<Item = (TokenInfo, U256)>,
    ) -> Vec<ImportedBridgeExit> {
        let mut res = Vec::new();

        let exits: Vec<BridgeExit> = events
            .into_iter()
            .map(|(token, amount)| exit_to_b(token, amount))
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
                    rollup_index: *NETWORK_A,
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

    /// Local state associated with this forest.
    pub fn local_state(&self) -> LocalNetworkState {
        LocalNetworkState::from(self.state_b.clone())
    }

    /// Apply a sequence of events and return the corresponding [`Certificate`].
    pub fn apply_bridge_exits(
        &mut self,
        imported_bridge_events: impl IntoIterator<Item = (TokenInfo, U256)>,
        bridge_exits: impl IntoIterator<Item = BridgeExit>,
    ) -> Certificate {
        let prev_local_exit_root = self.state_b.exit_tree.get_root();

        let imported_bridge_exits = self.imported_bridge_exits(imported_bridge_events);
        let bridge_exits = bridge_exits
            .into_iter()
            .inspect(|exit| {
                self.state_b.exit_tree.add_leaf(exit.hash()).unwrap();
            })
            .collect();

        let new_local_exit_root = self.state_b.exit_tree.get_root();

        let (_combined_hash, signature, _signer) =
            compute_signature_info(new_local_exit_root, &imported_bridge_exits, &self.wallet);

        Certificate {
            network_id: NETWORK_B,
            height: 0,
            prev_local_exit_root,
            new_local_exit_root,
            bridge_exits,
            imported_bridge_exits,
            signature,
            metadata: Default::default(),
        }
    }

    /// Apply a sequence of events and return the corresponding [`Certificate`].
    pub fn apply_events(
        &mut self,
        imported_bridge_events: &[(TokenInfo, U256)],
        bridge_events: &[(TokenInfo, U256)],
    ) -> Certificate {
        let imported_bridge_events = imported_bridge_events.iter().cloned();
        let bridge_exits = bridge_events.iter().map(|(tok, amt)| exit_to_a(*tok, *amt));
        self.apply_bridge_exits(imported_bridge_events, bridge_exits)
    }

    /// Apply a sequence of events and return the corresponding [`Certificate`].
    pub fn apply_events_with_aggchain_proof(
        &mut self,
        imported_bridge_events: &[(TokenInfo, U256)],
        bridge_events: &[(TokenInfo, U256)],
    ) -> (Certificate, SP1VerifyingKey, [u8; 32], SP1Proof) {
        let certificate = self.apply_events(imported_bridge_events, bridge_events);

        let (aggchain_proof, aggchain_vkey, aggchain_params) =
            compute_aggchain_proof(AggchainECDSA {
                signer: certificate.signer().unwrap(),
                signature: certificate.signature.into(),
                commit_imported_bridge_exits: *commit_imported_bridge_exits(
                    certificate
                        .imported_bridge_exits
                        .iter()
                        .map(|i| i.global_index),
                ),
                prev_local_exit_root: *certificate.prev_local_exit_root,
                new_local_exit_root: *certificate.new_local_exit_root,
                l1_info_root: *certificate.l1_info_root().unwrap().unwrap(),
                origin_network: *NETWORK_B,
            });

        (certificate, aggchain_vkey, aggchain_params, aggchain_proof)
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
    exit(token_info, *NETWORK_A, amount)
}

fn exit_to_b(token_info: TokenInfo, amount: U256) -> BridgeExit {
    exit(token_info, *NETWORK_B, amount)
}
