use agglayer_interop_types::{
    aggchain_proof::{AggchainData, MultisigPayload},
    LocalExitRoot,
};
use agglayer_primitives::{Address, Digest, Hashable, Signature, B256};
use pessimistic_proof::{
    core::commitment::{SignatureCommitmentValues, SignatureCommitmentVersion},
    keccak::keccak256_combine,
    unified_bridge::{BridgeExit, LeafType, TokenInfo},
};
use unified_bridge::{
    ImportedBridgeExit, ImportedBridgeExitCommitmentValues, LocalExitTree, NetworkId,
};

use crate::{aggchain_data::MultisigCtx, Certificate, Height, SignerError, U256};

impl Default for Certificate {
    fn default() -> Self {
        let network_id = NetworkId::ETH_L1;
        let wallet = Self::wallet_for_test(network_id);
        // The LET depth can't be inferred to be the default of 32 due to the
        // limitations of the Rust compiler's type inference, so we specify it here.
        let local_exit_root = LocalExitTree::<32>::default().get_root().into();
        let height = Height::ZERO;
        let mut certificate = Self {
            network_id,
            height,
            prev_local_exit_root: local_exit_root,
            new_local_exit_root: local_exit_root,
            bridge_exits: Default::default(),
            imported_bridge_exits: Default::default(),
            aggchain_data: AggchainData::MultisigOnly {
                multisig: MultisigPayload(vec![None]),
            },
            metadata: Default::default(),
            custom_chain_data: vec![],
            l1_info_tree_leaf_count: None,
        };
        sign_multisig_1_of_1(&mut certificate, &wallet);
        certificate
    }
}

pub fn compute_signature_info(
    new_local_exit_root: LocalExitRoot,
    imported_bridge_exits: &[ImportedBridgeExit],
    wallet: &alloy::signers::local::PrivateKeySigner,
    height: Height,
    version: SignatureCommitmentVersion,
) -> (B256, Signature, Address) {
    use alloy::signers::SignerSync;
    let combined_hash = SignatureCommitmentValues {
        new_local_exit_root,
        commit_imported_bridge_exits: ImportedBridgeExitCommitmentValues {
            claims: imported_bridge_exits
                .iter()
                .map(|exit| exit.to_indexed_exit_hash())
                .collect(),
        },
        height: height.as_u64(),
        aggchain_params: None,
        certificate_id: Digest::default(),
    }
    .commitment(version);

    let signature = wallet
        .sign_hash_sync(&combined_hash)
        .expect("valid signature");
    let signature = Signature::new(signature.r(), signature.s(), signature.v());

    (combined_hash, signature, wallet.address().into())
}

/// Signs the certificate with a 1-of-1 multisig over the V5 multisig commitment.
pub fn sign_multisig_1_of_1(
    certificate: &mut Certificate,
    wallet: &alloy::signers::local::PrivateKeySigner,
) {
    use alloy::signers::SignerSync;

    let commitment = certificate.signature_commitment_values().multisig_commitment();
    let signature = wallet
        .sign_hash_sync(&commitment)
        .expect("valid signature");
    let signature = Signature::new(signature.r(), signature.s(), signature.v());

    certificate.aggchain_data = AggchainData::MultisigOnly {
        multisig: MultisigPayload(vec![Some(signature)]),
    };
}

/// Builds the multisig witness context for a 1-of-1 committee.
pub fn multisig_1_of_1_ctx(certificate: &Certificate, signer: Address) -> MultisigCtx {
    MultisigCtx {
        signers: vec![signer],
        threshold: 1,
        prehash: certificate.signature_commitment_values().multisig_commitment(),
    }
}

/// Re-signs a certificate loaded from legacy ECDSA JSON fixtures as multisig 1-of-1.
pub fn resign_loaded_certificate_as_multisig_1_of_1(certificate: &mut Certificate) {
    let wallet = Certificate::wallet_for_test(certificate.network_id);
    sign_multisig_1_of_1(certificate, &wallet);
}

impl Certificate {
    pub fn wallet_for_test(network_id: NetworkId) -> alloy::signers::local::PrivateKeySigner {
        let fake_priv_key = keccak256_combine([b"FAKEKEY:", network_id.to_be_bytes().as_slice()]);
        alloy::signers::local::PrivateKeySigner::from_slice(fake_priv_key.as_bytes())
            .expect("valid fake private key")
    }

    pub fn get_signer(&self) -> Address {
        Self::wallet_for_test(self.network_id).address().into()
    }

    pub fn new_for_test(network_id: NetworkId, height: Height) -> Self {
        Self::new_for_test_with_version(network_id, height, SignatureCommitmentVersion::V5)
    }

    pub fn new_for_test_with_version(
        network_id: NetworkId,
        height: Height,
        _version: SignatureCommitmentVersion,
    ) -> Self {
        // The LET depth can't be inferred to be the default of 32 due to the
        // limitations of the Rust compiler's type inference, so we specify it here.
        let local_exit_root = LocalExitTree::<32>::default().get_root().into();

        Self::new_for_test_custom(
            network_id,
            height,
            local_exit_root,
            0, // No bridge exits for basic test certificates
            AggchainDataType::MultisigOnly { num_signers: 1 },
            SignatureCommitmentVersion::V5,
        )
    }

