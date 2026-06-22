# Settlement attempt builder (issue 1318) — design

- Issue: <https://github.com/agglayer/agglayer/issues/1318>
- Related: 1319 (gas bump), 1321 (submit to L1), 1320 (save attempt), 1381
  (persist job), 1314 (wait for nonce on L1), 1382 (current result on L1)
- Status: approved design, pre-implementation
- Date: 2026-06-08

## Context

The settlement service runs one `SettlementTask` per settlement job
(`crates/agglayer-settlement-service/src/settlement_task.rs`).
The task's `run` loop submits and tracks L1 settlement transactions,
keying attempts by `(wallet Address, Nonce)`.

Several leaf helpers are still `todo!()` stubs, each tracked by its own issue.
This spec covers the builder marked by
`// XREF: https://github.com/agglayer/agglayer/issues/1318`,
currently:

```rust
fn build_next_attempt_with_new_nonce(
    &self,
) -> (Address, Nonce, SettlementAttemptNumber, TxEnvelope) {
    todo!()
}
```

where `type TxEnvelope = EthereumTxEnvelope<TxEip4844Variant>` (a *signed*
envelope).
Its sibling `build_next_attempt_with_nonce` (issue 1319) bumps gas for a
retry on an existing nonce; submission (issue 1321) is separate.

The issue title — "build a settlement tx for a given job with a given nonce
and gas parameters" — describes a generic builder primitive,
while the XREF marker sits on the *new-nonce* function.
We reconcile this by implementing both:
a reusable primitive plus the new-nonce orchestration that calls it.

### Key constraints discovered during exploration

1. `SettlementTask` has **no signer**, and the crate does not depend on any
   signer crate.
   But the function must return a *signed* `TxEnvelope`;
   its `tx.tx_hash()` is what the caller records to RocksDB before submission
   (`save_attempt_to_db_and_submit_to_l1`, `settlement_task.rs:449`).
2. The stub signature cannot work as written.
   alloy's `TransactionBuilder::build(wallet)` is `async`, returns `Result`,
   and requires a `NetworkWallet`.
   The current function is synchronous and infallible.
3. The service builds the envelope **before** submitting,
   so it cannot rely on alloy's send-time fillers (nonce/gas/fee);
   it must resolve those values itself.

## Goals

- Implement a reusable primitive that builds and signs an EIP-1559 settlement
  transaction for `self.job` given a wallet, nonce, chain id, and resolved gas
  parameters.
- Implement `build_next_attempt_with_new_nonce` to select a wallet,
  resolve a fresh nonce and base gas parameters, allocate an attempt number,
  and call the primitive.
- Give `SettlementTask` access to a signing wallet.
- Build only; do not submit (submission is issue 1321).

## Non-goals (out of scope)

- Loading signing keys from configuration (keystore / GCP KMS wiring).
  The task receives an already-constructed `EthereumWallet`.
- Multi-wallet selection policy beyond "use the default signer".
- The per-retry gas bump (issue 1319).
- Persisting the attempt to RocksDB (issue 1320) and submission (issue 1321).
- Integrating the settlement service into the node
  (it is still `#![allow(dead_code)]`).

## Decisions (from brainstorming)

1. **Scope**: build a reusable primitive `build_attempt(...)` and wire it into
   `build_next_attempt_with_new_nonce`.
2. **Signer source**: add an `EthereumWallet` field to `SettlementTask`.
   alloy's `EthereumWallet` is itself a multi-signer container keyed by address
   and implements `NetworkWallet<Ethereum>`;
   `agglayer-signer`'s `ConfiguredSigner` (local keystore or GCP KMS)
   implements `TxSigner<Signature>`, so it plugs in via
   `EthereumWallet::from(signer)`.
3. **Gas handling**: align with the existing convention in
   `agglayer-contracts` (`adjust_gas_estimate`, `settler.rs`),
   adapted to the new per-field config and the job's explicit gas limit;
   use a small local helper rather than depending on `agglayer-contracts`.

