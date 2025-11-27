use std::collections::{BTreeMap, BTreeSet};

use agglayer_types::{primitives::Hashable, Certificate, Digest};
use pessimistic_proof::{
    local_exit_tree::{data::LocalExitTreeData, LocalExitTree},
    unified_bridge::BridgeExit,
};

///
/// Request
pub struct AggoracleRequest {
    /// Network who submits this request
    pub destination_network: Network,

    /// Checkpoint list up to which the destination network already synced
    pub last_synced_ler: Vec<SyncCheckpoint>,
}

pub struct SyncCheckpoint {
    /// Network from which the sync is done
    pub network: Network,

    /// LER (included) up to which the sync is done
    pub ler: Ler,
}

///
/// Response
pub struct PreconfResponse {
    pub preconfirmations: Vec<PreconfirmationWithBridges>,
}

pub struct PreconfirmationWithBridges {
    /// Origin network of these bridge exits
    pub origin_network: Network,

    /// Preconfirmed local exit root to inject by the aggoracle and against
    /// which all the claims are done
    pub new_ler: Ler,

    /// Signature information on this preconfirmation, to be verified by the
    /// aggoracle
    pub signatures: PreconfSignatureInformation,

    /// List of bridge exits with their inclusion proof against `new_ler`
    pub bridges: Vec<BridgeWithMerkleProof>,
}

pub struct PreconfSignatureInformation {
    /// Multisig of the origin network and agglayer signature
    pub signatures: SignatureInfo,

    /// Agglayer params to reconstruct the signed commitment
    pub agglayer_params: Digest,
}

pub struct BridgeWithMerkleProof {
    /// Leaf index of the bridge exit that is claimed
    pub leaf_index: LeafIndex,

    /// Merkle proof of the bridge exit
    pub merkle_proof: MerkleProof,
}

pub type MerkleProof = ();
pub type SignatureInfo = ();
pub type Network = u32;
pub type FromNetwork = Network;
pub type ToNetwork = Network;
pub type LeafIndex = u32;
pub type LeafCount = u32;
pub type Height = u64;
pub type Ler = Digest;

pub struct PreconfirmedState {
    pub origin_network: Network,

    /// Preconfirmed LER
    pub new_ler: Ler,

    /// Leaf count
    pub new_leaf_count: LeafCount,

    /// All the signatures on the this preconfirmed state
    /// Multisig from origin network along with agglayer signature
    /// And all values to re-compute the signed message
    pub signatures: SignatureInfo,
}

/// CF
pub struct InMemoryDb {
    /// Which network received which bridge exit (leaf index) from which network
    /// Only total ordered keys, no need values for this cf
    pub received_bridge_exit_cf: BTreeSet<(ToNetwork, FromNetwork, LeafIndex)>,

    /// Lookup from origin network and LER to leaf count
    pub ler_2_leaf_count_cf: BTreeMap<(Network, Ler), LeafCount>,

    /// Full local exit tree data for a network, latest one
    pub full_let_cf: BTreeMap<Network, LocalExitTreeData>,

    /// Latest preconfirmed certificate per network
    pub latest_preconfirmed_state: BTreeMap<Network, PreconfirmedState>,
}

impl InMemoryDb {
    /// Returns the preconfirmation response for a given couple (from, to)
    /// network
    pub fn read_sync_from_one_network(
        &self,
        from_network_checkpoint: SyncCheckpoint,
        target_network: ToNetwork,
    ) -> PreconfirmationWithBridges {
        // get the latest state
        let SyncCheckpoint { network, ler } = from_network_checkpoint;
        let latest_synced_leaf_count = self.ler_2_leaf_count_cf.get(&(network, ler)).unwrap();

        let full_let = self.full_let_cf.get(&network).unwrap();

        todo!();
    }

    /// Read for aggoracle request
    pub fn read_destination_chain_injection(&self, request: AggoracleRequest) {
        let mut result = PreconfResponse {
            preconfirmations: todo!(),
        };
    }

    /// Write all the bridge exit indices
    pub fn write_bridges_indices(
        &mut self,
        from: FromNetwork,
        to: ToNetwork,
        bridge_indices: Vec<LeafIndex>,
    ) {
        todo!();
        // let mut to_add = bridge_indices
        //     .iter()
        //     .map(|&leaf_idx| (to, from, leaf_idx))
        //     .collect();
        // self.received_bridge_exit_cf.append(to_add);
    }

    /// Write latest preconfirmed state
    /// New leaf count is `new_state.local_exit_tree.leaf_count()`
    pub fn write_latest_preconfirmed_state(
        &mut self,
        certificate: Certificate,
        prev_leaf_count: LeafCount,
        new_leaf_count: LeafCount,
    ) {
        let origin_network = certificate.network_id.to_u32();
        let preconf_state = PreconfirmedState {
            origin_network,
            new_ler: certificate.new_local_exit_root.into(),
            new_leaf_count,
            signatures: (), // from certificate + agglayer-node signature
        };

        // Traversal of certificate.bridge_exits and call
        // write_bridge_indices accordingly
        // TODO: much better way of doing by just having one write call per destination
        // network
        for (idx, b) in certificate.bridge_exits.iter().enumerate() {
            let target_network = b.dest_network.to_u32();
            self.write_bridges_indices(
                origin_network,
                certificate.network_id.to_u32(),
                vec![idx as u32],
            );
        }
    }
}

#[test]
fn test_aggoracle_request() {
    todo!();
}
