use agglayer_primitives::{
    bytes::{BigEndian, ByteOrder as _},
    keccak::keccak256_combine,
    Digest,
};
use hex_literal::hex;

use crate::aggchain_data::{aggchain_proof::AggchainProof, AggchainData, MultiSignature, Vkey};

struct ConsensusType(u32);

pub enum AggchainHashValues {
    ConsensusType1 {
        aggchain_vkey: Option<Vkey>,
        aggchain_params: Option<Digest>,
        multisig_hash: Option<Digest>,
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
    /// Value if no multisig.
    pub const EMPTY_MULTISIG_HASH: Digest = Digest(hex!(
        "290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563"
    ));

    /// Value if no aggchain vkey.
    pub const EMPTY_AGGCHAIN_VKEY: Digest = Digest::ZERO;

    /// Value if no aggchain params.
    pub const EMPTY_AGGCHAIN_PARAMS: Digest = Digest::ZERO;

    /// Computes the aggchain hash with the right default values.
    pub fn hash(&self) -> Digest {
        let consensus_type: u32 = ConsensusType::from(self).0;

        match self {
            AggchainHashValues::ConsensusType1 {
                aggchain_vkey: aggchain_vkey_u32,
                aggchain_params,
                multisig_hash,
            } => {
                let aggchain_vkey_hash = aggchain_vkey_u32
                    .map(|vkey| {
                        let mut aggchain_vkey_hash = [0u8; 32];
                        BigEndian::write_u32_into(&vkey, &mut aggchain_vkey_hash);
                        aggchain_vkey_hash
                    })
                    .unwrap_or(*Self::EMPTY_AGGCHAIN_VKEY);

                let aggchain_params = aggchain_params.unwrap_or(Self::EMPTY_AGGCHAIN_PARAMS);
                let multisig_hash = multisig_hash.unwrap_or(Self::EMPTY_MULTISIG_HASH);

                keccak256_combine([
                    &consensus_type.to_be_bytes(),
                    aggchain_vkey_hash.as_slice(),
                    aggchain_params.as_slice(),
                    multisig_hash.as_slice(),
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
                multisig_hash: Some(
                    MultiSignature {
                        signatures: vec![(0, *signature)],
                        expected_signers: vec![*signer],
                        threshold: 1,
                    }
                    .multisig_hash(),
                ),
            },
            AggchainData::MultisigOnly(multisig) => AggchainHashValues::ConsensusType1 {
                aggchain_vkey: None,
                aggchain_params: None,
                multisig_hash: Some(multisig.multisig_hash()),
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
                multisig_hash: Some(multisig.multisig_hash()),
            },
        }
    }
}
