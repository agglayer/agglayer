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
use tracing::error;

use crate::{Clock, ClockRef, Error, Event, BROADCAST_CHANNEL_SIZE};

/// Time based [`Clock`] implementation.
///
/// Simulate blockchain block production by increasing Block height by 1 every
/// second. Epoch duration can be configured when creating the Clock.
pub struct TimeClock {
    genesis: DateTime<Utc>,
    current_block: Arc<AtomicU64>,
    epoch_duration: NonZeroU64,
    current_epoch: Arc<AtomicU64>,
}

#[async_trait::async_trait]
impl Clock for TimeClock {
    async fn spawn(mut self) -> Result<ClockRef, Error> {
        let (sender, _receiver) = broadcast::channel(BROADCAST_CHANNEL_SIZE);

        self.compute_block_height();
        _ = self.compute_epoch_number();

        let clock_ref = ClockRef {
            sender: sender.clone(),
            current_epoch: self.current_epoch.clone(),
            current_block_height: self.current_block.clone(),
        };
        tokio::spawn(async move {
            self.run(sender).await;
        });

        Ok(clock_ref)
    }
}

impl TimeClock {
    /// Create a new TimeClock instance based on the current datetime and an
    /// Epoch.
    pub fn new_now(epoch_duration: NonZeroU64) -> Self {
        Self::new(Utc::now(), epoch_duration)
    }

    /// Create a new TimeClock instance based on a genesis datetime and an Epoch
    /// duration.
    pub fn new(genesis: DateTime<Utc>, epoch_duration: NonZeroU64) -> Self {
        Self {
            genesis,
            current_block: Arc::new(AtomicU64::new(0)),
            epoch_duration,
            current_epoch: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Run the Clock task.
    async fn run(&mut self, sender: broadcast::Sender<Event>) {
        let mut interval = interval_at(Instant::now(), Duration::from_secs(1));

        let current_block = self.compute_block_height();
        let current_epoch = self.calculate_epoch_number(current_block);
        self.current_epoch.store(current_epoch, Ordering::Relaxed);

        loop {
            interval.tick().await;

            let current_block = self.current_block.fetch_add(1, Ordering::Release) + 1;

            if current_block % self.epoch_duration == 0 {
                match self.compute_epoch_number() {
                    Ok(epoch_ended) => {
                        _ = sender.send(Event::EpochEnded(epoch_ended));
                    }
                    Err(current_epoch) => {
                        error!(
                            "Unexpected error computing the current Epoch: current_epoch={}, current_block={}",
                            current_epoch,
                            current_block
                        );
                    }
                }
            }
        }
    }

    /// Computes the Block height of this [`TimeClock`].
    ///
    /// This method is used to compute the Block height based on the
    /// genesis datetime and the current datetime.
    ///
    /// The Block height is the number of seconds since the genesis datetime.
    fn compute_block_height(&mut self) -> u64 {
        let block_height = self.calculate_block_height();

        self.current_block.store(block_height, Ordering::Release);

        block_height
    }

    /// Computes the current Epoch of this [`TimeClock`].
    ///
    /// This method is used to compute the current Epoch number based on the
    /// Block height and the Epoch duration.
    ///
    /// To define the current Epoch number, the Block height is divided
    /// by the Epoch duration.
    fn compute_epoch_number(&mut self) -> Result<u64, u64> {
        let current_block = self.current_block.load(Ordering::Acquire);

        let current_epoch = self.calculate_epoch_number(current_block);

        self.current_epoch.compare_exchange(
            current_epoch.saturating_sub(1),
            current_epoch,
            Ordering::Acquire,
            Ordering::Relaxed,
        )
    }

    fn calculate_epoch_number(&self, current_block: u64) -> u64 {
        current_block / self.epoch_duration
    }

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
    use std::num::NonZeroU64;

    use chrono::{Duration, Utc};

    use crate::{Clock, Event, TimeClock};

    #[tokio::test]
    async fn test_time_clock() {
        let genesis = Utc::now()
            .checked_sub_signed(Duration::seconds(30))
            .unwrap();

        let clock = TimeClock::new(genesis, NonZeroU64::new(5).unwrap());

        let clock_ref = clock.spawn().await.unwrap();

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

        let clock_ref = clock.spawn().await.unwrap();

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
}
