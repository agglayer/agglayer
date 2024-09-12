use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use agglayer_clock::Event;
use futures_util::{future::BoxFuture, Stream, StreamExt};
use pessimistic_proof::bridge_exit::NetworkId;
use pessimistic_proof::{certificate::Certificate, local_state::LocalNetworkStateData, ProofError};
use tokio::{
    sync::mpsc::Receiver,
    task::{JoinHandle, JoinSet},
};
use tokio_util::sync::{CancellationToken, WaitForCancellationFutureOwned};
use tracing::{debug, error, info};

#[cfg(test)]
mod tests;

const MAX_POLL_READS: usize = 1_000;

/// Global State composed of each network state for all networks.
/// Eventually, each state will live only in the networks themselves.
type GlobalState = BTreeMap<NetworkId, LocalNetworkStateData>;

/// The Certificate orchestrator receives the certificates from CDKs.
///
/// Each certificate reception triggers the generation of a pessimistic proof.
/// At the end of the epoch, the Certificate Orchestrator collects a set of
/// pessimistic proofs generated so far and settles them on the L1.
pub struct CertificateOrchestrator<C, E, A, I, P> {
    /// Epoch packing task resolver.
    epoch_packing_tasks: JoinSet<Result<(), Error>>,
    /// Epoch packing task builder.
    epoch_packing_task_builder: Arc<E>,
    /// Certifier task resolver.
    certifier_tasks: JoinSet<Result<CertifierOutput<P>, Error>>,
    /// Certifier task builder.
    certifier_task_builder: Arc<A>,
    /// Global network state.
    global_state: GlobalState,
    /// Clock stream to receive EpochEnded events.
    clock: C,
    /// Certificates received from each CDKs.
    received_certificates: HashMap<NetworkId, VecDeque<I>>,
    /// Set of proofs generated in the ongoing epoch.
    pending_proofs: HashMap<NetworkId, VecDeque<P>>,
    /// Proof to aggregate and settle for each epoch.
    pub(crate) to_pack: BTreeMap<u64, VecDeque<P>>,
    /// Receiver for certificates coming from CDKs.
    data_receiver: Receiver<I>,
    /// Cancellation token for graceful shutdown.
    cancellation_token: Pin<Box<WaitForCancellationFutureOwned>>,
}

impl<C, E, A, I, P> CertificateOrchestrator<C, E, A, I, P> {
    /// Creates a new CertificateOrchestrator instance.
    pub(crate) fn new(
        clock: C,
        data_receiver: Receiver<I>,
        cancellation_token: CancellationToken,
        epoch_packing_task_builder: E,
        certifier_task_builder: A,
    ) -> Self {
        Self {
            epoch_packing_tasks: JoinSet::new(),
            certifier_tasks: JoinSet::new(),
            clock,
            epoch_packing_task_builder: Arc::new(epoch_packing_task_builder),
            certifier_task_builder: Arc::new(certifier_task_builder),
            global_state: Default::default(),
            data_receiver,
            received_certificates: HashMap::new(),
            pending_proofs: HashMap::new(),
            to_pack: BTreeMap::default(),
            cancellation_token: Box::pin(cancellation_token.cancelled_owned()),
        }
    }
}

