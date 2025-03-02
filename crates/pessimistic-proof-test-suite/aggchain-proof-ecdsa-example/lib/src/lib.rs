use alloy_primitives::{Address, PrimitiveSignature};
use pessimistic_proof_core::aggchain_proof::AggchainProofPublicValues;
use serde::{Deserialize, Serialize};

pub type Digest = [u8; 32];

#[derive(Serialize, Deserialize)]
pub struct AggchainECDSA {
    /// Previous local exit root.
    pub prev_local_exit_root: Digest,
    /// New local exit root.
    pub new_local_exit_root: Digest,
    /// Commitment to the imported bridge exits indexes.
    pub commit_imported_bridge_exits: Digest,
    /// L1 info root used to import bridge exits.
    pub l1_info_root: Digest,
    /// Origin network for which the proof was generated.
    pub origin_network: u32,
    /// Signer (aggchain_params).
    pub signer: Address,
    /// Signature of the commitment.
    pub signature: PrimitiveSignature,
}

impl AggchainECDSA {
    /// Field which is re-constructed by the L1.
    /// The eth address of 20bytes is padded to 32bytes.
    pub fn aggchain_params(&self) -> [u8; 32] {
        let mut aggchain_params = [0; 32];
        aggchain_params[12..32].copy_from_slice(self.signer.0.as_slice());
        aggchain_params
    }

    pub fn public_values(&self) -> AggchainProofPublicValues {
        AggchainProofPublicValues {
            prev_local_exit_root: self.prev_local_exit_root.into(),
            new_local_exit_root: self.new_local_exit_root.into(),
            l1_info_root: self.l1_info_root.into(),
            origin_network: self.origin_network,
            commit_imported_bridge_exits: self.commit_imported_bridge_exits.into(),
            aggchain_params: self.aggchain_params().into(),
        }
    }
}
