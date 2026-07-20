# Settlement attempt builder (issue 1318) implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task.
> Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build and sign an EIP-1559 settlement transaction for a job with a
fresh nonce and config-derived gas parameters, ready to record to RocksDB
before submission.

**Architecture:** Add an `EthereumWallet` to `SettlementTask`; add a reusable
async `build_attempt` primitive that signs a fully-resolved EIP-1559 request;
add `build_next_attempt_with_new_nonce` that resolves wallet/nonce/chain-id/gas
(RPC + config) and calls the primitive, integrated with the existing
`retry!`/`RetryCallbackError` machinery.

**Tech Stack:** Rust, alloy 1.7.3 (`TransactionRequest`, `EthereumWallet`,
`Provider`), tokio, Anvil (`alloy` `node-bindings`, dev-only),
`agglayer-config` (`SettlementTransactionConfig`, `Multiplier`).

**Spec:** `docs/knowledge-base/src/ai-agents/specs/2026-06-08-settlement-attempt-builder-design.md`

---

## File structure

- Modify: `crates/agglayer-settlement-service/src/settlement_task.rs`
  - `SettlementTask` struct gains `wallet: Arc<EthereumWallet>`.
  - New: `GasParams`, `BuildAttemptError`, `apply_multiplier_u128`,
    `clamp_u128`, `next_attempt_number`, `resolve_base_gas_params`,
    `build_attempt`; rewritten `build_next_attempt_with_new_nonce`;
    optional `is_wallet_privkey_known`.
  - Run-loop call site updated to `await` + `retry!`.
  - Test helpers updated; new unit tests added.
- Modify: `crates/agglayer-settlement-service/src/settlement_service.rs`
  - `SettlementService` struct gains `wallet: Arc<EthereumWallet>`;
    `start` gains a wallet parameter; `request_new_settlement` passes it to
    `SettlementTask::create`. Test helper updated.

All changes are within one crate (`agglayer-settlement-service`).
No new non-dev dependency is required: `EthereumWallet` is in `alloy::network`
(already pulled via the `full` feature), and tests use
`alloy::signers::local::PrivateKeySigner` and `alloy::node_bindings::Anvil`
(already used by `utils.rs` tests).

---

## Task 1: Add the signing wallet to `SettlementTask` and `SettlementService`

Structural change only: thread a wallet through constructors so the crate
still compiles and all existing tests pass. No new builder behavior yet.

**Files:**
- Modify: `crates/agglayer-settlement-service/src/settlement_task.rs`
  (imports; struct ~142-151; `create` ~158-187; `load` ~189-212;
  test helpers ~1099-1126 and load test ~1319)
- Modify: `crates/agglayer-settlement-service/src/settlement_service.rs`
  (imports; struct ~23-31; `start` ~58-75; `request_new_settlement` ~125-132;
  test helper ~315-327)

- [ ] **Step 1: Add the `wallet` field and import to `settlement_task.rs`**

In the `alloy::{...}` import block, add `EthereumWallet` to the `network`
group:

```rust
    network::{BlockResponse as _, EthereumWallet, ReceiptResponse as _},
```

Add the field to the struct (after `store`):

```rust
pub struct SettlementTask<L1Provider, SettlementStore> {
    id: SettlementJobId,
    job: SettlementJob,
    tx_config: Arc<SettlementTransactionConfig>,
    provider: Arc<L1Provider>,
    store: Arc<SettlementStore>,
    wallet: Arc<EthereumWallet>,
    control: TaskControl,
    attempts:
        BTreeMap<(Address, Nonce), BTreeMap<SettlementAttemptNumber, ActiveSettlementAttempt>>,
}
```

- [ ] **Step 2: Thread `wallet` through `create` and `load`**

`create` — add the parameter and set the field:

