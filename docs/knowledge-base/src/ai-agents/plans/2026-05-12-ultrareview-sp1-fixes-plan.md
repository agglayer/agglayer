# Ultrareview SP1 Fixes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task.
> Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix the confirmed ultrareview SP1 v6 issues
while making unreadable pending proofs re-prove instead of blocking recovery.

**Architecture:** Use narrow compatibility handling at the pending store read boundary
for unreadable proof rows.
Keep SP1-heavy vkey work off async runtime threads
and make policy/error boundaries explicit without broad refactors.

**Tech Stack:** Rust, RocksDB-backed storage, SP1 SDK,
`prover_executor::sp1_fast`, `eyre`, `thiserror`,
repository cargo test workflows.

---

## File Structure

- Modify `crates/agglayer-storage/src/stores/pending/mod.rs`
  for unreadable generated proof handling in `get_proof` and `multi_get_proof`.
- Modify `crates/agglayer-storage/src/stores/pending/tests.rs`
  for pending proof decode behavior tests.
- Modify `crates/agglayer-storage/Cargo.toml` to move `sp1-sdk-v5` to dev-dependencies.
- Modify `crates/agglayer-sp1/src/error.rs` for serialization-specific error variants.
- Modify `crates/agglayer-sp1/src/ext.rs` for serialization errors,
  combined vkey hash helper,
  and explicit executable policy.
- Modify `crates/agglayer-types/src/aggchain_data/aggchain_proof.rs`
  if the payload needs a combined hash helper.
- Modify `crates/agglayer-aggregator-notifier/src/certifier/l1_context.rs` and `mod.rs`
  to offload vkey hashing and pass explicit policy.
- Modify `crates/pessimistic-proof-test-suite/src/runner.rs` for fallible `get_vkey`.

## Tasks

### Task 1: Pending Store Ignores Unreadable Proof Rows

- [ ] Write failing tests in `crates/agglayer-storage/src/stores/pending/tests.rs`
  that write invalid raw bytes to `ProofPerCertificateColumn`
  and assert `get_proof` returns `Ok(None)`.
- [ ] Add a failing multi-get test that writes one valid proof and one invalid proof
  and expects `[Some(_), None]`.
- [ ] Run `cargo test -p agglayer-storage pending::tests::get_proof_ignores_unreadable_pending_proof --lib`
  and confirm failure.
- [ ] Implement pending-store-only proof reads that map decode failures to `None`
  while preserving DB/key encoding errors.
- [ ] Run the two pending proof tests and confirm they pass.

### Task 2: SP1 Vkey Hashing Off Async Runtime

- [ ] Add or update tests around aggchain vkey hash helpers
  if existing tests do not cover combined hash output consistency.
- [ ] Add a combined helper that deserializes the vkey once
  and returns both `[u8; 32]` and `[u32; 8]`.
- [ ] Wrap certifier aggchain vkey hash computation in `sp1_fast` from `fetch_aggchain_proof_ctx`.
- [ ] Run the affected notifier certifier tests.

### Task 3: SP1 Error and Policy API Cleanup

- [ ] Add `SerializeSp1Proof` and `SerializeSp1Vkey` to `ProofError`.
- [ ] Map serialization failures in `current_sp1_stark_with_context` to the new variants.
- [ ] Change `executable_sp1` to accept `&AcceptancePolicy`
  and update certifier callers to pass `AcceptancePolicy::DEFAULT`.
- [ ] Run `cargo test -p agglayer-sp1`.

### Task 4: Fallible Test-Suite Vkey Setup

- [ ] Change `Runner::get_vkey` to return `eyre::Result<SP1VerifyingKey>`.
- [ ] Propagate `setup` errors with `map_err(|e| eyre!(e))`.
- [ ] Run `cargo test -p pessimistic-proof-test-suite --lib` if available.

### Task 5: Dependency Boundary and Verification

- [ ] Move `sp1-sdk-v5.workspace = true` from `agglayer-storage` dependencies to dev-dependencies.
- [ ] Run formatting for touched Rust files.
- [ ] Run targeted tests for touched crates.
- [ ] Run a workspace check or narrower cargo check based on compile time.

## Self-Review

The plan covers the approved storage policy,
the certifier async-runtime concern,
duplicate vkey deserialization,
recoverable setup panic,
serialization error precision,
executable policy explicitness,
and storage dependency boundary.
It intentionally avoids a destructive DB migration
because unreadable pending proofs should be lazily ignored and re-proven.
