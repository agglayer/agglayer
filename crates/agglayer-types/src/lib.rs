use std::collections::{BTreeMap, BTreeSet};

use pessimistic_proof::global_index::GlobalIndex;
pub use pessimistic_proof::keccak::digest::Digest;
use pessimistic_proof::keccak::keccak256_combine;
use pessimistic_proof::local_balance_tree::{LocalBalanceTree, LOCAL_BALANCE_TREE_DEPTH};
use pessimistic_proof::local_exit_tree::hasher::Keccak256Hasher;
use pessimistic_proof::local_exit_tree::{LocalExitTree, LocalExitTreeError};
use pessimistic_proof::local_state::StateCommitment;
use pessimistic_proof::multi_batch_header::signature_commitment;
use pessimistic_proof::nullifier_tree::{FromBool, NullifierTree, NULLIFIER_TREE_DEPTH};
use pessimistic_proof::utils::smt::{Smt, SmtError};
use pessimistic_proof::LocalNetworkState;
use pessimistic_proof::{
    bridge_exit::{BridgeExit, TokenInfo},
    imported_bridge_exit::{commit_imported_bridge_exits, ImportedBridgeExit},
    local_balance_tree::LocalBalancePath,
    multi_batch_header::MultiBatchHeader,
    nullifier_tree::{NullifierKey, NullifierPath},
    ProofError,
};
use serde::{Deserialize, Serialize};
use sp1_sdk::provers::ProofOpts;
use sp1_sdk::{
    MockProver, Prover, SP1Context, SP1Proof, SP1ProofKind, SP1ProofWithPublicValues,
    SP1PublicValues, SP1Stdin,
};

pub type EpochNumber = u64;
pub type CertificateIndex = u64;
pub type CertificateId = Digest;
pub type Height = u64;
pub type Metadata = Digest;

pub use agglayer_primitives as primitives;
// Re-export common primitives again as agglayer-types root types
pub use agglayer_primitives::{Address, Signature, SignatureError, B256, U256, U512};
pub use pessimistic_proof::bridge_exit::NetworkId;
use sp1_sdk::SP1VerificationError;

/// ELF of the pessimistic proof program
pub(crate) const ELF: &[u8] =
    include_bytes!("../../pessimistic-proof-program/elf/riscv32im-succinct-zkvm-elf");

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionMode {
    Default,
    DryRun,
}

impl ExecutionMode {
    pub const fn prefix(&self) -> &'static str {
        match self {
            ExecutionMode::Default => "",
            ExecutionMode::DryRun => "(Dry run) ",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EpochConfiguration {
    /// The genesis block where the AggLayer starts.
    pub genesis_block: u64,
    /// The duration of an epoch in blocks.
    pub epoch_duration: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CertificateHeader {
    pub network_id: NetworkId,
    pub height: Height,
    pub epoch_number: Option<EpochNumber>,
    pub certificate_index: Option<CertificateIndex>,
    pub certificate_id: CertificateId,
    pub prev_local_exit_root: Digest,
    pub new_local_exit_root: Digest,
    pub metadata: Metadata,
    pub status: CertificateStatus,
    pub settlement_tx_hash: Option<Digest>,
}

#[derive(Debug, thiserror::Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Error {
    /// The imported bridge exits should refer to one and the same L1 info root.
    #[error("Imported bridge exits refer to multiple L1 info root")]
    MultipleL1InfoRoot,
    /// The certificate refers to a new local exit root which differ from the
    /// one computed by the agglayer.
    #[error(
        "Mismatch on the certificate new local exit root. declared: {declared:?}, computed: \
         {computed:?}"
    )]
    MismatchNewLocalExitRoot { computed: Digest, declared: Digest },
    /// The given token balance cannot overflow.
    #[error("Token balance cannot overflow. token: {0:?}")]
    BalanceOverflow(TokenInfo),
    /// The given token balance cannot be negative.
    #[error("Token balance cannot be negative. token: {0:?}")]
    BalanceUnderflow(TokenInfo),
    /// The balance proof for the given token cannot be generated.
    #[error("Unable to generate the balance proof. token: {token:?}, error: {source}")]
    BalanceProofGenerationFailed {
        source: pessimistic_proof::utils::smt::SmtError,
        token: TokenInfo,
    },
    /// The nullifier path for the given imported bridge exit cannot be
    /// generated.
    #[error(
        "Unable to generate the nullifier path. global_index: {global_index:?}, error: {source}"
    )]
    NullifierPathGenerationFailed {
        source: pessimistic_proof::utils::smt::SmtError,
        global_index: GlobalIndex,
    },
    /// The operation cannot be applied on the local exit tree.
    #[error(transparent)]
    InvalidLocalExitTreeOperation(#[from] LocalExitTreeError),
    #[error(
        "Incorrect L1 Info Root for the leaf count {leaf_count}. declared: {declared}, retrieved \
         from L1: {retrieved}"
    )]
    /// Invalid or unsettled L1 Info Root
    L1InfoRootIncorrect {
        leaf_count: u32,
        declared: Digest,
        retrieved: Digest,
    },
    /// The operation cannot be applied on the smt.
    #[error(transparent)]
    InvalidSmtOperation(#[from] SmtError),
}