#[buildstructor::buildstructor]
impl<C, E, A, I, P> CertificateOrchestrator<C, E, A, I, P>
where
    P: Send + Unpin + 'static,
    I: Send + Unpin + 'static + CertificateInput,
    C: Stream<Item = Event> + Unpin + Send + 'static,
    A: Certifier<Input = I, Proof = P>,
    E: EpochPacker<Item = P>,
{
    /// Function that setups and starts the CertificateOrchestrator.
    ///
    /// The available methods are:
    ///
    /// - `builder`: Creates a new builder instance.
    /// - `clock`: Sets clock stream to receive EpochEnded events.
    /// - `data_receiver`: Sets the receiver for certificates coming from CDKs.
    /// - `cancellation_token`: Sets the cancellation token for graceful
    ///   shutdown.
    /// - `epoch_packing_builder`: Sets the task builder for epoch packing.
    /// - `start`: Starts the CertificateOrchestrator.
    ///
    /// # Examples
    /// ```
    /// # use agglayer_certificate_orchestrator::Error;
    /// # use agglayer_certificate_orchestrator::EpochPacker;
    /// # use agglayer_certificate_orchestrator::Certifier;
    /// # use agglayer_certificate_orchestrator::CertifierResult;
    /// # use agglayer_certificate_orchestrator::CertifierOutput;
    /// # use agglayer_certificate_orchestrator::CertificateInput;
    /// # use agglayer_certificate_orchestrator::CertificateOrchestrator;
    /// # use tokio_stream::wrappers::BroadcastStream;
    /// # use tokio_util::sync::CancellationToken;
    /// # use futures_util::future::BoxFuture;
    /// # use tokio_stream::StreamExt;
    /// # use pessimistic_proof::bridge_exit::NetworkId;
    /// # use pessimistic_proof::LocalNetworkState;
    ///
    /// # #[derive(Clone)]
    /// # pub struct Empty;
    /// # impl CertificateInput for Empty {
    /// #     fn network_id(&self) -> NetworkId {
    /// #         NetworkId::new(0)
    /// #     }
    /// # }
    ///
    /// # #[derive(Clone)]
    /// # pub struct AggregatorNotifier {}
    ///
    /// # impl AggregatorNotifier {
    /// #     pub(crate) fn new() -> Self {
    /// #         Self {}
    /// #     }
    /// # }
    ///
    /// impl EpochPacker for AggregatorNotifier {
    ///     type Item = ();
    ///     fn pack<T: IntoIterator<Item = ()>>(
    ///         &self,
    ///         epoch: u64,
    ///         to_pack: T,
    ///     ) -> Result<BoxFuture<Result<(), Error>>, Error> {
    ///         Ok(Box::pin(async move { Ok(()) }))
    ///     }
    /// }
    ///
    /// impl Certifier for AggregatorNotifier {
    ///     type Input = Empty;
    ///     type Proof = ();
    ///
    ///     fn certify(
    ///         &self,
    ///         local_state: LocalNetworkState,
    ///         certificate: Self::Input,
    ///     ) -> CertifierResult<Self::Proof> {
    ///         Ok(Box::pin(async move {
    ///             Ok(CertifierOutput {
    ///                 proof: (),
    ///                 new_state: LocalNetworkState::default(),
    ///                 network: NetworkId::new(0),
    ///             })
    ///         }))
    ///     }
    /// }
    ///
    /// async fn start() -> Result<(), ()> {
    ///     let (sender, receiver) = tokio::sync::broadcast::channel(1);
    ///     let clock_stream = BroadcastStream::new(sender.subscribe()).filter_map(|value| value.ok());
    ///     let notifier = AggregatorNotifier::new();
    ///     let data_receiver = tokio::sync::mpsc::channel(1).1;
    ///
    ///     CertificateOrchestrator::builder()
    ///         .clock(clock_stream)
    ///         .data_receiver(data_receiver)
    ///         .cancellation_token(CancellationToken::new())
    ///         .epoch_packing_task_builder(notifier.clone())
    ///         .certifier_task_builder(notifier)
    ///         .start()
    ///         .await
    ///         .unwrap();
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// This function can't fail but returns a Result for convenience and future
    ///
    /// evolution.
    #[builder(entry = "builder", exit = "start", visibility = "pub")]
    pub async fn start(
        clock: C,
        data_receiver: Receiver<I>,
        cancellation_token: CancellationToken,
        epoch_packing_task_builder: E,
        certifier_task_builder: A,
    ) -> anyhow::Result<JoinHandle<()>> {
        let orchestrator = Self::new(
            clock,
            data_receiver,
            cancellation_token,
            epoch_packing_task_builder,
            certifier_task_builder,
        );

        let handle = tokio::spawn(orchestrator);

        Ok(handle)
    }
}

impl<C, E, A, I, P> CertificateOrchestrator<C, E, A, I, P>
where
    I: Send + Unpin + 'static + CertificateInput,
    P: Send + Unpin + 'static,
    A: Certifier<Input = I, Proof = P>,
    E: EpochPacker<Item = P>,
{
    fn receive_proof(&mut self, certifier_outputs: CertifierOutput<P>) {
        let network_id = certifier_outputs.network;
        self.pending_proofs
            .entry(network_id)
            .or_default()
            .push_back(certifier_outputs.proof);

        self.global_state
            .entry(network_id)
            .and_modify(|s| *s = certifier_outputs.new_state);
    }

    fn receive_certificate(&mut self, certificate: I) -> Result<(), Error> {
        let network: NetworkId = certificate.network_id();
        let local_state = self.global_state.entry(network).or_default().clone();

        self.received_certificates
            .entry(network)
            .or_default()
            .push_back(certificate.clone());

        let task = self.certifier_task_builder.clone();
        self.certifier_tasks
            .spawn(async move { task.certify(local_state, certificate)?.await });

        Ok(())
    }
}

