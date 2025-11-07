use std::{
    collections::{BTreeMap, HashMap},
    fmt,
    io::Write,
    process::{Command, Stdio},
};

use agglayer_primitives::Hashable;
use agglayer_types::{
    aggchain_data::CertificateAggchainDataCtx, aggchain_proof::AggchainData,
    compute_signature_info, Certificate, Digest, Error as TypesError, Height, L1WitnessCtx,
    LocalNetworkStateData, PessimisticRootInput, U256,
};
use aggregation_proof_core::{AgglayerRer, AggregationWitness, PessimisticProof, RangePP};
use pessimistic_proof::{
    core::commitment::{PessimisticRootCommitmentVersion, SignatureCommitmentVersion},
    local_exit_tree::data::LocalExitTreeData,
    multi_batch_header::MultiBatchHeader,
    unified_bridge::{BridgeExit, Claim, ClaimFromPreconf, GlobalIndex, LeafType, MerkleProof},
    NetworkState, PessimisticProofOutput,
};
use petgraph::{
    dot::Dot,
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
    Direction::{Incoming, Outgoing},
};
use sp1_sdk::{HashableKey, ProverClient, SP1Stdin};
use unified_bridge::{ImportedBridgeExit, NetworkId, RollupIndex};

use crate::{sample_data::USDC, PESSIMISTIC_PROOF_ELF};

fn letter_to_network_id(letter: char) -> Option<NetworkId> {
    match letter {
        'A'..='Z' => Some((letter as u32 - 'A' as u32) + 1),
        'a'..='z' => Some((letter as u32 - 'a' as u32) + 1),
        _ => None,
    }
    .map(NetworkId::new)
}

fn network_label(network_id: NetworkId) -> char {
    let id: u32 = network_id.to_u32();

    char::from_u32('A' as u32 + id - 1).unwrap()
}

pub fn render_ascii<N, E>(graph: &DiGraph<N, E>) -> String
where
    N: std::fmt::Debug,
    E: std::fmt::Debug,
{
    let dot = format!("{:?}", Dot::new(graph));

    let child = Command::new("graph-easy")
        .arg("--from=dot")
        .arg("--as=boxart")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn();

    match child {
        Ok(mut child) => {
            if let Some(stdin) = child.stdin.as_mut() {
                let _ = stdin.write_all(dot.as_bytes());
            }
            let output = child.wait_with_output().unwrap();
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout).to_string()
            } else {
                format!("[graph-easy failed]\nDOT:\n{}", dot)
            }
        }
        Err(_) => format!(
            "[graph-easy not installed. Try: `brew install graphviz graph-easy`]\nDOT:\n{}",
            dot
        ),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeKind {
    /// Certificate A_N+1 is Next to A_N
    Next,
    /// Cert A0 claimed by B2 if B2 contains an imported bridge exit from A0
    ClaimedBy,
}

#[derive(Debug, Clone)]
pub struct CertNode {
    pub id: usize,
    pub height: u64,
    pub network: NetworkId,
}

impl fmt::Debug for CertificateRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let letter = network_label(self.multi_batch_header.origin_network);
        write!(f, "{}{}", letter, self.certificate.height)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CertificateHandle(NodeIndex);
impl CertificateHandle {
    pub fn node_index(self) -> NodeIndex {
        self.0
    }

    pub fn claims_from(self, sender: CertificateHandle, dag: &mut CertGraphBuilder) {
        let src_net = dag.graph[sender.0].network;
        let dst_net = dag.graph[self.0].network;
        assert_ne!(
            src_net, dst_net,
            "invalid cert_graph: cannot claim from same network ({} == {})",
            src_net, dst_net
        );
        dag.graph.add_edge(sender.0, self.0, EdgeKind::ClaimedBy);
    }
}

#[derive(Debug, Default)]
pub struct CertGraphBuilder {
    graph: DiGraph<CertNode, EdgeKind>,
    last_by_network: HashMap<NetworkId, (CertificateHandle, u64)>,
    next_id: usize,
}

