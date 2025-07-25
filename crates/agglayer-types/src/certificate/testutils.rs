use agglayer_interop_types::{aggchain_proof::AggchainData, LocalExitRoot};
use agglayer_primitives::{Address, Signature};
use pessimistic_proof::{
    core::commitment::SignatureCommitmentValues,
    keccak::{keccak256_combine, Keccak256Hasher},
};
use unified_bridge::{
    CommitmentVersion, ImportedBridgeExit, ImportedBridgeExitCommitmentValues, LocalExitTree,
    NetworkId,
};

use crate::{Certificate, Digest, Height};

impl Default for Certificate {
    fn default() -> Self {
        let network_id = NetworkId::ETH_L1;
        let wallet = Self::wallet_for_test(network_id);
        let local_exit_root = LocalExitTree::<Keccak256Hasher>::default()
            .get_root()
            .into();
        let height = Height::ZERO;
        let (_new_local_exit_root, signature, _signer) =
            compute_signature_info(local_exit_root, &[], &wallet, height, CommitmentVersion::V2);
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
    version: CommitmentVersion,
) -> (Digest, Signature, Address) {
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
    }
    .commitment(version);

    let signature = wallet
        .sign_hash_sync(&agglayer_primitives::B256::new(combined_hash.0))
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
        Self::new_for_test_with_version(network_id, height, CommitmentVersion::V2)
    }

    pub fn new_for_test_with_version(
        network_id: NetworkId,
        height: Height,
        version: CommitmentVersion,
    ) -> Self {
        let wallet = Self::wallet_for_test(network_id);
        let local_exit_root = LocalExitTree::<Keccak256Hasher>::default()
            .get_root()
            .into();
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

    pub fn with_new_local_exit_root(mut self, new_local_exit_root: LocalExitRoot) -> Self {
        self.new_local_exit_root = new_local_exit_root;
        self
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use unified_bridge::CommitmentVersion;

    use crate::Certificate;

    #[rstest]
    fn can_retrieve_correct_signer(
        #[values(CommitmentVersion::V2, CommitmentVersion::V3)] version: CommitmentVersion,
    ) {
        let certificate = Certificate::new_for_test_with_version(2.into(), 1.into(), version);
        let expected_signer = certificate.get_signer();

        // Can retrieve the correct signer address from the signature
        assert_eq!(
            certificate.retrieve_signer(version).unwrap(),
            expected_signer
        );

        // Check that the signature is valid
        assert!(certificate.verify_cert_signature(expected_signer).is_ok())
    }
}
