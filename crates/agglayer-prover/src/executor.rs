use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

use agglayer_config::prover::ProverConfig;
use futures::{Future, TryFutureExt};
use pessimistic_proof::{
    local_exit_tree::hasher::Keccak256Hasher, multi_batch_header::MultiBatchHeader,
    LocalNetworkState,
};
use sp1_sdk::{
    provers::ProofOpts, CpuProver, NetworkProver, Prover, SP1Context, SP1ProofKind,
    SP1ProofWithPublicValues, SP1ProvingKey, SP1Stdin, SP1VerifyingKey,
};
use tokio::task::spawn_blocking;
use tower::{
    limit::ConcurrencyLimitLayer, timeout::TimeoutLayer, util::BoxCloneService, Service,
    ServiceBuilder, ServiceExt,
};
use tracing::error;

#[cfg(test)]
mod tests;

/// ELF of the pessimistic proof program
const ELF: &[u8] =
    include_bytes!("../../pessimistic-proof-program/elf/riscv32im-succinct-zkvm-elf");

#[derive(Clone)]
pub struct Executor {
    network: BoxCloneService<Request, Response, Error>,
    local: BoxCloneService<Request, Response, Error>,
}

impl Executor {
    pub fn build_network_service<S>(
        timeout: Duration,
        service: S,
    ) -> BoxCloneService<Request, Response, Error>
    where
        S: Service<Request, Response = Response, Error = Error> + Send + Clone + 'static,
        <S as Service<Request>>::Future: std::marker::Send,
    {
        BoxCloneService::new(
            ServiceBuilder::new()
                .layer(TimeoutLayer::new(timeout))
                .service(service)
                .map_err(|error| match error.downcast::<Error>() {
                    Ok(error) => *error,
                    Err(error) => Error::ProverFailed(anyhow::Error::msg(error.to_string())),
                }),
        )
    }

    pub fn build_local_service<S>(
        timeout: Duration,
        concurrency: usize,
        service: S,
    ) -> BoxCloneService<Request, Response, Error>
    where
        S: Service<Request, Response = Response, Error = Error> + Send + Clone + 'static,
        <S as Service<Request>>::Future: std::marker::Send,
    {
        BoxCloneService::new(
            ServiceBuilder::new()
                .layer(TimeoutLayer::new(timeout))
                .layer(ConcurrencyLimitLayer::new(concurrency))
                .service(service)
                .map_err(|error| match error.downcast::<Error>() {
                    Ok(error) => *error,
                    Err(error) => Error::ProverFailed(anyhow::Error::msg(error.to_string())),
                }),
        )
    }

    #[cfg(test)]
    pub fn new_with_services(
        network: BoxCloneService<Request, Response, Error>,
        local: BoxCloneService<Request, Response, Error>,
    ) -> Self {
        Self { network, local }
    }

    pub fn new(config: &ProverConfig) -> Self {
        let network_prover = NetworkProver::new();
        let (_proving_key, verification_key) = network_prover.setup(ELF);
        let network = Self::build_network_service(
            config.network_prover.proving_timeout,
            NetworkExecutor {
                prover: Arc::new(network_prover),
                verification_key,
            },
        );

        let prover = CpuProver::new();
        let (proving_key, verification_key) = prover.setup(ELF);

        let local = Self::build_local_service(
            config.cpu_prover.proving_timeout,
            config.cpu_prover.max_concurrency_limit,
            LocalExecutor {
                prover: Arc::new(prover),
                proving_key,
                verification_key,
            },
        );

        Self { network, local }
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    pub(crate) initial_state: LocalNetworkState,
    pub(crate) batch_header: MultiBatchHeader<Keccak256Hasher>,
}

impl From<Request> for SP1Stdin {
    fn from(request: Request) -> Self {
        let mut stdin = SP1Stdin::new();

        stdin.write(&request.initial_state);
        stdin.write(&request.batch_header);

        stdin
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub(crate) proof: SP1ProofWithPublicValues,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to execute prover")]
    UnableToExecuteProver,

    #[error("Prover failed to generate proof: {0}")]
    ProverFailed(#[from] anyhow::Error),

    #[error("Proof verification failed: {0}")]
    SP1VerificationFailed(#[from] sp1_sdk::SP1VerificationError),
}

impl Service<Request> for Executor {
    type Response = Response;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let network = self.network.call(req.clone());
        let mut local = self.local.clone();

        let fut = async move {
            match network.await {
                Ok(res) => Ok(res),
                Err(err) => {
                    error!("Network prover failed: {:?}", err);

                    local.ready().await?.call(req).await
                }
            }
        };

        // Return the response as an immediate future
        Box::pin(fut)
    }
}

#[derive(Clone)]
struct LocalExecutor {
    proving_key: SP1ProvingKey,
    verification_key: SP1VerifyingKey,
    prover: Arc<CpuProver>,
}

impl Service<Request> for LocalExecutor {
    type Response = Response;

    type Error = Error;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: Request) -> Self::Future {
        let prover = self.prover.clone();
        let stdin = SP1Stdin::new();
        let opts = ProofOpts::default();
        let kind = SP1ProofKind::default();

        let proving_key = self.proving_key.clone();
        let verification_key = self.verification_key.clone();

        Box::pin(
            spawn_blocking(move || {
                let context = SP1Context::default();
                let proof = prover
                    .prove(&proving_key, stdin, opts, context, kind)
                    .map_err(Error::ProverFailed)?;

                prover.verify(&proof, &verification_key)?;

                Ok(Response { proof })
            })
            .map_err(|_| Error::UnableToExecuteProver)
            .and_then(|res| async { res }),
        )
    }
}

#[derive(Clone)]
struct NetworkExecutor {
    prover: Arc<NetworkProver>,
    verification_key: SP1VerifyingKey,
}

impl Service<Request> for NetworkExecutor {
    type Response = Response;

    type Error = Error;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let prover = self.prover.clone();
        let stdin = req.into();

        let verification_key = self.verification_key.clone();

        let fut = async move {
            let proof = prover
                .prove(
                    ELF,
                    stdin,
                    sp1_sdk::proto::network::ProofMode::default(),
                    None,
                )
                .await
                .map_err(Error::ProverFailed)?;

            prover.verify(&proof, &verification_key)?;

            Ok(Response { proof })
        };

        Box::pin(fut)
    }
}
