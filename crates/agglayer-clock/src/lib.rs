//! This crate is responsible for managing the Clock pace.
//!
//! The Clock is responsible for providing information about Epoch timing by
//! exposing references to the data and by broadcasting `EpochChange` events.

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
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
}

/// The ClockRef is a reference to the Clock instance.
pub struct ClockRef {
    pub(crate) sender: broadcast::Sender<Event>,
    /// The current Epoch number.
    /// This value is updated by the Clock task.
    pub(crate) current_epoch: Arc<AtomicU64>,
    /// The Block height.
    /// This value is updated by the Clock task.
    pub(crate) block_height: Arc<AtomicU64>,
}

impl ClockRef {
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
        self.current_epoch.load(Ordering::Acquire)
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
pub enum Error {}
