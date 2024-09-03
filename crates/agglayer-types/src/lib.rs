use std::collections::BTreeMap;

pub use pessimistic_proof::local_exit_tree::hasher::Keccak256Hasher;
use pessimistic_proof::{
    bridge_exit::BridgeExit, imported_bridge_exit::ImportedBridgeExit, keccak::Digest,
    multi_batch_header::MultiBatchHeader, LocalNetworkState,
};
use reth_primitives::{Address, Signature, U256};
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Converstion error: {0}")]
    TypeConversion(&'static str),
}

pub type EpochNumber = u64;
pub type CertificateIndex = u64;
pub type CertificateId = [u8; 32];
pub type Hash = [u8; 32];
pub type Height = u64;
pub type NetworkId = u32;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificateHeader {
    pub certificate_id: CertificateId,
    pub network_id: NetworkId,
    pub height: Height,
    pub epoch_number: Option<EpochNumber>,
    pub certificate_index: Option<CertificateIndex>,
    pub local_exit_root: Hash,
}

/// Proof is a wrapper around all the different types of proofs that can be
/// generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Proof {
    SP1(sp1_sdk::SP1ProofWithPublicValues),
}

impl Proof {
    pub fn new_for_test() -> Self {
        Proof::SP1(sp1_sdk::SP1ProofWithPublicValues {
            proof: sp1_sdk::SP1Proof::Core(Vec::new()),
            stdin: sp1_sdk::SP1Stdin::new(),
            public_values: sp1_core_machine::io::SP1PublicValues::new(),
            sp1_version: String::new(),
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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

    pub fn hash(&self) -> CertificateId {
        let data = [
            &(self.network_id as u64).to_be_bytes(),
            &self.height.to_be_bytes(),
            self.prev_local_exit_root.as_slice(),
            self.new_local_exit_root.as_slice(),
        ]
        .concat();

        pessimistic_proof::keccak::keccak256(data.as_slice())
    }

    pub fn into_pessimistic_proof_input(
        &self,
        state: &LocalNetworkState,
        signer: Address,
    ) -> Result<MultiBatchHeader<Keccak256Hasher>, Error> {
        let prev_balance_root = state.balance_tree.root;
        let prev_nullifier_root = state.nullifier_set.root;

        let balances_proofs = {
            #[allow(clippy::let_and_return)]
            let res = BTreeMap::new();
            // let mutated_tokens: BTreeSet<_> = self
            //     .imported_bridge_exits
            //     .iter()
            //     .map(|exit| exit.bridge_exit.token_info)
            //     .chain(self.bridge_exits)
            //     .collect();
            //
            // for token in mutated_tokens {
            //     let initial_balance =
            //         U256::from_be_bytes(self.local_balance_tree.get(token).
            // unwrap_or_default());     let path = if initial_balance.is_zero()
            // {         self.local_balance_tree
            //             .get_inclusion_proof_zero(token)
            //             .unwrap()
            //     } else {
            //         self.local_balance_tree.get_inclusion_proof(token).unwrap()
            //     };
            //     res.insert(token, (initial_balance, path));
            // }
            res
        };

        // let imported_bridge_exit = self.imported_bridge_exits.iter().map(|exit| {
        //     let nullifier_path = state
        //         .nullifier_set.get_non
        //     (exit.clone(), nullifier_path)
        // });

        Ok(MultiBatchHeader::<Keccak256Hasher> {
            origin_network: self.network_id.into(),
            prev_local_exit_root: self.prev_local_exit_root,
            new_local_exit_root: self.new_local_exit_root,
            bridge_exits: self.bridge_exits.clone(),
            imported_bridge_exits: Vec::new(),
            balances_proofs,
            prev_balance_root,
            new_balance_root: state.balance_tree.root,
            prev_nullifier_root,
            new_nullifier_root: state.nullifier_set.root,
            signer,
            signature: self.signature,
            imported_rollup_exit_root: [0; 32],
            imported_mainnet_exit_root: [0; 32],
            imported_exits_root: None,
        })
    }
}
