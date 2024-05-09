//! This crate is responsible for managing the clock pace.
//!
//! The Clock is responsible for providing information about Epoch timing by
//! exposing references to the data and by broadcasting `EpochChange` events.

use std::sync::{atomic::AtomicU64, Arc};

use tokio::sync::broadcast;

mod time;

pub use time::TimeClock;

const BROADCAST_CHANNEL_SIZE: usize = 100;

/// The Clock trait is responsible for exposing methods to access relevant
/// information regarding the current block and epoch numbers.
#[async_trait::async_trait]
pub trait Clock {
    /// Compute Epoch/Block numbers and spawn the clock task.
    async fn spawn(self) -> Result<ClockRef, Error>;
    /// Return a reference to the current block number.
    fn block_ref(&self) -> Arc<AtomicU64>;
    /// Return a reference to the current epoch number.
    fn epoch_ref(&self) -> Arc<AtomicU64>;
}

/// The ClockRef is a reference to the Clock instance.
pub struct ClockRef {
    pub(crate) sender: broadcast::Sender<Event>,
}

impl ClockRef {
    /// Subscribe to the Clock events.
    ///
    /// # Errors
    ///
    /// This function can't fail but return a Result for convenience and future
    /// evolution.
    pub fn subscribe(&self) -> Result<broadcast::Receiver<Event>, Error> {
        Ok(self.sender.subscribe())
    }
}

/// Events broadcasted by the Clock.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// Notify an Epoch change with the associated epoch_number.
    EpochChange(u64),
}

/// Errors that can be returned by the Clock.
#[derive(Debug, thiserror::Error)]
pub enum Error {}
