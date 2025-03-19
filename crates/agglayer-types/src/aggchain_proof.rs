use agglayer_primitives::Signature;
use serde::{Deserialize, Serialize};
use sp1_core_machine::reduce::SP1ReduceProof;
use sp1_prover::InnerSC;

use crate::Digest;

// Aggchain data submitted via the [`Certificate`].
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "testutils", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum AggchainData {
    ECDSA {
        signature: Signature,
    },
    Generic {
        /// proof of the aggchain proof.
        proof: Proof,
        /// Chain-specific commitment forwarded through the PP.
        aggchain_params: Digest,
    },
}

pub type SP1StarkProof = SP1ReduceProof<InnerSC>;

/// Proof that is part of the aggchain proof submitted via the [`Certificate`].
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Proof {
    SP1Stark(Box<SP1StarkProof>),
}

#[cfg(feature = "testutils")]
impl<'a> arbitrary::Arbitrary<'a> for Proof {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        use bincode::Options as _;
        let bytes = <&[u8]>::arbitrary(u)?;
        let proof = std::panic::catch_unwind(|| {
            bincode::options()
                .with_limit(bytes.len() as u64)
                .deserialize(bytes)
        })
        .map_err(|_| arbitrary::Error::IncorrectFormat)?
        .map_err(|e| match *e {
            bincode::ErrorKind::SizeLimit => arbitrary::Error::NotEnoughData,
            _ => arbitrary::Error::IncorrectFormat,
        })?;
        Ok(Proof::SP1Stark(Box::new(proof)))
    }
}

#[cfg(feature = "testutils")]
impl std::cmp::PartialEq for Proof {
    fn eq(&self, other: &Self) -> bool {
        bincode::serialize(self).unwrap() == bincode::serialize(other).unwrap()
    }
}

#[cfg(feature = "testutils")]
impl std::cmp::Eq for Proof {}

#[test]
fn regression_sp1_serialization_roundtrip_fail() {
    // Conclusion: sp1 serialization is not deterministic, removed the equality
    // check.
    use bincode::Options;
    let bytes = hex::decode("00000000000000000000000000000000000000000000fb00000100000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000").unwrap();
    let input: SP1StarkProof = bincode::options()
        .deserialize(&bytes)
        .expect("failed first deserialization, would be fine");
    let serialized: Vec<u8> = bincode::DefaultOptions::new()
        .with_big_endian()
        .with_fixint_encoding()
        .serialize(&input)
        .expect("failed serialization, unexpected");
    let _output: SP1StarkProof = bincode::DefaultOptions::new()
        .with_big_endian()
        .with_fixint_encoding()
        .deserialize(&serialized)
        .expect("failed second deserialization, is unexpected");
}