#[derive(Clone, Debug, Serialize, Deserialize, thiserror::Error, PartialEq, Eq)]
pub enum CertificateStatusError {
    /// Failure on the pessimistic proof execution, either natively or in the
    /// prover.
    #[error("({generation_type}) proof generation error: {}", source.to_string())]
    ProofGenerationError {
        generation_type: GenerationType,
        source: ProofError,
    },
    /// Failure on the proof verification.
    #[error("proof verification failed")]
    ProofVerificationFailed(#[from] ProofVerificationError),
    /// Failure on the pessimistic proof witness generation from the
    /// [`LocalNetworkStateData`] and the provided [`Certificate`].
    #[error(transparent)]
    TypeConversionError(#[from] Error),
    #[error("Trusted sequencer address not found for network: {0}")]
    TrustedSequencerNotFound(NetworkId),
    #[error("Internal error")]
    InternalError(String),
    #[error("Settlement error: {0}")]
    SettlementError(String),
    #[error("Pre certification error: {0}")]
    PreCertificationError(String),
    #[error("Certification error: {0}")]
    CertificationError(String),
    #[error("L1 Info root not found for l1 leaf count: {0}")]
    L1InfoRootNotFound(u32),
}

#[derive(Clone, Debug, Serialize, Deserialize, thiserror::Error, PartialEq, Eq)]
pub enum GenerationType {
    Native,
    Prover,
}

impl std::fmt::Display for GenerationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenerationType::Native => write!(f, "native"),
            GenerationType::Prover => write!(f, "prover"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, thiserror::Error, PartialEq, Eq)]
pub enum ProofVerificationError {
    #[error("Version mismatch: {0}")]
    VersionMismatch(String),
    #[error("Core machine verification error: {0}")]
    Core(String),
    #[error("Recursion verification error: {0}")]
    Recursion(String),
    #[error("Plonk verification error: {0}")]
    Plonk(String),
    #[error("Groth16 verification error: {0}")]
    Groth16(String),
    #[error("Invalid public values")]
    InvalidPublicValues,
}

impl From<SP1VerificationError> for ProofVerificationError {
    fn from(err: SP1VerificationError) -> Self {
        match err {
            SP1VerificationError::VersionMismatch(version) => {
                ProofVerificationError::VersionMismatch(version)
            }
            SP1VerificationError::Core(core) => ProofVerificationError::Core(core.to_string()),
            SP1VerificationError::Recursion(recursion) => {
                ProofVerificationError::Recursion(recursion.to_string())
            }
            SP1VerificationError::Plonk(error) => ProofVerificationError::Plonk(error.to_string()),
            SP1VerificationError::Groth16(error) => {
                ProofVerificationError::Groth16(error.to_string())
            }
            SP1VerificationError::InvalidPublicValues => {
                ProofVerificationError::InvalidPublicValues
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CertificateStatus {
    Pending,
    Proven,
    Candidate,
    InError { error: CertificateStatusError },
    Settled,
}

impl std::fmt::Display for CertificateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CertificateStatus::Pending => write!(f, "Pending"),
            CertificateStatus::Proven => write!(f, "Proven"),
            CertificateStatus::Candidate => write!(f, "Candidate"),
            CertificateStatus::InError { error } => write!(f, "InError: {}", error),
            CertificateStatus::Settled => write!(f, "Settled"),
        }
    }
}

/// Proof is a wrapper around all the different types of proofs that can be
/// generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Proof {
    SP1(sp1_sdk::SP1ProofWithPublicValues),
}

impl Proof {
    pub fn dummy() -> Self {
        Self::SP1(SP1ProofWithPublicValues {
            proof: SP1Proof::Core(vec![]),
            stdin: SP1Stdin::new(),
            public_values: SP1PublicValues::new(),
            sp1_version: "".to_string(),
        })
    }
    pub fn new_for_test(
        state: &LocalNetworkState,
        multi_batch_header: &MultiBatchHeader<Keccak256Hasher>,
    ) -> Self {
        let mock = MockProver::new();
        let (p, _v) = mock.setup(ELF);

        let mut stdin = SP1Stdin::new();
        stdin.write(state);
        stdin.write(multi_batch_header);

        let opts = ProofOpts::default();
        let context = SP1Context::default();
        let kind = SP1ProofKind::Plonk;
        let proof = mock.prove(&p, stdin, opts, context, kind).unwrap();

        Proof::SP1(proof)
    }
}

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
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Certificate {
    /// NetworkID of the origin network.
    pub network_id: NetworkId,
    /// Simple increment to count the Certificate per network.
    pub height: Height,
    /// Previous local exit root.
    pub prev_local_exit_root: Digest,
    /// New local exit root.
    pub new_local_exit_root: Digest,
    /// List of bridge exits included in this state transition.
    pub bridge_exits: Vec<BridgeExit>,
    /// List of imported bridge exits included in this state transition.
    pub imported_bridge_exits: Vec<ImportedBridgeExit>,
    /// Signature committed to the bridge exits and imported bridge exits.
    pub signature: Signature,
    /// Fixed size field of arbitrary data for the chain needs.
    pub metadata: Metadata,
}

#[cfg(any(test, feature = "testutils"))]
impl Default for Certificate {
    fn default() -> Self {
        let network_id = Default::default();
        let wallet = Self::wallet_for_test(network_id);
        let exit_root = LocalExitTree::<Keccak256Hasher>::default().get_root();
        let (_new_local_exit_root, signature) = compute_signature_info(exit_root, &[], &wallet);
        Self {
            network_id,
            height: Default::default(),
            prev_local_exit_root: exit_root,
            new_local_exit_root: exit_root,
            bridge_exits: Default::default(),
            imported_bridge_exits: Default::default(),
            signature,
            metadata: Default::default(),
        }
    }
}

#[cfg(any(test, feature = "testutils"))]
pub fn compute_signature_info(
    new_local_exit_root: Digest,
    imported_bridge_exits: &[ImportedBridgeExit],
    wallet: &ethers::signers::LocalWallet,
) -> (Digest, Signature) {
    let combined_hash = pessimistic_proof::multi_batch_header::signature_commitment(
        new_local_exit_root,
        imported_bridge_exits,
    );
    let signature = wallet.sign_hash(combined_hash.0.into()).unwrap();
    let signature = Signature::new(
        U256::from_limbs(signature.r.0),
        U256::from_limbs(signature.s.0),
        signature.recovery_id().unwrap().is_y_odd(),
    );

    (combined_hash, signature)
}

impl Certificate {
    #[cfg(any(test, feature = "testutils"))]
    pub fn wallet_for_test(network_id: NetworkId) -> ethers::signers::LocalWallet {
        let fake_priv_key = keccak256_combine([b"FAKEKEY:", network_id.to_be_bytes().as_slice()]);
        ethers::signers::LocalWallet::from_bytes(fake_priv_key.as_bytes()).unwrap()
    }