impl<C, E, A, I, P> Future for CertificateOrchestrator<C, E, A, I, P>
where
    P: Send + Unpin + 'static,
    I: Send + Unpin + 'static + CertificateInput,
    C: Stream<Item = Event> + Send + Unpin + 'static,
    A: Certifier<Input = I, Proof = P>,
    E: EpochPacker<Item = P>,
{
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Check if the orchestrator has been cancelled and should shutdown.
        if self.cancellation_token.as_mut().poll(cx).is_ready() {
            debug!("Certificate orchestrator cancelled by token");

            return Poll::Ready(());
        }

        // Poll the notification tasks to check for
        match self.certifier_tasks.poll_join_next(cx) {
            Poll::Ready(Some(Ok(Err(error)))) => {
                error!("Error during p-proof generation: {:?}", error)
            }
            Poll::Ready(Some(Err(error))) => {
                error!("Critical error during p-proof generation: {:?}", error);
            }
            Poll::Ready(Some(Ok(Ok(certifier_outputs)))) => {
                debug!("Received the successfully generated p-proof");
                self.receive_proof(certifier_outputs);
            }
            _ => {}
        }

        // Poll the notification tasks to check if any have errored.
        match self.epoch_packing_tasks.poll_join_next(cx) {
            Poll::Ready(Some(Ok(Err(error)))) => {
                error!("Error during epoch packing: {:?}", error)
            }
            Poll::Ready(Some(Err(error))) => {
                error!("Critical error during epoch packing: {:?}", error);
            }
            Poll::Ready(Some(Ok(Ok(())))) => {
                info!("Successfully settled the epoch");
            }
            _ => {}
        }

        if let Some((epoch, proofs)) = self.to_pack.pop_first() {
            debug!(
                "Start the settlement of {} p-proofs for the epoch {}",
                self.pending_proofs.len(),
                epoch
            );

            // Settle the p-proofs for the ended epoch
            let task = self.epoch_packing_task_builder.clone();
            self.epoch_packing_tasks
                .spawn(async move { task.pack(epoch, proofs)?.await });
        }

        let mut received = vec![];
        if let Poll::Ready(1usize..) =
            self.data_receiver
                .poll_recv_many(cx, &mut received, MAX_POLL_READS)
        {
            for certificate in received {
                if let Err(e) = self.receive_certificate(certificate) {
                    error!("Failed to handle the Certificate: {e:?}");
                }
            }

            return self.poll(cx);
        }

        if let Poll::Ready(Some(Event::EpochEnded(epoch))) = self.clock.poll_next_unpin(cx) {
            debug!("Epoch change event received: {}", epoch);

            // Gather one proof per network to settle for the current epoch
            let to_settle = self
                .pending_proofs
                .values_mut()
                .filter_map(|v| v.pop_front())
                .collect();

            self.to_pack.insert(epoch, to_settle);

            return self.poll(cx);
        }

        Poll::Pending
    }
}

/// Epoch Packer used to gather all the proofs generated on-the-go
/// and to submit them in a settlement tx to the L1.
pub trait EpochPacker: Clone + Unpin + Send + Sync + 'static {
    type Item;

    /// Pack a set of proofs for settlement on the L1
    fn pack<T: IntoIterator<Item = Self::Item>>(
        &self,
        epoch: u64,
        to_pack: T,
    ) -> Result<BoxFuture<Result<(), Error>>, Error>;
}

pub trait CertificateInput: Clone {
    fn network_id(&self) -> NetworkId;
}

impl CertificateInput for Certificate {
    fn network_id(&self) -> NetworkId {
        self.network_id
    }
}

pub struct CertifierOutput<P> {
    pub proof: P,
    pub new_state: LocalNetworkStateData,
    pub network: NetworkId,
}

pub type CertifierResult<'a, P> = Result<BoxFuture<'a, Result<CertifierOutput<P>, Error>>, Error>;

/// Apply one Certificate on top of a local state and computes one proof.
pub trait Certifier: Clone + Unpin + Send + Sync + 'static {
    type Input: CertificateInput;
    type Proof;

    fn certify(
        &self,
        full_state: LocalNetworkStateData,
        batch_header: Self::Input,
    ) -> CertifierResult<Self::Proof>;
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("proof verification failed")]
    ProofVerificationFailed,
    #[error("prover execution failed: {0}")]
    ProverExecutionFailed(#[from] anyhow::Error),
    #[error("native execution failed: {0:?}")]
    NativeExecutionFailed(#[from] ProofError),
    #[error("Type error: {0}")]
    Types(#[from] pessimistic_proof::certificate::Error),
}
