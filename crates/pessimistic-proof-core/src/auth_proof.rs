//! Auth proof data structures.
//!
//! The auth-proof is the extra custom logic which is verified within the
//! pessimistic-proof.
//!
//! For now, this is constraint to be either one ECDSA signature, or one SP1
//! plonk proof proving a specified statement which can be abstracted here.
use agglayer_primitives::{Address, Signature};
use serde::{Deserialize, Serialize};
use sha2::{Digest as Sha256Digest, Sha256};
use sp1_zkvm::lib::utils::words_to_bytes_le;

use crate::{
    keccak::{digest::Digest, keccak256_combine},
    proof::PessimisticConsensusType,
};

pub type Vkey = [u32; 8];
pub type PlonkVkey = Vec<u8>;
pub type PlonkProof = Vec<u8>;

/// Auth Proof which is either one ECDSA signature, or one Plonk proof.
/// Contains all the necessary data for verification.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AuthProofData {
    /// ECDSA signature.
    ECDSA(AuthProofECDSAData),
    /// Plonk proof generated with SP1 and its metadata.
    SP1(AuthProofSP1Data),
}

impl AuthProofData {
    /// Returns the auth hash
    pub fn auth_hash(&self) -> Digest {
        match &self {
            AuthProofData::ECDSA(auth_proof_ecdsa) => keccak256_combine([
                &(PessimisticConsensusType::ECDSA as u32).to_be_bytes(),
                auth_proof_ecdsa.signer.as_slice(),
            ]),
            AuthProofData::SP1(auth_proof_sp1) => keccak256_combine([
                &auth_proof_sp1.auth_type.to_be_bytes(),
                words_to_bytes_le(&auth_proof_sp1.auth_vkey).as_slice(),
                auth_proof_sp1.auth_plonk_vkey.as_slice(),
                auth_proof_sp1.auth_params.as_slice(),
            ]),
        }
    }
}

/// ECDSA variant of the auth proof.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthProofECDSAData {
    /// Signer committing to the state transition.
    pub signer: Address,
    /// Signature committing to the state transition.
    pub signature: Signature,
}

/// SP1 variant of the auth proof.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthProofSP1Data {
    /// Chain-specific commitment forwarded by the PP.
    pub auth_params: Digest,
    /// Auth type.
    pub auth_type: u32,
    /// SP1 verifying key for the SP1 auth proof program.
    pub auth_vkey: Vkey,
    /// Plonk verifying key for a specific SP1 version.
    pub auth_plonk_vkey: PlonkVkey,
    /// Snark plonk proof.
    pub plonk_proof: PlonkProof,
}

/// Public values to verify the SP1 auth proof.
pub struct AuthProofPublicValues {
    /// Previous local exit root.
    pub prev_local_exit_root: Digest,
    /// L1 info root used to import bridge exits.
    pub l1_info_root: Digest,
    /// Origin network for which the proof was generated.
    pub origin_network: u32,
    /// Commitment to the imported bridge exits indexes.
    pub commit_imported_bridge_exits: Digest,
    /// Chain-specific commitment forwarded by the PP.
    pub auth_params: Digest,
}

impl AuthProofPublicValues {
    pub fn hash(&self) -> [u8; 32] {
        let public_values = [
            self.prev_local_exit_root.as_slice(),
            self.l1_info_root.as_slice(),
            &self.origin_network.to_be_bytes(),
            self.commit_imported_bridge_exits.as_slice(),
            self.auth_params.as_slice(),
        ]
        .concat();

        Sha256::digest(&public_values).into()
    }
}