```rust
    pub async fn create(
        job: SettlementJob,
        tx_config: Arc<SettlementTransactionConfig>,
        provider: Arc<L1Provider>,
        store: Arc<SettlementStore>,
        wallet: Arc<EthereumWallet>,
        control: TaskControl,
    ) -> eyre::Result<(SettlementJobId, Self)> {
        // ...unchanged id generation...
        let this = Self {
            id,
            job,
            tx_config,
            provider,
            store,
            wallet,
            control,
            attempts: BTreeMap::new(),
        };
        this.save_settlement_job_to_db().await?;
        Ok((id, this))
    }
```

`load` — add the parameter and set the field in the `Pending` branch:

```rust
    pub async fn load(
        id: SettlementJobId,
        tx_config: Arc<SettlementTransactionConfig>,
        provider: Arc<L1Provider>,
        store: Arc<SettlementStore>,
        wallet: Arc<EthereumWallet>,
        control: TaskControl,
    ) -> eyre::Result<StoredSettlementJob<L1Provider, SettlementStore>> {
        let (job, result) = Self::load_settlement_job_from_db(id).await?;
        if let Some(result) = result {
            Ok(StoredSettlementJob::Completed(job, result))
        } else {
            let mut this = SettlementTask {
                id,
                job,
                tx_config,
                provider,
                store,
                wallet,
                control,
                attempts: BTreeMap::new(),
            };
            this.load_settlement_attempts_from_db().await?;
            Ok(StoredSettlementJob::Pending(this))
        }
    }
```

- [ ] **Step 3: Add `wallet` to `SettlementService` and `start`**

In `settlement_service.rs`, add the import:

```rust
use alloy::{network::EthereumWallet, providers::Provider};
```

(replace the existing `use alloy::providers::Provider;` line).

Add the field to the struct:

```rust
pub struct SettlementService<L1Provider, SettlementStore> {
    tx_config: Arc<SettlementTransactionConfig>,
    provider: Arc<L1Provider>,
    store: Arc<SettlementStore>,
    wallet: Arc<EthereumWallet>,
    cancellation_token: CancellationToken,
    task_controls: Arc<Mutex<HashMap<SettlementJobId, TaskControlHandle>>>,
    result_watchers:
        Arc<Mutex<HashMap<SettlementJobId, watch::Receiver<Option<SettlementJobResult>>>>>,
}
```

In `start`, add the parameter and set the field:

```rust
    pub async fn start(
        _config: SettlementServiceConfig,
        tx_config: Arc<SettlementTransactionConfig>,
        provider: Arc<L1Provider>,
        store: Arc<SettlementStore>,
        wallet: Arc<EthereumWallet>,
        cancellation_token: CancellationToken,
    ) -> eyre::Result<Self> {
        let this = Self {
            tx_config,
            provider,
            store,
            wallet,
            cancellation_token,
            task_controls: Arc::new(Mutex::new(HashMap::new())),
            result_watchers: Arc::new(Mutex::new(HashMap::new())),
        };
        Ok(this)
    }
```

In `request_new_settlement`, pass the wallet into `create`:

```rust
        let (job_id, mut task) = SettlementTask::create(
            job,
            self.tx_config.clone(),
            self.provider.clone(),
            self.store.clone(),
            self.wallet.clone(),
            task_control,
        )
        .await?;
```

- [ ] **Step 4: Update test helpers to construct a wallet**

In `settlement_task.rs` tests, add imports inside `mod tests`:

```rust
    use alloy::{network::EthereumWallet, signers::local::PrivateKeySigner};
```

Add a helper near `mk_control`:

```rust
    fn mk_random_wallet() -> Arc<EthereumWallet> {
        Arc::new(EthereumWallet::from(PrivateKeySigner::random()))
    }
```

Change `mk_task_with_id` to accept a wallet and set the field:

