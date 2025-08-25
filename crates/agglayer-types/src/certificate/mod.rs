use agglayer_interop_types::{aggchain_proof::AggchainData, LocalExitRoot};
use agglayer_primitives::{Address, Hashable, Signature, B256};
use pessimistic_proof::{core::commitment::SignatureCommitmentValues, keccak::keccak256_combine};
use unified_bridge::{
    BridgeExit, CommitmentVersion, ImportedBridgeExit, ImportedBridgeExitCommitmentValues,
    NetworkId,
};

use crate::{Digest, Error, SignerError};

mod header;
mod height;
mod id;
mod index;
mod metadata;
#[cfg(feature = "testutils")]
mod testutils;
mod v0;
mod v1;

pub use header::{CertificateHeader, CertificateStatus, SettlementTxHash};
pub use height::Height;
pub use id::CertificateId;
pub use index::CertificateIndex;
pub use metadata::Metadata;
#[cfg(feature = "testutils")]
pub use testutils::compute_signature_info;
pub use v0::FieldsV0;
pub use v1::FieldsV1;

/// Holds certificate fields that are specific to particular version(s).
#[derive(Clone, Debug, derive_more::From, serde::Serialize, serde::Deserialize)]
pub enum VersionFields {
    V0(v0::FieldsV0),
    V1(v1::FieldsV1),
}

impl VersionFields {
    pub const fn metadata(&self) -> Option<&Metadata> {
        match self {
            Self::V0(FieldsV0 { metadata }) => Some(metadata),
            Self::V1(FieldsV1 {}) => None,
        }
    }

    fn metadata_slice(&self) -> &[u8] {
        self.metadata().map_or(&[], |m| m.as_slice())
    }

    const fn id_preimage_prefix(&self) -> &[u8] {
        match self {
            Self::V0(_) => FieldsV0::ID_PREIMAGE_PREFIX,
            Self::V1(_) => FieldsV1::ID_PREIMAGE_PREFIX,
        }
    }
}

/// Represents the data submitted by the chains to the AggLayer.
///
/// The bridge exits plus the imported bridge exits define
/// the state transition, resp. the amount that goes out and the amount that
/// comes in.
///
/// The bridge exits refer to the [`BridgeExit`] emitted by
/// the origin network of the [`Certificate`].
///
/// The imported bridge exits refer to the [`BridgeExit`] received and imported
/// by the origin network of the [`Certificate`].
///
/// Note: be mindful to update the [`Self::hash`] method accordingly
/// upon modifying the fields of this structure.
#[derive(Clone, Debug)]
pub struct Certificate {
    /// NetworkID of the origin network.
    pub network_id: NetworkId,
    /// Simple increment to count the Certificate per network.
    pub height: Height,
    /// Previous local exit root.
    pub prev_local_exit_root: LocalExitRoot,
    /// New local exit root.
    pub new_local_exit_root: LocalExitRoot,
    /// List of bridge exits included in this state transition.
    pub bridge_exits: Vec<BridgeExit>,
    /// List of imported bridge exits included in this state transition.
    pub imported_bridge_exits: Vec<ImportedBridgeExit>,
    /// Aggchain data which is either one ECDSA or Generic proof.
    pub aggchain_data: AggchainData,
    pub custom_chain_data: Vec<u8>,
    pub l1_info_tree_leaf_count: Option<u32>,

    pub extra_fields: VersionFields,
}

impl serde::Serialize for Certificate {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        todo!()
    }
}

impl<'de> serde::Deserialize<'de> for Certificate {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        todo!()
    }
}

impl Certificate {
    pub fn hash(&self) -> CertificateId {
        let commit_bridge_exits =
            keccak256_combine(self.bridge_exits.iter().map(|exit| exit.hash()));
        let commit_imported_bridge_exits =
            keccak256_combine(self.imported_bridge_exits.iter().map(|exit| exit.hash()));

        CertificateId::new(keccak256_combine([
            self.extra_fields.id_preimage_prefix(),
            self.network_id.to_be_bytes().as_slice(),
            self.height.as_u64().to_be_bytes().as_slice(),
            self.prev_local_exit_root.as_ref(),
            self.new_local_exit_root.as_ref(),
            commit_bridge_exits.as_slice(),
            commit_imported_bridge_exits.as_slice(),
            self.extra_fields.metadata_slice(),
        ]))
    }

    pub const fn metadata(&self) -> Option<&Metadata> {
        self.extra_fields.metadata()
    }

    /// Returns the L1 Info Tree leaf count considered for this [`Certificate`].
    /// Corresponds to the highest L1 Info Tree leaf index considered by the
    /// imported bridge exits.
    pub fn l1_info_tree_leaf_count(&self) -> Option<u32> {
        self.l1_info_tree_leaf_count.or_else(|| {
            self.imported_bridge_exits
                .iter()
                .map(|i| i.l1_leaf_index() + 1)
                .max()
        })
    }

