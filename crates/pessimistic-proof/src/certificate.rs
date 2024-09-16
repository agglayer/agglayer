use std::collections::{BTreeMap, BTreeSet};

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

/// Represents the data submitted by the chains to the AggLayer.
///
/// The bridge exits plus the imported bridge exits define
/// the state transition, resp. the amount that goes out and the amount that
/// comes in.
///
/// The bridge exits refer to the [`BridgeExit`] emitted by
/// the origin network of the [`Certificate`].
///
/// The imported bridge exits refer to the [`BridgeExit`] received and imported
/// by the origin network of the [`Certificate`].
///
/// Note: be mindful to update the [`Self::hash`] method accordingly
/// upon modifying the fields of this structure.
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
            // Set of dedup tokens mutated in the transition
            let mutated_tokens: BTreeSet<TokenInfo> = {
                let imported_tokens = self
                    .imported_bridge_exits
                    .iter()
                    .map(|exit| exit.bridge_exit.token_info);
                let exported_tokens = self.bridge_exits.iter().map(|exit| exit.token_info);
                imported_tokens.chain(exported_tokens).collect()
            };

            // Get the proof against the initial balance for each token
            mutated_tokens
                .into_iter()
                .map(|token| {
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
                    (token, (initial_balance, path))
                })
                .collect()
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
