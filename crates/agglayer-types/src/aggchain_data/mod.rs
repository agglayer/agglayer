use agglayer_interop_types::aggchain_proof::AggchainData;

mod aggchain_proof;
mod global;
mod multisig;

// TODO: better naming for all the structures in this module, including this
// module name
pub use crate::aggchain_data::{
    aggchain_proof::{Ctx as AggchainProofCtx, Payload as AggchainProofPayload},
    global::{
        AggchainDataError, Ctx as CertificateAggchainDataCtx, Payload as CertificateAggchainData,
    },
    multisig::{Ctx as MultisigCtx, MultisigError, Payload as MultisigPayload},
    PayloadWithCtx as CertificateAggchainDataWithCtx,
};

/// Represents the payload from the chain, with the context fetched from the L1.
#[derive(Clone, Debug)]
pub struct PayloadWithCtx<Payload, Context>(pub Payload, pub Context);

// FIXME: To remove, global::Payload should replace all the aggchain data from
// the Certificate and API
// NOTE: This is temporary to have minimal backward compatibility
impl TryFrom<AggchainData> for global::Payload {
    type Error = AggchainDataError;
    fn try_from(value: AggchainData) -> Result<Self, Self::Error> {
        Ok(match value {
            AggchainData::ECDSA { signature } => global::Payload::LegacyEcdsa { signature },
            AggchainData::Generic {
                proof,
                aggchain_params,
                signature,
                public_values,
            } => global::Payload::AggchainProofOnly {
                signature: *signature.ok_or(AggchainDataError::MissingSignature)?,
                aggchain_proof: aggchain_proof::Payload {
                    proof,
                    aggchain_params,
                    public_values,
                },
            },
        })
    }
}
