use std::{path::PathBuf, time::Instant};

use agglayer_types::{Address, Certificate, U256};
use clap::Parser;
use pessimistic_proof::bridge_exit::{NetworkId, TokenInfo};
use pessimistic_proof::PessimisticProofOutput;
use pessimistic_proof_test_suite::{
    runner::Runner,
    sample_data::{self as data},
};
use serde::{Deserialize, Serialize};
use sp1_sdk::HashableKey;
use tracing::{info, warn};
use uuid::Uuid;

/// The arguments for the pp generator.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct PPGenArgs {
    /// The number of bridge exits.
    #[clap(long, default_value = "10")]
    n_exits: usize,

    /// The number of imported bridge exits.
    #[clap(long, default_value = "10")]
    n_imported_exits: usize,

    /// The optional output directory to write the proofs in JSON. If not set,
    /// the proof is simply logged.
    #[clap(long)]
    proof_dir: Option<PathBuf>,

    /// The optional path to the custom sample data.
    #[clap(long)]
    sample_path: Option<PathBuf>,
}

fn get_events(n: usize, path: Option<PathBuf>) -> Vec<(TokenInfo, U256)> {
    if let Some(p) = path {
        data::sample_bridge_exits(p)
            .cycle()
            .take(n)
            .map(|e| (e.token_info, e.amount))
            .collect::<Vec<_>>()
    } else {
        data::sample_bridge_exits_01()
            .cycle()
            .take(n)
            .map(|e| (e.token_info, e.amount))
            .collect::<Vec<_>>()
    }
}

pub fn main() {
    sp1_sdk::utils::setup_logger();

    let args = PPGenArgs::parse();

    let mut state = data::sample_state_00();

    let old_state = state.state_b.clone();

    let bridge_exits = get_events(args.n_exits, args.sample_path.clone());
    let imported_bridge_exits = get_events(args.n_imported_exits, args.sample_path);

    let certificate = state.apply_events(&imported_bridge_exits, &bridge_exits);

    info!(
        "Certificate {}: [{}]",
        certificate.hash(),
        serde_json::to_string(&certificate).unwrap()
    );

    let l1_info_root = certificate.l1_info_root().unwrap().unwrap_or_default();
    let multi_batch_header = old_state
        .make_multi_batch_header(&certificate, state.get_signer(), l1_info_root)
        .unwrap();

    info!(
        "Generating the proof for {} bridge exit(s) and {} imported bridge exit(s)",
        bridge_exits.len(),
        imported_bridge_exits.len()
    );

    let start = Instant::now();
    let (proof, vk, new_roots) = Runner::new()
        .generate_plonk_proof(&old_state.into(), &multi_batch_header)
        .expect("proving failed");
    let duration = start.elapsed();
    info!(
        "Successfully generated the plonk proof with a latency of {:?}",
        duration
    );

    let vkey = vk.bytes32().to_string();
    info!("vkey: {}", vkey);

    let fixture = PessimisticProofFixture {
        certificate,
        pp_inputs: new_roots.into(),
        signer: state.get_signer(),
        vkey: vkey.clone(),
        public_values: format!("0x{}", hex::encode(proof.public_values.as_slice())),
        proof: format!("0x{}", hex::encode(proof.bytes())),
    };

    if let Some(proof_dir) = args.proof_dir {
        // Save the plonk proof to a json file.
        let proof_path = proof_dir.join(format!(
            "{}-exits-v{}-{}.json",
            args.n_exits,
            &vkey[..8],
            Uuid::new_v4()
        ));
        if let Err(e) = std::fs::create_dir_all(&proof_dir) {
            warn!("Failed to create directory: {e}");
        }
        info!("Writing the proof to {:?}", proof_path);
        std::fs::write(proof_path, serde_json::to_string_pretty(&fixture).unwrap())
            .expect("failed to write fixture");
    } else {
        info!("Proof: {:?}", fixture);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct VerifierInputs {
    /// The previous local exit root.
    pub prev_local_exit_root: String,
    /// The previous pessimistic root.
    pub prev_pessimistic_root: String,
    /// The l1 info root against which we prove the inclusion of the
    /// imported bridge exits.
    pub l1_info_root: String,
    /// The origin network of the pessimistic proof.
    pub origin_network: NetworkId,
    /// The consensus hash.
    pub consensus_hash: String,
    /// The new local exit root.
    pub new_local_exit_root: String,
    /// The new pessimistic root which commits to the balance and nullifier
    /// tree.
    pub new_pessimistic_root: String,
}

impl From<PessimisticProofOutput> for VerifierInputs {
    fn from(v: PessimisticProofOutput) -> Self {
        Self {
            prev_local_exit_root: format!("0x{}", hex::encode(v.prev_local_exit_root)),
            prev_pessimistic_root: format!("0x{}", hex::encode(v.prev_pessimistic_root)),
            l1_info_root: format!("0x{}", hex::encode(v.l1_info_root)),
            origin_network: v.origin_network.into(),
            consensus_hash: format!("0x{}", hex::encode(v.consensus_hash)),
            new_local_exit_root: format!("0x{}", hex::encode(v.new_local_exit_root)),
            new_pessimistic_root: format!("0x{}", hex::encode(v.new_pessimistic_root)),
        }
    }
}

/// A fixture that can be used to test the verification of SP1 zkVM proofs
/// inside Solidity.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct PessimisticProofFixture {
    certificate: Certificate,
    pp_inputs: VerifierInputs,
    signer: Address,
    vkey: String,
    public_values: String,
    proof: String,
}
