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
use tracing::debug;
#[cfg(not(test))]
use tracing::error;

use crate::{Clock, ClockRef, Error, Event, BROADCAST_CHANNEL_SIZE};

/// Block based [`Clock`] implementation.
pub struct BlockClock<P> {
    provider: Arc<P>,
    genesis_block: u64,
    current_block: Arc<AtomicU64>,
    epoch_duration: NonZeroU64,
    current_epoch: Arc<AtomicU64>,
}

#[async_trait::async_trait]
impl<P> Clock for BlockClock<P>
where
    P: Middleware + 'static,
    <P as Middleware>::Provider: PubsubClient,
{
    async fn spawn(mut self) -> Result<ClockRef, Error> {
        let (sender, _receiver) = broadcast::channel(BROADCAST_CHANNEL_SIZE);

        let clock_ref = ClockRef {
            sender: sender.clone(),
            current_epoch: self.current_epoch.clone(),
            current_block_height: self.current_block.clone(),
        };

        // Spawn the Clock task directly
        tokio::spawn(async move {
            self.run(sender).await;
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
            current_block: Arc::new(AtomicU64::new(0)),
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
        let expecting_epoch = current_epoch.saturating_sub(1);

        match self.current_epoch.compare_exchange(
            expecting_epoch,
            current_epoch,
            Ordering::Acquire,
            Ordering::Relaxed,
        ) {
            Ok(previous) => Ok(previous),
            Err(stored) => Err((stored, expecting_epoch)),
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

impl<P> BlockClock<P>
where
    P: Middleware,
    <P as Middleware>::Provider: PubsubClient,
{
    /// Run the Clock task.
    async fn run(&mut self, sender: broadcast::Sender<Event>) {
        // Start by setting the current Block height based on the current L1 Block
        // number. If the current L1 Block number is less than the genesis block
        // number, we exit the process with an error message.
        if let Ok(current_l1_block) = self.provider.get_block_number().await {
            let current_l1_block = current_l1_block.as_u64();
            if current_l1_block < self.genesis_block {
                let error_message = format!(
                    "The current block height is less than the genesis block number: {} < {}",
                    current_l1_block, self.genesis_block
                );

                #[cfg(not(test))]
                {
                    error!("{}", error_message);
                    std::process::exit(1);
                }

                #[cfg(test)]
                panic!("{}", error_message);
            }

            let current_block = self.calculate_block_number(current_l1_block);
            match self.current_block.compare_exchange(
                0,
                current_block,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(0) => {
                    debug!("The current block height was set to: {}", current_block);
                    if let Err((previous, expected)) = self.update_epoch_number(current_block) {
                        let error_message = format!(
                            "Failed to set the current Epoch number: previous={}, expected={}",
                            previous, expected
                        );

                        #[cfg(not(test))]
                        {
                            error!("{}", error_message);
                            std::process::exit(1);
                        }

                        #[cfg(test)]
                        panic!("{}", error_message);
                    }
                }
                Ok(block) => {
                    let error_message = format!(
                        "The current block height was already set to a non-zero value: {}",
                        block
                    );

                    #[cfg(not(test))]
                    {
                        error!("{}", error_message);
                        std::process::exit(1);
                    }

                    #[cfg(test)]
                    panic!("{}", error_message);
                }
                Err(block) => {
                    let error_message = format!(
                        "Failed to set the current block height, already set to: {}",
                        block
                    );

                    #[cfg(not(test))]
                    {
                        error!("{}", error_message);
                        std::process::exit(1);
                    }

                    #[cfg(test)]
                    panic!("{}", error_message);
                }
            }
        }

        let provider = self.provider.clone();
        // Subscribe to the L1 Block stream and listen for new Blocks.
        let mut stream = provider.subscribe_blocks().await.unwrap();

        while let Some(block) = stream.next().await {
            debug!(
                "L1 Block received: timestamp={}, number={}, hash={}",
                block.timestamp,
                block.number.unwrap(),
                block.hash.unwrap()
            );

            // Increase the Block height by 1. The `fetch_add` method returns the previous
            // value, so we need to add 1 to it to get the current Block height.
            if let Some(current_block) = self
                .current_block
                .fetch_add(1, Ordering::Release)
                .checked_add(1)
            {
                // If the current Block height is a multiple of the Epoch duration, the current
                // Epoch has ended. In this case, we need to update the new Epoch number and
                // send an `EpochEnded` event to the subscribers.
                if current_block % self.epoch_duration == 0 {
                    match self.update_epoch_number(current_block) {
                        Err((previous, expected)) => {
                            let error_message = format!(
                                "Failed to set the current Epoch number: previous={}, expected={}",
                                previous, expected
                            );

                            #[cfg(not(test))]
                            {
                                error!("{}", error_message);
                                std::process::exit(1);
                            }

                            #[cfg(test)]
                            panic!("{}", error_message);
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

#[cfg(test)]
mod tests {
    use std::{num::NonZeroU64, time::Duration};

    use ethers::{
        providers::{Provider, Ws},
        utils::Anvil,
    };
    use tokio::sync::broadcast;

    use crate::{BlockClock, Clock, ClockRef, Event, BROADCAST_CHANNEL_SIZE};

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
        let clock_ref = clock.spawn().await.unwrap();
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
        let clock_ref = clock.spawn().await.unwrap();
        assert_eq!(clock_ref.current_epoch(), 0);

        let mut recv = clock_ref.subscribe().unwrap();

        assert_eq!(recv.recv().await, Ok(Event::EpochEnded(0)));
        assert_eq!(clock_ref.current_epoch(), 1);
        assert!(clock_ref.current_block_height() >= 3);
    }

    #[tokio::test]
    #[should_panic]
    async fn test_block_clock_starting_with_wrong_genesis() {
        let anvil = Anvil::new().block_time(1u64).spawn();
        let client = Provider::<Ws>::connect(anvil.ws_endpoint()).await.unwrap();

        tokio::time::sleep(Duration::from_secs(1)).await;
        let mut clock = BlockClock::new(client, 2, NonZeroU64::new(3).unwrap());
        let (sender, _receiver) = broadcast::channel(BROADCAST_CHANNEL_SIZE);

        let mut fut = Box::pin(clock.run(sender));

        _ = futures::poll!(&mut fut);
        tokio::time::sleep(Duration::from_millis(10)).await;
        _ = futures::poll!(&mut fut);
    }

    #[tokio::test]
    #[should_panic]
    async fn test_time_clock_overflow() {
        let anvil = Anvil::new().block_time(1u64).spawn();
        let client = Provider::<Ws>::connect(anvil.ws_endpoint()).await.unwrap();

        let mut clock = BlockClock::new(client, 2, NonZeroU64::new(3).unwrap());
        let blocks = clock.current_block.clone();
        let (sender, _receiver) = broadcast::channel(BROADCAST_CHANNEL_SIZE);

        let clock_ref = ClockRef {
            sender: sender.clone(),
            current_epoch: clock.current_epoch.clone(),
            current_block_height: clock.current_block.clone(),
        };

        let mut fut = Box::pin(clock.run(sender));
        let mut recv = clock_ref.subscribe().unwrap();

        _ = futures::poll!(&mut fut);
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        _ = futures::poll!(&mut fut);
        assert_eq!(recv.try_recv(), Ok(Event::EpochEnded(15)));

        // Overflow the block height on next poll
        blocks.store(u64::MAX - 1, std::sync::atomic::Ordering::SeqCst);

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        _ = futures::poll!(&mut fut);
    }
}
