use std::{
    num::NonZeroU64,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use chrono::{DateTime, Utc};
use tokio::{
    sync::broadcast,
    time::{interval_at, Instant},
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};

use crate::{Clock, ClockRef, Error, Event, BROADCAST_CHANNEL_SIZE};

/// Time based [`Clock`] implementation.
///
/// Simulate blockchain block production by increasing Block height by 1 every
/// second. Epoch duration can be configured when creating the Clock.
pub struct TimeClock {
    genesis: DateTime<Utc>,
    current_block: Arc<AtomicU64>,
    epoch_duration: Arc<NonZeroU64>,
    current_epoch: Arc<AtomicU64>,
}

#[async_trait::async_trait]
impl Clock for TimeClock {
    async fn spawn(mut self, cancellation_token: CancellationToken) -> Result<ClockRef, Error> {
        let (sender, _receiver) = broadcast::channel(BROADCAST_CHANNEL_SIZE);

        let clock_ref = ClockRef {
            sender: sender.clone(),
            block_height: self.current_block.clone(),
            block_per_epoch: self.epoch_duration.clone(),
        };

        // Spawn the Clock task directly
        tokio::spawn(async move {
            self.run(sender, cancellation_token).await;
        });

        Ok(clock_ref)
    }
}

impl TimeClock {
    /// Create a new [`TimeClock`] instance based on the current datetime and an
    /// Epoch.
    pub fn new_now(epoch_duration: NonZeroU64) -> Self {
        Self::new(Utc::now(), epoch_duration)
    }

    /// Create a new [`TimeClock`] instance based on a genesis datetime and an
    /// Epoch duration.
    pub fn new(genesis: DateTime<Utc>, epoch_duration: NonZeroU64) -> Self {
        Self {
            genesis,
            current_block: Arc::new(AtomicU64::new(0)),
            epoch_duration: Arc::new(epoch_duration),
            current_epoch: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Run the Clock task.
    async fn run(
        &mut self,
        sender: broadcast::Sender<Event>,
        cancellation_token: CancellationToken,
    ) {
        let mut interval = interval_at(Instant::now(), Duration::from_secs(1));

        // Compute the current Block height and Epoch number
        let current_block = self.update_block_height();
        let current_epoch = Self::calculate_epoch_number(current_block, *self.epoch_duration);

        // Use compare_exchange to ensure the initial current_epoch is 0
        if let Err(stored_value) = self.current_epoch.compare_exchange(
            0,
            current_epoch,
            Ordering::AcqRel,
            Ordering::Relaxed,
        ) {
            let error_message = format!(
                "The current_epoch has already been modified. Shutting down the Clock task. \
                 Stored value: {}",
                stored_value
            );
            #[cfg(not(test))]
            {
                error!("{}", error_message);
                std::process::exit(1);
            }
            #[cfg(test)]
            panic!("{}", error_message);
        }

        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    debug!("Clock task cancelled");
                    break;
                }
                _ = interval.tick() => {

                    // Increase the Block height by 1.
                    // The `fetch_add` method returns the previous value, so we need to add 1 to it
                    // to get the current Block height.
                    if let Some(current_block) = self
                        .current_block
                            .fetch_add(1, Ordering::Release)
                            .checked_add(1)
                    {
                        // If the current Block height is a multiple of the Epoch duration,
                        // the current Epoch has ended. In this case, we need to update the
                        // new Epoch number and send an `EpochEnded` event to the subscribers.
                        if current_block % *self.epoch_duration == 0 {
                            match self.update_epoch_number() {
                                Ok(epoch_ended) => {
                                    if let Err(error) = sender.send(Event::EpochEnded(epoch_ended)) {
                                        error!("Failed to send EpochEnded event to subscribers: {error}");
                                    }
                                }
                                Err((current_epoch, expected)) => {
                                    error!(
                                        "Unexpected error computing the current Epoch: current_epoch={}, \
                                        expected_epoch={}, current_block={}",
                                        current_epoch, expected, current_block
                                    );
                                    cancellation_token.cancel();
                                }
                            }
                        }
                    } else {
                       error!("Block height overflowed the u64 limit. \
                           This is an unexpected situation and could lead to unexpected behavior. \
                           Please report this issue to the developers. https://github.com/agglayer/agglayer/issues/new \
                           The node will now kill itself to prevent further damage.");

                        cancellation_token.cancel();
                    }
                }
            }
        }
    }

    /// Updates the Block height of this [`TimeClock`].
    ///
    /// This method is used to update the Block height based on the
    /// genesis datetime and the current datetime.
    ///
    /// The Block height is the number of seconds since the genesis datetime.
    fn update_block_height(&mut self) -> u64 {
        let block_height = self.calculate_block_height();

        self.current_block.store(block_height, Ordering::Release);

        block_height
    }

    /// Updates the current Epoch of this [`TimeClock`].
    ///
    /// This method is used to update the current Epoch number based on the
    /// Block height and the Epoch duration.
    ///
    /// To define the current Epoch number, the Epoch duration divides the Block
    /// height.
    fn update_epoch_number(&mut self) -> Result<u64, (u64, u64)> {
        let current_block = self.current_block.load(Ordering::Acquire);

        let current_epoch = Self::calculate_epoch_number(current_block, *self.epoch_duration);
        let expected_epoch = current_epoch.saturating_sub(1);

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

    /// Calculate the Block height.
    fn calculate_block_height(&self) -> u64 {
        std::cmp::max(
            Utc::now()
                .naive_utc()
                .signed_duration_since(self.genesis.naive_utc())
                .num_seconds(),
            0,
        ) as u64
    }
}

#[cfg(test)]
mod tests {
    use std::{num::NonZeroU64, sync::atomic::Ordering};

    use chrono::{Duration, Utc};
    use tokio::sync::broadcast;
    use tokio_util::sync::CancellationToken;

    use crate::{Clock, ClockRef, Event, TimeClock, BROADCAST_CHANNEL_SIZE};

    #[tokio::test]
    async fn test_time_clock() {
        let genesis = Utc::now()
            .checked_sub_signed(Duration::seconds(30))
            .unwrap();

        let clock = TimeClock::new(genesis, NonZeroU64::new(5).unwrap());

        let token = CancellationToken::new();
        let clock_ref = clock.spawn(token.clone()).await.unwrap();

        let mut recv = clock_ref.subscribe().unwrap();

        assert_eq!(recv.recv().await, Ok(Event::EpochEnded(6)));
        assert_eq!(clock_ref.current_epoch(), 7);
        assert!(clock_ref.current_block_height() >= 30);
    }

    #[tokio::test]
    async fn test_time_clock_catchup() {
        let genesis = Utc::now()
            .checked_sub_signed(Duration::seconds(30))
            .unwrap();

        let clock = TimeClock::new(genesis, NonZeroU64::new(2).unwrap());

        let token = CancellationToken::new();
        let clock_ref = clock.spawn(token.clone()).await.unwrap();

        let mut recv = clock_ref.subscribe().unwrap();
        assert_eq!(recv.recv().await, Ok(Event::EpochEnded(15)));
        assert!(recv.try_recv().is_err());
        assert_eq!(clock_ref.current_epoch(), 16);
        assert!(clock_ref.current_block_height() >= 30);
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        assert_eq!(recv.recv().await, Ok(Event::EpochEnded(16)));
        assert_eq!(recv.recv().await, Ok(Event::EpochEnded(17)));

        assert_eq!(clock_ref.current_epoch(), 18);
        assert!(clock_ref.current_block_height() >= 35);
    }

    #[tokio::test]
    async fn test_time_clock_overflow() {
        let genesis = Utc::now()
            .checked_sub_signed(Duration::seconds(30))
            .unwrap();

        let mut clock = TimeClock::new(genesis, NonZeroU64::new(2).unwrap());
        let blocks = clock.current_block.clone();
        let (sender, _receiver) = broadcast::channel(BROADCAST_CHANNEL_SIZE);

        let clock_ref = ClockRef {
            sender: sender.clone(),
            block_height: clock.current_block.clone(),
            block_per_epoch: clock.epoch_duration.clone(),
        };

        let token = CancellationToken::new();
        let mut fut = Box::pin(clock.run(sender, token.clone()));
        let mut recv = clock_ref.subscribe().unwrap();

        _ = futures::poll!(&mut fut);
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        _ = futures::poll!(&mut fut);
        assert_eq!(recv.try_recv(), Ok(Event::EpochEnded(15)));

        // Overflow the block height on next poll
        blocks.store(u64::MAX - 1, std::sync::atomic::Ordering::SeqCst);

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        _ = futures::poll!(&mut fut);

        assert!(token.is_cancelled());
    }

    #[tokio::test]
    #[should_panic]
    async fn test_initial_epoch_update_error() {
        let genesis = Utc::now()
            .checked_sub_signed(Duration::seconds(30))
            .unwrap();

        let mut clock = TimeClock::new(genesis, NonZeroU64::new(5).unwrap());
        clock.current_epoch.store(1, Ordering::Relaxed);

        let (sender, _receiver) = broadcast::channel(BROADCAST_CHANNEL_SIZE);

        let token = CancellationToken::new();
        let _ = clock.run(sender, token).await;
    }
}
