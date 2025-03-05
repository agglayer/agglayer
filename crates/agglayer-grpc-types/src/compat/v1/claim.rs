use agglayer_types::{Claim, ClaimFromMainnet, ClaimFromRollup};

use super::Error;
use crate::protocol::types::v1;

impl TryFrom<v1::ClaimFromMainnet> for ClaimFromMainnet {
    type Error = Error;

    fn try_from(value: v1::ClaimFromMainnet) -> Result<Self, Self::Error> {
        Ok(ClaimFromMainnet {
            proof_leaf_mer: required_field!(value, proof_leaf_mer),
            proof_ger_l1root: required_field!(value, proof_ger_l1root),
            l1_leaf: required_field!(value, l1_leaf),
        })
    }
}

impl From<ClaimFromMainnet> for v1::ClaimFromMainnet {
    fn from(value: ClaimFromMainnet) -> Self {
        v1::ClaimFromMainnet {
            proof_leaf_mer: Some(value.proof_leaf_mer.into()),
            proof_ger_l1root: Some(value.proof_ger_l1root.into()),
            l1_leaf: Some(value.l1_leaf.into()),
        }
    }
}

impl TryFrom<v1::ClaimFromRollup> for ClaimFromRollup {
    type Error = Error;

    fn try_from(value: v1::ClaimFromRollup) -> Result<Self, Self::Error> {
        Ok(ClaimFromRollup {
            proof_leaf_ler: required_field!(value, proof_leaf_ler),
            proof_ler_rer: required_field!(value, proof_ler_rer),
            proof_ger_l1root: required_field!(value, proof_ger_l1root),
            l1_leaf: required_field!(value, l1_leaf),
        })
    }
}

impl From<ClaimFromRollup> for v1::ClaimFromRollup {
    fn from(value: ClaimFromRollup) -> Self {
        v1::ClaimFromRollup {
            proof_leaf_ler: Some(value.proof_leaf_ler.into()),
            proof_ler_rer: Some(value.proof_ler_rer.into()),
            proof_ger_l1root: Some(value.proof_ger_l1root.into()),
            l1_leaf: Some(value.l1_leaf.into()),
        }
    }
}

impl TryFrom<v1::imported_bridge_exit::Claim> for Claim {
    type Error = Error;

    fn try_from(value: v1::imported_bridge_exit::Claim) -> Result<Self, Self::Error> {
        Ok(match value {
            v1::imported_bridge_exit::Claim::Mainnet(claim_from_mainnet) => {
                Claim::Mainnet(Box::new(
                    claim_from_mainnet
                        .try_into()
                        .map_err(|e: Error| e.inside_field("claim_from_mainnet"))?,
                ))
            }
            v1::imported_bridge_exit::Claim::Rollup(claim_from_rollup) => Claim::Rollup(Box::new(
                claim_from_rollup
                    .try_into()
                    .map_err(|e: Error| e.inside_field("claim_from_rollup"))?,
            )),
        })
    }
}

impl From<Claim> for v1::imported_bridge_exit::Claim {
    fn from(value: Claim) -> Self {
        match value {
            Claim::Mainnet(claim_from_mainnet) => {
                v1::imported_bridge_exit::Claim::Mainnet((*claim_from_mainnet).into())
            }
            Claim::Rollup(claim_from_rollup) => {
                v1::imported_bridge_exit::Claim::Rollup((*claim_from_rollup).into())
            }
        }
    }
}
