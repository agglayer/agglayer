use agglayer_interop_types::{aggchain_proof::AggchainData, LocalExitRoot};
use agglayer_primitives::{Address, Digest, Hashable, Signature, B256};
use pessimistic_proof::{
    core::commitment::{SignatureCommitmentValues, SignatureCommitmentVersion},
    keccak::keccak256_combine,
    unified_bridge::{BridgeExit, LeafType, TokenInfo},
};
use unified_bridge::{
    ImportedBridgeExit, ImportedBridgeExitCommitmentValues, LocalExitTree, NetworkId,
};

use crate::{Certificate, Height, SignerError, U256};

impl Default for Certificate {
    fn default() -> Self {
        let network_id = NetworkId::ETH_L1;
        let wallet = Self::wallet_for_test(network_id);
        // The LET depth can't be inferred to be the default of 32 due to the
        // limitations of the Rust compiler's type inference, so we specify it here.
        let local_exit_root = LocalExitTree::<32>::default().get_root().into();
        let height = Height::ZERO;
        let (_new_local_exit_root, signature, _signer) = compute_signature_info(
            local_exit_root,
            &[],
            &wallet,
            height,
            SignatureCommitmentVersion::V2,
        );
        Self {
            network_id,
            height,
            prev_local_exit_root: local_exit_root,
            new_local_exit_root: local_exit_root,
            bridge_exits: Default::default(),
            imported_bridge_exits: Default::default(),
            aggchain_data: AggchainData::ECDSA { signature },
            metadata: Default::default(),
            custom_chain_data: vec![],
            l1_info_tree_leaf_count: None,
        }
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
        Self::new_for_test_with_version(network_id, height, SignatureCommitmentVersion::V2)
    }

    pub fn new_for_test_with_version(
        network_id: NetworkId,
        height: Height,
        version: SignatureCommitmentVersion,
    ) -> Self {
        let wallet = Self::wallet_for_test(network_id);
        // The LET depth can't be inferred to be the default of 32 due to the
        // limitations of the Rust compiler's type inference, so we specify it here.
        let local_exit_root = LocalExitTree::<32>::default().get_root().into();
        let (_, signature, _signer) =
            compute_signature_info(local_exit_root, &[], &wallet, height, version);

        Self {
            network_id,
            height,
            prev_local_exit_root: local_exit_root,
            new_local_exit_root: local_exit_root,
            bridge_exits: Default::default(),
            imported_bridge_exits: Default::default(),
            aggchain_data: AggchainData::ECDSA { signature },
            metadata: Default::default(),
            custom_chain_data: vec![],
            l1_info_tree_leaf_count: None,
        }
    }

    /// Generate a certificate with random bridge exits and custom parameters.
    ///
    /// # Arguments
    /// * `network_id` - The network ID
    /// * `height` - The certificate height
    /// * `prev_local_exit_root` - The previous local exit root
    /// * `num_bridge_exits` - Number of random bridge exits to generate
    /// * `aggchain_data_type` - Type of AggchainData variant to use
    /// * `version` - Signature commitment version
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
        let (_, signature, _signer) =
            compute_signature_info(new_local_exit_root, &[], &wallet, height, version);

        let aggchain_data = match aggchain_data_type {
            AggchainDataType::Ecdsa => AggchainData::ECDSA { signature },
            AggchainDataType::Generic => {
                // Generic variant with proof, aggchain_params, and signature
                let aggchain_params = Digest::from(rng.random::<[u8; 32]>());
                let proof = create_dummy_stark_proof();
                AggchainData::Generic {
                    proof,
                    aggchain_params,
                    signature: Some(Box::new(signature)),
                    public_values: None,
                }
            }
            AggchainDataType::MultisigOnly { num_signers } => {
                // Generate multisig with specified number of signers
                let threshold = num_signers.div_ceil(2); // Majority threshold
                let signatures: Vec<Option<Signature>> = (0..num_signers)
                    .map(|i| {
                        if i < threshold {
                            Some(signature) // Reuse the same signature for
                                            // simplicity
                        } else {
                            None
                        }
                    })
                    .collect();

                AggchainData::MultisigOnly {
                    multisig: agglayer_interop_types::aggchain_proof::MultisigPayload(signatures),
                }
            }
            AggchainDataType::MultisigAndAggchainProof { num_signers } => {
                // Generate both multisig and aggchain proof
                let threshold = num_signers.div_ceil(2);
                let signatures: Vec<Option<Signature>> = (0..num_signers)
                    .map(|i| if i < threshold { Some(signature) } else { None })
                    .collect();

                let aggchain_params = Digest::from(rng.random::<[u8; 32]>());
                let proof = create_dummy_stark_proof();

                AggchainData::MultisigAndAggchainProof {
                    multisig: agglayer_interop_types::aggchain_proof::MultisigPayload(signatures),
                    aggchain_proof: agglayer_interop_types::aggchain_proof::AggchainProof {
                        proof,
                        aggchain_params,
                        public_values: None,
                    },
                }
            }
        };

