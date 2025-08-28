use agglayer_primitives::{
    bytes::{BigEndian, ByteOrder as _},
    keccak::keccak256_combine,
    Digest,
};

use crate::aggchain_data::{aggchain_proof::AggchainProof, AggchainData, MultiSignature, Vkey};

struct ConsensusType(u32);

pub enum AggchainHashValues {
    ConsensusType1 {
        aggchain_vkey: Option<Vkey>,
        aggchain_params: Option<Digest>,
        signers_commit: Option<Digest>,
    },
}

impl From<&AggchainHashValues> for ConsensusType {
    fn from(value: &AggchainHashValues) -> Self {
        match value {
            AggchainHashValues::ConsensusType1 { .. } => Self(1),
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
    pub fn empty_threshold() -> u32 {
        0u32
    }

    pub fn empty_aggchain_params() -> Digest {
        Digest::default()
    }

    /// Computes the aggchain hash with the right default values.
    pub(crate) fn hash(&self) -> Digest {
        let consensus_type: u32 = ConsensusType::from(self).0;

        match self {
            AggchainHashValues::ConsensusType1 {
                aggchain_vkey,
                aggchain_params,
                signers_commit,
            } => {
                let aggchain_vkey_hash = {
                    let vkey = aggchain_vkey.unwrap_or_else(Self::empty_sp1_vkey);
                    let mut aggchain_vkey_hash = [0u8; 32];
                    BigEndian::write_u32_into(&vkey, &mut aggchain_vkey_hash);
                    aggchain_vkey_hash
                };

                let aggchain_params = aggchain_params.unwrap_or_else(Self::empty_aggchain_params);
                let signers = signers_commit.unwrap_or_else(Self::empty_signers);

                keccak256_combine([
                    &consensus_type.to_be_bytes(),
                    aggchain_vkey_hash.as_slice(),
                    aggchain_params.as_slice(),
                    signers.as_slice(),
                ])
            }
        }
    }
}

impl From<&AggchainData> for AggchainHashValues {
    fn from(value: &AggchainData) -> Self {
        match value {
            AggchainData::LegacyEcdsa { signer, signature } => AggchainHashValues::ConsensusType1 {
                aggchain_vkey: None,
                aggchain_params: None,
                signers_commit: Some(
                    MultiSignature {
                        signatures: vec![(0, *signature)],
                        expected_signers: vec![*signer],
                        threshold: 1,
                    }
                    .signers_commit(),
                ),
            },
            AggchainData::MultisigOnly(multisig) => AggchainHashValues::ConsensusType1 {
                aggchain_vkey: None,
                aggchain_params: None,
                signers_commit: Some(multisig.signers_commit()),
            },
            AggchainData::AggchainProofOnly(AggchainProof {
                aggchain_params,
                aggchain_vkey,
            }) => AggchainHashValues::ConsensusType1 {
                aggchain_vkey: Some(*aggchain_vkey),
                aggchain_params: Some(*aggchain_params),
                signers_commit: None,
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
            },
        }
    }
}
