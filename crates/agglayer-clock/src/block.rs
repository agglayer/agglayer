use std::{
    num::NonZeroU64,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use agglayer_types::EpochNumber;
use alloy::{
    network::Ethereum,
    providers::{
        fillers::{BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller},
        Identity, Provider, ProviderBuilder, RootProvider, WsConnect,
    },
    pubsub::{ConnectionHandle, PubSubConnect, Subscription},
    rpc::{client::ClientBuilder, types::Header},
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
    RootProvider,
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
    P: Provider + 'static,
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
        // Initialize metrics for clock startup
        agglayer_telemetry::clock::record_clock_startup();

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
        connection: WsConnect,
        genesis_block: u64,
        epoch_duration: NonZeroU64,
        reconnect_attempt_timeout: Duration,
        reconnect_attempt_interval: Duration,
        total_reconnect_timeout: Duration,
    ) -> Result<Self, BlockClockError> {
        let ws = WsConnectWithRetries {
            connection,
            reconnect_attempt_timeout,
            reconnect_attempt_interval,
            total_reconnect_timeout,
        };
        info!(
            genesis_block = genesis_block,
            epoch_duration = epoch_duration.get(),
            "Creating BlockClock with WebSocket connection"
        );

        let client = ClientBuilder::default().pubsub(ws).await?;
        let provider = ProviderBuilder::new().on_client(client);

        // Mark connection as successful
        agglayer_telemetry::clock::record_connection_established();

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
    SetEpochNumber(EpochNumber, EpochNumber),
    #[error("Failed to notify the start of the Clock task")]
    UnableToNotifyStart,
    #[error("Transport initialization: {0}")]
    Transport(#[from] alloy::transports::RpcError<TransportErrorKind>),
    #[error("L1 block channel unexpectedly closed")]
    L1BlockChannelClosed,
}

impl<P> BlockClock<P>
where
    P: Provider + 'static,
{
    /// Run the Clock task.
    async fn run(
        &mut self,
        sender: broadcast::Sender<Event>,
        start_sender: oneshot::Sender<()>,
        cancellation_token: CancellationToken,
    ) -> Result<(), BlockClockError> {
        info!(
            genesis_block = self.genesis_block,
            epoch_duration = self.epoch_duration.get(),
            "Starting BlockClock task"
        );

        // Start by setting the current Block height based on the current L1 Block
        // number. If the current L1 Block number is less than the genesis block
        // number, we walk the L1 block stream until reaching the genesis block.
        self.latest_seen_block = self.provider.get_block_number().await.map_err(|e| {
            error!(error = %e, "Failed to get initial block number from L1");
            agglayer_telemetry::clock::record_connection_lost();
            BlockClockError::GetBlockNumber
        })?;

        info!(
            current_l1_block = self.latest_seen_block,
            genesis_block = self.genesis_block,
            "Retrieved current L1 block number"
        );

        let provider = self.provider.clone();

        // Subscribe to the L1 Block stream.
        let mut stream = provider.subscribe_blocks().await.map_err(|e| {
            error!(error = %e, "Failed to subscribe to L1 block stream");
            agglayer_telemetry::clock::record_connection_lost();
            BlockClockError::SubscribeBlocks
        })?;

        info!("Successfully subscribed to L1 block stream");
        agglayer_telemetry::clock::record_connection_established();

        // Wait for genesis block if needed
        while self.latest_seen_block < self.genesis_block {
            let header = Self::recv_block(&mut stream).await?;
            self.latest_seen_block = header.number;

            debug!(
                current_block = self.latest_seen_block,
                genesis_block = self.genesis_block,
                "Waiting for genesis block"
            );
        }

        info!(
            genesis_block = self.genesis_block,
            "Reached genesis L1 block, starting epoch tracking"
        );

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
                let current_epoch = self.current_epoch.load(Ordering::Acquire);
                info!(
                    initial_block_height = current_block,
                    initial_epoch = current_epoch,
                    "Initialized block clock state"
                );
                self.reinitialize_epoch_number(current_block);

                // Update initial metrics
                agglayer_telemetry::clock::CURRENT_BLOCK_HEIGHT.record(current_block, &[]);
                agglayer_telemetry::clock::CURRENT_EPOCH.record(current_epoch, &[]);
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
                block_result = Self::recv_block(&mut stream) => {
                    let block = block_result.map_err(|e| {
                        error!(error = %e, "Failed to receive block from stream");
                        agglayer_telemetry::clock::record_connection_lost();
                        e
                    })?;

                    if block.number <= self.latest_seen_block {
                        trace!(
                            block_number = block.number,
                            latest_seen = self.latest_seen_block,
                            "Skipping already processed block"
                        );
                        continue;
                    }

                    debug!(
                        block_number = block.number,
                        block_hash = %block.hash,
                        timestamp = block.timestamp,
                        blocks_to_process = block.number - self.latest_seen_block,
                        "Received new L1 block"
                    );

                    // Overwrite the block number to simulate an overflow
                    // This is used for testing purposes only and doesn't affect the production
                    // code.
                    fail::fail_point!("block_clock::BlockClock::run::overwrite_block_number_on_new_block");

                    // Process all blocks up to the received one
                    while self.latest_seen_block < block.number {
                        self.latest_seen_block += 1;
                        trace!(
                            processing_block = self.latest_seen_block,
                            target_block = block.number,
                            "Processing block"
                        );
                        self.update_and_notify(&sender)?;
                    }
                }
            }
        }

        info!("BlockClock task stopped");
        agglayer_telemetry::clock::record_clock_shutdown();

        Ok(())
    }

    async fn recv_block(stream: &mut Subscription<Header>) -> Result<Header, BlockClockError> {
        #[cfg(test)]
        {
            // The default sleep fail point directive issues a blocking sleep.
            // That does not play nice with code that is meant to be executed
            // in an async runtime. Here, we abuse the return value injection
            // to specify the timeout and use the tokio sleep instead.
            fn get_delay() -> Duration {
                fail::fail_point!("block_clock::BlockClock::recv_block::before", |d| {
                    d.map(|d| Duration::from_secs(d.parse().unwrap()))
                        .unwrap_or_default()
                });
                Duration::default()
            }
            tokio::time::sleep(get_delay()).await;
        }

        loop {
            match stream.recv().await {
                Ok(block) => break Ok(block),
                Err(broadcast::error::RecvError::Closed) => {
                    break Err(BlockClockError::L1BlockChannelClosed)
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!(
                        lagged_messages = n,
                        "Block subscription lagged behind, some blocks may have been missed"
                    );
                    agglayer_telemetry::clock::record_subscription_lag(n);
                }
            }
        }
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
            // Record block processing metrics
            agglayer_telemetry::clock::CURRENT_BLOCK_HEIGHT.record(current_block, &[]);

            // If the current Block height is a multiple of the Epoch duration, the current
            // Epoch has ended. In this case, we need to update the new Epoch number and
            // send an `EpochEnded` event to the subscribers.
            if current_block % *self.epoch_duration == 0 {
                match self.update_epoch_number(current_block) {
                    Err((previous, expected)) => {
                        return Err(BlockClockError::SetEpochNumber(previous, expected));
                    }
                    Ok(epoch_ended) => {
                        info!(
                            epoch_number = epoch_ended.as_u64(),
                            block_height = current_block,
                            epoch_duration = self.epoch_duration.get(),
                            "Epoch ended, broadcasting event"
                        );

                        // Record new current epoch (the epoch we just entered)
                        agglayer_telemetry::clock::record_current_epoch(epoch_ended.as_u64() + 1);

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
    fn update_epoch_number(
        &mut self,
        current_block: u64,
    ) -> Result<EpochNumber, (EpochNumber, EpochNumber)> {
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
            Ok(previous) => Ok(EpochNumber::new(previous)),
            Err(stored) => Err((EpochNumber::new(stored), EpochNumber::new(expected_epoch))),
        }
    }
}

struct WsConnectWithRetries {
    connection: WsConnect,
    reconnect_attempt_timeout: Duration,
    reconnect_attempt_interval: Duration,
    total_reconnect_timeout: Duration,
}

#[derive(PartialEq, Eq, Clone, Debug, thiserror::Error)]
#[error("Attempt to establish L1 connection timed out")]
struct ConnectionTimeout;

impl PubSubConnect for WsConnectWithRetries {
    fn is_local(&self) -> bool {
        self.connection.is_local()
    }

    async fn connect(&self) -> TransportResult<ConnectionHandle> {
        tokio::time::timeout(self.reconnect_attempt_timeout, self.connection.connect())
            .await
            .unwrap_or_else(|_| {
                let err = Box::new(ConnectionTimeout);
                let err = alloy::transports::TransportErrorKind::Custom(err);
                Err(err.into())
            })
            .inspect(|_| {
                info!("Successfully connected to L1 WebSocket");
                agglayer_telemetry::clock::record_connection_established();
            })
            .inspect_err(|e| {
                warn!(error = %e, "Failed to connect to L1 WebSocket");
                agglayer_telemetry::clock::record_connection_lost();
            })
    }

    async fn try_reconnect(&self) -> TransportResult<ConnectionHandle> {
        agglayer_telemetry::clock::record_reconnection_attempt();

        backoff::future::retry(
            ExponentialBackoff {
                max_interval: self.reconnect_attempt_interval,
                max_elapsed_time: Some(self.total_reconnect_timeout),
                ..Default::default()
            },
            || async {
                info!("Attempting to reconnect to L1 WebSocket");
                // This fail point is used to insert delay in the reconnection to make the block
                // progress when the client is disconnected
                fail::fail_point!("block_clock::PubSubConnect::try_reconnect::add_delay");

                Ok(self.connect().await?)
            },
        )
        .await
    }
}
