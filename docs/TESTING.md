# Testing guide

This document describes conventions, tools, and patterns
for writing tests in the Agglayer repository.

## Test tiers

The project uses a three-tier testing strategy.
Each tier has a distinct purpose and tradeoff profile.

### Unit tests

Located inline (`#[cfg(test)] mod tests`) or in sibling `tests.rs` files.
Run with:

```sh
cargo nextest run --workspace
```

Use unit tests to verify individual functions and error paths in isolation.
They should be fast (< 1 second each) and deterministic.

### Integration tests

Located in `tests/integrations/` as a separate workspace member.
Run with:

```sh
cargo nextest run --workspace -P integrations
```

These tests spin up a Docker-based L1 node and a full Agglayer node,
exercising end-to-end flows such as certificate settlement and storage backups.
They run serially (one at a time) due to shared resource constraints.

### End-to-end tests

Managed externally via Kurtosis CDK.
Triggered in CI on merge queue or manual dispatch.
See `docs/knowledge-base/e2e-tests.md` for setup instructions.

## Test runner: nextest

All tests **must** pass under
[cargo-nextest](https://nexte.st/).
Some tests (notably in `agglayer-jsonrpc-api`) explicitly assert
that the `NEXTEST` environment variable is set,
because nextest provides per-test process isolation
that avoids port-binding and concurrency issues.

Run doc tests separately (nextest does not support them):

```sh
cargo test --doc --workspace
```

## Naming conventions

- Use descriptive names that express the behavior under test.
- Avoid the `test_` prefix; the `#[test]` attribute already identifies tests.
- Name pattern: `<scenario_under_test>` or `<action>_<expected_outcome>`.

Good:

```rust
#[test]
fn returns_error_on_duplicate_nullifier() { .. }

#[tokio::test]
async fn certificate_settlement_retries_on_l1_failure() { .. }
```

Avoid:

```rust
#[test]
fn test_thing() { .. }
```

## Test structure: arrange / act / assert

Every test should follow three clear phases:

1. **Arrange** -- set up inputs, mocks, and preconditions.
2. **Act** -- call the function or method under test.
3. **Assert** -- verify the outcome.

Keep each test focused on **one behavior**.
If a test grows beyond ~50 lines, consider splitting it
or extracting setup into a helper function.

## Frameworks and when to use them

| Crate | Purpose | When to reach for it |
|---|---|---|
| `rstest` | Parameterized tests, fixtures | Testing multiple input variants or sharing setup |
| `insta` | Snapshot testing | Serialization formats, API responses, error messages |
| `mockall` | Trait mocking | Isolating a component from its dependencies |
| `fail` | Fault injection (failpoints) | Simulating timeouts, disconnections, panics |
| `bolero` | Fuzz / property-based testing | Parsers, serialization round-trips, crypto inputs |
| `test-log` | Tracing in tests | Any async test where log output aids debugging |

### Snapshot testing guidelines (insta)

Snapshot tests are appropriate for:

- JSON-RPC response formats
- Protobuf / bincode serialization stability
- Error message rendering (`Display`, `Debug`)
- Configuration serialization

Review snapshot changes carefully in PRs.
Run `cargo insta review` to accept or reject changes interactively.

### Failpoint guidelines (fail crate)

Failpoints allow injecting faults at specific code locations:

```rust
// In production code:
fail::fail_point!("certifier::before_verifying_proof", |_| {
    Err(CertificationError::InternalError("injected".into()))
});

// In test code:
let scenario = FailScenario::setup();
fail::cfg("certifier::before_verifying_proof", "return()").unwrap();
// ... run test ...
scenario.teardown();
```

Use failpoints for:

- Simulating prover timeouts
- Testing reconnection logic (e.g., block clock disconnection)
- Bypassing expensive proof verification in integration tests

## The `testutils` feature pattern

Many crates expose a `testutils` Cargo feature
that gates test helper code for cross-crate reuse.
This avoids polluting production builds while sharing test infrastructure.

```toml
# In the providing crate's Cargo.toml:
[features]
testutils = []

# In the consuming crate's Cargo.toml:
[dev-dependencies]
agglayer-storage = { path = "../agglayer-storage", features = ["testutils"] }
```

Crates that currently expose `testutils`:

- `agglayer-types` -- `Certificate::new_for_test`, wallet helpers
- `agglayer-storage` -- `TempDBDir`, mock stores (`MockStateStore`, etc.)
- `agglayer-config` -- test config builders
- `agglayer-clock` -- clock test helpers
- `agglayer-contracts` -- contract test helpers
- `agglayer-aggregator-notifier` -- `MockL1Rpc`
- `pessimistic-proof` / `pessimistic-proof-core` -- proof test helpers

When adding new test helpers that other crates will need,
put them behind the `testutils` feature in the appropriate crate
rather than duplicating code across test modules.

## Common test helpers

| Helper | Location | Purpose |
|---|---|---|
| `TempDBDir` | `agglayer-storage::tests` | RAII temp directory for DB tests |
| `MockStateStore` et al. | `agglayer-storage::tests::mocks` | Mock store traits via `mockall` |
| `TestContext` | `agglayer-jsonrpc-api::testutils` | Full RPC server test harness |
| `Certificate::new_for_test` | `agglayer-types` | Deterministic test certificates |
| `Forest` | `pessimistic-proof-test-suite` | Test data builder for proofs |

## Anti-patterns to avoid

### Hardcoded sleeps for synchronization

Do not use `tokio::time::sleep` to wait for async operations to complete.
Instead use:

- `CancellationToken` or shutdown signals for server lifecycle
- `tokio::time::pause()` / `advance()` for deterministic timing in time-sensitive tests
- Channel receives or condition variables for event-based synchronization

If a sleep is genuinely necessary (e.g., waiting for an external process),
document the reason clearly in a comment.

### Duplicated mock definitions

If the same `mockall::mock!` block appears in more than one file,
extract it into a shared `testutils` module gated behind the `testutils` feature.

### Tests that verify multiple behaviors

Each test should assert one logical behavior.
Long tests (> 80 lines) that verify multiple outcomes
make failures harder to diagnose and are more brittle.

### Undocumented `#[ignore]`

Always provide a reason:

```rust
#[ignore = "requires external L1 endpoint"]
```

## Coverage

Coverage runs in CI using `grcov` with LLVM instrumentation.
Results are uploaded to Codecov.

To run coverage locally:

```sh
cargo nextest run --workspace -F coverage --cargo-profile codecov
```

## Fuzz testing

Fuzz tests use the `bolero` crate and run via `cargo-bolero`:

```sh
cargo make fuzz-all
```

The default fuzz duration is 60 seconds per target.
Fuzz targets live alongside their unit tests
and are gated with `#[cfg_attr(not(kani), test)]` or similar.