    #[cfg(any(test, feature = "testutils"))]
    pub fn get_signer(&self) -> Address {
        use ethers::signers::Signer;
        Self::wallet_for_test(self.network_id).address().0.into()
    }

    #[cfg(any(test, feature = "testutils"))]
    pub fn new_for_test(network_id: NetworkId, height: Height) -> Self {
        let wallet = Self::wallet_for_test(network_id);
        let exit_root = LocalExitTree::<Keccak256Hasher>::default().get_root();
        let (_, signature) = compute_signature_info(exit_root, &[], &wallet);

        Self {
            network_id,
            height,
            prev_local_exit_root: exit_root,
            new_local_exit_root: exit_root,
            bridge_exits: Default::default(),
            imported_bridge_exits: Default::default(),
            signature,
            metadata: Default::default(),
        }
    }

    #[cfg(any(test, feature = "testutils"))]
    pub fn with_new_local_exit_root(mut self, new_local_exit_root: Digest) -> Self {
        self.new_local_exit_root = new_local_exit_root;
        self
    }

    pub fn hash(&self) -> CertificateId {
        let commit_bridge_exits =
            keccak256_combine(self.bridge_exits.iter().map(|exit| exit.hash()));
        let commit_imported_bridge_exits =
            keccak256_combine(self.imported_bridge_exits.iter().map(|exit| exit.hash()));

        keccak256_combine([
            self.network_id.to_be_bytes().as_slice(),
            self.height.to_be_bytes().as_slice(),
            self.prev_local_exit_root.as_slice(),
            self.new_local_exit_root.as_slice(),
            commit_bridge_exits.as_slice(),
            commit_imported_bridge_exits.as_slice(),
            self.metadata.as_slice(),
        ])
    }

