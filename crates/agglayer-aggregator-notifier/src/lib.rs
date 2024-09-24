#![cfg_attr(feature = "coverage", feature(coverage_attribute))]

use std::sync::Arc;

use agglayer_certificate_orchestrator::{Certifier, CertifierOutput, EpochPacker, Error};
use agglayer_config::certificate_orchestrator::prover::ProverConfig;
use agglayer_storage::stores::{
    PendingCertificateReader, PendingCertificateWriter, StateReader, StateWriter,
};
use agglayer_types::{Certificate, Height, LocalNetworkStateData, NetworkId, Proof};
use error::NotifierError;
use futures::future::BoxFuture;
use pessimistic_proof::{generate_pessimistic_proof, LocalNetworkState};
use reth_primitives::Address;
use serde::Serialize;
use sp1::SP1;
use sp1_sdk::{CpuProver, MockProver, NetworkProver};
use tracing::{debug, error, info};

/// ELF of the pessimistic proof program
const ELF: &[u8] =
    include_bytes!("../../pessimistic-proof-program/elf/riscv32im-succinct-zkvm-elf");

mod error;
mod proof;
mod sp1;
#[cfg(test)]
mod tests;

/// The trait that must be implemented by the prover
pub(crate) trait AggregatorProver<I>: Send + Sync {
    fn prove(
        &self,
        local_state: LocalNetworkState,
        certificate: I,
    ) -> BoxFuture<'_, Result<Proof, anyhow::Error>>;
    fn verify(&self, proof: &Proof) -> Result<(), anyhow::Error>;
}

/// The notifier that will be used to notify the aggregator
/// using a prover that implements the [`AggregatorProver`] trait
pub struct AggregatorNotifier<I, PendingStore, StateStore> {
    /// The prover that will be used to generate the proof
    prover: Arc<dyn AggregatorProver<I>>,
    pending_store: Arc<PendingStore>,
    #[allow(unused)]
    state_store: Arc<StateStore>,
}
impl<I, PendingStore, StateStore> AggregatorNotifier<I, PendingStore, StateStore>
where
    I: Serialize,
{
    /// Try to create a new notifier using the given configuration
    pub fn try_new(
        config: &ProverConfig,
        pending_store: Arc<PendingStore>,
        state_store: Arc<StateStore>,
    ) -> Result<Self, NotifierError> {
        Ok(Self {
            prover: match config {
                ProverConfig::SP1Network {} => Arc::new(SP1::new(NetworkProver::new(), ELF)),
                ProverConfig::SP1Local {} => Arc::new(SP1::new(CpuProver::new(), ELF)),
                ProverConfig::SP1Mock {} => Arc::new(SP1::new(MockProver::new(), ELF)),
            },
            pending_store,
            state_store,
        })
    }
}

impl<PendingStore, StateStore> Certifier
    for AggregatorNotifier<Certificate, PendingStore, StateStore>
where
    PendingStore: PendingCertificateReader + PendingCertificateWriter + Clone + 'static,
    StateStore: StateWriter + StateReader + 'static,
{
    fn certify(
        &self,
        mut state: LocalNetworkStateData,
        network_id: NetworkId,
        height: Height,
    ) -> Result<BoxFuture<Result<CertifierOutput, Error>>, Error> {
        // Fetch certificate from storage
        let certificate = self
            .pending_store
            .get_certificate(network_id, height)?
            .ok_or(Error::CertificateNotFound(network_id, height))?;

        let certificate_id = certificate.hash();

        if self.pending_store.get_proof(certificate_id)?.is_some() {
            return Err(Error::ProofAlreadyExists(network_id, height));
        }

        let signer = Address::new([0; 20]); // TODO: put the trusted sequencer address

        let initial_state = LocalNetworkState::from(state.clone());
        let multi_batch_header = state.apply_certificate(&certificate, signer)?;

        // Perform the native pp execution
        generate_pessimistic_proof(initial_state.clone(), &multi_batch_header)
            .map_err(Error::NativeExecutionFailed)?;

        info!(
            "Successfully executed the native pp for the Certificate {:?}",
            certificate.hash()
        );

        let proving_request = self.prover.prove(initial_state, certificate.clone()); // TODO: should be batch

        Ok(Box::pin(async move {
            let proof = proving_request
                .await
                .map_err(Error::ProverExecutionFailed)?;

            if let Err(error) = self.prover.verify(&proof) {
                error!("Failed to verify the p-proof: {:?}", error);

                Err(Error::ProofVerificationFailed)
            } else {
                info!("Successfully generated and verified the p-proof!");

                // TODO: Check if the key already exists
                self.pending_store
                    .insert_generated_proof(&certificate.hash(), &proof)?;

                Ok(CertifierOutput {
                    certificate,
                    height,
                    new_state: state,
                    network: multi_batch_header.origin_network,
                })
            }
        }))
    }
}

impl<I, PendingStore, StateStore> EpochPacker for AggregatorNotifier<I, PendingStore, StateStore>
where
    PendingStore: PendingCertificateReader + Clone + 'static,
    StateStore: StateReader + 'static,
    I: Clone + 'static,
{
    type Item = I;
    fn pack<T: IntoIterator<Item = Self::Item>>(
        &self,
        epoch: u64,
        to_pack: T,
    ) -> Result<BoxFuture<Result<(), Error>>, Error> {
        let to_pack = to_pack.into_iter().collect::<Vec<_>>();

        debug!(
            "Start the settlement for the epoch {} with {} p-proofs",
            epoch,
            to_pack.len()
        );

        Ok(Box::pin(async move {
            // TODO: Submit the settlement tx for each proof
            // No aggregation for now, we settle each PP individually
            Ok(())
        }))
    }
}
