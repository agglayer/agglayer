use agglayer_interop_types::{aggchain_proof::AggchainData, LocalExitRoot};
use agglayer_primitives::{Address, Hashable, Signature, B256};
use pessimistic_proof::{
    core::commitment::{SignatureCommitmentValues, SignatureCommitmentVersion},
    keccak::keccak256_combine,
};
use unified_bridge::{
    BridgeExit, ImportedBridgeExit, ImportedBridgeExitCommitmentValues, NetworkId,
};

use crate::{
    aggchain_data::{MultisigCtx, MultisigPayload, PayloadWithCtx},
    Digest, Error, SignerError,
};

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

    pub fn signature_commitment_values(&self) -> SignatureCommitmentValues {
        SignatureCommitmentValues::from(self)
    }

    pub fn verify_legacy_ecdsa(
        &self,
        expected_signer: Address,
        signature: &Signature,
    ) -> Result<(), SignerError> {
        let signature_commitment_values = self.signature_commitment_values();

        let recovered_expected_signer = [
            SignatureCommitmentVersion::V3,
            SignatureCommitmentVersion::V2,
        ]
        .iter()
        .any(|version| {
            let commitment = signature_commitment_values.commitment(*version);
            match signature.recover_address_from_prehash(&commitment) {
                Ok(recovered) => recovered == expected_signer,
                Err(_) => false,
            }
        });

        recovered_expected_signer
            .then_some(())
            .ok_or(SignerError::InvalidPessimisticProofSignature { expected_signer })
    }

    pub fn verify_aggchain_proof_signature(
        &self,
        expected_signer: Address,
        signature: &Option<Box<Signature>>,
    ) -> Result<(), SignerError> {
        let signature_commitment_values = self.signature_commitment_values();

        let signature = signature.as_ref().ok_or(SignerError::Missing)?;
        let commitment = signature_commitment_values.commitment(SignatureCommitmentVersion::V4); // NOTE: will be upgraded to V5 eventually
        let recovered = signature
            .recover_address_from_prehash(&commitment)
            .map_err(SignerError::Recovery)?;

        (recovered == expected_signer)
            .then_some(())
            .ok_or(SignerError::InvalidPessimisticProofSignature { expected_signer })
    }

    pub fn verify_multisig(
        &self,
        signatures: &Vec<Signature>,
        ctx: MultisigCtx,
    ) -> Result<(), SignerError> {
        let multisig_with_ctx = PayloadWithCtx(MultisigPayload::from(signatures.clone()), ctx);

        // Verify the multisig from the chain payload and the L1 context
        let _witness_data: pessimistic_proof::core::MultiSignature = multisig_with_ctx
            .try_into()
            .map_err(SignerError::InvalidMultisig)?;

        Ok(())
    }

    /// Retrieve the signer from the certificate signature.
    pub fn retrieve_signer(
        &self,
        version: SignatureCommitmentVersion,
    ) -> Result<Address, SignerError> {
        let (signature, commitment) = match &self.aggchain_data {
            AggchainData::ECDSA { signature } => {
                let commitment = SignatureCommitmentValues::from(self).commitment(version);
                (signature, commitment)
            }
            AggchainData::Generic { signature, .. } => {
                let signature = signature.as_ref().ok_or(SignerError::Missing)?;
                let commitment = SignatureCommitmentValues::from(self)
                    .commitment(SignatureCommitmentVersion::V4);
                (signature.as_ref(), commitment)
            }
            AggchainData::MultisigOnly(_signatures) => todo!(), // return vec signers on V5
            AggchainData::MultisigAndAggchainProof { multisig: _, .. } => todo!(), /* return vec
                                                                  * signers on
                                                                  * V5 */
        };

        signature
            .recover_address_from_prehash(&commitment)
            .map_err(SignerError::Recovery)
    }

    pub fn aggchain_params(&self) -> Option<Digest> {
        match &self.aggchain_data {
            AggchainData::ECDSA { .. } => None,
            AggchainData::Generic {
                aggchain_params, ..
            } => Some(*aggchain_params),
            AggchainData::MultisigOnly(_) => None,
            AggchainData::MultisigAndAggchainProof {
                aggchain_proof:
                    agglayer_interop_types::aggchain_proof::AggchainProof {
                        aggchain_params, ..
                    },
                ..
            } => Some(*aggchain_params),
        }
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
            aggchain_params: certificate.aggchain_params(),
            certificate_id: certificate.hash().into(),
        }
    }
}