    /// Returns the L1 Info Tree leaf count considered for this [`Certificate`].
    /// Corresponds to the highest L1 Info Tree leaf index considered by the
    /// imported bridge exits.
    pub fn l1_info_tree_leaf_count(&self) -> Option<u32> {
        self.imported_bridge_exits
            .iter()
            .map(|i| i.l1_leaf_index() + 1)
            .max()
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

    pub fn signer(&self) -> Option<Address> {
        // retrieve signer
        let combined_hash =
            signature_commitment(self.new_local_exit_root, &self.imported_bridge_exits);

        self.signature
            .recover_address_from_prehash(&B256::new(combined_hash.0))
            .ok()
    }
}

/// Local state data of one network.
/// The AggLayer tracks the [`LocalNetworkStateData`] for all networks.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct LocalNetworkStateData {
    /// The local exit tree without leaves.
    pub exit_tree: LocalExitTree<Keccak256Hasher>,
    /// The full local balance tree.
    pub balance_tree: Smt<Keccak256Hasher, LOCAL_BALANCE_TREE_DEPTH>,
    /// The full nullifier tree.
    pub nullifier_tree: Smt<Keccak256Hasher, NULLIFIER_TREE_DEPTH>,
}

impl From<LocalNetworkStateData> for LocalNetworkState {
    fn from(state: LocalNetworkStateData) -> Self {
        LocalNetworkState {
            exit_tree: state.exit_tree,
            balance_tree: LocalBalanceTree::new_with_root(state.balance_tree.root),
            nullifier_tree: NullifierTree::new_with_root(state.nullifier_tree.root),
        }
    }
}

impl LocalNetworkStateData {
    /// Prune the SMTs
    pub fn prune_stale_nodes(&mut self) -> Result<(), Error> {
        self.balance_tree.traverse_and_prune()?;
        self.nullifier_tree.traverse_and_prune()?;

        Ok(())
    }