impl CertGraphBuilder {
    /// Append the Next Certificate in the DAG for the given origin network.
    pub fn add_cert(&mut self, network_letter: char) -> CertificateHandle {
        let network: NetworkId = letter_to_network_id(network_letter).unwrap();

        let new_height = self
            .last_by_network
            .get(&network)
            .map(|&(_, h)| h)
            .unwrap_or_default()
            + 1;

        let node = CertNode {
            id: self.next_id,
            height: new_height,
            network,
        };
        self.next_id += 1;
        let idx = self.graph.add_node(node);
        let handle = CertificateHandle(idx);
        if let Some(prev) = self.last_by_network.insert(network, (handle, new_height)) {
            self.graph.add_edge(prev.0 .0, idx, EdgeKind::Next);
        }
        handle
    }

    pub fn graph(&self) -> &DiGraph<CertNode, EdgeKind> {
        &self.graph
    }

    pub fn build(&self) -> CertGraph {
        self.try_into().unwrap()
    }
}

/// Exhaustive information for one state transition
#[derive(Clone)]
pub struct CertificateRecord {
    /// Id of the record, for graph lookup
    pub id: usize,
    /// Origin network of this certificate.
    pub network: NetworkId,
    /// [`Certificate`] for this node record
    pub certificate: Certificate,
    /// [`MultiBatchHeader`] for this node record
    pub multi_batch_header: MultiBatchHeader,
    /// State before the state transition.
    pub state_before: LocalNetworkStateData,
    /// State after the state transition.
    pub state_after: LocalNetworkStateData,
}

#[derive(Debug)]
pub struct CertGraph {
    graph: DiGraph<CertificateRecord, EdgeKind>,
}

impl TryFrom<&CertGraphBuilder> for CertGraph {
    type Error = TypesError;