## Design

### Data types

```rust
/// Fully-resolved gas parameters for a single settlement attempt.
struct GasParams {
    gas_limit: u128,
    max_fee_per_gas: u128,
    max_priority_fee_per_gas: u128,
}

/// Error surfaced while building a settlement attempt.
#[derive(Debug, thiserror::Error)]
enum BuildAttemptError {
    #[error("L1 RPC error while building settlement attempt: {0}")]
    Transport(#[from] TransportError),
    #[error("failed to build/sign settlement transaction: {0}")]
    Build(TransactionBuilderError<Ethereum>),
}

/// Retry policy for building a settlement attempt. Transport failures retry
/// while transient; opaque signer-backend failures retry a bounded number of
/// times (to ride out a transient remote-signer blip without looping forever
/// on a permanent misconfiguration); structural build errors are permanent.
struct BuildRetryPolicy {
    signer_failures: u32,
}

impl BuildRetryPolicy {
    const MAX_SIGNER_BUILD_RETRIES: u32 = 3;

    fn should_retry(&mut self, error: &BuildAttemptError) -> bool {
        match error {
            BuildAttemptError::Transport(error) => crate::utils::is_transient_alloy_error(error),
            error if error.is_signer_backend() => {
                self.signer_failures += 1;
                self.signer_failures <= Self::MAX_SIGNER_BUILD_RETRIES
            }
            BuildAttemptError::Build(_) => false,
        }
    }
}
```

### Task field and constructor threading

Add a signing wallet to the task and the service:

```rust
pub struct SettlementTask<L1Provider, SettlementStore> {
    // ...existing fields...
    wallet: Arc<EthereumWallet>,
}
```

- `SettlementTask::create` and `SettlementTask::load` gain a
  `wallet: Arc<EthereumWallet>` parameter and store it.
- `SettlementService` gains a `wallet: Arc<EthereumWallet>` field;
  `SettlementService::start` gains a corresponding parameter and passes the
  wallet into `SettlementTask::create`.
- Optional adjacent win: implement the existing `is_wallet_privkey_known`
  stub as `self.wallet.has_signer_for(&wallet)`
  (a one-liner enabled by the new field; removes a `todo!()` panic).
  Flagged as optional; include only if it keeps the change focused.

### Primitive: `build_attempt`

Pure builder (async only because signing is async); no RPC calls.

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

Setting both `max_fee_per_gas` and `max_priority_fee_per_gas`
(and no blob fields) yields an EIP-1559 envelope (`TxEnvelope::Eip1559`).
The `wallet` argument sets the `from` field and must have a registered signer
in `self.wallet`; otherwise `build` fails with a signer-missing error.

### Orchestration: `build_next_attempt_with_new_nonce`

```rust
async fn build_next_attempt_with_new_nonce(
    &self,
) -> Result<
    (Address, Nonce, SettlementAttemptNumber, TxEnvelope),
    RetryCallbackError<BuildAttemptError>,
> {
    let wallet = self.wallet.default_signer_address();
    let attempt_number = self.next_attempt_number();

    crate::utils::retry_callback_until_success(
        &self.tx_config.retry_on_transient_failure,
        &self.control.cancellation_token,
        || async {
            let nonce = self
                .provider
                .get_transaction_count(wallet)
                .pending()
                .await?;
            let chain_id = self.provider.get_chain_id().await?;
            let estimate = self.provider.estimate_eip1559_fees().await?;
            let gas = self.resolve_base_gas_params(&estimate);
            let tx = self
                .build_attempt(wallet, Nonce(nonce), chain_id, gas)
                .await
                .map_err(BuildAttemptError::Build)?;
            Ok((wallet, Nonce(nonce), attempt_number, tx))
        },
        BuildAttemptError::is_transient,
    )
    .await
}
```