```rust
    fn mk_task_with_id(
        job_id: SettlementJobId,
        store: Arc<MockStateStore>,
        attempts: BTreeMap<
            (Address, Nonce),
            BTreeMap<SettlementAttemptNumber, ActiveSettlementAttempt>,
        >,
        wallet: Arc<EthereumWallet>,
    ) -> SettlementTask<impl Provider + 'static, MockStateStore> {
        SettlementTask {
            id: job_id,
            job: mk_job(),
            tx_config: Arc::new(SettlementTransactionConfig::default()),
            provider: Arc::new(mk_provider()),
            store,
            wallet,
            control: mk_control(),
            attempts,
        }
    }
```

Update `mk_task` to pass a random wallet:

```rust
    fn mk_task(
        store: Arc<MockStateStore>,
        attempts: BTreeMap<
            (Address, Nonce),
            BTreeMap<SettlementAttemptNumber, ActiveSettlementAttempt>,
        >,
    ) -> SettlementTask<impl Provider + 'static, MockStateStore> {
        mk_task_with_id(mk_job_id(1), store, attempts, mk_random_wallet())
    }
```

Update the one direct caller of `mk_task_with_id` (in
`load_settlement_attempts_from_db_hydrates_attempts_and_results`):

```rust
        let mut task = mk_task_with_id(job_id, Arc::new(store), BTreeMap::new(), mk_random_wallet());
```

- [ ] **Step 5: Update the `settlement_service.rs` test helper**

Add the import inside `mod tests`:

```rust
    use alloy::{network::EthereumWallet, signers::local::PrivateKeySigner};
```

Pass a wallet into `start` in `mk_service`:

```rust
    async fn mk_service(
        store: Arc<MockStateStore>,
    ) -> SettlementService<impl Provider + 'static, MockStateStore> {
        SettlementService::start(
            SettlementServiceConfig::default(),
            Arc::new(SettlementTransactionConfig::default()),
            Arc::new(mk_provider()),
            store,
            Arc::new(EthereumWallet::from(PrivateKeySigner::random())),
            CancellationToken::new(),
        )
        .await
        .expect("settlement service should start")
    }
```

- [ ] **Step 6: Verify the crate compiles and existing tests pass**

Run: `cargo nextest run -p agglayer-settlement-service`
Expected: PASS (all pre-existing tests still green; no behavior changed).

- [ ] **Step 7: Commit**

```bash
git add crates/agglayer-settlement-service/src/settlement_task.rs \
        crates/agglayer-settlement-service/src/settlement_service.rs
git commit -S -m "feat(settlement): add signing wallet to settlement task"
```

---

## Task 2: `next_attempt_number` helper

Allocate the next per-job attempt number from in-memory state.

**Files:**
- Modify: `crates/agglayer-settlement-service/src/settlement_task.rs`
  (add method near `all_attempt_keys` ~476; add test in `mod tests`)

- [ ] **Step 1: Write the failing test**

Add to `mod tests`:

```rust
    #[test]
    fn next_attempt_number_starts_at_zero_and_increments_past_max() {
        let store = Arc::new(MockStateStore::new());

        let empty = mk_task(store.clone(), BTreeMap::new());
        assert_eq!(empty.next_attempt_number(), SettlementAttemptNumber(0));

        let wallet = Address::from([1; 20]);
        let nonce = Nonce(7);
        let attempts = BTreeMap::from([(
            (wallet, nonce),
            BTreeMap::from([
                (
                    SettlementAttemptNumber(2),
                    mk_active_attempt(wallet, nonce, mk_tx_hash(1), None),
                ),
                (
                    SettlementAttemptNumber(5),
                    mk_active_attempt(wallet, nonce, mk_tx_hash(2), None),
                ),
            ]),
        )]);
        let task = mk_task(store, attempts);
        assert_eq!(task.next_attempt_number(), SettlementAttemptNumber(6));
    }
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo nextest run -p agglayer-settlement-service next_attempt_number_starts_at_zero`
Expected: FAIL — `no method named next_attempt_number`.

- [ ] **Step 3: Implement `next_attempt_number`**

