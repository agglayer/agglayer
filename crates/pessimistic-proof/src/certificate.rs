use std::collections::BTreeMap;

use reth_primitives::{Address, Signature, U256};
use serde::{Deserialize, Serialize};

use crate::bridge_exit::NetworkId;
use crate::keccak::keccak256_combine;
use crate::local_exit_tree::hasher::Keccak256Hasher;
use crate::local_state::{LocalNetworkStateData, StateCommitment};
use crate::{
    bridge_exit::BridgeExit,
    imported_bridge_exit::ImportedBridgeExit,
    keccak::Digest,
    multi_batch_header::MultiBatchHeader,
    nullifier_tree::{NullifierKey, NullifierPath},
};
use crate::{bridge_exit::TokenInfo, local_balance_tree::LocalBalancePath};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Conversion error: {0}")]
    TypeConversion(&'static str),
}

pub type Height = u64;

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct Certificate {
    pub network_id: NetworkId,
    pub height: Height,
    pub prev_local_exit_root: Digest,
    pub new_local_exit_root: Digest,
    pub bridge_exits: Vec<BridgeExit>,
    pub imported_bridge_exits: Vec<ImportedBridgeExit>,
    pub signature: Signature,
}

impl Certificate {
    pub fn new_for_test(network_id: NetworkId, height: Height) -> Self {
        Self {
            network_id,
            height,
            prev_local_exit_root: [0; 32],
            new_local_exit_root: [0; 32],
            bridge_exits: Vec::new(),
            imported_bridge_exits: Vec::new(),
            signature: Signature {
                r: U256::ZERO,
                s: U256::ZERO,
                odd_y_parity: false,
            },
        }
    }

    pub fn hash(&self) -> Digest {
        keccak256_combine([
            self.network_id.to_be_bytes().as_slice(),
            self.height.to_be_bytes().as_slice(),
            self.prev_local_exit_root.as_slice(),
            self.new_local_exit_root.as_slice(),
        ])
    }

    pub fn into_pessimistic_proof_input(
        &self,
        state: &LocalNetworkStateData,
        signer: Address,
    ) -> Result<MultiBatchHeader<Keccak256Hasher>, Error> {
        let prev_balance_root = state.balance_tree.root;
        let prev_nullifier_root = state.nullifier_set.root;

        let balances_proofs: BTreeMap<TokenInfo, (U256, LocalBalancePath<Keccak256Hasher>)> = {
            let mut res = BTreeMap::new();

            let mutated_tokens = {
                let imported_tokens = self
                    .imported_bridge_exits
                    .iter()
                    .map(|exit| exit.bridge_exit.token_info);
                let exported_tokens = self.bridge_exits.iter().map(|exit| exit.token_info);
                imported_tokens.chain(exported_tokens)
            };

            mutated_tokens.for_each(|token| {
                let initial_balance =
                    U256::from_be_bytes(state.balance_tree.get(token).unwrap_or_default());
                let path = if initial_balance.is_zero() {
                    // TODO: dont clone once get_inclusion_proof_zero doesnt mutate anymore
                    state
                        .balance_tree
                        .clone()
                        .get_inclusion_proof_zero(token)
                        .unwrap()
                } else {
                    state.balance_tree.get_inclusion_proof(token).unwrap()
                };
                res.insert(token, (initial_balance, path));
            });
            res
        };

        let imported_bridge_exits: Vec<(ImportedBridgeExit, NullifierPath<Keccak256Hasher>)> = self
            .imported_bridge_exits
            .iter()
            .map(|exit| {
                let null_key = NullifierKey {
                    network_id: exit.global_index.network_id(),
                    let_index: exit.global_index.leaf_index,
                };
                let nullifier_path = state
                    .nullifier_set
                    .get_non_inclusion_proof(null_key)
                    .unwrap();
                (exit.clone(), nullifier_path)
            })
            .collect();

        Ok(MultiBatchHeader::<Keccak256Hasher> {
            origin_network: self.network_id,
            prev_local_exit_root: self.prev_local_exit_root,
            bridge_exits: self.bridge_exits.clone(),
            imported_bridge_exits,
            balances_proofs,
            prev_balance_root,
            prev_nullifier_root,
            signer,
            signature: self.signature,
            imported_rollup_exit_root: [0; 32],
            imported_mainnet_exit_root: [0; 32],
            imported_exits_root: None,
            target: StateCommitment {
                exit_root: self.new_local_exit_root,
                balance_root: Default::default(), // computed by agglayer
                nullifier_root: Default::default(), // computed by agglayer
            },
        })
    }
}
