//! Thread-safe nonce manager for L1 transaction submissions.
//!
//! This module provides a `NonceManager` that coordinates nonce assignment
//! across concurrent transaction submissions to prevent "nonce too low" errors.

use std::{collections::HashMap, sync::Arc};

use alloy::providers::Provider;
use parking_lot::Mutex;
use tracing::{debug, error, info, warn};

use crate::L1RpcError;

/// Information about a nonce assignment
#[derive(Debug, Clone)]
pub struct NonceAssignment {
    /// The nonce to use for the transaction
    pub nonce: u64,
    /// The address this nonce is for (using alloy's Address which implements
    /// Hash)
    pub address: alloy::primitives::Address,
}

/// Thread-safe nonce manager that coordinates nonce assignment across
/// concurrent transaction submissions.
///
/// The manager maintains a cache of the next expected nonce for each address
/// and ensures that nonces are assigned atomically without gaps or duplicates.
#[derive(Clone)]
pub struct NonceManager<P> {
    /// The underlying L1 provider
    provider: Arc<P>,
    /// Cache of next nonces per address
    /// Key: Address (alloy Address which implements Hash), Value: Next nonce to
    /// use
    nonces: Arc<Mutex<HashMap<alloy::primitives::Address, u64>>>,
}

impl<P> NonceManager<P>
where
    P: Provider + 'static,
{
    /// Create a new NonceManager with the given provider
    pub fn new(provider: Arc<P>) -> Self {
        Self {
            provider,
            nonces: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get the next nonce for the given address and increment the internal
    /// counter.
    ///
    /// This method is thread-safe and ensures that each call returns a unique,
    /// sequential nonce value.
    ///
    /// # Arguments
    ///
    /// * `address` - The Ethereum address to get the nonce for
    /// * `force_refresh` - If true, queries L1 for the current nonce regardless
    ///   of cache
    ///
    /// # Returns
    ///
    /// Returns the next nonce to use for a transaction from this address
    pub async fn get_next_nonce(
        &self,
        address: alloy::primitives::Address,
        force_refresh: bool,
    ) -> Result<NonceAssignment, L1RpcError> {
        // Check if we need to refresh from L1
        let should_refresh = {
            let nonces = self.nonces.lock();
            force_refresh || !nonces.contains_key(&address)
        }; // Lock is dropped here

        if should_refresh {
            // Query current nonce from L1 (without holding any lock)
            let l1_nonce = self
                .provider
                .get_transaction_count(address)
                .await
                .map_err(|e| {
                    error!(?e, %address, "Failed to query nonce from L1");
                    L1RpcError::FailedToQueryNonce {
                        address: address.into(),
                        source: e.into(),
                    }
                })?;

            debug!(%address, nonce = l1_nonce, "Queried nonce from L1");

            // Update cache with L1 nonce, but only if it's higher than cached value
            // (in case another thread updated it while we were querying)
            let mut nonces = self.nonces.lock();
            let cached_nonce = nonces.get(&address).copied().unwrap_or(0);
            let new_nonce = l1_nonce.max(cached_nonce);

            if new_nonce != l1_nonce {
                warn!(
                    %address,
                    l1_nonce,
                    cached_nonce,
                    new_nonce,
                    "Nonce mismatch: using higher value"
                );
            }

            nonces.insert(address, new_nonce);
        } // Lock is dropped here

        // Get and increment nonce atomically
        let nonce = {
            let mut nonces = self.nonces.lock();
            let nonce = nonces.get(&address).copied().unwrap_or(0);
            nonces.insert(address, nonce + 1);
            nonce
        }; // Lock is dropped here

        debug!(%address, nonce, next_nonce = nonce + 1, "Assigned nonce");

        Ok(NonceAssignment { nonce, address })
    }

    /// Report that a transaction with the given nonce failed.
    ///
    /// This method should be called when a transaction fails with a
    /// nonce-related error (e.g., "nonce too low"). It will reset the
    /// cached nonce for the address so that the next call to
    /// `get_next_nonce` will refresh from L1.
    pub fn report_nonce_error(&self, address: alloy::primitives::Address, failed_nonce: u64) {
        let mut nonces = self.nonces.lock();

        warn!(
            %address,
            failed_nonce,
            "Transaction failed with nonce error, invalidating cache"
        );

        // Remove from cache to force refresh on next request
        nonces.remove(&address);
    }

    /// Manually set the next nonce for an address.
    ///
    /// This is useful for recovery scenarios or when you have external
    /// knowledge about the correct nonce value.
    ///
    /// # Safety
    ///
    /// Use with caution - this bypasses the normal nonce management and can
    /// lead to nonce conflicts if used incorrectly.
    pub fn set_next_nonce(&self, address: alloy::primitives::Address, nonce: u64) {
        let mut nonces = self.nonces.lock();

        info!(
            %address,
            nonce,
            "Manually setting next nonce"
        );

        nonces.insert(address, nonce);
    }

    /// Get the cached nonce for an address without incrementing.
    ///
    /// Returns `None` if the address is not in the cache.
    pub fn peek_nonce(&self, address: alloy::primitives::Address) -> Option<u64> {
        let nonces = self.nonces.lock();
        nonces.get(&address).copied()
    }

    /// Clear the nonce cache for a specific address.
    ///
    /// The next call to `get_next_nonce` for this address will query L1.
    pub fn invalidate_cache(&self, address: alloy::primitives::Address) {
        let mut nonces = self.nonces.lock();
        nonces.remove(&address);
        debug!(%address, "Invalidated nonce cache");
    }

    /// Clear all cached nonces.
    ///
    /// All subsequent calls to `get_next_nonce` will query L1.
    pub fn clear_all_caches(&self) {
        let mut nonces = self.nonces.lock();
        nonces.clear();
        info!("Cleared all nonce caches");
    }

    /// Get a reference to the underlying provider
    pub fn provider(&self) -> &Arc<P> {
        &self.provider
    }
}

impl<P> std::fmt::Debug for NonceManager<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NonceManager")
            .field("nonces", &self.nonces.lock().clone())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    // TODO: Fix tests - these need to be updated for the new alloy API
    /*
    use alloy::providers::{ProviderBuilder};
    use alloy::providers::mock::Asserter;

    #[tokio::test]
    async fn test_nonce_manager_basic() {
        let asserter = Asserter::new();
        let provider = Arc::new(ProviderBuilder::new().on_mocked_client(asserter.clone()));
        let manager = NonceManager::new(provider);

        let address = agglayer_types::Address::ZERO;

        // Mock the get_transaction_count call to return 100
        asserter.mock_response(serde_json::json!(
            "0x64" // 100 in hex
        ));

        let assignment = manager.get_next_nonce(address, false).await.unwrap();
        assert_eq!(assignment.nonce, 100);
        assert_eq!(assignment.address, address);

        // Next call should use cached value (101) without querying L1
        let assignment = manager.get_next_nonce(address, false).await.unwrap();
        assert_eq!(assignment.nonce, 101);
    }

    #[tokio::test]
    async fn test_force_refresh() {
        let asserter = Asserter::new();
        let provider = Arc::new(ProviderBuilder::new().on_mocked_client(asserter.clone()));
        let manager = NonceManager::new(provider);

        let address = agglayer_types::Address::ZERO;

        // First call - mock response 100
        asserter.mock_response(serde_json::json!("0x64"));
        let assignment = manager.get_next_nonce(address, false).await.unwrap();
        assert_eq!(assignment.nonce, 100);

        // Force refresh - mock response 105 (simulating 5 txs were submitted externally)
        asserter.mock_response(serde_json::json!("0x69"));
        let assignment = manager.get_next_nonce(address, true).await.unwrap();
        assert_eq!(assignment.nonce, 105);
    }

    #[tokio::test]
    async fn test_concurrent_nonce_assignment() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let asserter = Asserter::new();
        let provider = Arc::new(ProviderBuilder::new().on_mocked_client(asserter.clone()));
        let manager = Arc::new(NonceManager::new(provider));

        let address = agglayer_types::Address::ZERO;

        // Mock initial nonce as 1000
        asserter.mock_response(serde_json::json!("0x3e8"));

        // Spawn multiple concurrent tasks to get nonces
        let num_tasks = 10;
        let mut handles = vec![];
        let seen_nonces = Arc::new(Mutex::new(std::collections::HashSet::new()));

        for _ in 0..num_tasks {
            let manager = manager.clone();
            let seen_nonces = seen_nonces.clone();
            let handle = tokio::spawn(async move {
                let assignment = manager.get_next_nonce(address, false).await.unwrap();
                let mut nonces = seen_nonces.lock();
                assert!(
                    nonces.insert(assignment.nonce),
                    "Duplicate nonce assigned: {}",
                    assignment.nonce
                );
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all nonces were unique and in range [1000, 1010)
        let seen_nonces = seen_nonces.lock();
        assert_eq!(seen_nonces.len(), num_tasks);
        for nonce in seen_nonces.iter() {
            assert!(*nonce >= 1000 && *nonce < 1000 + num_tasks as u64);
        }
    }

    #[tokio::test]
    async fn test_report_nonce_error() {
        let asserter = Asserter::new();
        let provider = Arc::new(ProviderBuilder::new().on_mocked_client(asserter.clone()));
        let manager = NonceManager::new(provider);

        let address = agglayer_types::Address::ZERO;

        // Initial query - mock response 100
        asserter.mock_response(serde_json::json!("0x64"));
        let assignment = manager.get_next_nonce(address, false).await.unwrap();
        assert_eq!(assignment.nonce, 100);

        // Report error
        manager.report_nonce_error(address, 100);

        // Next call should refresh from L1 - mock response 102
        asserter.mock_response(serde_json::json!("0x66"));
        let assignment = manager.get_next_nonce(address, false).await.unwrap();
        assert_eq!(assignment.nonce, 102);
    }
    */
}
