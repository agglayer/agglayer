pub use bincode::Options;
use hex_literal::hex;
use pessimistic_proof_core::keccak::keccak256_combine;
use pessimistic_proof_core::multi_batch_header::MultiBatchHeader;
use pessimistic_proof_core::ProofError;
use pessimistic_proof_core::{
    bridge_exit::NetworkId,
    keccak::digest::Digest,
    local_state::{local_exit_tree::hasher::Keccak256Hasher, LocalNetworkState, StateCommitment},
};
use serde::{Deserialize, Serialize};

/// Outputs of the pessimistic proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PessimisticProofOutput {
    /// The previous local exit root.
    pub prev_local_exit_root: Digest,
    /// The previous pessimistic root.
    pub prev_pessimistic_root: Digest,
    /// The l1 info root against which we prove the inclusion of the imported
    /// bridge exits.
    pub l1_info_root: Digest,
    /// The origin network of the pessimistic proof.
    pub origin_network: NetworkId,
    /// The consensus hash.
    pub consensus_hash: Digest,
    /// The new local exit root.
    pub new_local_exit_root: Digest,
    /// The new pessimistic root which commits to the balance and nullifier
    /// tree.
    pub new_pessimistic_root: Digest,
}

impl PessimisticProofOutput {
    pub fn bincode_options() -> impl bincode::Options {
        bincode::DefaultOptions::new()
            .with_big_endian()
            .with_fixint_encoding()
    }

    pub fn display_to_hex(&self) -> String {
        format!(
            "prev_local_exit_root: {}, prev_pessimistic_root: {}, l1_info_root: {}, \
             origin_network: {}, consensus_hash: {}, new_local_exit_root: {}, \
             new_pessimistic_root: {}",
            self.prev_local_exit_root,
            self.prev_pessimistic_root,
            self.l1_info_root,
            self.origin_network,
            self.consensus_hash,
            self.new_local_exit_root,
            self.new_pessimistic_root,
        )
    }
}

const PESSIMISTIC_CONSENSUS_TYPE: u32 = 0;

const EMPTY_LER: Digest = Digest(hex!(
    "27ae5ba08d7291c96c8cbddcc148bf48a6d68c7974b94356f53754ef6171d757"
));

const EMPTY_PP_ROOT: Digest = Digest(hex!(
    "c89c9c0f2ebd19afa9e5910097c43e56fb4aff3a06ddee8d7c9bae09bc769184"
));

/// Proves that the given [`MultiBatchHeader`] can be applied on the given
/// [`LocalNetworkState`].
pub fn generate_pessimistic_proof(
    initial_network_state: LocalNetworkState,
    batch_header: &MultiBatchHeader<Keccak256Hasher>,
) -> Result<PessimisticProofOutput, ProofError> {
    let StateCommitment {
        exit_root: prev_ler,
        ler_leaf_count: prev_ler_leaf_count,
        balance_root: prev_lbr,
        nullifier_root: prev_nr,
    } = initial_network_state.roots();
    let prev_pessimistic_root = keccak256_combine([
        prev_lbr.as_slice(),
        prev_nr.as_slice(),
        prev_ler_leaf_count.to_le_bytes().as_slice(),
    ]);

    let consensus_hash = keccak256_combine([
        &PESSIMISTIC_CONSENSUS_TYPE.to_be_bytes(),
        batch_header.signer.as_slice(),
    ]);

    let new_pessimistic_root = keccak256_combine([
        batch_header.target.balance_root.as_slice(),
        batch_header.target.nullifier_root.as_slice(),
        batch_header.target.ler_leaf_count.to_le_bytes().as_slice(),
    ]);

    let mut network_state = initial_network_state;
    let computed_target = network_state.apply_batch_header(batch_header)?;

    if computed_target.exit_root != batch_header.target.exit_root {
        return Err(ProofError::InvalidNewLocalExitRoot {
            declared: batch_header.target.exit_root,
            computed: computed_target.exit_root,
        });
    }

    if computed_target.balance_root != batch_header.target.balance_root {
        return Err(ProofError::InvalidNewBalanceRoot {
            declared: batch_header.target.balance_root,
            computed: computed_target.balance_root,
        });
    }

    if computed_target.nullifier_root != batch_header.target.nullifier_root {
        return Err(ProofError::InvalidNewNullifierRoot {
            declared: batch_header.target.nullifier_root,
            computed: computed_target.nullifier_root,
        });
    }

    // NOTE: Hack to comply with the L1 contracts which assume `0x00..00` for the
    // empty roots of the different trees involved. Therefore, we do
    // one mapping of empty tree hash <> 0x00..0 on the public inputs.
    let (prev_local_exit_root, prev_pessimistic_root) = {
        let prev_ler = if prev_ler == EMPTY_LER {
            [0; 32].into()
        } else {
            prev_ler
        };

        let prev_pp_root = if prev_pessimistic_root == EMPTY_PP_ROOT {
            [0; 32].into()
        } else {
            prev_pessimistic_root
        };

        (prev_ler, prev_pp_root)
    };

    Ok(PessimisticProofOutput {
        prev_local_exit_root,
        prev_pessimistic_root,
        l1_info_root: batch_header.l1_info_root,
        origin_network: batch_header.origin_network.into(),
        consensus_hash,
        new_local_exit_root: batch_header.target.exit_root,
        new_pessimistic_root,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_tree_roots() {
        let empty_state = LocalNetworkState::default();

        let ler = empty_state.exit_tree.get_root();
        let ppr = keccak256_combine([
            empty_state.balance_tree.root.as_slice(),
            empty_state.nullifier_tree.root.as_slice(),
            empty_state.exit_tree.leaf_count.to_le_bytes().as_slice(),
        ]);

        assert_eq!(EMPTY_LER, ler);
        assert_eq!(EMPTY_PP_ROOT, ppr);
    }
}
