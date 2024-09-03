use std::path::PathBuf;

use clap::Parser;
use pessimistic_proof::{
    bridge_exit::{BridgeExit, NetworkId},
    PessimisticProofOutput,
};
use pessimistic_proof_test_suite::{runner::Runner, sample_data as data};
use reth_primitives::Address;
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

    /// The optional output directory to write the proofs in JSON. If not set,
    /// the proof is simply logged.
    #[clap(long)]
    proof_dir: Option<PathBuf>,

    /// The optional path to the custom sample data.
    #[clap(long)]
    sample_path: Option<PathBuf>,
}

pub fn main() {
    sp1_sdk::utils::setup_logger();

    let args = PPGenArgs::parse();

    let mut state = data::sample_state_01();
    let bridge_exits = if let Some(p) = args.sample_path {
        data::sample_bridge_exits(p)
            .take(args.n_exits)
            .collect::<Vec<_>>()
    } else {
        data::sample_bridge_exits_01()
            .take(args.n_exits)
            .collect::<Vec<_>>()
    };

    let withdrawals = bridge_exits
        .iter()
        .map(|be| (be.token_info, be.amount))
        .collect::<Vec<_>>();

    let old_state = state.local_state();
    let batch_header = state.apply_events(&[], &withdrawals);

    info!(
        "Generating the proof for {} bridge exits",
        withdrawals.len()
    );

    let (proof, vk, new_roots) = Runner::new()
        .generate_plonk_proof(&old_state, &batch_header)
        .expect("proving failed");

    info!("Successfully generated the plonk proof");

    let vkey = vk.bytes32().to_string();
    let fixture = PessimisticProofFixture {
        bridge_exits,
        pp_inputs: new_roots.into(),
        signer: batch_header.signer,
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
    /// The global exit root against which we prove the inclusion of the
    /// imported bridge exits.
    pub selected_ger: String,
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
            selected_ger: format!("0x{}", hex::encode(v.selected_ger)),
            origin_network: v.origin_network,
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
    bridge_exits: Vec<BridgeExit>,
    pp_inputs: VerifierInputs,
    signer: Address,
    vkey: String,
    public_values: String,
    proof: String,
}