    fn try_from(dag: &CertGraphBuilder) -> Result<Self, Self::Error> {
        let order = petgraph::algo::toposort(&dag.graph, None)
            .expect("cycle in blueprint certificate graph");

        let mut nets: HashMap<NetworkId, NetworkCtx> = HashMap::new();

        // tracker of which bridge exit corresponds to which claim
        let mut claim_tracker: HashMap<(NodeIndex, NodeIndex), (u32, BridgeExit)> = HashMap::new();

        let mut graph = DiGraph::<CertificateRecord, EdgeKind>::new();
        let mut node_map: HashMap<NodeIndex, NodeIndex> = HashMap::new();

        for idx in order {
            let CertNode {
                id,
                height,
                network: net,
            } = dag.graph[idx];

            let net_ctx = nets.entry(net).or_insert_with(NetworkCtx::default);
            // Generate the bridge exits for the upcoming claims (guaranteed to be in the
            // right order because of the topological sort)
            let bridge_exits = dag
                .graph
                .edges_directed(idx, Outgoing)
                .filter(|&e| *e.weight() == EdgeKind::ClaimedBy)
                .map(|e| {
                    let tgt = e.target();
                    let tgt_net = dag.graph[tgt].network;
                    assert_ne!(net, tgt_net, "cannot bridge exit to same network");

                    let exit = mk_exit(net, tgt_net);
                    let leaf_idx = net_ctx.add_bridge_exit_for_claim(&exit).unwrap();
                    // tag the bridge exit for when it'd get claimed by the corresponding node
                    // later
                    claim_tracker.insert((idx, tgt), (leaf_idx, exit.clone()));
                    exit
                })
                .collect::<Vec<_>>();

            // Collect the imported bridge exit from the ClaimedBy edges.
            let imported = dag
                .graph
                .edges_directed(idx, Incoming)
                .filter(|&e| *e.weight() == EdgeKind::ClaimedBy)
                .map(|e| {
                    let src = e.source();
                    let src_net = dag.graph[src].network;
                    let src_ctx = nets.get(&src_net).expect("source ctx exists");

                    let (leaf_idx, exported_exit) = claim_tracker[&(src, idx)].clone();
                    let proof = src_ctx.proof_for_index(leaf_idx).unwrap();

                    let global_index = mk_global_index_from_network(src_net, leaf_idx);

                    assert_eq!(global_index.network_id(), src_net);

                    ImportedBridgeExit {
                        bridge_exit: exported_exit,
                        global_index,
                        claim_data: Claim::Preconf(Box::new(ClaimFromPreconf {
                            proof_leaf_ler: proof,
                        })),
                    }
                })
                .collect::<Vec<_>>();

            let certificate = {
                let prev_ler = nets
                    .entry(net)
                    .or_insert_with(NetworkCtx::default)
                    .state
                    .exit_tree
                    .get_root()
                    .into();
                let new_ler = nets[&net].preview_new_ler(&bridge_exits)?;
                let wallet = Certificate::wallet_for_test(net);
                let height = Height::new(height);
                let (_hash, signature, _signer) = compute_signature_info(
                    new_ler.into(),
                    &imported,
                    &wallet,
                    height,
                    SignatureCommitmentVersion::V3,
                );

                Certificate {
                    network_id: net,
                    height,
                    prev_local_exit_root: prev_ler,
                    new_local_exit_root: new_ler.into(),
                    bridge_exits: bridge_exits.clone(),
                    imported_bridge_exits: imported.clone(),
                    aggchain_data: AggchainData::ECDSA { signature },
                    metadata: Default::default(),
                    custom_chain_data: vec![],
                    l1_info_tree_leaf_count: None,
                }
            };

            let witness = L1WitnessCtx {
                l1_info_root: Digest::ZERO,
                prev_pessimistic_root: PessimisticRootInput::Computed(
                    PessimisticRootCommitmentVersion::V3,
                ),
                aggchain_data_ctx: CertificateAggchainDataCtx::LegacyEcdsa {
                    signer: certificate.get_signer(),
                },
            };

            let record = {
                let net_ctx = nets.get_mut(&net).unwrap();
                let state_before = net_ctx.state.clone();
                let mbh = net_ctx.state.apply_certificate(&certificate, witness)?;
                let state_after = net_ctx.state.clone();

                CertificateRecord {
                    id,
                    network: net,
                    certificate,
                    multi_batch_header: mbh,
                    state_before,
                    state_after,
                }
            };

            let new_idx = graph.add_node(record);
            node_map.insert(idx, new_idx);
        }

        for e in dag.graph.edge_references() {
            let src = node_map[&e.source()];
            let tgt = node_map[&e.target()];
            graph.add_edge(src, tgt, *e.weight());
        }

        println!("Graph representation:\n{}", render_ascii(&graph));

        Ok(Self { graph })
    }
}

impl CertGraph {
    pub fn graph(&self) -> &DiGraph<CertificateRecord, EdgeKind> {
        &self.graph
    }

    pub fn record(&self, network_letter: char, height: u64) -> &CertificateRecord {
        let network: NetworkId = letter_to_network_id(network_letter).unwrap();
        let h = Height::new(height).as_u64();
        self.graph
            .node_weights()
            .find(|rec| rec.network == network && rec.multi_batch_header.height == h)
            .expect(format!("no record for {network_letter}{h}").as_str())
    }

    pub fn edge_count(&self, kind: EdgeKind) -> usize {
        self.graph
            .edge_references()
            .filter(|e| *e.weight() == kind)
            .count()
    }

    pub fn iter_topological_order(&self) -> Vec<NodeIndex> {
        petgraph::algo::toposort(self.graph(), None).unwrap()
    }