    /// Generate a certificate with random bridge exits and custom parameters.
    pub fn new_for_test_custom(
        network_id: NetworkId,
        height: Height,
        prev_local_exit_root: LocalExitRoot,
        num_bridge_exits: usize,
        aggchain_data_type: AggchainDataType,
        version: SignatureCommitmentVersion,
    ) -> Self {
        use rand::{Rng, SeedableRng};
        // Use a constant seed for deterministic, repeatable tests
        // Seed is derived from network_id and height for variety while maintaining
        // determinism
        let mut seed = [0u8; 32];
        seed[0..4].copy_from_slice(&network_id.to_u32().to_le_bytes());
        seed[4..12].copy_from_slice(&height.as_u64().to_le_bytes());
        // Remaining bytes stay as 0 for determinism
        let mut rng = rand::rngs::StdRng::from_seed(seed);

        // Generate deterministic bridge exits
        let bridge_exits: Vec<BridgeExit> = (0..num_bridge_exits)
            .map(|_| {
                let origin_network = NetworkId::new(rng.random_range(0..100));
                let token_address = Address::from(rng.random::<[u8; 20]>());
                BridgeExit {
                    leaf_type: LeafType::Transfer,
                    token_info: TokenInfo {
                        origin_network,
                        origin_token_address: token_address,
                    },
                    dest_network: NetworkId::new(rng.random_range(0..100)),
                    dest_address: Address::from(rng.random::<[u8; 20]>()),
                    amount: U256::from(rng.random::<u128>()),
                    metadata: Some(Digest::from(rng.random::<[u8; 32]>())),
                }
            })
            .collect();

        // Calculate new_local_exit_root based on bridge_exits
        let new_local_exit_root = if bridge_exits.is_empty() {
            prev_local_exit_root
        } else {
            let mut local_exit_tree = LocalExitTree::<32>::default();
            for exit in &bridge_exits {
                local_exit_tree
                    .add_leaf(exit.hash())
                    .expect("Failed to add leaf");
            }
            local_exit_tree.get_root().into()
        };

        let wallet = Self::wallet_for_test(network_id);

        let mut certificate = Self {
            network_id,
            height,
            prev_local_exit_root,
            new_local_exit_root,
            bridge_exits,
            imported_bridge_exits: Default::default(),
            aggchain_data: AggchainData::MultisigOnly {
                multisig: MultisigPayload(vec![None]),
            },
            metadata: Default::default(),
            custom_chain_data: vec![],
            l1_info_tree_leaf_count: None,
        };

        certificate.aggchain_data = match aggchain_data_type {
            AggchainDataType::MultisigOnly { num_signers: 1 } => {
                sign_multisig_1_of_1(&mut certificate, &wallet);
                return certificate;
            }
            AggchainDataType::MultisigOnly { num_signers } => {
                let (_, signature, _) =
                    compute_signature_info(new_local_exit_root, &[], &wallet, height, version);
                let threshold = num_signers.div_ceil(2);
                let signatures: Vec<Option<Signature>> = (0..num_signers)
                    .map(|i| if i < threshold { Some(signature) } else { None })
                    .collect();
                AggchainData::MultisigOnly {
                    multisig: MultisigPayload(signatures),
                }
            }
            AggchainDataType::Generic => {
                let (_, signature, _) =
                    compute_signature_info(new_local_exit_root, &[], &wallet, height, version);
                let aggchain_params = Digest::from(rng.random::<[u8; 32]>());
                let proof = create_dummy_stark_proof();
                AggchainData::Generic {
                    proof,
                    aggchain_params,
                    signature: Some(Box::new(signature)),
                    public_values: None,
                }
            }
            AggchainDataType::MultisigAndAggchainProof { num_signers } => {
                let (_, signature, _) =
                    compute_signature_info(new_local_exit_root, &[], &wallet, height, version);
                let threshold = num_signers.div_ceil(2);
                let signatures: Vec<Option<Signature>> = (0..num_signers)
                    .map(|i| if i < threshold { Some(signature) } else { None })
                    .collect();

                let aggchain_params = Digest::from(rng.random::<[u8; 32]>());
                let proof = create_dummy_stark_proof();

                AggchainData::MultisigAndAggchainProof {
                    multisig: MultisigPayload(signatures),
                    aggchain_proof: agglayer_interop_types::aggchain_proof::AggchainProof {
                        proof,
                        aggchain_params,
                        public_values: None,
                    },
                }
            }
        };

        certificate
    }

    pub fn with_new_local_exit_root(mut self, new_local_exit_root: LocalExitRoot) -> Self {
        self.new_local_exit_root = new_local_exit_root;
        self
    }
}

