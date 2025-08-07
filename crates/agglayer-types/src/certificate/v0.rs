use agglayer_interop_types::{aggchain_proof::AggchainData, LocalExitRoot};
use agglayer_primitives::{Address, Signature};
use unified_bridge::{BridgeExit, CommitmentVersion, ImportedBridgeExit, NetworkId};

use crate::{
    certificate::{common, CertificateId, Fields, Height, Metadata, Version},
    Digest, Error, SignerError,
};

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
pub struct CertificateV0 {
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

impl CertificateV0 {
    #[inline]
    pub fn fields(&self) -> Fields {
        Fields {
            version: Version::V0,
            network_id: self.network_id,
            height: self.height,
            prev_local_exit_root: &self.prev_local_exit_root,
            new_local_exit_root: &self.new_local_exit_root,
            bridge_exits: &self.bridge_exits,
            imported_bridge_exits: &self.imported_bridge_exits,
            aggchain_data: &self.aggchain_data,
            l1_info_tree_leaf_count: self.l1_info_tree_leaf_count,
        }
    }

    pub fn hash(&self) -> CertificateId {
        self.fields().hash([self.metadata.as_slice()])
    }

    /// Returns the L1 Info Tree leaf count considered for this [`Certificate`].
    /// Corresponds to the highest L1 Info Tree leaf index considered by the
    /// imported bridge exits.
    pub fn l1_info_tree_leaf_count(&self) -> Option<u32> {
        self.fields().l1_info_tree_leaf_count()
    }

    /// Returns the L1 Info Root considered for this [`Certificate`].
    /// Fails if multiple L1 Info Root are considered among the inclusion proofs
    /// of the imported bridge exits.
    pub fn l1_info_root(&self) -> Result<Option<Digest>, Error> {
        self.fields().l1_info_root()
    }

    /// Verify the extra certificate signature.
    pub fn verify_extra_signature(
        &self,
        expected_signer: Address,
        signature: Signature,
    ) -> Result<(), SignerError> {
        common::verify_extra_signature(
            self.hash(),
            self.l1_info_tree_leaf_count,
            expected_signer,
            signature,
        )
    }

    /// Verify the signature on the PP commitment.
    pub fn verify_cert_signature(&self, expected_signer: Address) -> Result<(), SignerError> {
        self.fields().verify_cert_signature(expected_signer)
    }

    /// Retrieve the signer from the certificate signature.
    pub fn retrieve_signer(&self, version: CommitmentVersion) -> Result<Address, SignerError> {
        self.fields().retrieve_signer(version)
    }
}
