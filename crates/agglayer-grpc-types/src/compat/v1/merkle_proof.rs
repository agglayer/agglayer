use agglayer_types::{Digest, MerkleProof};

use crate::protocol::types::v1;

use super::Error;

impl TryFrom<v1::MerkleProof> for MerkleProof {
    type Error = Error;

    fn try_from(value: v1::MerkleProof) -> Result<Self, Self::Error> {
        let siblings: Vec<Digest> = value
            .siblings
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(|e| Error::ParsingField("siblings", Box::new(e)))?;
        let siblings: [Digest; 32] =
            siblings
                .try_into()
                .map_err(|s: Vec<_>| Error::WrongVectorLength {
                    expected: 32,
                    actual: s.len(),
                })?;
        Ok(MerkleProof::new(required_field!(value, root), siblings))
    }
}
