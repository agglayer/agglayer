use alloy_primitives::PrimitiveSignature;
use k256::ecdsa;

use super::{Address, U256, B256, SignatureError};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Signature(PrimitiveSignature);

impl Signature {
    pub fn from_signature_and_parity(sig: ecdsa::Signature, v: bool) -> Self {
        PrimitiveSignature::from_signature_and_parity(sig, v).into()
    }

    pub fn new(r: U256, s: U256, v: bool) -> Self {
        PrimitiveSignature::new(r, s, v).into()
    }

    pub fn recover_address_from_prehash(&self, prehash: &B256) -> Result<Address, SignatureError> {
        self.0.recover_address_from_prehash(prehash)
    }
}

impl From<PrimitiveSignature> for Signature {
    fn from(ps: PrimitiveSignature) -> Self {
        Self(ps)
    }
}
