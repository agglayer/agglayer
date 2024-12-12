//! This crate is responsible for managing the Clock pace.
//!
//! The Clock is responsible for providing information about Epoch timing by
//! exposing references to the data and by broadcasting `EpochChange` events.

use std::{
    num::NonZeroU64,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use tokio::sync::broadcast;

mod block;
mod time;

pub use block::BlockClock;
pub use time::TimeClock;
use tokio_util::sync::CancellationToken;

const BROADCAST_CHANNEL_SIZE: usize = 100;

/// The Clock trait is responsible for exposing methods to access relevant
/// information regarding the Block height and Epoch numbers.
#[async_trait::async_trait]
pub trait Clock {
    /// Spawn the Clock task and return a [`ClockRef`] to interact with it.
    async fn spawn(self, cancellation_token: CancellationToken) -> Result<ClockRef, Error>;

    /// Calculate an Epoch number based on a Block number.
    fn calculate_epoch_number(from_block: u64, epoch_duration: NonZeroU64) -> u64 {
        from_block / epoch_duration
    }
}

/// The ClockRef is a reference to the Clock instance.
#[derive(Clone)]
pub struct ClockRef {
    pub(crate) sender: broadcast::Sender<Event>,
    /// The Block height.
    /// This value is updated by the Clock task.
    pub(crate) block_height: Arc<AtomicU64>,
    /// The number of Blocks per Epoch.
    block_per_epoch: Arc<NonZeroU64>,
}

impl ClockRef {
    #[doc(hidden)]
    pub fn new(
        sender: broadcast::Sender<Event>,
        block_height: Arc<AtomicU64>,
        block_per_epoch: Arc<NonZeroU64>,
    ) -> Self {
        Self {
            sender,
            block_height,
            block_per_epoch,
        }
    }

    #[cfg(feature = "testutils")]
    pub fn update_block_height(&self, n: u64) {
        self.block_height.store(n, Ordering::SeqCst);
    }

    #[cfg(feature = "testutils")]
    pub fn get_sender(&self) -> broadcast::Sender<Event> {
        self.sender.clone()
    }

    /// Subscribe to the Clock events.
    ///
    /// # Errors
    ///
    /// This function can't fail but returns a Result for convenience and future
    /// evolution.
    pub fn subscribe(&self) -> Result<broadcast::Receiver<Event>, Error> {
        Ok(self.sender.subscribe())
    }

    /// Returns the current Epoch.
    pub fn current_epoch(&self) -> u64 {
        self.current_block_height() / *self.block_per_epoch
    }

    /// Returns the current Block height.
    pub fn current_block_height(&self) -> u64 {
        self.block_height.load(Ordering::Acquire)
    }
}

/// Events broadcasted by the Clock.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// Notify that an Epoch just ended with the associated Epoch number.
    EpochEnded(u64),
}

/// Errors that can be returned by the Clock.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("The Clock failed to start")]
    UnableToStart,
    #[error(transparent)]
    BlockClock(#[from] block::BlockClockError),
}
