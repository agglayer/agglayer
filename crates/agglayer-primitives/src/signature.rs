use alloy_primitives::PrimitiveSignature;
use k256::ecdsa;

use super::{Address, SignatureError, B256, U256};

/// A wrapper over [PrimitiveSignature] with custom serialization.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(from = "compat::Signature", into = "compat::Signature")]
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

    pub fn as_primitive_signature(&self) -> &PrimitiveSignature {
        &self.0
    }

    pub fn as_bytes(&self) -> [u8; 65] {
        self.0.as_bytes()
    }

    pub fn r(&self) -> U256 {
        self.0.r()
    }

    pub fn s(&self) -> U256 {
        self.0.s()
    }

    pub fn v(&self) -> bool {
        self.0.v()
    }
}

impl From<PrimitiveSignature> for Signature {
    fn from(ps: PrimitiveSignature) -> Self {
        Self(ps)
    }
}

impl From<Signature> for PrimitiveSignature {
    fn from(value: Signature) -> Self {
        value.0
    }
}

impl TryFrom<&[u8]> for Signature {
    type Error = SignatureError;

    fn try_from(sig: &[u8]) -> Result<Self, Self::Error> {
        sig.try_into().map(Self)
    }
}

impl std::str::FromStr for Signature {
    type Err = <PrimitiveSignature as std::str::FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
    }
}

/// Helpers for serialization / deserialization format compatibility.
mod compat {
    use super::U256;

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Signature {
        r: U256,
        s: U256,
        odd_y_parity: bool,
    }

    impl Signature {
        fn new(r: U256, s: U256, odd_y_parity: bool) -> Self {
            Self { r, s, odd_y_parity }
        }
    }

    impl From<super::Signature> for Signature {
        fn from(sig: super::Signature) -> Self {
            Self::new(sig.0.r(), sig.0.s(), sig.0.v())
        }
    }

    impl From<Signature> for super::Signature {
        fn from(sig: Signature) -> Self {
            let Signature { r, s, odd_y_parity } = sig;
            Self::new(r, s, odd_y_parity)
        }
    }
}