        Self {
            network_id,
            height,
            prev_local_exit_root,
            new_local_exit_root,
            bridge_exits,
            imported_bridge_exits: Default::default(),
            aggchain_data,
            metadata: Default::default(),
            custom_chain_data: vec![],
            l1_info_tree_leaf_count: None,
        }
    }

    pub fn with_new_local_exit_root(mut self, new_local_exit_root: LocalExitRoot) -> Self {
        self.new_local_exit_root = new_local_exit_root;
        self
    }
}

/// Enum to specify which AggchainData variant to use in test certificates
pub enum AggchainDataType {
    /// Legacy ECDSA signature
    Ecdsa,
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
    use sp1_sdk::Prover;

    // Use empty ELF file for minimal setup
    const EMPTY_ELF: &[u8] =
        include_bytes!("../../../agglayer-storage/src/types/certificate/tests/empty.elf");

    let (proof, vkey) = {
        let client = sp1_sdk::ProverClient::builder().mock().build();
        let (proving_key, verif_key) = client.setup(EMPTY_ELF);
        let dummy_proof = sp1_sdk::SP1ProofWithPublicValues::create_mock_proof(
            &proving_key,
            sp1_sdk::SP1PublicValues::new(),
            sp1_sdk::SP1ProofMode::Compressed,
            sp1_sdk::SP1_CIRCUIT_VERSION,
        );
        let proof = dummy_proof.proof.try_as_compressed().unwrap();
        (proof, verif_key)
    };

    agglayer_interop_types::aggchain_proof::Proof::SP1Stark(
        agglayer_interop_types::aggchain_proof::SP1StarkWithContext {
            proof,
            vkey,
            version: "test".to_string(),
        },
    )
}

impl Certificate {
    /// Retrieve the signer from the certificate signature.
    pub fn retrieve_signer(
        &self,
        version: SignatureCommitmentVersion,
    ) -> Result<Address, SignerError> {
        let (signature, commitment) = match &self.aggchain_data {
            AggchainData::ECDSA { signature } => {
                let commitment = SignatureCommitmentValues::from(self).commitment(version);
                (signature, commitment)
            }
            AggchainData::Generic { signature, .. } => {
                let signature = signature.as_ref().ok_or(SignerError::Missing)?;
                let commitment = SignatureCommitmentValues::from(self)
                    .commitment(SignatureCommitmentVersion::V4);
                (signature.as_ref(), commitment)
            }
            AggchainData::MultisigOnly { .. } => unimplemented!("adapt tests for multisig"),
            AggchainData::MultisigAndAggchainProof { .. } => {
                unimplemented!("adapt tests for multisig")
            }
        };

        signature
            .recover_address_from_prehash(&commitment)
            .map_err(SignerError::Recovery)
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
        #[values(SignatureCommitmentVersion::V2, SignatureCommitmentVersion::V3)]
        version: SignatureCommitmentVersion,
    ) {
        let certificate = Certificate::new_for_test_with_version(2.into(), 1.into(), version);
        let expected_signer = certificate.get_signer();

        // Can retrieve the correct signer address from the signature
        assert_eq!(
            certificate.retrieve_signer(version).unwrap(),
            expected_signer
        );

        // Check that the signature is valid
        let agglayer_types::aggchain_proof::AggchainData::ECDSA { signature } =
            certificate.aggchain_data
        else {
            panic!("inconsistent test data")
        };

        assert!(certificate
            .verify_legacy_ecdsa(expected_signer, &signature)
            .is_ok())
    }

    #[test]
    fn test_new_for_test_custom_ecdsa() {
        use unified_bridge::LocalExitTree;

        use crate::certificate::testutils::AggchainDataType;

        let cert = Certificate::new_for_test_custom(
            1.into(),
            Height::ZERO,
            LocalExitTree::<32>::default().get_root().into(),
            5,
            AggchainDataType::Ecdsa,
            SignatureCommitmentVersion::V2,
        );

        assert_eq!(cert.network_id, 1.into());
        assert_eq!(cert.height, Height::ZERO);
        assert_eq!(cert.bridge_exits.len(), 5);
        assert!(matches!(cert.aggchain_data, AggchainData::ECDSA { .. }));
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
    fn test_new_for_test_custom_multisig_only() {
        use unified_bridge::LocalExitTree;

        use crate::certificate::testutils::AggchainDataType;

        let cert = Certificate::new_for_test_custom(
            3.into(),
            Height::ZERO,
            LocalExitTree::<32>::default().get_root().into(),
            2,
            AggchainDataType::MultisigOnly { num_signers: 5 },
            SignatureCommitmentVersion::V2,
        );

        assert_eq!(cert.bridge_exits.len(), 2);
        assert!(matches!(
            cert.aggchain_data,
            AggchainData::MultisigOnly { .. }
        ));
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