Add as a method on `SettlementTask` (near `all_attempt_keys`):

```rust
    fn next_attempt_number(&self) -> SettlementAttemptNumber {
        let next = self
            .attempts
            .values()
            .flat_map(|attempts_for_nonce| attempts_for_nonce.keys())
            .map(|attempt_number| attempt_number.0)
            .max()
            .map_or(0, |max| max.saturating_add(1));
        SettlementAttemptNumber(next)
    }
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo nextest run -p agglayer-settlement-service next_attempt_number_starts_at_zero`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/agglayer-settlement-service/src/settlement_task.rs
git commit -S -m "feat(settlement): allocate next settlement attempt number"
```

---

## Task 3: Gas types and `resolve_base_gas_params`

Pure config-driven gas resolution mirroring `agglayer-contracts::adjust_gas_estimate`.

**Files:**
- Modify: `crates/agglayer-settlement-service/src/settlement_task.rs`
  (imports; new types + helpers + method; test in `mod tests`)

- [ ] **Step 1: Add imports**

Add to the `alloy::{...}` block:

```rust
    eips::{eip1559::Eip1559Estimation, BlockNumberOrTag},
```

(replace the existing `eips::BlockNumberOrTag,` line).

Add after the alloy import block:

```rust
use agglayer_config::Multiplier;
```

- [ ] **Step 2: Write the failing test**

Add to `mod tests` (note `SettlementTransactionConfig` and `Multiplier` are in
scope via `super::*` and the new import):

```rust
    #[test]
    fn resolve_base_gas_params_applies_multiplier_floor_and_ceiling() {
        // `Multiplier` is in scope via the module-level import + `super::*`.
        let mut config = SettlementTransactionConfig::default();
        config.gas_limit_multiplier_factor = Multiplier::from_u64_per_1000(2000); // 2.0x
        config.gas_limit_ceiling = U256::from(150_000u64);
        config.max_fee_per_gas_multiplier_factor = Multiplier::from_u64_per_1000(1000);
        config.max_fee_per_gas_floor = 1_000_000_000; // 1 gwei
        config.max_fee_per_gas_ceiling = 50_000_000_000; // 50 gwei
        config.max_priority_fee_per_gas_multiplier_factor = Multiplier::from_u64_per_1000(1000);
        config.max_priority_fee_per_gas_floor = 2_000_000_000; // 2 gwei
        config.max_priority_fee_per_gas_ceiling = 50_000_000_000; // 50 gwei

        let mut task = mk_task(Arc::new(MockStateStore::new()), BTreeMap::new());
        task.tx_config = Arc::new(config);
        task.job = SettlementJob { gas_limit: 100_000, ..mk_job() };

        // Estimate above the fee ceiling and below the priority floor.
        let estimate = Eip1559Estimation {
            max_fee_per_gas: 80_000_000_000, // 80 gwei -> clamps to 50 gwei
            max_priority_fee_per_gas: 100_000_000, // 0.1 gwei -> raised to 2 gwei floor
        };

        let gas = task.resolve_base_gas_params(&estimate);

        // 100_000 * 2.0 = 200_000, capped to ceiling 150_000.
        assert_eq!(gas.gas_limit, 150_000);
        assert_eq!(gas.max_fee_per_gas, 50_000_000_000);
        assert_eq!(gas.max_priority_fee_per_gas, 2_000_000_000);
    }
```

`mk_job` builds `SettlementJob` with `gas_limit: 100_000`; the spread sets the
limit explicitly for clarity. `U256` is already imported in `mod tests`.

- [ ] **Step 3: Run the test to verify it fails**

Run: `cargo nextest run -p agglayer-settlement-service resolve_base_gas_params_applies`
Expected: FAIL — `GasParams` / `resolve_base_gas_params` not found.

- [ ] **Step 4: Implement the types and helpers**

Add near the top of the file (after the `TxEnvelope` alias):

```rust
/// Fully-resolved gas parameters for a single settlement attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct GasParams {
    gas_limit: u64,
    max_fee_per_gas: u128,
    max_priority_fee_per_gas: u128,
}

