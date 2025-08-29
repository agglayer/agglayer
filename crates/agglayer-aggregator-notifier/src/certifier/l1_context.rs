use agglayer_certificate_orchestrator::CertificationError;
use agglayer_contracts::{aggchain::AggchainContract, RollupContract};
use agglayer_types::{
    aggchain_data::{
        AggchainProofCtx, AggchainProofPayload, CertificateAggchainData,
        CertificateAggchainDataCtx, MultisigCtx,
    },
    aggchain_proof::AggchainData,
    Address, Certificate, Digest, L1WitnessCtx, PessimisticRootInput, B256,
};
use eyre::Context as _;
use pessimistic_proof::core::commitment::{SignatureCommitmentValues, SignatureCommitmentVersion};
use prover_executor::sp1_fast;
use sp1_sdk::HashableKey;

use crate::CertifierClient;

impl<PendingStore, L1Rpc> CertifierClient<PendingStore, L1Rpc>
where
    L1Rpc: RollupContract + AggchainContract + Send + Sync + 'static,
{
    /// Fetch all the necessary context from the L1 for the witness generation.
    pub async fn fetch_l1_context(
        &self,
        certificate: &Certificate,
    ) -> Result<L1WitnessCtx, CertificationError> {
        let network_id = certificate.network_id;

        let prev_pessimistic_root = self
            .l1_rpc
            .get_prev_pessimistic_root(network_id.to_u32())
            .await
            .map_err(|_| CertificationError::LastPessimisticRootNotFound(network_id))?;

        let l1_info_root = self.fetch_l1_info_root(certificate).await?;

        let aggchain_data_paylaod = CertificateAggchainData::try_from(
            certificate.aggchain_data.clone(),
        )
        .map_err(|source| CertificationError::Types {
            source: agglayer_types::Error::InvalidChainData(source),
        })?;

        // Fetch context based on the aggchain data type that we received from the
        // chain.
        let aggchain_data_ctx: CertificateAggchainDataCtx = match aggchain_data_paylaod {
            CertificateAggchainData::LegacyEcdsa { .. } => {
                let signer = self
                    .l1_rpc
                    .get_trusted_sequencer_address(
                        network_id.to_u32(),
                        self.config.proof_signers.clone(),
                    )
                    .await
                    .map_err(|_| CertificationError::TrustedSequencerNotFound(network_id))?;
                CertificateAggchainDataCtx::LegacyEcdsa { signer }
            }
            CertificateAggchainData::MultisigOnly(_) => CertificateAggchainDataCtx::MultisigOnly(
                self.fetch_multisig_ctx(certificate).await?,
            ),
            CertificateAggchainData::MultisigAndAggchainProof { aggchain_proof, .. } => {
                CertificateAggchainDataCtx::MultisigAndAggchainProof {
                    multisig_ctx: self.fetch_multisig_ctx(certificate).await?,
                    aggchain_proof_ctx: self
                        .fetch_aggchain_proof_ctx(certificate, &aggchain_proof)
                        .await?,
                }
            }
        };

        Ok(L1WitnessCtx {
            prev_pessimistic_root: PessimisticRootInput::Fetched(prev_pessimistic_root.into()),
            l1_info_root,
            aggchain_data_ctx,
        })
    }

    pub async fn fetch_aggchain_proof_ctx(
        &self,
        certificate: &Certificate,
        aggchain_proof_payload: &AggchainProofPayload,
    ) -> Result<AggchainProofCtx, CertificationError> {
        // Fetching rollup contract address
        let rollup_address = self
            .l1_rpc
            .get_rollup_contract_address(certificate.network_id.to_u32())
            .await
            .map_err(CertificationError::RollupContractAddressNotFound)?;

        let aggchain_vkey_selector = certificate
            .custom_chain_data
            .first_chunk::<2>()
            .ok_or(CertificationError::Types {
                source: agglayer_types::Error::InvalidCustomChainDataLength {
                    expected_at_least: 2,
                    actual: certificate.custom_chain_data.len(),
                },
            })
            .map(|bytes| u16::from_be_bytes(*bytes))?;

        let aggchain_vkey = self
            .l1_rpc
            .get_aggchain_vkey_hash(rollup_address, aggchain_vkey_selector)
            .await
            .map_err(|source| CertificationError::UnableToFindAggchainVkey { source })?;

        let vkey = aggchain_proof_payload.aggchain_vkey_from_proof();

        let vkey_hash_bytes = sp1_fast(|| vkey.vk.hash_bytes())
            .context("Failed to hash SP1 vkey")
            .map_err(CertificationError::Other)?;

        let proof_vk_hash = agglayer_contracts::aggchain::AggchainVkeyHash::new(vkey_hash_bytes);

        if aggchain_vkey != proof_vk_hash {
            return Err(CertificationError::AggchainProofVkeyMismatch {
                expected: aggchain_vkey.to_hex(),
                actual: proof_vk_hash.to_hex(),
            });
        }

        let vkey_hash_u32 = sp1_fast(|| vkey.vk.hash_u32())
            .context("Failed to hash SP1 vkey")
            .map_err(CertificationError::Other)?;

        Ok(AggchainProofCtx {
            aggchain_vkey: vkey_hash_u32,
        })
    }

    pub async fn fetch_multisig_ctx(
        &self,
        certificate: &Certificate,
    ) -> Result<MultisigCtx, CertificationError> {
        let (_multisig_signers, _multisig_threshold): (Vec<Address>, usize) = {
            // TODO: To fetch from the L1: https://github.com/agglayer/agglayer/issues/941
            Default::default()
        };

        Ok(MultisigCtx {
            signers: _multisig_signers,
            threshold: _multisig_threshold,
            prehash: B256::new(
                SignatureCommitmentValues::from(certificate)
                    .commitment(SignatureCommitmentVersion::V5)
                    .0,
            ),
        })
    }

    /// Fetch, verify consistency, and wait for the finalization of the l1 info
    /// root.
    pub async fn fetch_l1_info_root(
        &self,
        certificate: &Certificate,
    ) -> Result<Digest, CertificationError> {
        let certificate_id = certificate.hash();
        let declared_l1_info_root = certificate
            .l1_info_root()
            .map_err(|source| CertificationError::Types { source })?;

        let declared_l1_info_leaf_count = certificate.l1_info_tree_leaf_count();

        let l1_info_root = match (declared_l1_info_leaf_count, declared_l1_info_root) {
            // Use the default corresponding to the entry set by the event `InitL1InfoRootMap`
            (None, _) if matches!(certificate.aggchain_data, AggchainData::Generic { .. }) => {
                return Err(CertificationError::MissingL1InfoTreeLeafCountForGenericAggchainData);
            }
            (None, None) => self.l1_rpc.default_l1_info_tree_entry().1.into(),
            // Retrieve the event corresponding to the declared entry and await for finalization
            (Some(declared_leaf), declared_root) => {
                // Retrieve from contract and await for finalization
                let retrieved_root = self
                    .l1_rpc
                    .get_l1_info_root(declared_leaf)
                    .await
                    .map_err(|_| {
                        CertificationError::L1InfoRootNotFound(certificate_id, declared_leaf)
                    })?
                    .into();

                if let Some(declared_root) = declared_root {
                    // Check that the retrieved l1 info root is equal to the declared one
                    if declared_root != retrieved_root {
                        return Err(CertificationError::Types {
                            source: agglayer_types::Error::L1InfoRootIncorrect {
                                declared: declared_root,
                                retrieved: retrieved_root,
                                leaf_count: declared_leaf,
                            },
                        });
                    }
                }

                retrieved_root
            }
            // Inconsistent declared L1 info tree entry
            (l1_leaf @ None, l1_info_root @ Some(_)) => {
                return Err(CertificationError::Types {
                    source: agglayer_types::Error::InconsistentL1InfoTreeInformation {
                        l1_leaf,
                        l1_info_root,
                    },
                })
            }
        };

        Ok(l1_info_root)
    }
}
