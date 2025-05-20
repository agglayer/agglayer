use std::collections::{BTreeMap, BTreeSet};

pub use agglayer_interop_types::aggchain_proof;
use agglayer_interop_types::{
    aggchain_proof::AggchainData, BridgeExit, GlobalIndex, ImportedBridgeExit,
    ImportedBridgeExitCommitmentValues, TokenInfo,
};
pub use agglayer_primitives::Digest;
use agglayer_primitives::{keccak::Keccak256Hasher, FromBool, Hashable, SignatureError};
use agglayer_tries::{error::SmtError, smt::Smt};
use pessimistic_proof::{
    core::{
        self,
        commitment::{PessimisticRoot, SignatureCommitmentValues},
        Vkey,
    },
    error::ProofVerificationError,
    keccak::keccak256_combine,
    local_balance_tree::{LocalBalancePath, LocalBalanceTree, LOCAL_BALANCE_TREE_DEPTH},
    local_state::StateCommitment,
    multi_batch_header::MultiBatchHeader,
    nullifier_tree::{NullifierKey, NullifierPath, NullifierTree, NULLIFIER_TREE_DEPTH},
    LocalNetworkState, ProofError,
};
use serde::{Deserialize, Serialize};
use unified_bridge::CommitmentVersion;
pub type EpochNumber = u64;
pub type CertificateIndex = u64;
pub type CertificateId = Digest;
pub type Height = u64;
pub type Metadata = Digest;
pub use agglayer_interop_types::NetworkId;
pub use agglayer_primitives as primitives;
// Re-export common primitives again as agglayer-types root types
pub use agglayer_primitives::{Address, Signature, B256, U256, U512};
pub use pessimistic_proof::proof::Proof;
use unified_bridge::{LocalExitTree, LocalExitTreeError};

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
#[cfg_attr(feature = "testutils", derive(arbitrary::Arbitrary))]
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
    /// The certificate refers to a prev local exit root which differ from the
    /// one computed by the agglayer.
    #[error(
        "Mismatch on the certificate prev local exit root. declared: {declared:?}, computed: \
         {computed:?}"
    )]
    MismatchPrevLocalExitRoot { computed: Digest, declared: Digest },
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
    BalanceProofGenerationFailed { source: SmtError, token: TokenInfo },
    /// The nullifier path for the given imported bridge exit cannot be
    /// generated.
    #[error(
        "Unable to generate the nullifier path. global_index: {global_index:?}, error: {source}"
    )]
    NullifierPathGenerationFailed {
        source: SmtError,
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
    #[error(
        "Incorrect declared L1 Info Tree information: l1_leaf: {l1_leaf:?}, l1_root: \
         {l1_info_root:?}"
    )]
    InconsistentL1InfoTreeInformation {
        l1_leaf: Option<u32>,
        l1_info_root: Option<Digest>,
    },
    /// The operation cannot be applied on the smt.
    #[error(transparent)]
    InvalidSmtOperation(#[from] SmtError),

    /// Inconsistent GERs
    #[error("Inconsistent GER")]
    InconsistentGlobalExitRoot,

    #[error("AggchainVkey missing")]
    MissingAggchainVkey,

    #[error(
        "Invalid custom chain data length expected at least {expected_at_least}, actual {actual}"
    )]
    InvalidCustomChainDataLength {
        expected_at_least: usize,
        actual: usize,
    },
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
    #[error("Proof verification failed")]
    ProofVerificationFailed(#[source] ProofVerificationError),

    /// Failure on the pessimistic proof witness generation from the
    /// [`LocalNetworkStateData`] and the provided [`Certificate`].
    #[error("Cannot produce local network state from certificate")]
    TypeConversionError(#[source] Error),

    #[error("Trusted sequencer address not found for network: {0}")]
    TrustedSequencerNotFound(NetworkId),

    #[error("Last pessimistic root not found for network: {0}")]
    LastPessimisticRootNotFound(NetworkId),

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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CertificateStatus {
    /// Received certificate from the network, nothing checked yet.
    ///
    /// Certificate will stay in this state until rate-limiting is lifted or an
    /// epoch-change event is triggered. A pending certificate can then be
    /// processed by the agglayer to be proven, or it could end up in error.
    Pending,

    /// Pessimistic proof has been generated for the certificate and stored in
    /// the rocksdb in the agglayer node.
    Proven,

    /// Settlement of the certificate's proof has already been started on L1
    /// (and acknowledged by its RPC) by issuing a contract call to the
    /// RollupManager, but the associated transaction has not yet seen
    /// enough confirmations.
    ///
    /// The certificate can move from Candidate to Settled if the associated
    /// transaction is accepted and the transaction receipt is a success. If the
    /// transaction receipt fails, the certificate will end up in Error.
    Candidate,

    /// Hit some error while moving the certificate through the pipeline.
    ///
    /// For example, proving failed (Pending -> InError), L1 reorg'd (Candidate
    /// -> InError)... See the documentation of `CertificateStatusError` for
    /// more details.
    ///
    /// Note that a certificate can be InError in agglayer but settled on L1,
    /// eg. if there was an error in agglayer but the certificate was valid
    /// and settled on L1.
    InError { error: CertificateStatusError },

    /// Transaction to settle the certificate was completed successfully on L1.
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

#[cfg(any(test, feature = "testutils"))]
impl Default for Certificate {
    fn default() -> Self {
        let network_id = NetworkId::ETH_L1;
        let wallet = Self::wallet_for_test(network_id);
        let exit_root = LocalExitTree::<Keccak256Hasher>::default().get_root();
        let height: Height = 0u64;
        let (_new_local_exit_root, signature, _signer) =
            compute_signature_info(exit_root, &[], &wallet, height);
        Self {
            network_id,
            height,
            prev_local_exit_root: exit_root,
            new_local_exit_root: exit_root,
            bridge_exits: Default::default(),
            imported_bridge_exits: Default::default(),
            aggchain_data: AggchainData::ECDSA { signature },
            metadata: Default::default(),
            custom_chain_data: vec![],
            l1_info_tree_leaf_count: None,
        }
    }
}

#[cfg(any(test, feature = "testutils"))]
pub fn compute_signature_info(
    new_local_exit_root: Digest,
    imported_bridge_exits: &[ImportedBridgeExit],
    wallet: &ethers::signers::LocalWallet,
    height: Height,
) -> (Digest, Signature, Address) {
    use ethers::signers::Signer;

    let version = CommitmentVersion::V2;
    let combined_hash = SignatureCommitmentValues {
        new_local_exit_root,
        commit_imported_bridge_exits: ImportedBridgeExitCommitmentValues {
            claims: imported_bridge_exits
                .iter()
                .map(|exit| exit.to_indexed_exit_hash())
                .collect(),
        },
        height,
    }
    .commitment(version);

    let signature = wallet.sign_hash(combined_hash.0.into()).unwrap();
    let signature = Signature::new(
        U256::from_limbs(signature.r.0),
        U256::from_limbs(signature.s.0),
        signature.recovery_id().unwrap().is_y_odd(),
    );

    (combined_hash, signature, wallet.address().0.into())
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
        let (_, signature, _signer) = compute_signature_info(exit_root, &[], &wallet, height);

        Self {
            network_id,
            height,
            prev_local_exit_root: exit_root,
            new_local_exit_root: exit_root,
            bridge_exits: Default::default(),
            imported_bridge_exits: Default::default(),
            aggchain_data: AggchainData::ECDSA { signature },
            metadata: Default::default(),
            custom_chain_data: vec![],
            l1_info_tree_leaf_count: None,
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

    pub fn signer(&self) -> Result<Option<Address>, SignatureError> {
        match self.aggchain_data {
            AggchainData::ECDSA { signature } => {
                // retrieve signer
                let version = CommitmentVersion::V2;
                let combined_hash = SignatureCommitmentValues::from(self).commitment(version);

                signature
                    .recover_address_from_prehash(&B256::new(combined_hash.0))
                    .map(Some)
            }
            AggchainData::Generic {
                ref signature,
                aggchain_params,
                ..
            } => {
                let commitment = SignatureCommitmentValues::from(self)
                    .aggchain_proof_commitment(&aggchain_params);

                signature
                    .recover_address_from_prehash(&B256::new(commitment.0))
                    .map(Some)
            }
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
            height: certificate.height,
        }
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

impl From<LocalNetworkStateData> for pessimistic_proof::NetworkState {
    fn from(state: LocalNetworkStateData) -> Self {
        LocalNetworkState::from(state).into()
    }
}

/// The last pessimistic root can be either fetched from L1 or recomputed for a
/// given version.
pub enum PessimisticRootInput {
    /// Computed from the given version.
    Computed(CommitmentVersion),
    /// Fetched from the L1.
    Fetched(Digest),
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
        prev_pp_root: PessimisticRootInput,
        aggchain_vkey: Option<Vkey>,
    ) -> Result<MultiBatchHeader<Keccak256Hasher>, Error> {
        let gers_are_consistent = certificate
            .imported_bridge_exits
            .iter()
            .all(|ib| ib.valid_claim());

        if !gers_are_consistent {
            return Err(Error::InconsistentGlobalExitRoot);
        }

        // Retrieve the pp root
        let prev_pessimistic_root = match prev_pp_root {
            PessimisticRootInput::Fetched(settled_from_l1) => settled_from_l1,
            PessimisticRootInput::Computed(version) => PessimisticRoot {
                balance_root: self.balance_tree.root,
                nullifier_root: self.nullifier_tree.root,
                ler_leaf_count: self.exit_tree.leaf_count(),
                height: certificate.height,
                origin_network: certificate.network_id,
            }
            .compute_pp_root(version),
        };

        let prev_local_exit_root = self.exit_tree.get_root();
        if certificate.prev_local_exit_root != prev_local_exit_root {
            return Err(Error::MismatchPrevLocalExitRoot {
                computed: prev_local_exit_root,
                declared: certificate.prev_local_exit_root,
            });
        }

        for e in certificate.bridge_exits.iter() {
            self.exit_tree.add_leaf(e.hash())?;
        }

        let balances_proofs: BTreeMap<TokenInfo, (U256, LocalBalancePath<Keccak256Hasher>)> = {
            // Consider all the imported bridge exits except for the native token
            let imported_bridge_exits = certificate.imported_bridge_exits.iter().filter(|b| {
                b.bridge_exit.amount_token_info().origin_network != certificate.network_id
            });

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

        // Check that the certificate referred to the right target
        let computed = self.exit_tree.get_root();
        if computed != certificate.new_local_exit_root {
            return Err(Error::MismatchNewLocalExitRoot {
                declared: (*certificate.new_local_exit_root).into(),
                computed: (*computed).into(),
            });
        }

        let aggchain_proof = match &certificate.aggchain_data {
            AggchainData::ECDSA { signature } => {
                let signature = *signature;
                core::AggchainData::ECDSA { signer, signature }
            }
            AggchainData::Generic {
                aggchain_params, ..
            } => core::AggchainData::Generic {
                aggchain_params: *aggchain_params,
                aggchain_vkey: aggchain_vkey.ok_or(Error::MissingAggchainVkey)?,
            },
        };

        Ok(MultiBatchHeader::<Keccak256Hasher> {
            origin_network: certificate.network_id,
            bridge_exits: certificate.bridge_exits.clone(),
            imported_bridge_exits,
            balances_proofs,
            l1_info_root,
            aggchain_proof,
            height: certificate.height,
            prev_pessimistic_root,
        })
    }

    /// Generates the [`MultiBatchHeader`] from the state and a [`Certificate`].
    /// Does not mutate the current state.
    pub fn make_multi_batch_header(
        &self,
        certificate: &Certificate,
        signer: Address,
        l1_info_root: Digest,
        prev_pp_root: PessimisticRootInput,
        aggchain_vkey: Option<Vkey>,
    ) -> Result<MultiBatchHeader<Keccak256Hasher>, Error> {
        self.clone().apply_certificate(
            certificate,
            signer,
            l1_info_root,
            prev_pp_root,
            aggchain_vkey,
        )
    }

    pub fn get_roots(&self) -> StateCommitment {
        StateCommitment {
            exit_root: self.exit_tree.get_root(),
            ler_leaf_count: self.exit_tree.leaf_count(),
            balance_root: self.balance_tree.root,
            nullifier_root: self.nullifier_tree.root,
        }
    }
}
