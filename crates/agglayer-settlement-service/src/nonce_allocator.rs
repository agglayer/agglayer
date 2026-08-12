//! Per-wallet exclusive nonce allocation for concurrent settlement tasks.
//!
//! [`NonceAllocatorRegistry`] hands out gap-free monotonic nonces across
//! concurrent settlement tasks that share an L1 wallet. Callers compute a
//! store/L1 floor (`max(L1 pending, highest stored nonce + 1)`) and call
//! [`NonceAllocatorRegistry::reserve_at_floor`], which applies that floor under
//! one lock and returns an armed [`NonceReservation`]. Handout skips nonces
//! still in the reserved set so releasing a lower hole cannot collide with a
//! higher in-flight reservation. Dropping an armed reservation releases the
//! nonce; call [`NonceReservation::commit`] after the attempt is saved so the
//! nonce stays reserved through submit until L1 observation or
//! [`NonceAllocatorRegistry::mark_consumed`]. Gas bumps must reuse an existing
//! nonce and must not hand out again.
//! XREF: https://github.com/agglayer/agglayer/issues/1575

use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex};

use alloy::primitives::Address;

#[derive(Debug, Default)]
struct WalletNonceState {
    /// Next nonce to hand out.
    next_nonce: u64,
    /// Handed out but not yet observed as consumed on L1 / released.
    /// Handout never returns a value still in this set, so releasing a lower
    /// hole cannot re-issue a higher nonce that is still in flight.
    reserved: BTreeSet<u64>,
    /// Whether `next_nonce` has been initialized from a floor or tests.
    seeded: bool,
}

impl WalletNonceState {
    fn apply_floor(&mut self, floor: u64) {
        if !self.seeded {
            self.next_nonce = floor;
            self.seeded = true;
        } else {
            self.next_nonce = self.next_nonce.max(floor);
        }
    }

    /// Returns the next nonce that is not already reserved, advancing
    /// `next_nonce` past it.
    fn handout_next_unreserved(&mut self) -> u64 {
        loop {
            let nonce = self.next_nonce;
            self.next_nonce = self.next_nonce.saturating_add(1);
            // `insert` is true only when this nonce was not already reserved
            // (e.g. still held by another in-flight build after a lower release).
            if self.reserved.insert(nonce) {
                return nonce;
            }
            assert!(
                nonce < u64::MAX,
                "settlement nonce space exhausted while skipping reserved nonces"
            );
        }
    }
}

/// Shared per-wallet nonce allocator for concurrent settlement tasks of one
/// [`SettlementService`](crate::SettlementService).
///
/// Critical sections are short and never await, so this uses a std mutex.
/// That also lets [`NonceReservation`] release safely from `Drop`.
/// Entries are created on first use and never removed; the map is bounded by
/// the number of distinct wallets ever used, which is one per service
/// instance today (the provider's default signer).
pub(crate) struct NonceAllocatorRegistry {
    inner: Mutex<HashMap<Address, WalletNonceState>>,
}

impl NonceAllocatorRegistry {
    pub(crate) fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<Address, WalletNonceState>> {
        self.inner.lock().expect("nonce allocator mutex poisoned")
    }

    /// Seeds `next_nonce` from `seed` if not already seeded.
    ///
    /// Prefer [`Self::reserve_at_floor`] / [`Self::handout_at_floor`]: they
    /// apply the floor and hand out atomically. This helper is for tests that
    /// only need to initialize state.
    #[cfg(test)]
    pub(crate) fn seed_if_unseeded(&self, wallet: Address, seed: u64) {
        let mut guard = self.lock();
        let state = guard.entry(wallet).or_default();
        if !state.seeded {
            state.next_nonce = seed;
            state.seeded = true;
        }
    }

    /// Applies `floor` then hands out the next exclusive nonce for `wallet`.
    ///
    /// `floor` should be `max(L1 pending, highest stored nonce + 1)`. Under the
    /// mutex this raises `next_nonce` to at least `floor` (and seeds if needed),
    /// then returns the next value not already in `reserved`. Concurrent callers
    /// receive distinct nonces even when their floor reads raced; releasing a
    /// lower hole cannot re-issue a higher still-reserved nonce.
    ///
    /// Prefer [`Self::reserve_at_floor`] so failed builds / unsaved attempts
    /// release via [`NonceReservation`] drop.
    pub(crate) fn handout_at_floor(&self, wallet: Address, floor: u64) -> u64 {
        let mut guard = self.lock();
        let state = guard.entry(wallet).or_default();
        state.apply_floor(floor);
        state.handout_next_unreserved()
    }

