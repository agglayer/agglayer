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
use pessimistic_proof::core::{self, MultisigError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::aggchain_data::{aggchain_proof, multisig, PayloadWithCtx};

/// Represents the data needed from the API/Certificate to verify aggchain
/// proofs and multisig.
/// Made separately for now to not have to deal with storage and API design.
#[derive(Clone, Debug, strum_macros::Display)]
pub enum Payload {
    LegacyEcdsa {
        /// ECDSA Signature from the trusted sequencer.
        signature: Signature,
    },
    MultisigOnly(multisig::Payload),
    MultisigAndAggchainProof {
        multisig: multisig::Payload,
        aggchain_proof: aggchain_proof::Payload,
    },
}

/// Represents the context fetched from L1 and/or defined by the agglayer.
#[derive(Clone, Debug, strum_macros::Display)]
pub enum Context {
    LegacyEcdsa {
        /// Address of the trusted sequencer.
        signer: Address,
    },
    MultisigOnly(multisig::Ctx),
    MultisigAndAggchainProof {
        multisig_ctx: multisig::Ctx,
        aggchain_proof_ctx: aggchain_proof::Context,
    },
}

#[derive(Clone, Debug, Error, Deserialize, Serialize, Eq, PartialEq)]
pub enum AggchainDataError {
    #[error("Invalid variant: {0}")]
    InvalidVariant(String),

    #[error("Invalid multisig: {0}")]
    InvalidMultisig(#[source] MultisigError),

    #[error("Aggchain proof comes without its ECDSA")]
    MissingSignature,
}

// Generate the prover inputs from the chain payload and the L1 context.
impl TryInto<core::AggchainData> for PayloadWithCtx<Payload, Context> {
    type Error = AggchainDataError;

    fn try_into(self) -> Result<core::AggchainData, Self::Error> {
        let PayloadWithCtx(payload, ctx) = self;
        match (payload, ctx) {
            (Payload::LegacyEcdsa { signature }, Context::LegacyEcdsa { signer }) => {
                Ok(core::AggchainData::LegacyEcdsa { signer, signature })
            }
            (Payload::MultisigOnly(payload), Context::MultisigOnly(ctx)) => {
                let prehash = ctx.prehash;
                let multisig = core::MultiSignature::from(PayloadWithCtx(payload, ctx));
                multisig
                    .verify(prehash)
                    .map_err(AggchainDataError::InvalidMultisig)?;
                Ok(core::AggchainData::MultisigOnly(multisig))
            }
            (
                Payload::MultisigAndAggchainProof {
                    multisig,
                    aggchain_proof,
                },
                Context::MultisigAndAggchainProof {
                    multisig_ctx,
                    aggchain_proof_ctx,
                },
            ) => {
                let prehash = multisig_ctx.prehash;
                let multisig = core::MultiSignature::from(PayloadWithCtx(multisig, multisig_ctx));
                multisig
                    .verify(prehash)
                    .map_err(AggchainDataError::InvalidMultisig)?;
                Ok(core::AggchainData::MultisigAndAggchainProof {
                    multisig,
                    aggchain_proof: PayloadWithCtx(aggchain_proof, aggchain_proof_ctx).into(),
                })
            }
            (payload, context) => Err(AggchainDataError::InvalidVariant(format!(
                "payload: {payload}, context: {context}"
            ))),
        }
    }
}