/// Multiply `value` by a [`Multiplier`] (fixed-point, scaled by 1000),
/// saturating instead of overflowing. Mirrors `adjust_gas_estimate`.
fn apply_multiplier_u128(value: u128, multiplier: Multiplier) -> u128 {
    value.saturating_mul(u128::from(multiplier.as_u64_per_1000())) / 1000
}

fn clamp_u128(value: u128, floor: u128, ceiling: u128) -> u128 {
    value.max(floor).min(ceiling)
}
```

Add the resolution method on `SettlementTask` (near the build functions):

```rust
    fn resolve_base_gas_params(&self, estimate: &Eip1559Estimation) -> GasParams {
        let config = self.tx_config.as_ref();

        let max_fee_per_gas = clamp_u128(
            apply_multiplier_u128(
                estimate.max_fee_per_gas,
                config.max_fee_per_gas_multiplier_factor,
            ),
            config.max_fee_per_gas_floor,
            config.max_fee_per_gas_ceiling,
        );
        let max_priority_fee_per_gas = clamp_u128(
            apply_multiplier_u128(
                estimate.max_priority_fee_per_gas,
                config.max_priority_fee_per_gas_multiplier_factor,
            ),
            config.max_priority_fee_per_gas_floor,
            config.max_priority_fee_per_gas_ceiling,
        );

        let gas_limit_ceiling = u128::try_from(config.gas_limit_ceiling).unwrap_or(u128::MAX);
        let gas_limit_u128 =
            apply_multiplier_u128(self.job.gas_limit, config.gas_limit_multiplier_factor)
                .min(gas_limit_ceiling);
        let gas_limit = u64::try_from(gas_limit_u128).unwrap_or(u64::MAX);

        GasParams {
            gas_limit,
            max_fee_per_gas,
            max_priority_fee_per_gas,
        }
    }
```

- [ ] **Step 5: Run the test to verify it passes**

Run: `cargo nextest run -p agglayer-settlement-service resolve_base_gas_params_applies`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/agglayer-settlement-service/src/settlement_task.rs
git commit -S -m "feat(settlement): resolve base gas params from config"
```

---

## Task 4: `build_attempt` primitive and `BuildAttemptError`

Build and sign an EIP-1559 envelope from `self.job` and given parameters.

**Files:**
- Modify: `crates/agglayer-settlement-service/src/settlement_task.rs`
  (imports; `BuildAttemptError`; `build_attempt`; test in `mod tests`)

- [ ] **Step 1: Add imports**

Extend the `alloy::{...}` block:

```rust
    network::{
        BlockResponse as _, Ethereum, EthereumWallet, ReceiptResponse as _,
        TransactionBuilder as _, TransactionBuilderError,
    },
    rpc::types::TransactionRequest,
```

(the `network` group replaces the one edited in Task 1; keep `primitives`,
`providers`, `transports`, `consensus`, `eips` as already present).

- [ ] **Step 2: Add the error type**

Add near `GasParams`:

```rust
/// Error surfaced while building a settlement attempt.
#[derive(Debug, thiserror::Error)]
enum BuildAttemptError {
    #[error("L1 RPC error while building settlement attempt: {0}")]
    Transport(#[from] TransportError),
    #[error("failed to build or sign settlement transaction: {0}")]
    Build(#[from] TransactionBuilderError<Ethereum>),
}

impl BuildAttemptError {
    fn is_transient(&self) -> bool {
        match self {
            Self::Transport(error) => crate::utils::is_transient_alloy_error(error),
            // A build/sign failure with a configured wallet is non-recoverable.
            Self::Build(_) => false,
        }
    }
}
```

- [ ] **Step 3: Write the failing test**

Add to `mod tests` (add `use alloy::consensus::{Transaction as _, transaction::SignerRecoverable as _};`
at the top of the test that needs them, or to the module test imports):

