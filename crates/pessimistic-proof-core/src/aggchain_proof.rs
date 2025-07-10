//! Aggchain proof data structures.
//!
//! The aggchain-proof is the extra custom logic which is verified within the
//! pessimistic-proof.
//!
//! For now, this is constraint to be either one ECDSA signature, or one SP1
//! stark proof proving a specified statement which can be abstracted here.
use agglayer_primitives::{
    bytes::{BigEndian, ByteOrder as _},
    Address, Digest, Signature,
};
use serde::{Deserialize, Serialize};

use crate::keccak::keccak256_combine;

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum ConsensusType {
    ECDSA = 0,
    Generic = 1,
}

pub type Vkey = [u32; 8];

/// Aggchain Data which is either one ECDSA signature, or one generic proof.
/// Contains all the necessary data for verification.
#[derive(
    Clone, Debug, Serialize, Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub enum AggchainData {
    /// ECDSA signature.
    ECDSA {
        /// Signer committing to the state transition.
        signer: Address,
        /// Signature committing to the state transition.
        signature: Signature,
    },
    /// Generic proof and its metadata.
    Generic {
        /// Chain-specific commitment forwarded by the PP.
        aggchain_params: Digest,
        /// Verifying key for the aggchain proof program.
        aggchain_vkey: Vkey,
    },
}

impl AggchainData {
    /// Returns the aggchain hash
    pub fn aggchain_hash(&self) -> Digest {
        match &self {
            AggchainData::ECDSA { signer, .. } => keccak256_combine([
                &(ConsensusType::ECDSA as u32).to_be_bytes(),
                signer.as_slice(),
            ]),
            AggchainData::Generic {
                aggchain_params,
                aggchain_vkey,
            } => {
                let mut aggchain_vkey_hash = [0u8; 32];
                BigEndian::write_u32_into(aggchain_vkey, &mut aggchain_vkey_hash);

                keccak256_combine([
                    &(ConsensusType::Generic as u32).to_be_bytes(),
                    aggchain_vkey_hash.as_slice(),
                    aggchain_params.as_slice(),
                ])
            }
        }
    }
}
