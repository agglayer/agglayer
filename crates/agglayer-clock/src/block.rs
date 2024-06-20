use std::{
    num::NonZeroU64,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use ethers::providers::{Middleware, PubsubClient};
use futures::StreamExt as _;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};

use crate::{Clock, ClockRef, Error, Event, BROADCAST_CHANNEL_SIZE};

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
    epoch_duration: NonZeroU64,
    /// The current local Epoch number.
    current_epoch: Arc<AtomicU64>,
}

#[async_trait::async_trait]
impl<P> Clock for BlockClock<P>
where
    P: Middleware + 'static,
    <P as Middleware>::Provider: PubsubClient,
{
    async fn spawn(mut self, cancellation_token: CancellationToken) -> Result<ClockRef, Error> {
        let (sender, _receiver) = broadcast::channel(BROADCAST_CHANNEL_SIZE);

        let clock_ref = ClockRef {
            sender: sender.clone(),
            current_epoch: self.current_epoch.clone(),
            block_height: self.block_height.clone(),
        };

        // Spawn the Clock task directly
        tokio::spawn(async move {
            if let Err(error) = self.run(sender, cancellation_token.clone()).await {
                error!("{}", error);
                cancellation_token.cancel();
            }
        });

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
            epoch_duration,
            current_epoch: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Updates the current Epoch of this [`TimeClock`].
    ///
    /// This method is used to update the current Epoch number based on the
    /// Block height and the Epoch duration.
    ///
    /// To define the current Epoch number, the Epoch duration divides the Block
    /// height.
    fn update_epoch_number(&mut self, current_block: u64) -> Result<u64, (u64, u64)> {
        let current_epoch = self.calculate_epoch_number(current_block);
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

    /// Calculate an Epoch number based on a Block number.
    fn calculate_epoch_number(&self, from_block: u64) -> u64 {
        from_block / self.epoch_duration
    }

    /// Calculate a Block number based on an L1 Block number.
    fn calculate_block_number(&self, from_block: u64) -> u64 {
        from_block.saturating_sub(self.genesis_block)
    }
}

#[derive(Debug, thiserror::Error)]
enum BlockClockError {
    #[error("Failed to get the current L1 Block number")]
    GetBlockNumber,
    #[error("Failed to subscribe to the L1 Block stream")]
    SubscribeBlocks,
    #[error("The current block height is less than the genesis block number: {0} < {1}")]
    GenesisBlockNumber(u64, u64),
    #[error("The current block height was already set to a non-zero value: {0}")]
    BlockHeightAlreadySet(u64),
    #[error("Failed to set the current block height, already set to: {0}")]
    SetBlockHeight(u64),
    #[error("Failed to set the current Epoch number: previous={0}, expected={1}")]
    SetEpochNumber(u64, u64),
}

impl<P> BlockClock<P>
where
    P: Middleware,
    <P as Middleware>::Provider: PubsubClient,
{
    /// Run the Clock task.
    async fn run(
        &mut self,
        sender: broadcast::Sender<Event>,
        cancellation_token: CancellationToken,
    ) -> Result<(), BlockClockError> {
        // Start by setting the current Block height based on the current L1 Block
        // number. If the current L1 Block number is less than the genesis block
        // number, we exit the process with an println message.
        let current_l1_block = self
            .provider
            .get_block_number()
            .await
            .map_err(|_| BlockClockError::GetBlockNumber)?;

        let provider = self.provider.clone();

        // Subscribe to the L1 Block stream.
        let mut stream = provider
            .subscribe_blocks()
            .await
            .map_err(|_| BlockClockError::SubscribeBlocks)?;

        let current_l1_block = current_l1_block.as_u64();
        if current_l1_block < self.genesis_block {
            return Err(BlockClockError::GenesisBlockNumber(
                current_l1_block,
                self.genesis_block,
            ));
        };

        // Overwrite the block number to simulate an overflow
        // This is used for testing purposes only and doesn't affect the production
        // code.
        fail::fail_point!("block_clock::BlockClock::run::overwrite_block_number");

        // Calculate the local Block height based on the current L1 Block number.
        let current_block = self.calculate_block_number(current_l1_block);

        match self.block_height.compare_exchange(
            0,
            current_block,
            Ordering::Acquire,
            Ordering::Relaxed,
        ) {
            Ok(0) => {
                debug!("The current block height was set to: {}", current_block);
                self.update_epoch_number(current_block)
                    .map_err(|(previous, expected)| {
                        BlockClockError::SetEpochNumber(previous, expected)
                    })?;
            }
            Ok(block) => {
                return Err(BlockClockError::BlockHeightAlreadySet(block));
            }
            Err(block) => {
                return Err(BlockClockError::SetBlockHeight(block));
            }
        }

        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    debug!("Clock task cancelled");
                    break;
                }
                Some(block) = stream.next() => {
                    debug!(
                        "L1 Block received: timestamp={}, number={}, hash={}",
                        block.timestamp,
                        block.number.unwrap(),
                        block.hash.unwrap()
                    );

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
                        if current_block % self.epoch_duration == 0 {
                            match self.update_epoch_number(current_block) {
                                Err((previous, expected)) => {
                                    return Err(BlockClockError::SetEpochNumber(previous, expected));
                                }
                                Ok(epoch_ended) => {
                                    _ = sender.send(Event::EpochEnded(epoch_ended));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{num::NonZeroU64, time::Duration};

    use ethers::{
        providers::{Provider, Ws},
        utils::Anvil,
    };
    use fail::FailScenario;
    use tokio::sync::broadcast;
    use tokio_util::sync::CancellationToken;

    use crate::{block::BlockClockError, BlockClock, Clock, Event, BROADCAST_CHANNEL_SIZE};

    #[test]
    fn test_block_calculation() {
        assert_eq!(
            0,
            BlockClock::new((), 0, NonZeroU64::new(3).unwrap()).calculate_block_number(0)
        );
        assert_eq!(
            2,
            BlockClock::new((), 0, NonZeroU64::new(3).unwrap()).calculate_block_number(2)
        );
        assert_eq!(
            1,
            BlockClock::new((), 1, NonZeroU64::new(3).unwrap()).calculate_block_number(2)
        );
        assert_eq!(
            0,
            BlockClock::new((), 2, NonZeroU64::new(3).unwrap()).calculate_block_number(2)
        );
    }

    #[tokio::test]
    async fn test_block_clock() {
        let anvil = Anvil::new().block_time(1u64).spawn();
        let client = Provider::<Ws>::connect(anvil.ws_endpoint()).await.unwrap();

        let clock = BlockClock::new(client, 0, NonZeroU64::new(3).unwrap());
        let token = CancellationToken::new();
        let clock_ref = clock.spawn(token).await.unwrap();
        assert_eq!(clock_ref.current_epoch(), 0);

        let mut recv = clock_ref.subscribe().unwrap();

        assert_eq!(recv.recv().await, Ok(Event::EpochEnded(0)));
        assert_eq!(clock_ref.current_epoch(), 1);
        assert!(clock_ref.current_block_height() >= 3);
    }

    #[tokio::test]
    async fn test_block_clock_with_genesis() {
        let anvil = Anvil::new().block_time(1u64).spawn();
        let client = Provider::<Ws>::connect(anvil.ws_endpoint()).await.unwrap();

        tokio::time::sleep(Duration::from_secs(3)).await;

        let clock = BlockClock::new(client, 2, NonZeroU64::new(3).unwrap());
        let token = CancellationToken::new();
        let clock_ref = clock.spawn(token).await.unwrap();
        assert_eq!(clock_ref.current_epoch(), 0);

        let mut recv = clock_ref.subscribe().unwrap();

        assert_eq!(recv.recv().await, Ok(Event::EpochEnded(0)));
        assert_eq!(clock_ref.current_epoch(), 1);
        assert!(clock_ref.current_block_height() >= 3);
    }

    #[tokio::test]
    async fn test_block_clock_starting_with_wrong_genesis() {
        let anvil = Anvil::new().block_time(1u64).spawn();
        let client = Provider::<Ws>::connect(anvil.ws_endpoint()).await.unwrap();

        tokio::time::sleep(Duration::from_secs(1)).await;
        let mut clock = BlockClock::new(client, 2, NonZeroU64::new(3).unwrap());
        let (sender, _receiver) = broadcast::channel(BROADCAST_CHANNEL_SIZE);

        let token = CancellationToken::new();
        let handle = tokio::spawn(async move { clock.run(sender, token).await });

        let res = tokio::time::timeout(Duration::from_millis(100), handle)
            .await
            .expect("Timeout waiting for task to finish")
            .expect("Task Join error");

        assert!(matches!(
            res,
            Err(BlockClockError::GenesisBlockNumber(1, 2))
        ));
    }

    #[tokio::test]
    async fn test_block_clock_overflow() {
        let scenario = FailScenario::setup();
        let anvil = Anvil::new().block_time(1u64).spawn();
        let client = Provider::<Ws>::connect(anvil.ws_endpoint()).await.unwrap();

        let mut clock = BlockClock::new(client, 0, NonZeroU64::new(3).unwrap());
        let blocks = clock.block_height.clone();
        let (sender, _receiver) = broadcast::channel(BROADCAST_CHANNEL_SIZE);

        let token = CancellationToken::new();

        fail::cfg_callback(
            "block_clock::BlockClock::run::overwrite_block_number",
            move || {
                // Overflow the block height on next poll
                blocks.store(u64::MAX - 1, std::sync::atomic::Ordering::SeqCst);
            },
        )
        .unwrap();

        let handle = tokio::spawn(async move { clock.run(sender, token).await });

        let res = tokio::time::timeout(Duration::from_secs(10), handle)
            .await
            .expect("Timeout waiting for task to finish")
            .expect("Task Join error");

        assert!(matches!(
            res,
            Err(BlockClockError::SetBlockHeight(height)) if height == u64::MAX - 1
        ));
        scenario.teardown();
    }

    #[tokio::test]
    async fn test_block_clock_overflow_epoch() {
        let scenario = FailScenario::setup();
        let anvil = Anvil::new().block_time(1u64).spawn();

        let client = Provider::<Ws>::connect(anvil.ws_endpoint()).await.unwrap();

        let mut clock = BlockClock::new(client, 0, NonZeroU64::new(3).unwrap());
        let epoch = clock.current_epoch.clone();
        let (sender, _receiver) = broadcast::channel(BROADCAST_CHANNEL_SIZE);

        let token = CancellationToken::new();
        fail::cfg_callback(
            "block_clock::BlockClock::run::overwrite_block_number",
            move || {
                // Overflow the current_epoch on next poll
                epoch.store(u64::MAX, std::sync::atomic::Ordering::SeqCst);
            },
        )
        .unwrap();

        let handle = tokio::spawn(async move { clock.run(sender, token).await });

        let res = tokio::time::timeout(Duration::from_secs(10), handle)
            .await
            .expect("Timeout waiting for task to finish")
            .expect("Task Join error");

        assert!(matches!(
            res,
            Err(BlockClockError::SetEpochNumber(u64::MAX, 0))
        ));
        scenario.teardown();
    }
}
