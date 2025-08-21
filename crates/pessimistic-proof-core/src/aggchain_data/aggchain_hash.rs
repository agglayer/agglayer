use agglayer_primitives::{
    bytes::{BigEndian, ByteOrder as _},
    keccak::keccak256_combine,
    Address, Digest,
};

use crate::aggchain_data::{aggchain_proof::AggchainProof, AggchainData, Vkey};

type ConsensusType = u32;

pub enum AggchainHashValues {
    ConsensusType0 {
        signer: Address,
    },
    ConsensusType1 {
        aggchain_vkey: Option<Vkey>,
        aggchain_params: Option<Digest>,
        signers_commit: Option<Digest>,
        threshold: Option<usize>,
    },
}

impl From<&AggchainHashValues> for ConsensusType {
    fn from(value: &AggchainHashValues) -> Self {
        match value {
            AggchainHashValues::ConsensusType0 { .. } => 0,
            AggchainHashValues::ConsensusType1 { .. } => 1,
        }
    }
}

impl AggchainHashValues {
    /// Returns the commitment on signers in case of no signer.
    pub fn empty_signers() -> Digest {
        keccak256_combine([Digest::default()])
    }

    /// Returns the empty vkey used in case of no aggchain proof
    pub fn empty_sp1_vkey() -> Vkey {
        Vkey::default()
    }

    /// Returns empty threshold
    pub fn empty_threshold() -> usize {
        1
    }

    pub fn empty_aggchain_params() -> Digest {
        Digest::default()
    }

    /// Computes the aggchain hash with the right default values.
    pub(crate) fn hash(&self) -> Digest {
        let consensus_type: u32 = ConsensusType::from(self);

        match self {
            AggchainHashValues::ConsensusType0 { signer } => {
                keccak256_combine([&consensus_type.to_be_bytes(), signer.as_slice()])
            }
            AggchainHashValues::ConsensusType1 {
                aggchain_vkey,
                aggchain_params,
                signers_commit,
                threshold,
            } => {
                let aggchain_vkey_hash = {
                    let vkey = aggchain_vkey.unwrap_or_else(Self::empty_sp1_vkey);
                    let mut aggchain_vkey_hash = [0u8; 32];
                    BigEndian::write_u32_into(&vkey, &mut aggchain_vkey_hash);
                    aggchain_vkey_hash
                };

                let aggchain_params = aggchain_params.unwrap_or_else(Self::empty_aggchain_params);
                let signers = signers_commit.unwrap_or_else(Self::empty_signers);
                let threshold = threshold.unwrap_or_else(Self::empty_threshold);

                keccak256_combine([
                    &consensus_type.to_be_bytes(),
                    aggchain_vkey_hash.as_slice(),
                    aggchain_params.as_slice(),
                    signers.as_slice(),
                    &threshold.to_be_bytes(),
                ])
            }
        }
    }
}

impl From<&AggchainData> for AggchainHashValues {
    fn from(value: &AggchainData) -> Self {
        match value {
            AggchainData::LegacyEcdsa { signer, .. } => {
                AggchainHashValues::ConsensusType0 { signer: *signer }
            }
            AggchainData::MultisigOnly(multi_signature) => AggchainHashValues::ConsensusType1 {
                aggchain_vkey: None,
                aggchain_params: None,
                signers_commit: Some(multi_signature.signers_commit()),
                threshold: Some(multi_signature.threshold),
            },
            AggchainData::AggchainProofOnly(AggchainProof {
                aggchain_params,
                aggchain_vkey,
            }) => AggchainHashValues::ConsensusType1 {
                aggchain_vkey: Some(*aggchain_vkey),
                aggchain_params: Some(*aggchain_params),
                signers_commit: None,
                threshold: None,
            },
            AggchainData::MultisigAndAggchainProof {
                multisig,
                aggchain_proof:
                    AggchainProof {
                        aggchain_params,
                        aggchain_vkey,
                    },
            } => AggchainHashValues::ConsensusType1 {
                aggchain_vkey: Some(*aggchain_vkey),
                aggchain_params: Some(*aggchain_params),
                signers_commit: Some(multisig.signers_commit()),
                threshold: Some(multisig.threshold),
            },
        }
    }
}