    /// Apply the [`Certificate`] on the current state and returns the
    /// [`MultiBatchHeader`] associated to the state transition.
    pub fn apply_certificate(
        &mut self,
        certificate: &Certificate,
        signer: Address,
        l1_info_root: Digest,
    ) -> Result<MultiBatchHeader<Keccak256Hasher>, Error> {
        let prev_balance_root = self.balance_tree.root;
        let prev_nullifier_root = self.nullifier_tree.root;

        for e in certificate.bridge_exits.iter() {
            self.exit_tree.add_leaf(e.hash())?;
        }

        let balances_proofs: BTreeMap<TokenInfo, (U256, LocalBalancePath<Keccak256Hasher>)> = {
            // Consider all the imported bridge exits
            let imported_bridge_exits = certificate.imported_bridge_exits.iter();
            // Consider all the bridge exits except for the native token
            let bridge_exits = certificate
                .bridge_exits
                .iter()
                .filter(|b| b.amount_token_info().origin_network != certificate.network_id);

            // Set of dedup tokens mutated in the transition
            let mutated_tokens: BTreeSet<TokenInfo> = {
                let imported_tokens = imported_bridge_exits
                    .clone()
                    .map(|exit| exit.bridge_exit.amount_token_info());
                let exported_tokens = bridge_exits.clone().map(|exit| exit.amount_token_info());
                imported_tokens.chain(exported_tokens).collect()
            };

            let initial_balances: BTreeMap<_, _> = mutated_tokens
                .iter()
                .map(|&token| {
                    let balance =
                        U256::from_be_bytes(*self.balance_tree.get(token).unwrap_or_default());
                    (token, balance)
                })
                .collect();

            let mut new_balances = initial_balances.clone();
            for imported_bridge_exit in imported_bridge_exits {
                let token = imported_bridge_exit.bridge_exit.amount_token_info();
                new_balances.insert(
                    token,
                    new_balances[&token]
                        .checked_add(imported_bridge_exit.bridge_exit.amount)
                        .ok_or(Error::BalanceOverflow(token))?,
                );
            }

            for bridge_exit in bridge_exits {
                let token = bridge_exit.amount_token_info();
                new_balances.insert(
                    token,
                    new_balances[&token]
                        .checked_sub(bridge_exit.amount)
                        .ok_or(Error::BalanceUnderflow(token))?,
                );
            }

            // Get the proof against the initial balance for each token
            mutated_tokens
                .into_iter()
                .map(|token| {
                    let initial_balance = initial_balances[&token];

                    let balance_proof_error =
                        |source| Error::BalanceProofGenerationFailed { source, token };

                    let path = if initial_balance.is_zero() {
                        self.balance_tree
                            .get_inclusion_proof_zero(token)
                            .map_err(balance_proof_error)?
                    } else {
                        self.balance_tree
                            .get_inclusion_proof(token)
                            .map_err(balance_proof_error)?
                    };

                    self.balance_tree
                        .update(token, new_balances[&token].to_be_bytes().into())
                        .map_err(balance_proof_error)?;

                    Ok((token, (initial_balance, path)))
                })
                .collect::<Result<BTreeMap<_, _>, Error>>()?
        };

        let imported_bridge_exits: Vec<(ImportedBridgeExit, NullifierPath<Keccak256Hasher>)> =
            certificate
                .imported_bridge_exits
                .iter()
                .map(|exit| {
                    let nullifier_key: NullifierKey = exit.global_index.into();
                    let nullifier_error = |source| Error::NullifierPathGenerationFailed {
                        source,
                        global_index: exit.global_index,
                    };
                    let nullifier_path = self
                        .nullifier_tree
                        .get_non_inclusion_proof(nullifier_key)
                        .map_err(nullifier_error)?;
                    self.nullifier_tree
                        .insert(nullifier_key, Digest::from_bool(true))
                        .map_err(nullifier_error)?;
                    Ok((exit.clone(), nullifier_path))
                })
                .collect::<Result<Vec<_>, Error>>()?;

        let imported_hash =
            commit_imported_bridge_exits(imported_bridge_exits.iter().map(|(exit, _)| exit));

        // Check that the certificate referred to the right target
        let computed = self.exit_tree.get_root();
        if computed != certificate.new_local_exit_root {
            return Err(Error::MismatchNewLocalExitRoot {
                declared: (*certificate.new_local_exit_root).into(),
                computed: (*computed).into(),
            });
        }

        Ok(MultiBatchHeader::<Keccak256Hasher> {
            origin_network: certificate.network_id,
            prev_local_exit_root: certificate.prev_local_exit_root,
            bridge_exits: certificate.bridge_exits.clone(),
            imported_bridge_exits,
            balances_proofs,
            prev_balance_root,
            prev_nullifier_root,
            signer,
            signature: certificate.signature,
            imported_exits_root: Some(imported_hash),
            target: self.get_roots(),
            l1_info_root,
        })
    }

    /// Generates the [`MultiBatchHeader`] from the state and a [`Certificate`].
    /// Does not mutate the current state.
    pub fn make_multi_batch_header(
        &self,
        certificate: &Certificate,
        signer: Address,
        l1_info_root: Digest,
    ) -> Result<MultiBatchHeader<Keccak256Hasher>, Error> {
        self.clone()
            .apply_certificate(certificate, signer, l1_info_root)
    }

    pub fn get_roots(&self) -> StateCommitment {
        StateCommitment {
            exit_root: self.exit_tree.get_root(),
            ler_leaf_count: self.exit_tree.leaf_count,
            balance_root: self.balance_tree.root,
            nullifier_root: self.nullifier_tree.root,
        }
    }
}
