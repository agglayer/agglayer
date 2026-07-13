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
mod tests {
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn same_wallet_lock_blocks_until_guard_drops() {
        let locks = Arc::new(WalletNonceLocks::default());
        let wallet = Address::from([0xAB; 20]);

        let guard = locks.lock(wallet).await;

        let contender = tokio::spawn({
            let locks = locks.clone();
            async move { locks.lock(wallet).await }
        });

        // The second acquisition must not complete while the guard is held.
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(
            !contender.is_finished(),
            "lock must be exclusive per wallet"
        );

        drop(guard);

        tokio::time::timeout(Duration::from_secs(5), contender)
            .await
            .expect("contender should acquire the lock after the guard drops")
            .expect("contender task should not panic");
    }

    #[tokio::test]
    async fn different_wallets_lock_independently() {
        let locks = WalletNonceLocks::default();
        let wallet_a = Address::from([0xAA; 20]);
        let wallet_b = Address::from([0xBB; 20]);

        let _guard_a = locks.lock(wallet_a).await;

        // Must complete immediately: wallet B has its own lock.
        tokio::time::timeout(Duration::from_secs(5), locks.lock(wallet_b))
            .await
            .expect("distinct wallets must not contend");
    }
}
