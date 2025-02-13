//! Aggchain proof data structures.
//!
//! The aggchain-proof is the extra custom logic which is verified within the
//! pessimistic-proof.
//!
//! For now, this is constraint to be either one ECDSA signature, or one SP1
//! stark proof proving a specified statement which can be abstracted here.
use agglayer_primitives::{Address, Signature};
use serde::{Deserialize, Serialize};
use sha2::{Digest as Sha256Digest, Sha256};
use sp1_zkvm::lib::utils::words_to_bytes_le;

use crate::keccak::{digest::Digest, keccak256_combine};

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum AggchainType {
    ECDSA = 0,
    SP1 = 1,
}

pub type Vkey = [u32; 8];

/// Aggchain Proof which is either one ECDSA signature, or one stark proof.
/// Contains all the necessary data for verification.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AggchainProofData {
    /// ECDSA signature.
    ECDSA(AggchainProofECDSAData),
    /// STARK proof generated with SP1 and its metadata.
    SP1(AggchainProofSP1Data),
}

impl AggchainProofData {
    /// Returns the aggchain hash
    pub fn aggchain_hash(&self) -> Digest {
        match &self {
            AggchainProofData::ECDSA(aggchain_proof_ecdsa) => keccak256_combine([
                &(AggchainType::ECDSA as u32).to_be_bytes(),
                aggchain_proof_ecdsa.signer.as_slice(),
            ]),
            AggchainProofData::SP1(aggchain_proof_sp1) => keccak256_combine([
                &(AggchainType::SP1 as u32).to_be_bytes(),
                words_to_bytes_le(&aggchain_proof_sp1.aggchain_vkey).as_slice(),
                aggchain_proof_sp1.aggchain_params.as_slice(),
            ]),
        }
    }
}

/// ECDSA variant of the aggchain proof.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AggchainProofECDSAData {
    /// Signer committing to the state transition.
    pub signer: Address,
    /// Signature committing to the state transition.
    pub signature: Signature,
}

/// SP1 variant of the aggchain proof.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AggchainProofSP1Data {
    /// Chain-specific commitment forwarded by the PP.
    pub aggchain_params: Digest,
    /// SP1 verifying key for the SP1 aggchain proof program.
    pub aggchain_vkey: Vkey,
}

/// Public values to verify the SP1 aggchain proof.
#[derive(Serialize, Deserialize)]
pub struct AggchainProofPublicValues {
    /// Previous local exit root.
    pub prev_local_exit_root: Digest,
    /// New local exit root.
    pub new_local_exit_root: Digest,
    /// L1 info root used to import bridge exits.
    pub l1_info_root: Digest,
    /// Origin network for which the proof was generated.
    pub origin_network: u32,
    /// Commitment to the imported bridge exits indexes.
    pub commit_imported_bridge_exits: Digest,
    /// Chain-specific commitment forwarded by the PP.
    pub aggchain_params: Digest,
}

impl AggchainProofPublicValues {
    pub fn hash(&self) -> [u8; 32] {
        let public_values = [
            self.prev_local_exit_root.as_slice(),
            self.new_local_exit_root.as_slice(),
            self.l1_info_root.as_slice(),
            &self.origin_network.to_le_bytes(),
            self.commit_imported_bridge_exits.as_slice(),
            self.aggchain_params.as_slice(),
        ]
        .concat();

        Sha256::digest(&public_values).into()
    }
}
