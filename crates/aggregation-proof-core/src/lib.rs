use std::collections::BTreeMap;

use agglayer_bincode as bincode;
use agglayer_primitives::Digest;
use agglayer_tries::roots::LocalExitRoot;
use pessimistic_proof_core::PessimisticProofOutput;
use serde::{Deserialize, Serialize};
use sha2::{Digest as ShaDigest, Sha256};
use unified_bridge::{LETMerkleProof, LocalExitTree, NetworkId};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid range")]
    InvalidRange,
    #[error("not contiguous pp from the given network")]
    NotContiguous,
    #[error("invalid pp")]
    InvalidPP,
    #[error("invalid origin network")]
    InvalidOriginNetwork { intruder: NetworkId },
    #[error("wrong prev agglayer RER")]
    InvalidPrevAgglayerRer,
    #[error("wrong new agglayer RER")]
    InvalidNewAgglayerRer,
    #[error("invalid serialization. {0}")]
    InvalidSerialization(#[source] Box<agglayer_bincode::ErrorKind>),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PessimisticProof {
    /// Public values for the PP.
    pub public_values: PessimisticProofOutput,
    /// Index points to the corresponding sub LET inclusion proof for each
    /// imported LER.
    pub lookup_imported_lers: Vec<(u32, (NetworkId, LocalExitRoot))>,
    /// Config number, pointing to a specific multisig set
    pub config_number: usize,
    /// Index of the sub-l1 info tree inclusion proof
    pub sub_l1_info_tree_inclusion_proof_idx: u32,
}

impl PessimisticProof {
    /// Verify the public values from `self` on the next proof of the stark
    /// stream.
    ///
    /// NOTE: the order on this function call is important, it consumes the
    /// next proof in the sp1 stream.
    ///
    /// NOTE: the verification will panic upon wrong proof (enforced by the sp1
    /// zkvm)
    pub fn verify_pp_stark_proof(&self, pp_vkey_hash: &[u32; 8]) -> Result<(), Error> {
        let pv_serialized = PessimisticProofOutput::bincode_codec()
            .serialize(&self.public_values)
            .map_err(Error::InvalidSerialization)?;
        let pv_digest = Sha256::digest(pv_serialized);
        // panic upon invalid proof because sp1
        sp1_zkvm::lib::verify::verify_sp1_proof(pp_vkey_hash, &pv_digest.into());
        Ok(())
    }

    /// Commitment on the imported LERs of the given PP
    pub fn imported_ler_hash(&self) -> Digest {
        todo!() // on self.lookup_imported_lers
    }
}

/// Contiguous set of PP from one network
#[derive(Deserialize, Serialize, Debug)]
pub struct RangePP {
    /// Origin network of this range of PP
    pub origin_network: NetworkId,
    /// Contiguous range of PP for the given network
    pub proofs: Vec<PessimisticProof>,
}

impl RangePP {
    /// Ensures that the PPs from one chain are contiguous
    pub fn verify_pp_contiguity(&self) -> Result<(), Error> {
        // NOTE: the height is enforced to be increased within the pp root computation
        // at the leaf PP level
        let contiguous = self.proofs.windows(2).all(|pp| {
            let prev = &pp[0].public_values;
            let next = &pp[1].public_values;

            prev.new_local_exit_root == next.prev_local_exit_root
                && prev.new_pessimistic_root == next.prev_pessimistic_root
        });

        if !contiguous {
            return Err(Error::NotContiguous);
        }

        Ok(())
    }

    /// Ensures that all the starks are successfully verified
    pub fn verify_pp_validity(&self, pp_vkey_hash: &[u32; 8]) -> Result<(), Error> {
        // verify that all PP are from the same and correct network
        let intruder_pp = self
            .proofs
            .iter()
            .find(|pp| pp.public_values.origin_network != self.origin_network);

        if let Some(intruder_pp) = intruder_pp {
            return Err(Error::InvalidOriginNetwork {
                intruder: intruder_pp.public_values.origin_network,
            });
        }

        // verify all the starks
        for pp in &self.proofs {
            pp.verify_pp_stark_proof(pp_vkey_hash)?
        }

        Ok(())
    }
}

/// Witness for the aggregation proof.
#[derive(Deserialize, Serialize, Default, Debug)]
pub struct AggregationWitness {
    /// PP vkey hash considered for this aggregation
    pub pp_vkey_hash: [u32; 8],

