//! State related commitments involved in the pessimistic proof.
//!
//! The pessimistic proof has the "pessimistic root" as part of its public
//! inputs. Some logic in this file handles the migration on its computation.
use agglayer_primitives::{alloy_primitives::B256, keccak::keccak256_combine, Digest};
use agglayer_tries::roots::{LocalBalanceRoot, LocalExitRoot, LocalNullifierRoot};
use serde::{Deserialize, Serialize};
use unified_bridge::{
    ImportedBridgeExitCommitmentValues, ImportedBridgeExitCommitmentVersion, NetworkId,
};

use crate::{
    aggchain_data::AggchainHashValues,
    proof::{ConstrainedValues, EMPTY_PP_ROOT_V2},
    ProofError,
};

/// The pessimistic root is a public value of the PP which is settled in the L1.
#[derive(Debug, Clone, Copy)]
pub enum PessimisticRootCommitmentVersion {
    /// Legacy PP root commitment.
    V2,
    /// Add the height and the origin network.
    V3,
}

/// The state commitment of one [`super::NetworkState`].
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StateCommitment {
    pub exit_root: LocalExitRoot,
    pub ler_leaf_count: u32,
    pub balance_root: LocalBalanceRoot,
    pub nullifier_root: LocalNullifierRoot,
}

/// The parameters which compose the pessimistic root.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PessimisticRootCommitmentValues {
    pub balance_root: LocalBalanceRoot,
    pub nullifier_root: LocalNullifierRoot,
    pub ler_leaf_count: u32,
    pub height: u64,
    pub origin_network: NetworkId,
}

impl From<&ConstrainedValues> for PessimisticRootCommitmentValues {
    fn from(value: &ConstrainedValues) -> Self {
        Self {
            balance_root: value.initial_state_commitment.balance_root,
            nullifier_root: value.initial_state_commitment.nullifier_root,
            ler_leaf_count: value.initial_state_commitment.ler_leaf_count,
            height: value.height,
            origin_network: value.origin_network,
        }
    }
}

impl PessimisticRootCommitmentValues {
    /// Infer the version of the provided settled pessimistic root.
    pub fn infer_settled_pp_root_version(
        &self,
        settled_pp_root: Digest,
    ) -> Result<PessimisticRootCommitmentVersion, ProofError> {
        let computed_v3 = self.compute_pp_root(PessimisticRootCommitmentVersion::V3);
        if computed_v3 == settled_pp_root {
            return Ok(PessimisticRootCommitmentVersion::V3);
        }

        let computed_v2 = self.compute_pp_root(PessimisticRootCommitmentVersion::V2);
        if computed_v2 == settled_pp_root {
            return Ok(PessimisticRootCommitmentVersion::V2);
        }

        // NOTE: Return v2 to trigger the migration
        let is_initial_state = computed_v2 == EMPTY_PP_ROOT_V2 && self.height == 0;

        if settled_pp_root.0 == [0u8; 32] && is_initial_state {
            return Ok(PessimisticRootCommitmentVersion::V2);
        }

        Err(ProofError::InvalidPreviousPessimisticRoot {
            declared: settled_pp_root,
            computed_v2,
            computed_v3,
        })
    }

    /// Compute the pessimistic root for the provided version.
    pub fn compute_pp_root(&self, version: PessimisticRootCommitmentVersion) -> Digest {
        match version {
            PessimisticRootCommitmentVersion::V2 => keccak256_combine([
                self.balance_root.as_ref(),
                self.nullifier_root.as_ref(),
                self.ler_leaf_count.to_le_bytes().as_slice(),
            ]),
            PessimisticRootCommitmentVersion::V3 => keccak256_combine([
                self.balance_root.as_ref(),
                self.nullifier_root.as_ref(),
                self.ler_leaf_count.to_le_bytes().as_slice(),
                self.height.to_le_bytes().as_slice(),
                self.origin_network.to_le_bytes().as_slice(),
            ]),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SignatureCommitmentVersion {
    /// Legacy commitment for the signature.
    V2,
    /// Add the height.
    V3,
    /// Add the aggchain params.
    V4,
    /// Add the certificate id.
    V5,
}

/// The values which compose the signature.
pub struct SignatureCommitmentValues {
    pub new_local_exit_root: LocalExitRoot,
    pub commit_imported_bridge_exits: ImportedBridgeExitCommitmentValues,
    pub height: u64,
    pub aggchain_params: Option<Digest>,
    pub certificate_id: Digest,
}

impl SignatureCommitmentValues {
    pub fn new(constrained_values: &ConstrainedValues, aggchain_params: Option<Digest>) -> Self {
        Self {
            new_local_exit_root: constrained_values.final_state_commitment.exit_root,
            commit_imported_bridge_exits: constrained_values.commit_imported_bridge_exits.clone(),
            height: constrained_values.height,
            aggchain_params,
            certificate_id: constrained_values.certificate_id,
        }
    }
}

impl SignatureCommitmentValues {
    /// Returns the expected signed commitment for the provided version.
    #[inline]
    pub fn commitment(&self, version: SignatureCommitmentVersion) -> B256 {
        let commitment = match version {
            SignatureCommitmentVersion::V2 => keccak256_combine([
                self.new_local_exit_root.as_ref(),
                self.commit_imported_bridge_exits
                    .commitment(ImportedBridgeExitCommitmentVersion::V2)
                    .as_slice(),
            ]),
            SignatureCommitmentVersion::V3 => {
                // Added the height to avoid replay attack edge cases
                keccak256_combine([
                    self.new_local_exit_root.as_ref(),
                    self.commit_imported_bridge_exits
                        .commitment(ImportedBridgeExitCommitmentVersion::V3)
                        .as_slice(),
                    self.height.to_le_bytes().as_slice(),
                ])
            }
            SignatureCommitmentVersion::V4 => {
                // Added the aggchain params to support the aggchain proof
                keccak256_combine([
                    self.new_local_exit_root.as_ref(),
                    self.commit_imported_bridge_exits
                        .commitment(ImportedBridgeExitCommitmentVersion::V3)
                        .as_slice(),
                    self.height.to_le_bytes().as_slice(),
                    self.aggchain_params
                        .unwrap_or_else(AggchainHashValues::empty_aggchain_params)
                        .as_slice(),
                ])
            }
            SignatureCommitmentVersion::V5 => {
                // Added the certificate id to cover edge cases coming with the multisig
                keccak256_combine([
                    self.new_local_exit_root.as_ref(),
                    self.commit_imported_bridge_exits
                        .commitment(ImportedBridgeExitCommitmentVersion::V3)
                        .as_slice(),
                    self.height.to_le_bytes().as_slice(),
                    self.aggchain_params
                        .unwrap_or_else(AggchainHashValues::empty_aggchain_params)
                        .as_slice(),
                    self.certificate_id.as_slice(),
                ])
            }
        };

        B256::new(commitment.0)
    }

    #[inline]
    pub fn multisig_commitment(&self) -> B256 {
        self.commitment(SignatureCommitmentVersion::V5)
    }
}