```rust
    #[tokio::test]
    async fn build_attempt_produces_signed_eip1559_envelope() {
        use alloy::consensus::{transaction::SignerRecoverable as _, Transaction as _};

        let signer = PrivateKeySigner::random();
        let wallet_address = signer.address();
        let wallet = Arc::new(EthereumWallet::from(signer));

        let task = mk_task_with_id(
            mk_job_id(1),
            Arc::new(MockStateStore::new()),
            BTreeMap::new(),
            wallet,
        );

        let gas = GasParams {
            gas_limit: 100_000,
            max_fee_per_gas: 30_000_000_000,
            max_priority_fee_per_gas: 1_000_000_000,
        };

        let envelope = task
            .build_attempt(wallet_address, Nonce(9), 1337, gas)
            .await
            .expect("attempt should build");

        assert!(matches!(envelope, TxEnvelope::Eip1559(_)));
        assert_eq!(envelope.nonce(), 9);
        assert_eq!(envelope.chain_id(), Some(1337));
        assert_eq!(envelope.gas_limit(), 100_000);
        assert_eq!(envelope.max_fee_per_gas(), 30_000_000_000);
        assert_eq!(envelope.max_priority_fee_per_gas(), Some(1_000_000_000));
        assert_eq!(envelope.value(), mk_job().eth_value);
        assert_eq!(envelope.input().as_ref(), mk_job().calldata.as_ref());
        assert_eq!(
            envelope.to(),
            Some(mk_job().contract_address.into_alloy())
        );
        assert_eq!(envelope.recover_signer().unwrap(), wallet_address);
    }
```

- [ ] **Step 4: Run the test to verify it fails**

Run: `cargo nextest run -p agglayer-settlement-service build_attempt_produces_signed`
Expected: FAIL — `no method named build_attempt`.

- [ ] **Step 5: Implement `build_attempt`**

Replace the `build_next_attempt_with_nonce`/`build_next_attempt_with_new_nonce`
neighborhood by adding this method (leave `build_next_attempt_with_nonce`,
issue 1319, untouched for now):

```rust
    async fn build_attempt(
        &self,
        wallet: Address,
        nonce: Nonce,
        chain_id: u64,
        gas: GasParams,
    ) -> Result<TxEnvelope, TransactionBuilderError<Ethereum>> {
        let request = TransactionRequest::default()
            .from(wallet)
            .to(self.job.contract_address.into_alloy())
            .value(self.job.eth_value)
            .input(self.job.calldata.clone().into())
            .nonce(nonce.0)
            .gas_limit(gas.gas_limit)
            .max_fee_per_gas(gas.max_fee_per_gas)
            .max_priority_fee_per_gas(gas.max_priority_fee_per_gas)
            .with_chain_id(chain_id);

        request.build(self.wallet.as_ref()).await
    }
```

- [ ] **Step 6: Run the test to verify it passes**