    /// Execute the PP program on SP1 for the given state transition.
    pub fn execute_sp1(&self, idx: NodeIndex) -> ((), PessimisticProofOutput) {
        let record = &self.graph[idx];

        let mut stdin = SP1Stdin::new();
        let network_state: NetworkState = record.state_before.clone().into();
        stdin.write(&network_state);
        stdin.write(&record.multi_batch_header);

        let client = ProverClient::from_env();

        let (pv, report) = client.execute(PESSIMISTIC_PROOF_ELF, &stdin).run().unwrap();
        let pv_sp1_execute: PessimisticProofOutput = PessimisticProofOutput::bincode_codec()
            .deserialize(pv.as_slice())
            .unwrap();

        // print out the PP public values
        println!(
            "\n===== [{}{}] ({} gas) =====\n{pv_sp1_execute:?}\n",
            network_label(record.network.into()),
            record.multi_batch_header.height,
            report.gas.unwrap(),
        );

        ((), pv_sp1_execute)
    }

    /// Returns the range pp per network
    pub fn range_pp(&self) -> Vec<RangePPWithProof> {
        let mut all_networks: BTreeMap<NetworkId, Vec<PessimisticProofWithProof>> = BTreeMap::new();

        for idx in self.iter_topological_order() {
            let record = &self.graph[idx];

            let pvs = all_networks
                .entry(record.multi_batch_header.origin_network)
                .or_insert_with(Vec::new);

            let (sp1_proof, pv) = self.execute_sp1(idx); // todo: generate actual stark

            let pp = PessimisticProofWithProof {
                pp_metadata: PessimisticProof {
                    public_values: pv,
                    lookup_imported_lers: Vec::new(), // todo
                    config_number: 0,                 // todo
                    sub_l1_info_tree_inclusion_proof_idx: 0u32,
                },
                sp1_proof,
            };

            pvs.push(pp);
        }

        let range_pps = all_networks
            .into_iter()
            .map(|(origin_network, proofs)| RangePPWithProof {
                origin_network,
                proofs,
            })
            .collect::<Vec<_>>();

        range_pps
    }

    pub fn aggregation_witness(&self) -> AggregationWitness {
        let client = ProverClient::from_env();
        let (_pk, vk) = client.setup(PESSIMISTIC_PROOF_ELF);

        let pp_with_proof = self.range_pp(); // todo write proofs in stdin

        let pp_per_network: Vec<RangePP> = pp_with_proof.iter().map(Into::into).collect();

        let prev_agglayer_rer = AgglayerRer(
            pp_per_network
                .iter()
                .map(|range| (range.origin_network, range.first_ler()))
                .collect::<BTreeMap<_, _>>(),
        );

        let new_agglayer_rer = AgglayerRer(
            pp_per_network
                .iter()
                .map(|range| (range.origin_network, range.last_ler()))
                .collect::<BTreeMap<_, _>>(),
        );

        let witness = AggregationWitness {
            pp_vkey_hash: vk.hash_u32(),
            pp_per_network,
            sub_let_inclusion_proofs: Default::default(), // todo
            prev_agglayer_rer,
            new_agglayer_rer,
            target_l1_info_root: Default::default(), // todo
            sub_l1_info_tree_inclusion_proofs: Default::default(), // todo
        };

        witness
    }
}

#[derive(Debug)]
pub struct PessimisticProofWithProof {
    pub pp_metadata: PessimisticProof,
    pub sp1_proof: (), // todo: SP1Proof
}

/// Contiguous set of PP from one network
#[derive(Debug)]
pub struct RangePPWithProof {
    /// Origin network of this range of PP
    pub origin_network: NetworkId,
    /// Contiguous range of PP for the given network
    pub proofs: Vec<PessimisticProofWithProof>,
}

