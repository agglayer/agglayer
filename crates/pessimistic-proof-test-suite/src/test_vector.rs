use agglayer_types::Address;
use alloy::primitives::B256;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TestFile {
    pub specification: String,
    pub test_vectors: Vec<TestVector>,
}

#[derive(Debug, Deserialize)]
pub struct TestVector {
    pub inputs: Inputs,
    pub expected_output: ExpectedOutput,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Inputs {
    Multisig(MultisigInputs),
    Aggchain(AggchainInputs),
}

#[derive(Debug, Deserialize)]
pub struct MultisigInputs {
    pub threshold: u32,
    pub signers: Vec<Address>,
}

#[derive(Debug, Deserialize)]
pub struct AggchainInputs {
    pub aggchain_vkey: B256,
    pub aggchain_params: B256,
    pub multisig_hash: B256,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ExpectedOutput {
    Multisig { multisig_hash: B256 },
    Aggchain { aggchain_hash: B256 },
}

impl ExpectedOutput {
    pub fn as_hash(&self) -> B256 {
        match *self {
            ExpectedOutput::Multisig { multisig_hash } => multisig_hash,
            ExpectedOutput::Aggchain { aggchain_hash } => aggchain_hash,
        }
    }
}