    /// Returns the L1 Info Root considered for this [`Certificate`].
    /// Fails if multiple L1 Info Root are considered among the inclusion proofs
    /// of the imported bridge exits.
    pub fn l1_info_root(&self) -> Result<Option<Digest>, Error> {
        let Some(l1_info_root) = self
            .imported_bridge_exits
            .first()
            .map(|imported_bridge_exit| imported_bridge_exit.l1_info_root())
        else {
            return Ok(None);
        };

        if self
            .imported_bridge_exits
            .iter()
            .all(|exit| exit.l1_info_root() == l1_info_root)
        {
            Ok(Some(l1_info_root))
        } else {
            Err(Error::MultipleL1InfoRoot)
        }
    }

    /// Computes the commitment used to verify the extra signature on the
    /// agglayer only.
    /// The commitment is expected to be on the certificate id and the optional
    /// l1 info tree leaf count.
    fn extra_signature_commitment(&self) -> Digest {
        let certificate_id = self.hash();

        match self.l1_info_tree_leaf_count {
            Some(leaf_count) => keccak256_combine([
                certificate_id.as_digest().as_slice(),
                leaf_count.to_le_bytes().as_slice(),
            ]),
            None => *certificate_id,
        }
    }

    /// Verify the extra certificate signature.
    pub fn verify_extra_signature(
        &self,
        expected_signer: Address,
        signature: Signature,
    ) -> Result<(), SignerError> {
        let expected_commitment = self.extra_signature_commitment();

        let retrieved_signer = signature
            .recover_address_from_prehash(&B256::new(expected_commitment.0))
            .map_err(SignerError::Recovery)?;

        (expected_signer == retrieved_signer)
            .then_some(())
            .ok_or(SignerError::InvalidExtraSignature { expected_signer })
    }

    /// Verify the signature on the PP commitment.
    pub fn verify_cert_signature(&self, expected_signer: Address) -> Result<(), SignerError> {
        let pp_commitment_values = SignatureCommitmentValues::from(self);

        let recovered_expected_signer = match &self.aggchain_data {
            // Verify if one of the commitment version is signed.
            // NOTE: The legitimacy of the version is verified during the witness generation,
            // especially in order to forbid version rollback by the chain.
            AggchainData::ECDSA { signature } => [CommitmentVersion::V3, CommitmentVersion::V2]
                .iter()
                .any(|version| {
                    let commitment = B256::new(pp_commitment_values.commitment(*version).0);
                    match signature.recover_address_from_prehash(&commitment) {
                        Ok(recovered) => recovered == expected_signer,
                        Err(_) => false,
                    }
                }),
            AggchainData::Generic {
                signature,
                aggchain_params,
                ..
            } => {
                let signature = signature.as_ref().ok_or(SignerError::Missing)?;
                let commitment = B256::new(
                    pp_commitment_values
                        .aggchain_proof_commitment(aggchain_params)
                        .0,
                );
                let recovered = signature
                    .recover_address_from_prehash(&commitment)
                    .map_err(SignerError::Recovery)?;

                recovered == expected_signer
            }
        };

        recovered_expected_signer
            .then_some(())
            .ok_or(SignerError::InvalidPessimisticProofSignature { expected_signer })
    }

    /// Retrieve the signer from the certificate signature.
    pub fn retrieve_signer(&self, version: CommitmentVersion) -> Result<Address, SignerError> {
        let (signature, commitment) = match &self.aggchain_data {
            AggchainData::ECDSA { signature } => {
                let commitment = SignatureCommitmentValues::from(self).commitment(version);
                (signature, commitment)
            }
            AggchainData::Generic {
                signature,
                aggchain_params,
                ..
            } => {
                let signature = signature.as_ref().ok_or(SignerError::Missing)?;
                let commitment = SignatureCommitmentValues::from(self)
                    .aggchain_proof_commitment(aggchain_params);
                (signature.as_ref(), commitment)
            }
        };

        signature
            .recover_address_from_prehash(&B256::new(commitment.0))
            .map_err(SignerError::Recovery)
    }
}

impl From<&Certificate> for SignatureCommitmentValues {
    fn from(certificate: &Certificate) -> Self {
        Self {
            new_local_exit_root: certificate.new_local_exit_root,
            commit_imported_bridge_exits: ImportedBridgeExitCommitmentValues {
                claims: certificate
                    .imported_bridge_exits
                    .iter()
                    .map(|exit| exit.to_indexed_exit_hash())
                    .collect(),
            },
            height: certificate.height.as_u64(),
        }
    }
}
