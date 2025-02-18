use agglayer_types::{Claim, ClaimFromMainnet, ClaimFromRollup};

use crate::protocol::types::v1;

use super::Error;

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

impl TryFrom<v1::imported_bridge_exit::Claim> for Claim {
    type Error = Error;

    fn try_from(value: v1::imported_bridge_exit::Claim) -> Result<Self, Self::Error> {
        Ok(match value {
            v1::imported_bridge_exit::Claim::Mainnet(claim_from_mainnet) => {
                Claim::Mainnet(Box::new(
                    claim_from_mainnet
                        .try_into()
                        .map_err(|e| Error::ParsingField("claim_from_mainnet", Box::new(e)))?,
                ))
            }
            v1::imported_bridge_exit::Claim::Rollup(claim_from_rollup) => Claim::Rollup(Box::new(
                claim_from_rollup
                    .try_into()
                    .map_err(|e| Error::ParsingField("claim_from_rollup", Box::new(e)))?,
            )),
        })
    }
}
