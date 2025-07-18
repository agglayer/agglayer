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

pub use header::{CertificateHeader, CertificateStatus, SettlementTxHash};
pub use height::Height;
pub use id::CertificateId;
pub use index::CertificateIndex;
pub use metadata::Metadata;
#[cfg(feature = "testutils")]
pub use testutils::compute_signature_info;

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
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
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
    /// Fixed size field of arbitrary data for the chain needs.
    pub metadata: Metadata,
    /// Aggchain data which is either one ECDSA or Generic proof.
    #[serde(flatten)]
    pub aggchain_data: AggchainData,
    #[serde(default)]
    pub custom_chain_data: Vec<u8>,
    #[serde(default)]
    pub l1_info_tree_leaf_count: Option<u32>,
}

impl Certificate {
    pub fn hash(&self) -> CertificateId {
        let commit_bridge_exits =
            keccak256_combine(self.bridge_exits.iter().map(|exit| exit.hash()));
        let commit_imported_bridge_exits =
            keccak256_combine(self.imported_bridge_exits.iter().map(|exit| exit.hash()));

        CertificateId::new(keccak256_combine([
            self.network_id.to_be_bytes().as_slice(),
            self.height.as_u64().to_be_bytes().as_slice(),
            self.prev_local_exit_root.as_ref(),
            self.new_local_exit_root.as_ref(),
            commit_bridge_exits.as_slice(),
            commit_imported_bridge_exits.as_slice(),
            self.metadata.0.as_slice(),
        ]))
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

    /// Retrieve the signer from the provided signature.
    pub fn signer_from_signature(&self, signature: Signature) -> Result<Address, SignerError> {
        // TODO: Verify for both commitment versions and return the version
        let version = CommitmentVersion::V2;
        let commitment = SignatureCommitmentValues::from(self).commitment(version);

        signature
            .recover_address_from_prehash(&B256::new(commitment.0))
            .map_err(SignerError::Recovery)
    }

    pub fn signer(&self) -> Result<Address, SignerError> {
        let (signature, commitment) = match &self.aggchain_data {
            AggchainData::ECDSA { signature } => {
                let version = CommitmentVersion::V2;
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
