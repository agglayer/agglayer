use std::sync::Arc;

use agglayer_types::ProofVerificationError;
use futures::{future::BoxFuture, FutureExt as _};
use pessimistic_proof::{LocalNetworkState, ProofError};
use serde::Serialize;
use sp1_prover::components::DefaultProverComponents;
use sp1_sdk::{
    CpuProver, MockProver, NetworkProver, Prover as _, SP1ProvingKey, SP1Stdin, SP1VerifyingKey,
};
use tokio::task::spawn_blocking;
use tracing::error;

use super::Proof;

/// SP1 is a wrapper around the SP1 prover
pub(crate) struct SP1<P> {
    prover: Arc<P>,
    proving_key: Arc<SP1ProvingKey>,
    verifying_key: SP1VerifyingKey,
}

impl<P: sp1_sdk::Prover<DefaultProverComponents>> SP1<P> {
    pub(super) fn new(prover: P, elf: &[u8]) -> Self {
        let (proving_key, verifying_key) = prover.setup(elf);

        Self {
            prover: Arc::new(prover),
            proving_key: Arc::new(proving_key),
            verifying_key,
        }
    }
}

impl<I> super::AggregatorProver<I> for SP1<CpuProver>
where
    I: Serialize,
{
    fn prove(
        &self,
        initial_state: LocalNetworkState,
        certificate: I,
    ) -> BoxFuture<'_, Result<Proof, ProofError>> {
        let mut stdin = SP1Stdin::new();
        stdin.write(&initial_state);
        stdin.write(&certificate);

        let proving_key = self.proving_key.clone();
        let prover = self.prover.clone();

        async move {
            spawn_blocking(move || {
                prover
                    .prove(
                        &proving_key,
                        stdin,
                        Default::default(),
                        Default::default(),
                        Default::default(),
                    )
                    .map_err(|error| {
                        let error_description = error.to_string();
                        // TODO: Find a better solution than downcasting the error into a
                        // ProofError.
                        if let Ok(error) = error.downcast::<ProofError>() {
                            error
                        } else {
                            error!("Error while proving: {}", error_description);

                            ProofError::Unknown(error_description)
                        }
                    })
            })
            .await
            .map_err(|_| ProofError::Unknown("Unable to properly execute Proof task".to_string()))?
            .map(Proof::SP1)
        }
        .boxed()
    }

    fn verify(&self, proof: &Proof) -> Result<(), ProofVerificationError> {
        let Proof::SP1(proof) = proof;

        Ok(self.prover.verify(proof, &self.verifying_key)?)
    }
}

impl<I> super::AggregatorProver<I> for SP1<NetworkProver>
where
    I: Serialize,
{
    fn prove(
        &self,
        initial_state: LocalNetworkState,
        certificate: I,
    ) -> BoxFuture<'_, Result<Proof, ProofError>> {
        let mut stdin = SP1Stdin::new();
        stdin.write(&initial_state);
        stdin.write(&certificate);

        let proving_key = self.proving_key.clone();
        let prover = self.prover.clone();

        async move {
            prover
                .prove(
                    &proving_key.elf,
                    stdin,
                    Default::default(),
                    Default::default(),
                )
                .await
                .map_err(|error| {
                    let error_description = error.to_string();
                    if let Ok(error) = error.downcast::<ProofError>() {
                        error
                    } else {
                        error!("Error while proving: {}", error_description);

                        ProofError::Unknown(error_description)
                    }
                })
                .map(Proof::SP1)
        }
        .boxed()
    }

    fn verify(&self, proof: &Proof) -> Result<(), ProofVerificationError> {
        let Proof::SP1(proof) = proof;
        Ok(self.prover.verify(proof, &self.verifying_key)?)
    }
}

impl<I> super::AggregatorProver<I> for SP1<MockProver>
where
    I: Serialize,
{
    fn prove(
        &self,
        initial_state: LocalNetworkState,
        certificate: I,
    ) -> BoxFuture<'_, Result<Proof, ProofError>> {
        let mut stdin = SP1Stdin::new();
        stdin.write(&initial_state);
        stdin.write(&certificate);

        let proving_key = self.proving_key.clone();
        let prover = self.prover.clone();

        async move {
            spawn_blocking(move || {
                prover
                    .prove(
                        &proving_key,
                        stdin,
                        Default::default(),
                        Default::default(),
                        Default::default(),
                    )
                    .map_err(|error| {
                        let error_description = error.to_string();
                        if let Ok(error) = error.downcast::<ProofError>() {
                            error
                        } else {
                            error!("Error while proving: {}", error_description);

                            ProofError::Unknown(error_description)
                        }
                    })
            })
            .await
            .map_err(|_| ProofError::Unknown("Unable to properly execute Proof task".to_string()))?
            .map(Proof::SP1)
        }
        .boxed()
    }

    fn verify(&self, proof: &Proof) -> Result<(), ProofVerificationError> {
        let Proof::SP1(proof) = proof;

        Ok(self.prover.verify(proof, &self.verifying_key)?)
    }
}