Run: `cargo nextest run -p agglayer-settlement-service build_attempt_produces_signed`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/agglayer-settlement-service/src/settlement_task.rs
git commit -S -m "feat(settlement): build and sign an eip1559 settlement attempt"
```

---

## Task 5: `build_next_attempt_with_new_nonce` and run-loop wiring

Resolve wallet/nonce/chain-id/fees, call `build_attempt`, and `await` it in the
run loop via `retry!`.

**Files:**
- Modify: `crates/agglayer-settlement-service/src/settlement_task.rs`
  (rewrite stub ~738-744; call site ~404-406; new Anvil test in `mod tests`)

- [ ] **Step 1: Write the failing test**

Add to `mod tests` (Anvil-backed; mirrors the `utils.rs` test pattern):

```rust
    #[tokio::test]
    async fn build_next_attempt_with_new_nonce_uses_pending_nonce_and_default_wallet() {
        use alloy::{
            consensus::Transaction as _,
            node_bindings::Anvil,
            providers::ProviderBuilder,
            rpc::types::TransactionRequest,
        };

        let anvil = Anvil::new().spawn();
        let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
        let wallet_address = signer.address();
        let wallet = Arc::new(EthereumWallet::from(signer.clone()));
        let provider = ProviderBuilder::new()
            .wallet(EthereumWallet::from(signer))
            .connect_http(anvil.endpoint_url());

        // Bump the sender's nonce to 2 by mining two transactions.
        for _ in 0..2 {
            let tx = TransactionRequest::default()
                .to(anvil.addresses()[1])
                .value(U256::from(1));
            provider
                .send_transaction(tx)
                .await
                .unwrap()
                .get_receipt()
                .await
                .unwrap();
        }

        let task = SettlementTask {
            id: mk_job_id(1),
            job: mk_job(),
            tx_config: Arc::new(SettlementTransactionConfig::default()),
            provider: Arc::new(provider),
            store: Arc::new(MockStateStore::new()),
            wallet,
            control: mk_control(),
            attempts: BTreeMap::new(),
        };

        let (used_wallet, nonce, attempt_number, envelope) = task
            .build_next_attempt_with_new_nonce()
            .await
            .expect("attempt should build");

        assert_eq!(used_wallet, wallet_address);
        assert_eq!(nonce, Nonce(2));
        assert_eq!(attempt_number, SettlementAttemptNumber(0));
        assert_eq!(envelope.nonce(), 2);
        assert_eq!(envelope.to(), Some(mk_job().contract_address.into_alloy()));
        assert_eq!(envelope.chain_id(), Some(anvil.chain_id()));
        // Fees are within the configured bounds (defaults: floor 0, ceiling 100 gwei).
        assert!(envelope.max_fee_per_gas() <= 100_000_000_000);
    }
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo nextest run -p agglayer-settlement-service build_next_attempt_with_new_nonce_uses_pending`
Expected: FAIL — the stub `todo!()` panics (and the signature is not yet
`async`/`Result`, so it will also fail to compile against the test).

- [ ] **Step 3: Replace the stub implementation**

Replace:

```rust
    fn build_next_attempt_with_new_nonce(
        &self,
    ) -> (Address, Nonce, SettlementAttemptNumber, TxEnvelope) {
        // TODO: Build the next attempt with correct gas and other params. Use https://docs.rs/alloy/latest/alloy/rpc/types/struct.TransactionRequest.html#method.build
        // XREF: https://github.com/agglayer/agglayer/issues/1318
        todo!()
    }
```

with:

```rust
    async fn build_next_attempt_with_new_nonce(
        &self,
    ) -> Result<
        (Address, Nonce, SettlementAttemptNumber, TxEnvelope),
        RetryCallbackError<BuildAttemptError>,
    > {
        let wallet = self.wallet.default_signer().address();
        let attempt_number = self.next_attempt_number();

        crate::utils::retry_callback_until_success(
            &self.tx_config.retry_on_transient_failure,
            &self.control.cancellation_token,
            || async {
                let nonce = self.provider.get_transaction_count(wallet).pending().await?;
                let chain_id = self.provider.get_chain_id().await?;
                let estimate = self.provider.estimate_eip1559_fees().await?;
                let gas = self.resolve_base_gas_params(&estimate);
                let tx = self
                    .build_attempt(wallet, Nonce(nonce), chain_id, gas)
                    .await?;
                Ok((wallet, Nonce(nonce), attempt_number, tx))
            },
            BuildAttemptError::is_transient,
        )
        .await
    }
```

- [ ] **Step 4: Update the run-loop call site**

In `run`, replace lines ~404-406:

```rust
                let (wallet, nonce, attempt_number, tx) = self.build_next_attempt_with_new_nonce();
                self.save_attempt_to_db_and_submit_to_l1(wallet, nonce, attempt_number, tx)
                    .await;