- **Wallet selection**: `self.wallet.default_signer_address()`.
  Trivial in the single-wallet case; richer selection is future work.
- **Nonce**: `get_transaction_count(wallet).pending()`.
- **Chain id**: `get_chain_id()`.
- **Fees**: `estimate_eip1559_fees()`, then adjusted (see below).
- **Attempt number**: `next_attempt_number()` =
  highest attempt number currently in `self.attempts` plus one,
  or `0` when there are none.
  Computed once outside the retry closure (it does not depend on RPC).
- The whole resolve-and-sign body is one retryable closure:
  transient transport errors are retried with
  `retry_on_transient_failure`;
  a build/sign error is non-transient and surfaces as
  `RetryCallbackError::Error`.
  Re-resolving nonce/fees on retry is intentional and keeps a consistent
  snapshot.

### Gas resolution (base attempt)

Mirrors `agglayer-contracts::adjust_gas_estimate`,
using the new per-field config in `SettlementTransactionConfig`:

```rust
fn resolve_base_gas_params(&self, estimate: &Eip1559Estimation) -> GasParams {
    let cfg = &self.tx_config;

    let max_fee_per_gas = clamp_fee(
        apply_multiplier(estimate.max_fee_per_gas, cfg.max_fee_per_gas_multiplier_factor),
        cfg.max_fee_per_gas_floor,
        cfg.max_fee_per_gas_ceiling,
    );
    let max_priority_fee_per_gas = clamp_fee(
        apply_multiplier(
            estimate.max_priority_fee_per_gas,
            cfg.max_priority_fee_per_gas_multiplier_factor,
        ),
        cfg.max_priority_fee_per_gas_floor,
        cfg.max_priority_fee_per_gas_ceiling,
    );

    let gas_limit = apply_multiplier(self.job.gas_limit, cfg.gas_limit_multiplier_factor)
        .min(u128::try_from(cfg.gas_limit_ceiling).unwrap_or(u128::MAX));

    GasParams { gas_limit, max_fee_per_gas, max_priority_fee_per_gas }
}
```

- `apply_multiplier(value, m)` = `value.saturating_mul(m.as_u64_per_1000() as u128) / 1000`,
  matching the legacy `adjust` closure.
- `clamp_fee(v, floor, ceiling)` = `v.max(floor).min(ceiling)`.
- **gas_limit base** is `self.job.gas_limit` (the orchestrator-provided
  estimate), not an RPC `estimate_gas` call;
  this is the deliberate adaptation from the legacy RPC-estimate path,
  since the new `SettlementJob` carries an explicit gas limit.
- Defaults (`multiplier = 1.0`, `floor = 0`, `gas_limit_ceiling = 60_000_000`,
  fee ceilings = 100 gwei) make the base attempt
  `gas_limit = min(job.gas_limit, ceiling)` and
  `fee = clamp(estimate, 0, ceiling)`.

#### Judgment calls (intentional deviations from legacy)

1. `max_priority_fee_per_gas` is clamped to **both** `[floor, ceiling]`;
   legacy applied only `.min(ceiling)` to priority,
   which looks like an oversight now that the config has an explicit priority
   floor.
2. At the base/new-nonce attempt the per-field multiplier is applied **once**
   (default `1.0` ⇒ no change).
   Per-retry compounding belongs to issue 1319.

### Signature and call-site changes

- `build_next_attempt_with_new_nonce` becomes `async` and fallible
  (see signature above).
- The call site (`settlement_task.rs:404`) changes from a direct call to:

  ```rust
  let (wallet, nonce, attempt_number, tx) = retry!(
      self.build_next_attempt_with_new_nonce().await,
      "building next settlement attempt with a new nonce",
  );
  ```

  This reuses the existing `retry!` macro,
  which returns early on `Cancelled` and panics with a `NonRecoverableError`
  on a permanent error — consistent with how `tx_hash_on_l1_for_nonce` is used.
