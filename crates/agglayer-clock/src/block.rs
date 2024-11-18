use std::{
    num::NonZeroU64,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use alloy::{
    network::Ethereum,
    providers::{
        fillers::{BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller},
        Identity, Provider, ProviderBuilder, RootProvider, WsConnect,
    },
    pubsub::{ConnectionHandle, PubSubConnect, PubSubFrontend},
    rpc::client::ClientBuilder,
    transports::{impl_future, TransportErrorKind, TransportResult},
};
use backoff::ExponentialBackoff;
use tokio::sync::{broadcast, oneshot};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, trace, warn};

use crate::{Clock, ClockRef, Error, Event, BROADCAST_CHANNEL_SIZE};

#[cfg(test)]
mod tests;

type BlockProvider = FillProvider<
    JoinFill<
        Identity,
        JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
    >,
    RootProvider<PubSubFrontend>,
    PubSubFrontend,
    Ethereum,
>;

/// Block based [`Clock`] implementation.
pub struct BlockClock<P> {
    /// The L1 Middleware provider.
    provider: Arc<P>,
    /// The genesis block number from the L1 chain to calculate the current
    /// Block height.
    genesis_block: u64,
    /// The local Block height.
    block_height: Arc<AtomicU64>,
    /// The Epoch duration in Blocks.
    epoch_duration: Arc<NonZeroU64>,
    /// The current local Epoch number.
    current_epoch: Arc<AtomicU64>,
    /// The last seen block number.
    latest_seen_block: u64,
}

#[async_trait::async_trait]
impl<P> Clock for BlockClock<P>
where
    P: Provider<PubSubFrontend> + 'static,
{
    async fn spawn(mut self, cancellation_token: CancellationToken) -> Result<ClockRef, Error> {
        let (sender, _receiver) = broadcast::channel(BROADCAST_CHANNEL_SIZE);

        let clock_ref = ClockRef {
            sender: sender.clone(),
            block_height: self.block_height.clone(),
            block_per_epoch: self.epoch_duration.clone(),
        };

        let (start_sender, start_receiver) = oneshot::channel();
        // Spawn the Clock task directly
        tokio::spawn(async move {
            if let Err(error) = self
                .run(sender, start_sender, cancellation_token.clone())
                .await
            {
                error!("Block clock error: {}", error);
                cancellation_token.cancel();
            }
        });

        _ = start_receiver.await.map_err(|_| Error::UnableToStart)?;

        Ok(clock_ref)
    }
}

impl<P> BlockClock<P> {
    /// Create a new [`BlockClock`] instance based on a genesis block number and
    /// an Epoch duration.
    pub fn new(provider: P, genesis_block: u64, epoch_duration: NonZeroU64) -> Self {
        Self {
            provider: Arc::new(provider),
            genesis_block,
            block_height: Arc::new(AtomicU64::new(0)),
            epoch_duration: Arc::new(epoch_duration),
            current_epoch: Arc::new(AtomicU64::new(0)),
            latest_seen_block: 0,
        }
    }

    /// Calculate a Block number based on an L1 Block number.
    fn calculate_block_number(&self, from_block: u64) -> u64 {
        from_block.saturating_sub(self.genesis_block)
    }
}