    /// Like [`Self::handout_at_floor`], but returns an armed [`NonceReservation`]
    /// that releases on drop unless [`NonceReservation::commit`] is called.
    pub(crate) fn reserve_at_floor(
        self: &Arc<Self>,
        wallet: Address,
        floor: u64,
    ) -> NonceReservation {
        let nonce = self.handout_at_floor(wallet, floor);
        NonceReservation {
            registry: Arc::clone(self),
            wallet,
            nonce,
            armed: true,
        }
    }

    /// Hands out the next exclusive nonce for `wallet`.
    ///
    /// The wallet must already be seeded via [`Self::handout_at_floor`],
    /// [`Self::seed_if_unseeded`], or [`Self::seed_for_test`]. Prefer
    /// [`Self::handout_at_floor`] / [`Self::reserve_at_floor`] when a
    /// store/L1 floor is available. Skips nonces still present in `reserved`.
    #[cfg(test)]
    pub(crate) fn handout(&self, wallet: Address) -> u64 {
        let mut guard = self.lock();
        let state = guard
            .get_mut(&wallet)
            .expect("wallet must be seeded before nonce handout");
        state.handout_next_unreserved()
    }

    /// Rolls back a handed-out nonce that never became a saved settlement
    /// attempt.
    ///
    /// Returns `true` when `nonce` was removed from the reserved set. Also
    /// lowers `next_nonce` when this handout created a gap at the low end so
    /// the nonce can be handed out again. Has no effect after
    /// [`Self::mark_consumed`].
    pub(crate) fn release(&self, wallet: Address, nonce: u64) -> bool {
        let mut guard = self.lock();
        let Some(state) = guard.get_mut(&wallet) else {
            return false;
        };

        if !state.reserved.remove(&nonce) {
            return false;
        }

        state.next_nonce = state.next_nonce.min(nonce);
        true
    }

    /// Records that `nonce` is used (by us, externally, or via admin insert).
    ///
    /// Advances the allocator past `nonce` and removes it from the reserved
    /// set. Creates wallet state if missing so out-of-band store writes still
    /// raise the floor before the first handout.
    pub(crate) fn mark_consumed(&self, wallet: Address, nonce: u64) {
        let mut guard = self.lock();
        let state = guard.entry(wallet).or_default();
        state.reserved.remove(&nonce);
        state.next_nonce = state.next_nonce.max(nonce.saturating_add(1));
        state.seeded = true;
    }

    /// Syncs local state with the chain's pending transaction count.
    ///
    /// Only increases `next_nonce`; never lowers it. Prunes reserved nonces
    /// that are strictly below `chain_pending`.
    pub(crate) fn reconcile_next_pending(&self, wallet: Address, chain_pending: u64) {
        let mut guard = self.lock();
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
    pub(crate) fn seed_for_test(&self, wallet: Address, next_nonce: u64) {
        let mut guard = self.lock();
        let state = guard.entry(wallet).or_default();
        state.next_nonce = next_nonce;
        state.seeded = true;
        state.reserved.clear();
    }

    #[cfg(test)]
    fn next_nonce_for_test(&self, wallet: Address) -> Option<u64> {
        let guard = self.lock();
        guard.get(&wallet).map(|state| state.next_nonce)
    }

    #[cfg(test)]
    fn is_reserved_for_test(&self, wallet: Address, nonce: u64) -> bool {
        let guard = self.lock();
        guard
            .get(&wallet)
            .is_some_and(|state| state.reserved.contains(&nonce))
    }
}

impl Default for NonceAllocatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Exclusive handout that releases on drop unless [`Self::commit`] is called.
///
/// After the settlement attempt is persisted, call [`Self::commit`] so the
/// nonce stays in `reserved` through submit until L1 observation or
/// [`NonceAllocatorRegistry::mark_consumed`]. On build failure, cancellation,
/// panic before save, or any drop before commit, `Drop` releases the nonce so
/// peers can reuse it.
pub(crate) struct NonceReservation {
    registry: Arc<NonceAllocatorRegistry>,
    wallet: Address,
    nonce: u64,
    armed: bool,
}

impl NonceReservation {
    /// The handed-out nonce value.
    pub(crate) fn nonce(&self) -> u64 {
        self.nonce
    }

