use std::{
    num::NonZeroU64,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use agglayer_types::EpochNumber;
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
        }
    }

    /// Run the Clock task.
    async fn run(
        &mut self,
        sender: broadcast::Sender<Event>,
        cancellation_token: CancellationToken,
    ) {
        let mut interval = interval_at(Instant::now(), Duration::from_secs(1));

        // Initialize the current Block height based on genesis
        self.update_block_height();

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
                        // the current Epoch has ended. In this case, we calculate the epoch
                        // number on demand and send an `EpochEnded` event to the subscribers.
                        if current_block % *self.epoch_duration == 0 {
                            // Calculate the epoch that just ended (current_block / epoch_duration - 1)
                            let epoch_ended = EpochNumber::new(
                                Self::calculate_epoch_number(current_block, *self.epoch_duration)
                                    .saturating_sub(1)
                            );
                            if let Err(error) = sender.send(Event::EpochEnded(epoch_ended)) {
                                error!("Failed to send EpochEnded event to subscribers: {error}");
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

    /// Calculate an Epoch number based on a Block number.
    fn calculate_epoch_number(from_block: u64, epoch_duration: NonZeroU64) -> u64 {
        from_block / epoch_duration
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use agglayer_types::EpochNumber;
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

        assert_eq!(
            recv.recv().await,
            Ok(Event::EpochEnded(EpochNumber::new(6)))
        );
        assert_eq!(clock_ref.current_epoch(), EpochNumber::new(7));
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
        assert_eq!(
            recv.recv().await,
            Ok(Event::EpochEnded(EpochNumber::new(15)))
        );
        assert!(recv.try_recv().is_err());
        assert_eq!(clock_ref.current_epoch(), EpochNumber::new(16));
        assert!(clock_ref.current_block_height() >= 30);
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        assert_eq!(
            recv.recv().await,
            Ok(Event::EpochEnded(EpochNumber::new(16)))
        );
        assert_eq!(
            recv.recv().await,
            Ok(Event::EpochEnded(EpochNumber::new(17)))
        );

        assert_eq!(clock_ref.current_epoch(), EpochNumber::new(18));
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
        assert_eq!(recv.try_recv(), Ok(Event::EpochEnded(EpochNumber::new(15))));

        // Overflow the block height on next poll
        blocks.store(u64::MAX - 1, std::sync::atomic::Ordering::SeqCst);

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        _ = futures::poll!(&mut fut);

        assert!(token.is_cancelled());
    }
}
