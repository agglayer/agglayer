use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

use agglayer_config::prover::{AgglayerProverType, NetworkProverConfig, ProverConfig};
use agglayer_prover_types::Error;
use futures::{Future, TryFutureExt};
use pessimistic_proof::{
    local_exit_tree::hasher::Keccak256Hasher, multi_batch_header::MultiBatchHeader,
    LocalNetworkState,
};
use sp1_sdk::{
    network::{prover::NetworkProver, FulfillmentStrategy},
    CpuProver, Prover, ProverClient, SP1ProofWithPublicValues, SP1ProvingKey, SP1Stdin,
    SP1VerifyingKey,
};
use tokio::task::spawn_blocking;
use tower::{
    limit::ConcurrencyLimitLayer, timeout::TimeoutLayer, util::BoxCloneService, Service,
    ServiceBuilder, ServiceExt,
};
use tracing::{debug, error, info};

#[cfg(test)]
mod tests;

/// ELF of the pessimistic proof program
pub(crate) const ELF: &[u8] =
    include_bytes!("../../pessimistic-proof-program/elf/riscv32im-succinct-zkvm-elf");

#[derive(Clone)]
pub struct Executor {
    primary: BoxCloneService<Request, Response, Error>,
    fallback: Option<BoxCloneService<Request, Response, Error>>,
}

impl Executor {
    pub fn get_vkey() -> SP1VerifyingKey {
        let prover = CpuProver::new();
        let (_proving_key, verification_key) = prover.setup(ELF);

        verification_key
    }

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
                    Err(error) => Error::ProverFailed(error.to_string()),
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
                    Err(error) => Error::ProverFailed(error.to_string()),
                }),
        )
    }

    #[cfg(test)]
    pub fn new_with_services(
        primary: BoxCloneService<Request, Response, Error>,
        fallback: Option<BoxCloneService<Request, Response, Error>>,
    ) -> Self {
        Self { primary, fallback }
    }

    fn create_prover(
        prover_type: &AgglayerProverType,
    ) -> BoxCloneService<Request, Response, Error> {
        match prover_type {
            AgglayerProverType::NetworkProver(network_prover_config) => {
                debug!("Creating network prover executor...");
                let network_prover = ProverClient::builder().network().build();
                let (proving_key, verification_key) = network_prover.setup(ELF);
                Self::build_network_service(
                    network_prover_config
                        .proving_request_timeout
                        .unwrap_or_else(|| {
                            network_prover_config.proving_timeout
                                + NetworkProverConfig::DEFAULT_PROVING_TIMEOUT_PADDING
                        }),
                    NetworkExecutor {
                        prover: Arc::new(network_prover),
                        proving_key,
                        verification_key,
                        timeout: network_prover_config.proving_timeout,
                    },
                )
            }
            AgglayerProverType::CpuProver(cpu_prover_config) => {
                debug!("Creating CPU prover executor...");
                let prover = CpuProver::new();
                let (proving_key, verification_key) = prover.setup(ELF);

                Self::build_local_service(
                    cpu_prover_config.get_proving_request_timeout(),
                    cpu_prover_config.max_concurrency_limit,
                    LocalExecutor {
                        prover: Arc::new(prover),
                        proving_key,
                        verification_key,
                    },
                )
            }
            AgglayerProverType::MockProver(mock_prover_config) => {
                debug!("Creating Mock prover executor...");
                let prover = CpuProver::mock();
                let (proving_key, verification_key) = prover.setup(ELF);

                Self::build_local_service(
                    mock_prover_config.get_proving_request_timeout(),
                    mock_prover_config.max_concurrency_limit,
                    LocalExecutor {
                        prover: Arc::new(prover),
                        proving_key,
                        verification_key,
                    },
                )
            }
            AgglayerProverType::GpuProver(_) => todo!(),
        }
    }

    pub fn new(config: &ProverConfig) -> Self {
        let primary = Self::create_prover(&config.primary_prover);
        if let Some(fallback_prover) = &config.fallback_prover {
            let fallback = Some(Self::create_prover(fallback_prover));
            Self { primary, fallback }
        } else {
            Self {
                primary,
                fallback: None,
            }
        }
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

        let initial_state = pessimistic_proof::NetworkState::from(request.initial_state);

        stdin.write(&initial_state);
        stdin.write(&request.batch_header);

        stdin
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub(crate) proof: SP1ProofWithPublicValues,
}

impl Service<Request> for Executor {
    type Response = Response;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let primary = self.primary.call(req.clone());
        let fallback = self.fallback.clone();
        let fut = async move {
            match primary.await {
                Ok(res) => Ok(res),
                Err(err) => {
                    error!("Primary prover failed: {:?}", err);
                    if let Some(mut _fallback) = fallback {
                        // If fallback prover is set, try to use it
                        info!("Repeating proving request with fallback prover...");
                        _fallback.ready().await?.call(req).await
                    } else {
                        // Return primary prover error
                        Err(err)
                    }
                }
            }
        };

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

    fn call(&mut self, req: Request) -> Self::Future {
        let prover = self.prover.clone();
        let stdin = req.into();

        let proving_key = self.proving_key.clone();
        let verification_key = self.verification_key.clone();

        debug!("Proving with CPU prover");
        Box::pin(
            spawn_blocking(move || {
                debug!("Starting the proving of the requested MultiBatchHeader");
                let proof = prover
                    .prove(&proving_key, &stdin)
                    .plonk()
                    .run()
                    .map_err(|error| Error::ProverFailed(error.to_string()))?;

                debug!("Proving completed. Verifying the proof...");
                prover
                    .verify(&proof, &verification_key)
                    .map_err(|error| Error::ProofVerificationFailed(error.into()))?;

                debug!("Proof verification completed successfully");

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
    proving_key: SP1ProvingKey,
    verification_key: SP1VerifyingKey,
    timeout: Duration,
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
        let proving_key = self.proving_key.clone();
        let timeout = self.timeout;

        debug!("Proving with network prover with timeout: {:?}", timeout);
        let fut = async move {
            debug!("Starting the proving of the requested MultiBatchHeader");
            let proof = prover
                .prove(&proving_key, &stdin)
                .plonk()
                .timeout(timeout)
                .strategy(FulfillmentStrategy::Reserved)
                .run_async()
                .await
                .map_err(|error| Error::ProverFailed(error.to_string()))?;

            debug!("Proving completed. Verifying the proof...");
            prover
                .verify(&proof, &verification_key)
                .map_err(|error| Error::ProofVerificationFailed(error.into()))?;

            debug!("Proof verification completed successfully");
            Ok(Response { proof })
        };

        Box::pin(fut)
    }
}