    /// One contiguous range of PP per origin network
    pub pp_per_network: Vec<RangePP>,

    /// Sub-inclusions of pre-confirmed LERs per origin network.
    pub sub_let_inclusion_proofs: Vec<SubLetInclusionProof>,

    /// LER in the new agglayer RER
    /// todo: define an SMT for it, and put the read values as LUT to avoid
    /// multiple read of the same value from the smt
    pub prev_agglayer_rer: BTreeMap<NetworkId, LocalExitRoot>,

    /// LER in the new agglayer RER
    /// todo: define an SMT for it, and put the read values as LUT to avoid
    /// multiple read of the same value from the smt
    pub new_agglayer_rer: BTreeMap<NetworkId, LocalExitRoot>,

    /// Target L1 info root against which we prove the inclusion of all the PP's
    /// l1 info root.
    /// NOTE: chosen by the agglayer-node during witness generation for the
    /// aggregation proof, can be against the root of the latest synchronized
    /// l1 info tree
    pub target_l1_info_root: Digest,

    /// Sub-L1 info tree inclusion proofs.
    /// NOTE: Computed by the agglayer-node, needs synchronization of the l1
    /// info tree.
    pub sub_l1_info_tree_inclusion_proofs: Vec<SubLetInclusionProof>,
}

impl AggregationWitness {
    /// Execute all the aggregation proving statement and returns the public
    /// values upon success.
    pub fn verify(&self) -> Result<AggregationPublicValues, Error> {
        // Per network, verify all the PPs (validity and contiguity per network)
        for pp_range in &self.pp_per_network {
            // all the PPs from the current network must be contiguous
            pp_range.verify_pp_contiguity()?;

            // all the PPs must be successfully verified starks
            pp_range.verify_pp_validity(&self.pp_vkey_hash)?;
        }

        // Verify the agglayer rollup exit tree transition
        self.verify_agglayer_rer_transition()?;

        // Verify all the imported LER against the agglayer rollup exit tree
        //
        // NOTE: divided in two steps to avoid verifying multiple times the same
        // inclusion proofs (since one LER might be imported several times across
        // different PP in the aggregation)
        {
            // verify the pointers
            self.verify_imported_ler_pointers()?;
            // verify the actual inclusion proofs uniquely
            self.verify_imported_lers_inclusion()?;
        }

        // Verify the inclusion proofs on the PP l1 info root to have only one l1 info
        // root as public input of the aggregation proof
        {
            // verify the pointers
            self.verify_l1_info_tree_pointers()?;
            // verify the actual inclusion proofs uniquely
            self.verify_l1_info_tree_inclusion()?;
        }

        Ok(self.public_values())
    }

    /// Verify the transition of the agglayer rollup exit tree wrt to the PP
    /// which compose this aggregation.
    pub fn verify_agglayer_rer_transition(&self) -> Result<(), Error> {
        for pp_range in &self.pp_per_network {
            let RangePP {
                origin_network,
                proofs,
            } = pp_range;

            let first_prev_ler = proofs.first().unwrap().public_values.prev_local_exit_root;
            let last_new_ler = proofs.last().unwrap().public_values.new_local_exit_root;

            // starting LER corresponds to the prev LER of the first PP
            if first_prev_ler != *self.prev_agglayer_rer.get(&origin_network).unwrap() {
                return Err(Error::InvalidPrevAgglayerRer);
            }

            // ending LER corresponds to the new LER of the last PP
            if last_new_ler != *self.new_agglayer_rer.get(&origin_network).unwrap() {
                return Err(Error::InvalidNewAgglayerRer);
            }
        }

        Ok(())
    }

    /// Verify that each imported LER dereference to the right sub-let inclusion
    /// proof (without verifying any inclusion proof here)
    pub fn verify_imported_ler_pointers(&self) -> Result<bool, Error> {
        let all_pointers_are_valid = self.pp_per_network.iter().all(|pp_range| {
            pp_range.proofs.iter().all(|pp| {
                pp.lookup_imported_lers
                    .iter()
                    .all(|&(lut_idx, imported_ler)| {
                        imported_ler == self.sub_let_inclusion_proofs[lut_idx as usize].target()
                    })
            })
        });

        Ok(all_pointers_are_valid)
    }