impl From<&RangePPWithProof> for RangePP {
    fn from(value: &RangePPWithProof) -> Self {
        Self {
            origin_network: value.origin_network,
            proofs: value
                .proofs
                .iter()
                .map(|pp_with_proof| pp_with_proof.pp_metadata.clone())
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct NetworkCtx {
    /// Trees of the given network.
    state: LocalNetworkStateData,
    /// LET including its leaves to generate inclusion proofs.
    let_data: LocalExitTreeData,
}

impl NetworkCtx {
    /// Add one bridge exit for it to be claimed after
    fn add_bridge_exit_for_claim(&mut self, exit: &BridgeExit) -> Result<u32, TypesError> {
        let idx = self.let_data.leaf_count();
        self.let_data.add_leaf(exit.hash())?;
        Ok(idx as u32)
    }

    /// Returns the merkle proof for the given leaf index
    fn proof_for_index(&self, leaf_index: u32) -> Result<MerkleProof, TypesError> {
        Ok(MerkleProof {
            proof: self.let_data.get_proof(leaf_index)?,
            root: self.let_data.get_root(),
        })
    }

    /// Get new LER without mutating current state
    fn preview_new_ler(&self, exits: &[BridgeExit]) -> Result<Digest, TypesError> {
        let mut tmp = self.state.exit_tree.clone();
        for e in exits {
            tmp.add_leaf(e.hash())?;
        }
        Ok(tmp.get_root().into())
    }
}

fn mk_exit(origin: NetworkId, dest: NetworkId) -> BridgeExit {
    let mut token = USDC;
    token.origin_network = origin;

    BridgeExit {
        leaf_type: LeafType::Transfer,
        token_info: token,
        dest_network: dest,
        dest_address: Certificate::wallet_for_test(dest).address().into(),
        amount: U256::from(10u64),
        metadata: Some(Digest::ZERO),
    }
}

fn mk_global_index_from_network(src_net: NetworkId, leaf_index: u32) -> GlobalIndex {
    if src_net == NetworkId::ETH_L1 {
        GlobalIndex {
            mainnet_flag: true,
            rollup_index: RollupIndex::new(0).unwrap(),
            leaf_index,
        }
    } else {
        GlobalIndex {
            mainnet_flag: false,
            rollup_index: RollupIndex::try_from(src_net).unwrap(),
            leaf_index,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_chain_with_claim() {
        let cert_graph = {
            let mut dag = CertGraphBuilder::default();
            let _a1 = dag.add_cert('A');
            let a2 = dag.add_cert('A');
            let b1 = dag.add_cert('B');
            b1.claims_from(a2, &mut dag);

            dag.build()
        };

        assert_eq!(cert_graph.graph().node_count(), 3);
        assert_eq!(cert_graph.edge_count(EdgeKind::ClaimedBy), 1);

        let a2_rec = cert_graph.record('A', 2);
        assert_ne!(
            a2_rec.state_before.exit_tree.get_root(),
            a2_rec.state_after.exit_tree.get_root()
        );

        let b1_rec = cert_graph.record('B', 1);
        assert_eq!(b1_rec.certificate.imported_bridge_exits.len(), 1);
        assert_eq!(
            b1_rec.state_before.exit_tree.get_root(),
            b1_rec.state_after.exit_tree.get_root()
        );
    }

    #[test]
    fn contiguous_only_chain() {
        let cert_graph = {
            let mut dag = CertGraphBuilder::default();
            let _a1 = dag.add_cert('A');
            let _a2 = dag.add_cert('A');
            let _a3 = dag.add_cert('A');

            dag.build()
        };

        assert_eq!(cert_graph.graph().node_count(), 3);
    }

    #[test]
    fn chain_with_multiple_claims_from_same_sender() {
        let cert_graph = {
            let mut dag = CertGraphBuilder::default();
            let a1 = dag.add_cert('A');
            let b1 = dag.add_cert('B');
            let b2 = dag.add_cert('B');
            b1.claims_from(a1, &mut dag);
            b2.claims_from(a1, &mut dag);

            dag.build()
        };

        assert_eq!(cert_graph.record('A', 1).certificate.bridge_exits.len(), 2);
        assert_eq!(
            cert_graph
                .record('B', 1)
                .certificate
                .imported_bridge_exits
                .len(),
            1
        );
        assert_eq!(
            cert_graph
                .record('B', 2)
                .certificate
                .imported_bridge_exits
                .len(),
            1
        );
    }

    #[test]
    fn transitive_chain_claims() {
        let cert_graph = {
            let mut dag = CertGraphBuilder::default();
            let a1 = dag.add_cert('A');
            let b1 = dag.add_cert('B');
            let c1 = dag.add_cert('C');
            b1.claims_from(a1, &mut dag);
            c1.claims_from(b1, &mut dag);

            dag.build()
        };

        assert_eq!(cert_graph.record('A', 1).certificate.bridge_exits.len(), 1);
        assert_eq!(
            cert_graph
                .record('B', 1)
                .certificate
                .imported_bridge_exits
                .len(),
            1
        );
        assert_eq!(cert_graph.record('B', 1).certificate.bridge_exits.len(), 1);
        assert_eq!(
            cert_graph
                .record('C', 1)
                .certificate
                .imported_bridge_exits
                .len(),
            1
        );
    }

    #[test]
    fn parallel_networks_do_not_interfere() {
        let cert_graph = {
            let mut dag = CertGraphBuilder::default();
            let _a1 = dag.add_cert('A');
            let _a2 = dag.add_cert('A');
            let _b1 = dag.add_cert('B');
            let _b2 = dag.add_cert('B');

            dag.build()
        };

        assert_eq!(cert_graph.edge_count(EdgeKind::Next), 2);
        assert_eq!(cert_graph.edge_count(EdgeKind::ClaimedBy), 0);
    }

    #[test]
    fn exit_tree_progresses_only_for_senders() {
        let cert_graph = {
            let mut dag = CertGraphBuilder::default();
            let a1 = dag.add_cert('A');
            let b1 = dag.add_cert('B');
            b1.claims_from(a1, &mut dag);

            dag.build()
        };

        let a1_rec = cert_graph.record('A', 1);
        let b1_rec = cert_graph.record('B', 1);

        assert_ne!(
            a1_rec.state_before.exit_tree.get_root(),
            a1_rec.state_after.exit_tree.get_root()
        );
        assert_eq!(
            b1_rec.state_before.exit_tree.get_root(),
            b1_rec.state_after.exit_tree.get_root()
        );
    }

    #[test]
    fn sp1_executes_for_all_nodes_simple_chain() {
        let cert_graph = {
            let mut dag = CertGraphBuilder::default();
            let _a1 = dag.add_cert('A');
            let a2 = dag.add_cert('A');
            let b1 = dag.add_cert('B');
            b1.claims_from(a2, &mut dag);

            dag.build()
        };

        for idx in cert_graph.iter_topological_order() {
            cert_graph.execute_sp1(idx);
        }
    }

    #[test]
    fn sp1_executes_for_parallel_networks() {
        let cert_graph = {
            let mut dag = CertGraphBuilder::default();
            let _a1 = dag.add_cert('A');
            let _a2 = dag.add_cert('A');
            let _b1 = dag.add_cert('B');
            let _b2 = dag.add_cert('B');

            dag.build()
        };

        for idx in cert_graph.iter_topological_order() {
            cert_graph.execute_sp1(idx);
        }
    }

    #[test]
    fn sp1_executes_for_transitive_claims() {
        let cert_graph = {
            let mut dag = CertGraphBuilder::default();
            let a1 = dag.add_cert('A');
            let b1 = dag.add_cert('B');
            let c1 = dag.add_cert('C');
            b1.claims_from(a1, &mut dag);
            c1.claims_from(b1, &mut dag);

            dag.build()
        };

        for idx in cert_graph.iter_topological_order() {
            cert_graph.execute_sp1(idx);
        }
    }

    #[test]
    fn sp1_executes_clean_chain() {
        let cert_graph = {
            let mut dag = CertGraphBuilder::default();
            let _a1 = dag.add_cert('A');
            let a2 = dag.add_cert('A');
            let b1 = dag.add_cert('B');
            b1.claims_from(a2, &mut dag);
            let c1 = dag.add_cert('C');
            c1.claims_from(b1, &mut dag);

            dag.build()
        };

        for idx in cert_graph.iter_topological_order() {
            cert_graph.execute_sp1(idx);
        }
    }

    #[test]
    //      A1
    //     /  \
    //    B1  C1
    //     \  /
    //      D1
    fn diamond_claim() {
        let cert_graph = {
            let mut dag = CertGraphBuilder::default();

            let a1 = dag.add_cert('A');
            let b1 = dag.add_cert('B');
            let c1 = dag.add_cert('C');
            let d1 = dag.add_cert('D');

            b1.claims_from(a1, &mut dag);
            c1.claims_from(a1, &mut dag);
            d1.claims_from(b1, &mut dag);
            d1.claims_from(c1, &mut dag);

            dag.build()
        };

        assert_eq!(cert_graph.record('A', 1).certificate.bridge_exits.len(), 2);

        assert_eq!(
            cert_graph
                .record('B', 1)
                .certificate
                .imported_bridge_exits
                .len(),
            1
        );
        assert_eq!(
            cert_graph
                .record('C', 1)
                .certificate
                .imported_bridge_exits
                .len(),
            1
        );

        assert_eq!(
            cert_graph
                .record('D', 1)
                .certificate
                .imported_bridge_exits
                .len(),
            2
        );

        for idx in cert_graph.iter_topological_order() {
            cert_graph.execute_sp1(idx);
        }
    }

    #[test]
    fn parallel_chain_with_multiple_sends() {
        let cert_graph = {
            let mut dag = CertGraphBuilder::default();
            let a1 = dag.add_cert('A');
            let _a2 = dag.add_cert('A');
            let a3 = dag.add_cert('A');
            let b1 = dag.add_cert('B');
            let b2 = dag.add_cert('B');

            b1.claims_from(a1, &mut dag);
            b2.claims_from(a3, &mut dag);

            dag.build()
        };

        assert_eq!(cert_graph.record('A', 1).certificate.bridge_exits.len(), 1);
        assert_eq!(cert_graph.record('A', 3).certificate.bridge_exits.len(), 1);

        assert_eq!(
            cert_graph
                .record('B', 1)
                .certificate
                .imported_bridge_exits
                .len(),
            1
        );
        assert_eq!(
            cert_graph
                .record('B', 2)
                .certificate
                .imported_bridge_exits
                .len(),
            1
        );

        for idx in cert_graph.iter_topological_order() {
            cert_graph.execute_sp1(idx);
        }
    }

    #[test]
    fn deep_depth_dag() {
        let cert_graph = {
            let mut dag = CertGraphBuilder::default();

            let a1 = dag.add_cert('A');
            let b1 = dag.add_cert('B');
            let c1 = dag.add_cert('C');
            b1.claims_from(a1, &mut dag);
            c1.claims_from(a1, &mut dag);

            let d1 = dag.add_cert('D');
            let e1 = dag.add_cert('E');
            d1.claims_from(b1, &mut dag);
            e1.claims_from(c1, &mut dag);

            let f1 = dag.add_cert('F');
            let g1 = dag.add_cert('G');
            f1.claims_from(d1, &mut dag);
            g1.claims_from(e1, &mut dag);

            dag.build()
        };

        for idx in cert_graph.iter_topological_order() {
            cert_graph.execute_sp1(idx);
        }
    }

    #[test]
    fn messy_cert_graph() {
        let cert_graph = {
            let mut dag = CertGraphBuilder::default();

            let a1 = dag.add_cert('A');
            let b1 = dag.add_cert('B');
            let c1 = dag.add_cert('C');
            let d1 = dag.add_cert('D');

            let a2 = dag.add_cert('A');
            let b2 = dag.add_cert('B');
            let c2 = dag.add_cert('C');
            let d2 = dag.add_cert('D');

            a2.claims_from(b1, &mut dag);
            b2.claims_from(c1, &mut dag);
            c2.claims_from(d1, &mut dag);
            d2.claims_from(a1, &mut dag);

            a2.claims_from(c1, &mut dag);
            b2.claims_from(d1, &mut dag);

            dag.build()
        };

        for idx in cert_graph.iter_topological_order() {
            cert_graph.execute_sp1(idx);
        }
    }
}
