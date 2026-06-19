//! Per-wallet exclusive nonce allocation for concurrent settlement tasks.
//!
//! [`NonceAllocatorRegistry`] seeds from the chain pending count, hands out
//! monotonic nonces, and reconciles when L1 state diverges. Only new-nonce
//! builds call [`NonceAllocatorRegistry::reserve`]; gas bumps must reuse an
//! existing nonce and must not reserve again.
//! XREF: https://github.com/agglayer/agglayer/issues/1319

use std::collections::{BTreeSet, HashMap};

use alloy::{primitives::Address, providers::Provider, transports::TransportError};
use thiserror::Error;
use tokio::sync::Mutex;

/// Errors surfaced while reserving a settlement nonce.
#[derive(Debug, Error)]
pub enum NonceAllocatorError {
    #[error("L1 RPC error while reserving settlement nonce: {0}")]
    Transport(#[from] TransportError),
}

#[derive(Debug, Default)]
struct WalletNonceState {
    /// Next nonce to hand out.
    next_nonce: u64,
    /// Handed out but not yet observed as consumed on L1.
    reserved: BTreeSet<u64>,
    /// Whether `next_nonce` has been seeded from chain state or tests.
    seeded: bool,
}

/// Shared per-wallet nonce allocator for concurrent settlement tasks.
pub struct NonceAllocatorRegistry {
    inner: Mutex<HashMap<Address, WalletNonceState>>,
}

impl NonceAllocatorRegistry {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    /// Reserves the next exclusive nonce for `wallet`.
    ///
    /// On first use per wallet, seeds from `get_transaction_count(wallet).pending()`.
    /// The mutex is not held across the RPC call.
    pub async fn reserve<P: Provider>(
        &self,
        provider: &P,
        wallet: Address,
    ) -> Result<u64, NonceAllocatorError> {
        let needs_seed = {
            let guard = self.inner.lock().await;
            guard
                .get(&wallet)
                .map(|state| !state.seeded)
                .unwrap_or(true)
        };

        if needs_seed {
            let pending = provider
                .get_transaction_count(wallet)
                .pending()
                .await
                .map_err(NonceAllocatorError::Transport)?;

            let mut guard = self.inner.lock().await;
            let state = guard.entry(wallet).or_default();
            if !state.seeded {
                state.next_nonce = pending;
                state.seeded = true;
            }
        }

        let mut guard = self.inner.lock().await;
        let state = guard
            .get_mut(&wallet)
            .expect("wallet nonce state exists after seeding");

        let nonce = state.next_nonce;
        state.next_nonce = state.next_nonce.saturating_add(1);
        state.reserved.insert(nonce);
        Ok(nonce)
    }

    /// Records that `nonce` is used on L1 (by us or externally).
    ///
    /// Advances the allocator past `nonce` and removes it from the reserved set.
    pub async fn mark_consumed(&self, wallet: Address, nonce: u64) {
        let mut guard = self.inner.lock().await;
        let Some(state) = guard.get_mut(&wallet) else {
            return;
        };

        state.reserved.remove(&nonce);
        state.next_nonce = state.next_nonce.max(nonce.saturating_add(1));
    }

    /// Syncs local state with the chain's pending transaction count.
    ///
    /// Only increases `next_nonce`; never lowers it. Prunes reserved nonces that
    /// are strictly below `chain_pending`.
    pub async fn reconcile_next_pending(&self, wallet: Address, chain_pending: u64) {
        let mut guard = self.inner.lock().await;
        let Some(state) = guard.get_mut(&wallet) else {
            return;
        };

        state.next_nonce = state.next_nonce.max(chain_pending);
        state
            .reserved
            .retain(|reserved_nonce| *reserved_nonce >= chain_pending);
    }

    /// Seeds allocator state for unit tests without an L1 RPC.
    #[cfg(test)]
    pub async fn seed_for_test(&self, wallet: Address, next_nonce: u64) {
        let mut guard = self.inner.lock().await;
        let state = guard.entry(wallet).or_default();
        state.next_nonce = next_nonce;
        state.seeded = true;
        state.reserved.clear();
    }

    #[cfg(test)]
    async fn next_nonce_for_test(&self, wallet: Address) -> Option<u64> {
        let guard = self.inner.lock().await;
        guard.get(&wallet).map(|state| state.next_nonce)
    }
}

impl Default for NonceAllocatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use alloy::primitives::Address;

    use super::*;

    fn test_wallet() -> Address {
        Address::from([0xAB; 20])
    }

    #[tokio::test]
    async fn reserve_twice_returns_consecutive_nonces() {
        let registry = NonceAllocatorRegistry::new();
        let wallet = test_wallet();
        registry.seed_for_test(wallet, 5).await;

        assert_eq!(registry.reserve_seeded(wallet).await, 5);
        assert_eq!(registry.reserve_seeded(wallet).await, 6);
    }

    #[tokio::test]
    async fn concurrent_reserve_returns_distinct_nonces() {
        let registry = Arc::new(NonceAllocatorRegistry::new());
        let wallet = test_wallet();
        registry.seed_for_test(wallet, 0).await;

        let mut handles = Vec::new();
        for _ in 0..10 {
            let registry = registry.clone();
            handles.push(tokio::spawn(async move {
                registry.reserve_seeded(wallet).await
            }));
        }

        let mut nonces: Vec<u64> = Vec::new();
        for handle in handles {
            nonces.push(handle.await.expect("task should complete"));
        }

        nonces.sort_unstable();
        assert_eq!(nonces, (0..10).collect::<Vec<_>>());
    }

    #[tokio::test]
    async fn reconcile_never_decreases_next_nonce() {
        let registry = NonceAllocatorRegistry::new();
        let wallet = test_wallet();
        registry.seed_for_test(wallet, 10).await;

        registry.reconcile_next_pending(wallet, 7).await;
        assert_eq!(registry.next_nonce_for_test(wallet).await, Some(10));

        registry.reconcile_next_pending(wallet, 12).await;
        assert_eq!(registry.next_nonce_for_test(wallet).await, Some(12));
        assert_eq!(registry.reserve_seeded(wallet).await, 12);
    }

    #[tokio::test]
    async fn mark_consumed_advances_next_reservation() {
        let registry = NonceAllocatorRegistry::new();
        let wallet = test_wallet();
        registry.seed_for_test(wallet, 5).await;

        assert_eq!(registry.reserve_seeded(wallet).await, 5);
        registry.mark_consumed(wallet, 5).await;
        assert_eq!(registry.reserve_seeded(wallet).await, 6);
    }

    #[tokio::test]
    async fn reconcile_prunes_reserved_nonces_below_chain_pending() {
        let registry = NonceAllocatorRegistry::new();
        let wallet = test_wallet();
        registry.seed_for_test(wallet, 5).await;

        assert_eq!(registry.reserve_seeded(wallet).await, 5);
        assert_eq!(registry.reserve_seeded(wallet).await, 6);

        registry.reconcile_next_pending(wallet, 6).await;
        assert_eq!(registry.next_nonce_for_test(wallet).await, Some(7));
        assert_eq!(registry.reserve_seeded(wallet).await, 7);
    }

    impl NonceAllocatorRegistry {
        async fn reserve_seeded(&self, wallet: Address) -> u64 {
            let mut guard = self.inner.lock().await;
            let state = guard
                .get_mut(&wallet)
                .expect("wallet should be seeded for test");
            let nonce = state.next_nonce;
            state.next_nonce = state.next_nonce.saturating_add(1);
            state.reserved.insert(nonce);
            nonce
        }
    }
}