```

with:

```rust
                let (wallet, nonce, attempt_number, tx) = retry!(
                    self.build_next_attempt_with_new_nonce().await,
                    "building next settlement attempt with a new nonce",
                );
                self.save_attempt_to_db_and_submit_to_l1(wallet, nonce, attempt_number, tx)
                    .await;
```

- [ ] **Step 5: Run the test to verify it passes**

Run: `cargo nextest run -p agglayer-settlement-service build_next_attempt_with_new_nonce_uses_pending`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/agglayer-settlement-service/src/settlement_task.rs
git commit -S -m "feat(settlement): build next attempt with a fresh nonce"
```

---

## Task 6: Implement `is_wallet_privkey_known` (optional adjacent win)

The wallet field makes this stub a one-liner; implementing it removes a
`todo!()` panic on the resubmission path.

**Files:**
- Modify: `crates/agglayer-settlement-service/src/settlement_task.rs`
  (replace stub ~536-539; test in `mod tests`)

- [ ] **Step 1: Write the failing test**

```rust
    #[test]
    fn is_wallet_privkey_known_reflects_registered_signers() {
        let signer = PrivateKeySigner::random();
        let known = signer.address();
        let wallet = Arc::new(EthereumWallet::from(signer));
        let unknown = Address::from([9; 20]);

        let task = mk_task_with_id(
            mk_job_id(1),
            Arc::new(MockStateStore::new()),
            BTreeMap::new(),
            wallet,
        );

        assert!(task.is_wallet_privkey_known(known));
        assert!(!task.is_wallet_privkey_known(unknown));
    }
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo nextest run -p agglayer-settlement-service is_wallet_privkey_known_reflects`
Expected: FAIL — the current stub is `todo!()` and panics.

- [ ] **Step 3: Replace the stub**

Replace:

```rust
    fn is_wallet_privkey_known(&self, _wallet: Address) -> bool {
        // TODO: tie with the configuration
        todo!()
    }
```

with:

```rust
    fn is_wallet_privkey_known(&self, wallet: Address) -> bool {
        self.wallet.signer_by_address(wallet).is_some()
    }
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo nextest run -p agglayer-settlement-service is_wallet_privkey_known_reflects`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/agglayer-settlement-service/src/settlement_task.rs
git commit -S -m "feat(settlement): resolve known wallets from the signer set"
```

---

## Final verification

- [ ] **Step 1: Run the verify skill's checks for the crate**

Run: `cargo make blast-radius` then the recommended commands. Expect
`affected_crates` to include `agglayer-settlement-service`; run:

```bash
cargo check --workspace --tests --all-features
cargo make ci-all
cargo nextest run -p agglayer-settlement-service
```

Expected: all PASS (format, clippy, typos, and tests).

- [ ] **Step 2: Confirm no XREF/TODO remains for 1318**

Run: `rg -n "issues/1318" crates/agglayer-settlement-service/src/settlement_task.rs`
Expected: no matches (the marker is removed with the implemented function).

---

## Notes for the implementer

- `default_signer().address()` and `signer_by_address()` are inherent
  `EthereumWallet` methods, used deliberately to avoid the `NetworkWallet<N>`
  trait-method ambiguity (`EthereumWallet: NetworkWallet<N>` for all `N`).
- `TransactionRequest`'s `.from/.to/.value/.input/.nonce/.gas_limit/.max_fee_per_gas/.max_priority_fee_per_gas`
  are inherent builder methods; `.with_chain_id` and `.build` come from the
  `TransactionBuilder` trait (imported as `_`).
- The `retry!` macro is defined inside `run`; it is in scope at the call site
  and converts a permanent `BuildAttemptError` into a `NonRecoverableError`
  panic, matching how `tx_hash_on_l1_for_nonce` is consumed.
- Do not implement issue 1319 (`build_next_attempt_with_nonce`) here; it will
  reuse `build_attempt` with a bumped `GasParams`.
