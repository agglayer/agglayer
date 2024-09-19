#![cfg_attr(feature = "coverage", feature(coverage_attribute))]

use std::sync::Arc;

use agglayer_certificate_orchestrator::{Certifier, CertifierOutput, EpochPacker, Error};
use agglayer_config::certificate_orchestrator::prover::ProverConfig;
use agglayer_types::Certificate;
use error::NotifierError;
use futures::future::BoxFuture;
use pessimistic_proof::{local_state::LocalNetworkStateData, LocalNetworkState};
use proof::Proof;
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
#[derive(Clone)]
pub struct AggregatorNotifier<I> {
    /// The prover that will be used to generate the proof
    prover: Arc<dyn AggregatorProver<I>>,
}

impl<I> TryFrom<ProverConfig> for AggregatorNotifier<I>
where
    I: Serialize,
{
    type Error = NotifierError;

    #[cfg_attr(feature = "coverage", coverage(off))]
    fn try_from(config: ProverConfig) -> Result<Self, Self::Error> {
        match config {
            ProverConfig::SP1Network {} => Ok(Self {
                prover: Arc::new(SP1::new(NetworkProver::new(), ELF)),
            }),
            ProverConfig::SP1Local {} => Ok(Self {
                prover: Arc::new(SP1::new(CpuProver::new(), ELF)),
            }),
            ProverConfig::SP1Mock {} => Ok(Self {
                prover: Arc::new(SP1::new(MockProver::new(), ELF)),
            }),
        }
    }
}

impl Certifier for AggregatorNotifier<Certificate> {
    type Input = Certificate;
    type Proof = Proof;

    fn certify(
        &self,
        full_state: LocalNetworkStateData,
        certificate: Certificate,
    ) -> Result<BoxFuture<Result<CertifierOutput<Self::Proof>, Error>>, Error> {
        let signer = Address::new([0; 20]); // TODO: put the trusted sequencer address
        let mut batch_header = certificate.into_pessimistic_proof_input(&full_state, signer)?;
        let initial_state = LocalNetworkState::from(full_state.clone());
        let target = initial_state
            .clone()
            .apply_batch_header(&batch_header)
            .map_err(Error::NativeExecutionFailed)?;

        if batch_header.target.exit_root != target.exit_root {
            // state transition mismatch between execution and received certificate
            return Err(Error::NativeExecutionFailed(
                pessimistic_proof::ProofError::InvalidFinalLocalExitRoot,
            ));
        }

        batch_header.target.balance_root = target.balance_root;
        batch_header.target.nullifier_root = target.nullifier_root;

        // TODO: implement `apply_batch_header` for LocalNetworkStateData
        let new_state = full_state.clone();

        let proving_request = self
            .prover
            .prove(initial_state.clone(), certificate.clone()); // TODO: should be batch

        Ok(Box::pin(async move {
            let proof = proving_request
                .await
                .map_err(Error::ProverExecutionFailed)?;

            if let Err(error) = self.prover.verify(&proof) {
                error!("Failed to verify the p-proof: {:?}", error);

                Err(Error::ProofVerificationFailed)
            } else {
                info!("Successfully generated and verified the p-proof!");
                Ok(CertifierOutput {
                    proof,
                    new_state,
                    network: batch_header.origin_network,
                })
            }
        }))
    }
}

impl<I> EpochPacker for AggregatorNotifier<I>
where
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