    /// Verify that all sub LET inclusion proofs are valid and against the same
    /// and corresponds to agglayerRER
    pub fn verify_imported_lers_inclusion(&self) -> Result<bool, Error> {
        Ok(self.sub_let_inclusion_proofs.iter().all(|p| {
            let valid_inclusion_proof = p.verify_sub_inclusion();
            let valid_target_root =
                p.target_ler == *self.new_agglayer_rer.get(&p.origin_network).unwrap();

            valid_target_root && valid_inclusion_proof
        }))
    }

    /// Verify pointers on l1 info tree sub-inclusion
    pub fn verify_l1_info_tree_pointers(&self) -> Result<bool, Error> {
        let all_pointers_are_valid = self.pp_per_network.iter().all(|pp_range| {
            pp_range.proofs.iter().all(|pp| {
                // the L1 info root of the current PP must match with the target LER of the
                // sub-tree inclusion proof
                pp.public_values.l1_info_root
                    == self.sub_l1_info_tree_inclusion_proofs
                        [pp.sub_l1_info_tree_inclusion_proof_idx as usize]
                        .target_ler
                        .into()
            })
        });

        Ok(all_pointers_are_valid)
    }

    /// Verify that all sub l1 info tree inclusion proofs are valid
    pub fn verify_l1_info_tree_inclusion(&self) -> Result<bool, Error> {
        Ok(self.sub_l1_info_tree_inclusion_proofs.iter().all(|p| {
            let valid_inclusion_proof = p.verify_sub_inclusion();
            let valid_target_root = p.target_ler == self.target_l1_info_root.into();

            valid_target_root && valid_inclusion_proof
        }))
    }
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct AggregationPublicValues {
    /// Hash chain on all the pp inputs
    pub hash_chain_pp_inputs: Digest,
    /// Pessimistic proof vkey
    pub pp_vkey: [u32; 8],
    /// L1 info root
    pub l1_info_root: Digest,
}

impl AggregationPublicValues {
    pub fn bincode_codec() -> bincode::Codec<impl bincode::Options> {
        bincode::contracts()
    }
}

impl AggregationWitness {
    /// Computes and returns the public values.
    pub fn public_values(&self) -> AggregationPublicValues {
        AggregationPublicValues {
            hash_chain_pp_inputs: self.hash_chain_pub_values(),
            pp_vkey: self.pp_vkey_hash,
            l1_info_root: self.target_l1_info_root,
        }
    }

    /// Hash chain on the PP public values.
    pub fn hash_chain_pub_values(&self) -> Digest {
        Digest::default() // todo
    }
}

/// Sub-inclusion proof from LET_n to LET_n+k.
/// frontier (32 * 32bytes) of LET_n + inclusion proof of `next_leaf` against
/// LER_n+k (target_ler)
#[derive(Deserialize, Serialize, Debug)]
pub struct SubLetInclusionProof {
    /// Origin network of the LET for which we do this sub-LET inclusion proof.
    pub origin_network: NetworkId,
    /// Frontier of LET_n
    pub base_let: LocalExitTree,
    /// LER of LET_(n+k)
    pub target_ler: LocalExitRoot,
    /// Inclusion proof against `target_ler` for `next_leaf` at the index
    /// `LET_n.leaf_count`
    pub inclusion_proof: LETMerkleProof,
    /// Next leaf in the target ler
    pub next_leaf: Digest,
}

impl SubLetInclusionProof {
    /// Returns the target LER for this sub-LET inclusion proof
    pub fn target(&self) -> (NetworkId, LocalExitRoot) {
        (self.origin_network, self.target_ler)
    }

    /// Verify the sub-inclusion of frontier to target ler
    /// todo: to bring in the interop repo
    /// cf. https://github.com/agglayer/agglayer/pull/364/files
    pub fn verify_sub_inclusion(&self) -> bool {
        // Check that `next_leaf` is the leaf at index `self.leaf_count` in the other
        // LET
        if !self.inclusion_proof.verify(
            self.next_leaf,
            self.base_let.leaf_count,
            self.target_ler.into(),
        ) {
            return false;
        }

        // Check that the frontier is a subset of the Merkle proof
        let mut index = self.base_let.leaf_count;
        let mut height = 0;
        while index != 0 {
            if index & 1 == 1 {
                if self.base_let.frontier[height] != self.inclusion_proof.siblings[height] {
                    return false;
                }
            }
            height += 1;
            index >>= 1;
        }

        true
    }
}