impl BlockClock<BlockProvider> {
    pub async fn new_with_ws(
        ws: WsConnect,
        genesis_block: u64,
        epoch_duration: NonZeroU64,
        max_reconnection_elapsed_time: Duration,
    ) -> Result<Self, BlockClockError> {
        let ws = WsConnectWithRetries(ws, Some(max_reconnection_elapsed_time));
        let client = ClientBuilder::default().pubsub(ws).await?;
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .on_client(client);

        Ok(Self::new(provider, genesis_block, epoch_duration))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BlockClockError {
    #[error("Failed to get the current L1 Block number")]
    GetBlockNumber,
    #[error("Failed to subscribe to the L1 Block stream")]
    SubscribeBlocks,
    #[error("The current block height was already set to a non-zero value: {0}")]
    BlockHeightAlreadySet(u64),
    #[error("Failed to set the current block height, already set to: {0}")]
    SetBlockHeight(u64),
    #[error("Failed to set the current Epoch number: previous={0}, expected={1}")]
    SetEpochNumber(u64, u64),
    #[error("Failed to notify the start of the Clock task")]
    UnableToNotifyStart,
    #[error("Transport initialization: {0}")]
    Transport(#[from] alloy::transports::RpcError<TransportErrorKind>),
    #[error("Transport error: {0}")]
    TransportError(#[from] tokio::sync::broadcast::error::RecvError),
}

impl<P> BlockClock<P>
where
    P: Provider<PubSubFrontend> + 'static,
{
    /// Run the Clock task.
    async fn run(
        &mut self,
        sender: broadcast::Sender<Event>,
        start_sender: oneshot::Sender<()>,
        cancellation_token: CancellationToken,
    ) -> Result<(), BlockClockError> {
        info!("Starting the BlockClock task");
        // Start by setting the current Block height based on the current L1 Block
        // number. If the current L1 Block number is less than the genesis block
        // number, we walk the L1 block stream until reaching the genesis block.
        self.latest_seen_block = self
            .provider
            .get_block_number()
            .await
            .map_err(|_| BlockClockError::GetBlockNumber)?;

        debug!("Current L1 Block number: {}", self.latest_seen_block);
        let provider = self.provider.clone();

        // Subscribe to the L1 Block stream.
        let mut stream = provider
            .subscribe_blocks()
            .await
            .map_err(|_| BlockClockError::SubscribeBlocks)?;

        debug!("Successfully subscribed to the L1 Block stream");

        while self.latest_seen_block < self.genesis_block {
            let header = stream.recv().await?;
            self.latest_seen_block = header.number;

            debug!("Current L1 Block number: {}", self.latest_seen_block);
        }

        info!("Node reached the genesis L1 block {}", self.genesis_block);

        // Calculate the local Block height based on the current L1 Block number.
        let current_block = self.calculate_block_number(self.latest_seen_block);

        // Overwrite the block number to simulate an overflow
        // This is used for testing purposes only and doesn't affect the production
        // code.
        fail::fail_point!("block_clock::BlockClock::run::overwrite_block_number");

        match self.block_height.compare_exchange(
            0,
            current_block,
            Ordering::Acquire,
            Ordering::Relaxed,
        ) {
            Ok(0) => {
                debug!("The current block height was set to: {}", current_block);
                self.reinitialize_epoch_number(current_block);
            }
            Ok(block) => {
                return Err(BlockClockError::BlockHeightAlreadySet(block));
            }
            Err(block) => {
                return Err(BlockClockError::SetBlockHeight(block));
            }
        }

        start_sender
            .send(())
            .map_err(|_| BlockClockError::UnableToNotifyStart)?;

        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    warn!("Clock task cancelled");
                    break;
                }
                block_result = stream.recv() => {
                    let block = block_result?;
                    if block.number <= self.latest_seen_block {
                        trace!("Skipping block: number={}, latest_seen_block={}", block.number, self.latest_seen_block);
                        continue;
                    }
                    trace!(
                        "L1 Block received: timestamp={}, number={}, hash={}",
                        block.timestamp,
                        block.number,
                        block.hash
                    );

                    // Overwrite the block number to simulate an overflow
                    // This is used for testing purposes only and doesn't affect the production
                    // code.
                    fail::fail_point!("block_clock::BlockClock::run::overwrite_block_number_on_new_block");

                    // looping until we catch up
                    while self.latest_seen_block < block.number {
                        self.latest_seen_block += 1;
                        trace!("Updated the latest_seen_block: latest_seen_block={}, latest_received_block={}", self.latest_seen_block, block.number);
                        self.update_and_notify(&sender)?;
                    }
                }
            }
        }

        debug!("Clock task stopped");

        Ok(())
    }

    fn update_and_notify(
        &mut self,
        sender: &broadcast::Sender<Event>,
    ) -> Result<(), BlockClockError> {
        // Increase the Block height by 1. The `fetch_add` method returns the previous
        // value, so we need to add 1 to it to get the current Block height.
        if let Some(current_block) = self
            .block_height
            .fetch_add(1, Ordering::Release)
            .checked_add(1)
        {
            // If the current Block height is a multiple of the Epoch duration, the current
            // Epoch has ended. In this case, we need to update the new Epoch number and
            // send an `EpochEnded` event to the subscribers.
            if current_block % *self.epoch_duration == 0 {
                match self.update_epoch_number(current_block) {
                    Err((previous, expected)) => {
                        return Err(BlockClockError::SetEpochNumber(previous, expected));
                    }
                    Ok(epoch_ended) => {
                        info!("Clock detected the end of the Epoch: {}", epoch_ended);
                        _ = sender.send(Event::EpochEnded(epoch_ended));
                    }
                }
            }
        }

        Ok(())
    }

    /// Reinitialize the current Epoch number based on the current Block height.
    fn reinitialize_epoch_number(&mut self, current_block: u64) {
        let current_epoch =
            <Self as Clock>::calculate_epoch_number(current_block, *self.epoch_duration);
        self.current_epoch.store(current_epoch, Ordering::SeqCst);
    }

    /// Updates the current Epoch of this [`BlockClock`].
    ///
    /// This method is used to update the current Epoch number based on the
    /// Block height and the Epoch duration.
    ///
    /// To define the current Epoch number, the Epoch duration divides the Block
    /// height.
    fn update_epoch_number(&mut self, current_block: u64) -> Result<u64, (u64, u64)> {
        let current_epoch = Self::calculate_epoch_number(current_block, *self.epoch_duration);
        let expected_epoch = current_epoch.saturating_sub(1);

        // Overwrite the current_epoch to simulate an overflow
        // This is used for testing purposes only and doesn't affect the production
        // code.
        fail::fail_point!("block_clock::BlockClock::update_epoch_number::overwrite_epoch");

        match self.current_epoch.compare_exchange(
            expected_epoch,
            current_epoch,
            Ordering::Acquire,
            Ordering::Relaxed,
        ) {
            Ok(previous) => Ok(previous),
            Err(stored) => Err((stored, expected_epoch)),
        }
    }
}

struct WsConnectWithRetries(WsConnect, Option<Duration>);

impl PubSubConnect for WsConnectWithRetries {
    fn is_local(&self) -> bool {
        self.0.is_local()
    }

    fn connect(&self) -> impl_future!(<Output = TransportResult<ConnectionHandle>>) {
        self.0.connect()
    }

    async fn try_reconnect(&self) -> TransportResult<ConnectionHandle> {
        backoff::future::retry(
            ExponentialBackoff {
                max_elapsed_time: self.1,
                ..Default::default()
            },
            || async {
                debug!("Trying to reconnect to the L1 node");
                // This fail point is used to insert delay in the reconnection to make the block
                // progress when the client is disconnected
                fail::fail_point!("block_clock::PubSubConnect::try_reconnect::add_delay");

                Ok(self.0.try_reconnect().await?)
            },
        )
        .await
    }
}