    /// Keep the reservation after the attempt has been saved to the store.
    ///
    /// The nonce remains reserved until later `mark_consumed` / reconcile.
    pub(crate) fn commit(mut self) {
        self.armed = false;
    }
}

impl Drop for NonceReservation {
    fn drop(&mut self) {
        if self.armed {
            let _ = self.registry.release(self.wallet, self.nonce);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::thread;

    use alloy::primitives::Address;

    use super::*;

    fn test_wallet() -> Address {
        Address::from([0xAB; 20])
    }

    #[test]
    fn handout_twice_returns_consecutive_nonces() {
        let registry = NonceAllocatorRegistry::new();
        let wallet = test_wallet();
        registry.seed_for_test(wallet, 5);

        assert_eq!(registry.handout(wallet), 5);
        assert_eq!(registry.handout(wallet), 6);
    }

    #[test]
    fn concurrent_handout_returns_distinct_nonces() {
        let registry = Arc::new(NonceAllocatorRegistry::new());
        let wallet = test_wallet();
        registry.seed_for_test(wallet, 0);

        let mut handles = Vec::new();
        for _ in 0..10 {
            let registry = registry.clone();
            handles.push(thread::spawn(move || registry.handout(wallet)));
        }

        let mut nonces: Vec<u64> = Vec::new();
        for handle in handles {
            nonces.push(handle.join().expect("task should complete"));
        }

        nonces.sort_unstable();
        assert_eq!(nonces, (0..10).collect::<Vec<_>>());
    }

    #[test]
    fn seed_if_unseeded_is_idempotent() {
        let registry = NonceAllocatorRegistry::new();
        let wallet = test_wallet();

        registry.seed_if_unseeded(wallet, 5);
        registry.seed_if_unseeded(wallet, 9);

        assert_eq!(registry.next_nonce_for_test(wallet), Some(5));
        assert_eq!(registry.handout(wallet), 5);
    }

    #[test]
    fn handout_at_floor_seeds_and_returns_floor() {
        let registry = NonceAllocatorRegistry::new();
        let wallet = test_wallet();

        assert_eq!(registry.handout_at_floor(wallet, 5), 5);
        assert_eq!(registry.handout_at_floor(wallet, 5), 6);
    }

    #[test]
    fn handout_at_floor_advances_to_higher_store_floor() {
        let registry = NonceAllocatorRegistry::new();
        let wallet = test_wallet();
        registry.seed_for_test(wallet, 5);

        // Store/admin moved ahead of the in-memory cursor.
        assert_eq!(registry.handout_at_floor(wallet, 11), 11);
        assert_eq!(registry.next_nonce_for_test(wallet), Some(12));
    }

    #[test]
    fn handout_at_floor_ignores_lower_stale_floor() {
        let registry = NonceAllocatorRegistry::new();
        let wallet = test_wallet();
        registry.seed_for_test(wallet, 10);

        assert_eq!(registry.handout_at_floor(wallet, 7), 10);
    }

    #[test]
    fn concurrent_handout_at_floor_returns_distinct_nonces() {
        let registry = Arc::new(NonceAllocatorRegistry::new());
        let wallet = test_wallet();

        let mut handles = Vec::new();
        for _ in 0..10 {
            let registry = registry.clone();
            handles.push(thread::spawn(move || registry.handout_at_floor(wallet, 0)));
        }

        let mut nonces: Vec<u64> = Vec::new();
        for handle in handles {
            nonces.push(handle.join().expect("task should complete"));
        }

        nonces.sort_unstable();
        assert_eq!(nonces, (0..10).collect::<Vec<_>>());
    }

    #[test]
    fn reconcile_never_decreases_next_nonce() {
        let registry = NonceAllocatorRegistry::new();
        let wallet = test_wallet();
        registry.seed_for_test(wallet, 10);

        registry.reconcile_next_pending(wallet, 7);
        assert_eq!(registry.next_nonce_for_test(wallet), Some(10));

        registry.reconcile_next_pending(wallet, 12);
        assert_eq!(registry.next_nonce_for_test(wallet), Some(12));
        assert_eq!(registry.handout(wallet), 12);
    }

    #[test]
    fn mark_consumed_advances_next_reservation() {
        let registry = NonceAllocatorRegistry::new();
        let wallet = test_wallet();
        registry.seed_for_test(wallet, 5);

        assert_eq!(registry.handout(wallet), 5);
        registry.mark_consumed(wallet, 5);
        assert_eq!(registry.handout(wallet), 6);
    }

    #[test]
    fn mark_consumed_creates_state_for_unknown_wallet() {
        let registry = NonceAllocatorRegistry::new();
        let wallet = test_wallet();

        registry.mark_consumed(wallet, 10);
        assert_eq!(registry.next_nonce_for_test(wallet), Some(11));
        assert_eq!(registry.handout(wallet), 11);
    }

    #[test]
    fn reconcile_prunes_reserved_nonces_below_chain_pending() {
        let registry = NonceAllocatorRegistry::new();
        let wallet = test_wallet();
        registry.seed_for_test(wallet, 5);

        assert_eq!(registry.handout(wallet), 5);
        assert_eq!(registry.handout(wallet), 6);

        registry.reconcile_next_pending(wallet, 6);
        assert_eq!(registry.next_nonce_for_test(wallet), Some(7));
        assert_eq!(registry.handout(wallet), 7);
    }

    #[test]
    fn release_restores_next_nonce_for_single_handout() {
        let registry = NonceAllocatorRegistry::new();
        let wallet = test_wallet();
        registry.seed_for_test(wallet, 5);

        assert_eq!(registry.handout(wallet), 5);
        assert_eq!(registry.next_nonce_for_test(wallet), Some(6));
        assert!(registry.release(wallet, 5));
        assert_eq!(registry.next_nonce_for_test(wallet), Some(5));
        assert_eq!(registry.handout(wallet), 5);
    }

    #[test]
    fn release_lowest_hole_with_higher_reserved() {
        let registry = NonceAllocatorRegistry::new();
        let wallet = test_wallet();
        registry.seed_for_test(wallet, 5);

        assert_eq!(registry.handout(wallet), 5);
        assert_eq!(registry.handout(wallet), 6);
        assert!(registry.release(wallet, 5));
        assert_eq!(registry.next_nonce_for_test(wallet), Some(5));
        assert_eq!(registry.handout(wallet), 5);
        // 6 is still reserved by the in-flight handout; must not re-issue it.
        assert_eq!(registry.handout(wallet), 7);
    }

    #[test]
    fn handout_skips_still_reserved_after_lower_release() {
        // P0 regression: parallel builds + release of a lower nonce must not
        // hand the higher still-in-flight nonce to another task.
        let registry = NonceAllocatorRegistry::new();
        let wallet = test_wallet();
        registry.seed_for_test(wallet, 5);

        assert_eq!(registry.handout_at_floor(wallet, 5), 5);
        assert_eq!(registry.handout_at_floor(wallet, 5), 6);
        assert!(registry.release(wallet, 5));

        assert_eq!(registry.handout_at_floor(wallet, 5), 5);
        assert_eq!(registry.handout_at_floor(wallet, 5), 7);
    }

    #[test]
    fn release_is_noop_when_not_reserved() {
        let registry = NonceAllocatorRegistry::new();
        let wallet = test_wallet();
        registry.seed_for_test(wallet, 5);

        assert!(!registry.release(wallet, 5));
        assert_eq!(registry.next_nonce_for_test(wallet), Some(5));
    }

    #[test]
    fn release_is_noop_for_unknown_wallet() {
        let registry = NonceAllocatorRegistry::new();
        let wallet = test_wallet();

        assert!(!registry.release(wallet, 0));
    }

    #[test]
    fn reservation_drop_releases_nonce() {
        let registry = Arc::new(NonceAllocatorRegistry::new());
        let wallet = test_wallet();

        let nonce = {
            let reservation = registry.reserve_at_floor(wallet, 5);
            assert_eq!(reservation.nonce(), 5);
            assert!(registry.is_reserved_for_test(wallet, 5));
            reservation.nonce()
        };

        assert!(!registry.is_reserved_for_test(wallet, nonce));
        assert_eq!(registry.next_nonce_for_test(wallet), Some(5));
        assert_eq!(registry.handout_at_floor(wallet, 5), 5);
    }

    #[test]
    fn reservation_commit_keeps_nonce_reserved() {
        let registry = Arc::new(NonceAllocatorRegistry::new());
        let wallet = test_wallet();

        {
            let reservation = registry.reserve_at_floor(wallet, 5);
            assert_eq!(reservation.nonce(), 5);
            reservation.commit();
        }

        assert!(registry.is_reserved_for_test(wallet, 5));
        assert_eq!(registry.handout_at_floor(wallet, 5), 6);
    }
}