/// Enum to specify which AggchainData variant to use in test certificates
pub enum AggchainDataType {
    /// Generic proof with aggchain params
    Generic,
    /// Multisig only with specified number of signers
    MultisigOnly { num_signers: usize },
    /// Multisig and aggchain proof with specified number of signers
    MultisigAndAggchainProof { num_signers: usize },
}

/// Create a dummy STARK proof for testing purposes.
/// This creates a minimal SP1 proof that can be used in tests.
fn create_dummy_stark_proof() -> agglayer_interop_types::aggchain_proof::Proof {
    agglayer_sp1::testutils::dummy_sp1_stark_proof_with_version("test")
}

impl Certificate {
    /// Retrieve the signer from the certificate signature.
    pub fn retrieve_signer(
        &self,
        _version: SignatureCommitmentVersion,
    ) -> Result<Address, SignerError> {
        let commitment = self.signature_commitment_values().multisig_commitment();

        let signature = match &self.aggchain_data {
            AggchainData::MultisigOnly { multisig } => multisig
                .0
                .iter()
                .find_map(|signature| signature.as_ref())
                .ok_or(SignerError::Missing)?,
            AggchainData::Generic { signature, .. } => {
                let signature = signature.as_ref().ok_or(SignerError::Missing)?;
                let commitment = self
                    .signature_commitment_values()
                    .commitment(SignatureCommitmentVersion::V4);
                return signature
                    .recover_address_from_prehash(&commitment)
                    .map_err(SignerError::Recovery);
            }
            AggchainData::MultisigAndAggchainProof { multisig, .. } => multisig
                .0
                .iter()
                .find_map(|signature| signature.as_ref())
                .ok_or(SignerError::Missing)?,
            AggchainData::ECDSA { signature } => signature,
        };

        signature
            .recover_address_from_prehash(&commitment)
            .map_err(SignerError::Recovery)
    }

    /// Verifies the certificate's 1-of-1 multisig signature.
    pub fn verify_multisig_1_of_1(&self, expected_signer: Address) -> Result<(), SignerError> {
        let AggchainData::MultisigOnly { multisig } = &self.aggchain_data else {
            return Err(SignerError::Missing);
        };

        self.verify_multisig(
            multisig.clone().into(),
            multisig_1_of_1_ctx(self, expected_signer),
        )
    }
}

#[cfg(test)]
mod tests {
    use agglayer_interop_types::aggchain_proof::AggchainData;
    use pessimistic_proof::core::commitment::SignatureCommitmentVersion;
    use rstest::rstest;

    use crate::{Certificate, Height};

    #[rstest]
    fn can_retrieve_correct_signer(
        #[values(SignatureCommitmentVersion::V3, SignatureCommitmentVersion::V5)]
        version: SignatureCommitmentVersion,
    ) {
        let certificate = Certificate::new_for_test_with_version(2.into(), 1.into(), version);
        let expected_signer = certificate.get_signer();

        assert_eq!(
            certificate.retrieve_signer(version).unwrap(),
            expected_signer
        );

        assert!(certificate
            .verify_multisig_1_of_1(expected_signer)
            .is_ok())
    }

    #[test]
    fn test_new_for_test_custom_multisig_only() {
        use unified_bridge::LocalExitTree;

        use crate::certificate::testutils::AggchainDataType;

        let cert = Certificate::new_for_test_custom(
            3.into(),
            Height::ZERO,
            LocalExitTree::<32>::default().get_root().into(),
            2,
            AggchainDataType::MultisigOnly { num_signers: 1 },
            SignatureCommitmentVersion::V5,
        );

        assert_eq!(cert.bridge_exits.len(), 2);
        assert!(matches!(
            cert.aggchain_data,
            AggchainData::MultisigOnly { .. }
        ));
    }

    #[test]
    fn test_new_for_test_custom_generic() {
        use unified_bridge::LocalExitTree;

        use crate::certificate::testutils::AggchainDataType;

        let cert = Certificate::new_for_test_custom(
            2.into(),
            Height::ZERO,
            LocalExitTree::<32>::default().get_root().into(),
            3,
            AggchainDataType::Generic,
            SignatureCommitmentVersion::V2,
        );

        assert_eq!(cert.bridge_exits.len(), 3);
        assert!(matches!(cert.aggchain_data, AggchainData::Generic { .. }));
    }

    #[test]
    fn test_new_for_test_custom_multisig_and_aggchain_proof() {
        use unified_bridge::LocalExitTree;

        use crate::certificate::testutils::AggchainDataType;

        let cert = Certificate::new_for_test_custom(
            4.into(),
            Height::ZERO,
            LocalExitTree::<32>::default().get_root().into(),
            10,
            AggchainDataType::MultisigAndAggchainProof { num_signers: 3 },
            SignatureCommitmentVersion::V3,
        );

        assert_eq!(cert.bridge_exits.len(), 10);
        assert!(matches!(
            cert.aggchain_data,
            AggchainData::MultisigAndAggchainProof { .. }
        ));
    }
}