- `BuildAttemptError` derives `thiserror::Error` so the macro's
  `eyre::Error::from(error)` works.

### New imports / APIs

- `alloy::network::{Ethereum, EthereumWallet, TransactionBuilder, TransactionBuilderError}`
- `alloy::rpc::types::TransactionRequest`
- `alloy::eips::eip1559::Eip1559Estimation`
- `Provider` methods: `get_transaction_count(addr).pending()`,
  `get_chain_id()`, `estimate_eip1559_fees()`.
- Conversions: `agglayer_types::Address::into_alloy()` (already used at
  `settlement_task.rs:798`); `job.calldata` is already `alloy::primitives::Bytes`.

## Error handling

- Transient L1 RPC failures (nonce/chain id/fees) are retried in-place using
  the configured `retry_on_transient_failure` policy and the existing
  `retry_callback_until_success` helper.
- Opaque signer-backend failures (`TransactionBuilderError::Signer(Error::Other)`,
  how a remote signer such as GCP KMS surfaces a signing error) are retried a
  **bounded** number of times (`MAX_SIGNER_BUILD_RETRIES`). This rides out a
  transient KMS blip without panicking the task, yet a permanent signer
  misconfiguration (e.g. bad KMS permissions) stops retrying after the bound and
  surfaces through the non-recoverable path instead of looping until
  cancellation. The transient-vs-permanent signer distinction is opaque here;
  a precise classification belongs in the signer backend (follow-up).
- Structural signer errors (unsupported operation, chain-id mismatch, signature
  errors) and other build failures (`InvalidTransactionRequest`,
  `UnsupportedSignatureType`) are non-recoverable immediately: they propagate as
  `RetryCallbackError::Error` and become a `NonRecoverableError` panic at the
  call site, matching the run-loop's handling of permanent errors.

## Testing strategy

Unit tests in `settlement_task.rs` using an Anvil-backed wallet provider
(the pattern already used in `utils.rs` tests:
`Anvil::new().spawn()` + `ProviderBuilder::new().wallet(EthereumWallet::from(signer))`).

- `build_attempt` produces a signed EIP-1559 envelope with the expected
  `to`, `value`, `input`, `nonce`, `chain_id`, and recovered signer address.
- `build_next_attempt_with_new_nonce` returns the wallet's default signer
  address, a nonce equal to the account's pending transaction count,
  and an attempt number one past the highest existing attempt.
- `resolve_base_gas_params` clamps fees to the configured floor/ceiling and
  caps `gas_limit` at `gas_limit_ceiling`,
  and applies the multiplier (use a non-default config to assert scaling).
- Existing tests (`write_job_result_*`, `load_settlement_attempts_*`) are
  updated only to pass a wallet into the task test helpers
  (`mk_task`, `mk_task_with_id`, `mk_service`);
  a wallet built from a throwaway `PrivateKeySigner` is sufficient since those
  tests do not build transactions.

## Dependencies and follow-ups

- A dependency on `agglayer-signer` is **not** required for this issue:
  the task accepts an `EthereumWallet` directly,
  and tests construct it from `alloy-signer-local`'s `PrivateKeySigner`.
  Wiring `ConfiguredSigner` from config is a later integration step.
- `dev-dependencies` already enable `alloy` `node-bindings` for Anvil.
- Downstream issues consume this builder: 1320 (save attempt),
  1321 (submit), 1319 (gas bump reuses `build_attempt` + a bumped `GasParams`).

## Open assumptions

- The default signer of the task's `EthereumWallet` is an acceptable wallet
  for a new-nonce attempt (single-wallet deployments).
- `self.job.gas_limit` is the authoritative base for the gas limit,
  shaped only by `gas_limit_multiplier_factor` and `gas_limit_ceiling`.
- Attempt numbers are unique per job and monotonically allocated from
  `self.attempts`; coordination with in-memory insertion on save is owned by
  issue 1320.
