//! Stuff common to certificates across multiple versions.

use agglayer_interop_types::{aggchain_proof::AggchainData, LocalExitRoot};
use agglayer_primitives::{Address, Digest, Hashable, Signature, B256};
use pessimistic_proof::{core::commitment::SignatureCommitmentValues, keccak::keccak256_combine};
use unified_bridge::{
    BridgeExit, CommitmentVersion, ImportedBridgeExit, ImportedBridgeExitCommitmentValues,
    NetworkId,
};

use super::{CertificateId, Height, Version};
use crate::{Error, SignerError};

/// Core certificate fields common to all versions.
//
// TODO: This could be broken down into multiple more granular structs
// with different subsets of fields based on overlap between certificate
// version features. Or turned into a number of traits.
pub struct Fields<'a> {
    pub version: Version,
    pub network_id: NetworkId,
    pub height: Height,
    pub prev_local_exit_root: &'a LocalExitRoot,
    pub new_local_exit_root: &'a LocalExitRoot,
    pub bridge_exits: &'a [BridgeExit],
    pub imported_bridge_exits: &'a [ImportedBridgeExit],
    pub aggchain_data: &'a AggchainData,
    pub l1_info_tree_leaf_count: Option<u32>,
}

impl Fields<'_> {
    pub(crate) fn hash<const N: usize>(&self, extra: [&[u8]; N]) -> CertificateId {
        // Version 1+ fills the original position of network_id and height
        // with (u32::MAX, u64::MAX) and adds version.
        let commit_version_1plus = {
            let mut prefix = [0xff; 16];
            prefix[12..16].copy_from_slice(&self.version.as_u32().to_be_bytes());
            prefix
        };

        let commit_version: &[u8] = match self.version {
            // Backwards compatible, V0 is without a prefix.
            Version::V0 => &[],
            _ => &commit_version_1plus,
        };

        let commit_bridge_exits =
            keccak256_combine(self.bridge_exits.iter().map(|exit| exit.hash()));
        let commit_imported_bridge_exits =
            keccak256_combine(self.imported_bridge_exits.iter().map(|exit| exit.hash()));

        let network_id_bytes = self.network_id.to_be_bytes();
        let height_bytes = self.height.as_u64().to_be_bytes();

        let fields = [
            commit_version,
            network_id_bytes.as_slice(),
            height_bytes.as_slice(),
            self.prev_local_exit_root.as_ref(),
            self.new_local_exit_root.as_ref(),
            commit_bridge_exits.as_slice(),
            commit_imported_bridge_exits.as_slice(),
        ];

        CertificateId::new(keccak256_combine(fields.into_iter().chain(extra)))
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

    pub fn signature_commitment_values(&self) -> SignatureCommitmentValues {
        SignatureCommitmentValues {
            new_local_exit_root: *self.new_local_exit_root,
            commit_imported_bridge_exits: ImportedBridgeExitCommitmentValues {
                claims: self
                    .imported_bridge_exits
                    .iter()
                    .map(|exit| exit.to_indexed_exit_hash())
                    .collect(),
            },
            height: self.height.as_u64(),
        }
    }

    /// Verify the signature on the PP commitment.
    pub fn verify_cert_signature(&self, expected_signer: Address) -> Result<(), SignerError> {
        let pp_commitment_values = self.signature_commitment_values();

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
        let commitment_values = self.signature_commitment_values();

        let (signature, commitment) = match &self.aggchain_data {
            AggchainData::ECDSA { signature } => {
                let commitment = commitment_values.commitment(version);
                (signature, commitment)
            }
            AggchainData::Generic {
                signature,
                aggchain_params,
                ..
            } => {
                let signature = signature.as_ref().ok_or(SignerError::Missing)?;
                let commitment = commitment_values.aggchain_proof_commitment(aggchain_params);
                (signature.as_ref(), commitment)
            }
        };

        signature
            .recover_address_from_prehash(&B256::new(commitment.0))
            .map_err(SignerError::Recovery)
    }
}

/// Computes the commitment used to verify the extra signature on the
/// agglayer only.
/// The commitment is expected to be on the certificate id and the optional
/// l1 info tree leaf count.
fn extra_signature_commitment(
    certificate_id: CertificateId,
    l1_info_tree_leaf_count: Option<u32>,
) -> Digest {
    match l1_info_tree_leaf_count {
        Some(leaf_count) => keccak256_combine([
            certificate_id.as_digest().as_slice(),
            leaf_count.to_le_bytes().as_slice(),
        ]),
        None => *certificate_id,
    }
}

/// Verify the extra certificate signature.
pub fn verify_extra_signature(
    certificate_id: CertificateId,
    l1_info_tree_leaf_count: Option<u32>,
    expected_signer: Address,
    signature: Signature,
) -> Result<(), SignerError> {
    let expected_commitment = extra_signature_commitment(certificate_id, l1_info_tree_leaf_count);

    let retrieved_signer = signature
        .recover_address_from_prehash(&B256::new(expected_commitment.0))
        .map_err(SignerError::Recovery)?;

    (expected_signer == retrieved_signer)
        .then_some(())
        .ok_or(SignerError::InvalidExtraSignature { expected_signer })
}
