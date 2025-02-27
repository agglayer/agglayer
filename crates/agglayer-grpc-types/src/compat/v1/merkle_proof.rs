use agglayer_types::{Digest, MerkleProof};

use super::Error;
use crate::protocol::types::v1;

impl TryFrom<v1::MerkleProof> for MerkleProof {
    type Error = Error;

    fn try_from(value: v1::MerkleProof) -> Result<Self, Self::Error> {
        if value.siblings.len() != 32 {
            return Err(Error::WrongVectorLength {
                expected: 32,
                actual: value.siblings.len(),
            });
        }
        let siblings: Vec<Digest> = value
            .siblings
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(|e| Error::ParsingField("siblings", Box::new(e)))?;
        let siblings: [Digest; 32] = siblings.try_into().unwrap(); // Checked just two statements above
        Ok(MerkleProof::new(required_field!(value, root), siblings))
    }
}

impl From<MerkleProof> for v1::MerkleProof {
    fn from(value: MerkleProof) -> Self {
        v1::MerkleProof {
            root: Some(value.root.into()),
            siblings: value.siblings().iter().copied().map(Into::into).collect(),
        }
    }
}
