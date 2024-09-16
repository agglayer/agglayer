use std::{marker::PhantomData, task::Poll};

use futures_util::{future::BoxFuture, poll};
use pessimistic_proof::{bridge_exit::NetworkId, local_state::LocalNetworkStateData};
use tokio::sync::{broadcast, mpsc};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tokio_util::sync::CancellationToken;

use crate::{
    CertificateInput, CertificateOrchestrator, Certifier, CertifierOutput, CertifierResult,
    EpochPacker, Error,
};

// CertificateOrchestrator can be stopped
#[tokio::test]
async fn test_certificate_orchestrator_can_stop() {
    let (_clock_sender, receiver) = broadcast::channel(1);
    let clock = BroadcastStream::new(receiver).filter_map(|value| value.ok());

    let (_data_sender, data_receiver) = mpsc::channel(10);
    let cancellation_token = CancellationToken::new();

    let (check_sender, mut check_receiver) = mpsc::channel(1);
    let check = Check::<()>::builder().executed(check_sender).build();

    let mut orchestrator = CertificateOrchestrator::new(
        clock,
        data_receiver,
        cancellation_token.clone(),
        check.clone(),
        check.clone(),
    );

    cancellation_token.cancel();

    assert!(matches!(poll!(&mut orchestrator), Poll::Ready(())));

    assert!(orchestrator.to_pack.is_empty());
    assert!(check_receiver.try_recv().is_err());
}

// Can collect certificates and pack them at the end of an epoch
#[tokio::test]
async fn test_collect_certificates() {
    let (clock_sender, receiver) = broadcast::channel(1);
    let clock = BroadcastStream::new(receiver).filter_map(|value| value.ok());
    let (data_sender, data_receiver) = mpsc::channel(10);
    let cancellation_token = CancellationToken::new();

    let (check_sender, mut check_receiver) = mpsc::channel(1);
    let check = Check::builder()
        .executed(check_sender)
        .expected_epoch(1)
        .expected_certificates_len(1)
        .build();

    let mut orchestrator = CertificateOrchestrator::new(
        clock,
        data_receiver,
        cancellation_token,
        check.clone(),
        check.clone(),
    );

    _ = data_sender.send(()).await;
    _ = clock_sender.send(agglayer_clock::Event::EpochEnded(1));

    let _poll = poll!(&mut orchestrator);

    assert!(orchestrator.to_pack.is_empty());
    assert!(check_receiver.recv().await.is_some());
}

// A certificate received after an EpochEnded is stored for next epoch
#[tokio::test]
async fn test_collect_certificates_after_epoch() {
    let (clock_sender, receiver) = broadcast::channel(1);
    let clock = BroadcastStream::new(receiver).filter_map(|value| value.ok());
    let (data_sender, data_receiver) = mpsc::channel(10);
    let cancellation_token = CancellationToken::new();

    let (check_sender, mut check_receiver) = mpsc::channel(1);
    let check = Check::builder()
        .executed(check_sender)
        .expected_epoch(1)
        .expected_certificates_len(0)
        .build();

    let mut orchestrator = CertificateOrchestrator::new(
        clock,
        data_receiver,
        cancellation_token,
        check.clone(),
        check.clone(),
    );

    _ = clock_sender.send(agglayer_clock::Event::EpochEnded(1));
    let _poll = poll!(&mut orchestrator);

    _ = data_sender.send(()).await;

    let _poll = poll!(&mut orchestrator);

    assert!(!orchestrator.received_certificates.is_empty());
    assert!(check_receiver.recv().await.is_some());
}

// If no certificate is received, the orchestrator should send an empty payload
#[tokio::test]
async fn test_collect_certificates_when_empty() {
    let (clock_sender, receiver) = broadcast::channel(1);
    let clock = BroadcastStream::new(receiver).filter_map(|value| value.ok());
    let (_data_sender, data_receiver) = mpsc::channel(10);
    let cancellation_token = CancellationToken::new();

    let (check_sender, mut check_receiver) = mpsc::channel(1);
    let check = Check::<()>::builder()
        .executed(check_sender)
        .expected_epoch(1)
        .expected_certificates_len(0)
        .build();

    let mut orchestrator = CertificateOrchestrator::new(
        clock,
        data_receiver,
        cancellation_token,
        check.clone(),
        check.clone(),
    );

    _ = clock_sender.send(agglayer_clock::Event::EpochEnded(1));
    let _poll = poll!(&mut orchestrator);

    assert!(orchestrator.received_certificates.is_empty());
    assert!(check_receiver.recv().await.is_some());
}

#[derive(Clone)]
struct Check<I> {
    executed: mpsc::Sender<()>,
    expected_epoch: Option<u64>,
    expected_certificates_len: Option<usize>,
    _phantom_data: std::marker::PhantomData<I>,
}

#[buildstructor::buildstructor]
impl<I> Check<I> {
    #[builder]
    fn new(
        executed: mpsc::Sender<()>,
        expected_epoch: Option<u64>,
        expected_certificates_len: Option<usize>,
    ) -> Self {
        Self {
            executed,
            expected_epoch,
            expected_certificates_len,
            _phantom_data: PhantomData,
        }
    }
}

impl<I> EpochPacker for Check<I>
where
    I: Send + Sync + Unpin + Clone + 'static,
{
    type Item = I;
    fn pack<T>(&self, epoch: u64, to_pack: T) -> Result<BoxFuture<Result<(), Error>>, Error>
    where
        T: IntoIterator<Item = Self::Item>,
    {
        if let Some(expected_epoch) = self.expected_epoch {
            assert_eq!(epoch, expected_epoch);
        }
        if let Some(expected_certificates_len) = self.expected_certificates_len {
            assert!(to_pack.into_iter().count() == expected_certificates_len);
        }

        _ = self.executed.try_send(());

        Ok(Box::pin(async { Ok(()) }))
    }
}

impl CertificateInput for () {
    fn network_id(&self) -> NetworkId {
        0.into()
    }
}

impl<I> Certifier for Check<I>
where
    I: Send + Sync + Unpin + Clone + 'static + CertificateInput,
{
    type Proof = ();
    type Input = I;

    fn certify(
        &self,
        local_state: LocalNetworkStateData,
        certificate: I,
    ) -> CertifierResult<Self::Proof> {
        // TODO: check whether the initial state is the expected one
        _ = self.executed.try_send(());
        Ok(Box::pin(async move {
            Ok(CertifierOutput {
                proof: (),
                new_state: local_state,
                network: certificate.network_id(),
            })
        }))
    }
}
