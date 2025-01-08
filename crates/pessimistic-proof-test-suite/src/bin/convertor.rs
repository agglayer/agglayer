use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};

use agglayer_types::{Address, Height, Metadata, NetworkId, Signature, U256};
use pessimistic_proof::bridge_exit::{LeafType, TokenInfo};
use pessimistic_proof::global_index::GlobalIndex;
use pessimistic_proof::keccak::keccak256;
use serde::ser::SerializeSeq as _;
use serde::{Deserialize, Serialize};

fn digest<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: AsRef<[u8]>,
    S: serde::Serializer,
{
    let s = format!("0x{}", hex::encode(value.as_ref()));
    serializer.serialize_str(&s)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Certificate {
    /// NetworkID of the origin network.
    pub network_id: NetworkId,
    /// Simple increment to count the Certificate per network.
    pub height: Height,
    /// Previous local exit root.
    #[serde(serialize_with = "digest")]
    pub prev_local_exit_root: [u8; 32],
    /// New local exit root.
    #[serde(serialize_with = "digest")]
    pub new_local_exit_root: [u8; 32],
    /// List of bridge exits included in this state transition.
    pub bridge_exits: Vec<BridgeExit>,
    /// List of imported bridge exits included in this state transition.
    pub imported_bridge_exits: Vec<ImportedBridgeExit>,
    /// Signature committed to the bridge exits and imported bridge exits.
    pub signature: Signature,
    /// Fixed size field of arbitrary data for the chain needs.
    pub metadata: Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedBridgeExit {
    /// The bridge exit initiated on another network, called the "sending"
    /// network. Need to verify that the destination network matches the
    /// current network, and that the bridge exit is included in an imported
    /// LER
    pub bridge_exit: BridgeExit,
    /// The claim data
    pub claim_data: Claim,
    /// The global index of the imported bridge exit.
    pub global_index: GlobalIndex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Claim {
    Mainnet(Box<ClaimFromMainnet>),
    Rollup(Box<ClaimFromRollup>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimFromMainnet {
    /// Proof from bridge exit leaf to MER
    pub proof_leaf_mer: MerkleProof,
    /// Proof from GER to L1Root
    pub proof_ger_l1root: MerkleProof,
    /// L1InfoTree leaf
    pub l1_leaf: L1InfoTreeLeaf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub proof: LETMerkleProof,
    #[serde(serialize_with = "digest")]
    pub root: [u8; 32],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LETMerkleProof {
    #[serde(serialize_with = "list_digest")]
    pub(crate) siblings: [[u8; 32]; 32],
}

fn list_digest<S>(siblings: &[[u8; 32]; 32], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut seq = serializer.serialize_seq(Some(siblings.len()))?;
    for sibling in siblings {
        seq.serialize_element(&format!("0x{}", hex::encode(sibling)))?
    }
    seq.end()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L1InfoTreeLeaf {
    pub l1_info_tree_index: u32,
    #[serde(serialize_with = "digest")]
    pub rer: [u8; 32],
    #[serde(serialize_with = "digest")]
    pub mer: [u8; 32],
    pub inner: L1InfoTreeLeafInner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L1InfoTreeLeafInner {
    #[serde(serialize_with = "digest")]
    pub global_exit_root: [u8; 32],
    #[serde(serialize_with = "digest")]
    pub block_hash: [u8; 32],
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimFromRollup {
    /// Proof from bridge exit leaf to LER
    proof_leaf_ler: MerkleProof,
    /// Proof from LER to RER
    proof_ler_rer: MerkleProof,
    /// Proof from GER to L1Root
    proof_ger_l1root: MerkleProof,
    /// L1InfoTree leaf
    l1_leaf: L1InfoTreeLeaf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeExit {
    pub leaf_type: LeafType,

    /// Unique ID for the token being transferred.
    pub token_info: TokenInfo,

    /// Network which the token is transferred to
    pub dest_network: NetworkId,
    /// Address which will own the received token
    pub dest_address: Address,

    /// Token amount sent
    pub amount: U256,

    #[serde(serialize_with = "optional_digest")]
    pub metadata: Vec<u8>,
}

fn optional_digest<S>(value: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if value.is_empty() {
        serializer.serialize_none()
    } else {
        serializer.serialize_str(&format!("0x{}", hex::encode(keccak256(value))))
    }
}

fn process_file(input_path: &str, output_path: &str) -> io::Result<()> {
    // Read the entire input file content
    let mut input_file = File::open(input_path)?;
    let certificate: Certificate = serde_json::from_reader(&mut input_file).unwrap();

    let certificate = serde_json::to_vec(&certificate).unwrap();

    // Write the updated content to the output file
    let mut output_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(output_path)?;

    output_file.write_all(&certificate)?;
    Ok(())
}

fn main() -> io::Result<()> {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <input_file> <output_file>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    // Process the file
    if let Err(e) = process_file(input_path, output_path) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    println!("Processing complete. Output written to {}", output_path);
    Ok(())
}
