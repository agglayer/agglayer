//! Represents the payload received from the chain, and the context fetched from
//! the L1, for all the possible cases.
//!
//! 1. legacy ecdsa, kept as-is for now because of the v2~v3 migration struggles
//!   - needs one ecdsa signature only
//!
//! 2. current generic, receive one aggchain proof with trusted sequencer
//!    signature verified only on the agglayer
//!   - stark proof
//!   - single ecdsa verified on the agglayer (+ optional pv for debug)
//!   - aggchain params
//!
//! 3. generic + multisig (one or the other, or both)
//!   - aggchain proof + single ecdsa (trusted sequencer, not multisig) ->
//!     single signature verified only on the agglayer
//!   - aggchain proof + multisig (multi signer, not necessarily trusted
//!     sequencer)
//!   - multisig
use agglayer_primitives::{Address, Signature};
use pessimistic_proof::core::{self};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::aggchain_data::{
    aggchain_proof::{self},
    multisig::{self, MultisigError},
    PayloadWithCtx,
};

/// Represents the data needed from the API/Certificate to verify aggchain
/// proofs and multisig.
/// Made separately for now to not have to deal with storage and API design.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Payload {
    LegacyEcdsa {
        /// ECDSA Signature from the trusted sequencer.
        signature: Signature,
    },
    MultisigOnly(multisig::Payload),
    AggchainProofOnly {
        /// ECDSA Signature from the trusted sequencer.
        signature: Signature,
        aggchain_proof: aggchain_proof::Payload,
    },
    MultisigAndAggchainProof {
        multisig: multisig::Payload,
        aggchain_proof: aggchain_proof::Payload,
    },
}

/// Represents the context fetched from L1 and/or defined by the agglayer.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Ctx {
    LegacyEcdsa {
        /// Address of the trusted sequencer.
        signer: Address,
    },
    MultisigOnly(multisig::Ctx),
    AggchainProofOnly(aggchain_proof::Ctx),
    MultisigAndAggchainProof {
        multisig_ctx: multisig::Ctx,
        aggchain_proof_ctx: aggchain_proof::Ctx,
    },
}

#[derive(Clone, Debug, Error, Deserialize, Serialize, Eq, PartialEq)]
pub enum AggchainDataError {
    #[error("Invalid variant: {0}")]
    InvalidVariant(String),

    #[error("Invalid multisig: {0}")]
    InvalidMultisig(#[source] MultisigError),
}

// Generate the prover inputs from the chain payload and the L1 context.
impl TryInto<core::AggchainData> for PayloadWithCtx<Payload, Ctx> {
    type Error = AggchainDataError;

    fn try_into(self) -> Result<core::AggchainData, Self::Error> {
        let PayloadWithCtx(payload, ctx) = self;
        Ok(match (payload, ctx) {
            (Payload::LegacyEcdsa { signature }, Ctx::LegacyEcdsa { signer }) => {
                core::AggchainData::LegacyEcdsa { signer, signature }
            }
            (Payload::MultisigOnly(payload), Ctx::MultisigOnly(ctx)) => {
                core::AggchainData::MultisigOnly(
                    PayloadWithCtx(payload, ctx)
                        .try_into()
                        .map_err(AggchainDataError::InvalidMultisig)?,
                )
            }
            (Payload::AggchainProofOnly { aggchain_proof, .. }, Ctx::AggchainProofOnly(ctx)) => {
                core::AggchainData::AggchainProofOnly(PayloadWithCtx(aggchain_proof, ctx).into())
            }
            (
                Payload::MultisigAndAggchainProof {
                    multisig,
                    aggchain_proof,
                },
                Ctx::MultisigAndAggchainProof {
                    multisig_ctx,
                    aggchain_proof_ctx,
                },
            ) => core::AggchainData::MultisigAndAggchainProof {
                multisig: PayloadWithCtx(multisig, multisig_ctx)
                    .try_into()
                    .map_err(AggchainDataError::InvalidMultisig)?,
                aggchain_proof: PayloadWithCtx(aggchain_proof, aggchain_proof_ctx).into(),
            },
            (payload, ctx) => {
                return Err(AggchainDataError::InvalidVariant(format!(
                    "payload: {payload:?}, ctx: {ctx:?}"
                )))
            }
        })
    }
}

// For error and debugging
impl ToString for Payload {
    fn to_string(&self) -> String {
        match self {
            Payload::LegacyEcdsa { .. } => String::from("legacy_ecdsa"),
            Payload::MultisigOnly(_) => String::from("multisig_only"),
            Payload::AggchainProofOnly { .. } => String::from("aggchain_proof_only"),
            Payload::MultisigAndAggchainProof { .. } => String::from("multisig_and_aggchain_proof"),
        }
    }
}

// For error and debugging
impl ToString for Ctx {
    fn to_string(&self) -> String {
        match self {
            Ctx::LegacyEcdsa { .. } => String::from("legacy_ecdsa"),
            Ctx::MultisigOnly(_) => String::from("multisig_only"),
            Ctx::AggchainProofOnly { .. } => String::from("aggchain_proof_only"),
            Ctx::MultisigAndAggchainProof { .. } => String::from("multisig_and_aggchain_proof"),
        }
    }
}
