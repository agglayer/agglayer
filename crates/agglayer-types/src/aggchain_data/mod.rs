use agglayer_interop_types::aggchain_proof::AggchainData;
use serde::{Deserialize, Serialize};

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
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PayloadWithCtx<Payload, Context>(pub Payload, pub Context);

// FIXME: To remove, global::Payload should replace all the aggchain data from
// the Certificate and API
// NOTE: This is temporary to have minimal backward compatibility
impl From<AggchainData> for global::Payload {
    fn from(value: AggchainData) -> Self {
        match value {
            AggchainData::ECDSA { signature } => global::Payload::LegacyEcdsa { signature },
            AggchainData::Generic {
                proof,
                aggchain_params,
                signature,
                public_values,
            } => global::Payload::AggchainProofOnly {
                signature: *signature.unwrap(), /* this signature must be mandatory, need
                                                 * fixing backward compatibility */
                aggchain_proof: aggchain_proof::Payload {
                    proof,
                    aggchain_params,
                    public_values,
                },
            },
        }
    }
}
