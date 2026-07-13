//! Per-wallet async locks serializing the nonce read-to-save window across
//! concurrent settlement tasks.
//!
//! A settlement task computes its next nonce from
//! `max(L1 pending count, highest stored nonce + 1)` but only records the
//! choice when the attempt is saved. Holding the wallet's lock across that
//! whole window makes the read-and-record atomic with respect to other tasks
//! signing from the same wallet.
//! XREF: https://github.com/agglayer/agglayer/issues/1597

use std::{collections::HashMap, sync::Arc};

use alloy::primitives::Address;
use tokio::sync::OwnedMutexGuard;
use tracing::debug;

/// Registry of per-wallet async locks shared by all settlement tasks of one
/// [`SettlementService`](crate::SettlementService).
///
/// Entries are created on first use and never removed; the map is bounded by
/// the number of distinct wallets ever used, which is one per service
/// instance today (the provider's default signer).
#[derive(Debug, Default)]
pub(crate) struct WalletNonceLocks {
    locks: std::sync::Mutex<HashMap<Address, Arc<tokio::sync::Mutex<()>>>>,
}

impl WalletNonceLocks {
    /// Locks `wallet`, waiting until any concurrent holder releases it.
    ///
    /// The returned guard is owned so callers can move it across function
    /// boundaries and drop it at the exact release point.
    /// Dropping the returned future before acquisition leaves the lock
    /// untouched (`lock_owned` is cancel-safe).
    /// The underlying tokio mutex is FIFO-fair, so waiters on a busy wallet
    /// cannot starve.
    pub(crate) async fn lock(&self, wallet: Address) -> OwnedMutexGuard<()> {
        let lock = {
            let mut locks = self.locks.lock().expect("wallet nonce locks poisoned");
            locks.entry(wallet).or_default().clone()
        };
        match Arc::clone(&lock).try_lock_owned() {
            Ok(guard) => guard,
            Err(_) => {
                debug!(%wallet, "Waiting for the per-wallet settlement nonce lock");
                let guard = lock.lock_owned().await;
                debug!(%wallet, "Acquired the per-wallet settlement nonce lock");
                guard
            }
        }
    }
}

#[cfg(test)]
mod tests;
