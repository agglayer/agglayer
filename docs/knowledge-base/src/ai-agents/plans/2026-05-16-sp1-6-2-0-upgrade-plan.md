# SP1 6.2.0 Upgrade Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task.
> Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Upgrade the mandatory SP1 v6 dependency surface from `6.1.0` to `6.2.0`.

**Architecture:** Keep the upgrade as a dependency and compatibility change.
Update direct SP1 v6 pins, refresh dependency resolution, and make only compiler-required
source or fixture edits.
Preserve legacy v5 proof compatibility.

**Tech Stack:** Rust workspace, Cargo resolver, SP1 SDK/toolchain, Agglayer certificate proving.

---

## File Structure

- Modify `Cargo.toml` to update workspace SP1 v6 dependency pins.
- Modify `crates/pessimistic-proof-program/Cargo.toml` for standalone zkVM program pins.
- Modify `crates/pessimistic-proof-test-suite/aggchain-proof-ecdsa-example/program/Cargo.toml`
  for the ECDSA example zkVM program pin.
- Modify `Cargo.lock` by running Cargo dependency resolution.
- Modify Rust source or tests only if `6.2.0` API or metadata behavior requires it.

## Task 1: Update Manifest Pins

**Files:**

- Modify: `Cargo.toml`
- Modify: `crates/pessimistic-proof-program/Cargo.toml`
- Modify: `crates/pessimistic-proof-test-suite/aggchain-proof-ecdsa-example/program/Cargo.toml`

- [ ] **Step 1: Update workspace SP1 v6 pins**

In `Cargo.toml`, change the SP1 v6 dependency block from `=6.1.0` to `=6.2.0`.
Do not change `sp1-sdk-v5`.

```toml
# SP1 dependencies
# Note: Whenever these are updated, also consider updating the SP1 toolchain docker image version.
sp1-core-machine = "=6.2.0"
sp1-core-executor = "=6.2.0"
sp1-hypercube = "=6.2.0"
sp1-sdk = { version = "=6.2.0", features = ["blocking"] }
sp1-sdk-v5 = { package = "sp1-sdk", version = "=5.2.2" }
sp1-primitives = "=6.2.0"
sp1-prover = "=6.2.0"
sp1-zkvm = { version = "=6.2.0", default-features = false }
```

- [ ] **Step 2: Update pessimistic proof program pins**

In `crates/pessimistic-proof-program/Cargo.toml`, update the standalone SP1 pins.

```toml
sp1-zkvm = { version = "=6.2.0", features = ["verify"] }

[build-dependencies]
sp1-cli = "=6.2.0"
```

Keep the existing SP1 patch dependency tags unless Cargo resolution or SP1 release notes require
a different patch tag.
The existing comments state those patch tags cover the SP1 6.x line.

- [ ] **Step 3: Update ECDSA example program pin**

In `crates/pessimistic-proof-test-suite/aggchain-proof-ecdsa-example/program/Cargo.toml`,
update `sp1-zkvm`.

```toml
sp1-zkvm = "=6.2.0"
```

Keep the existing SP1 patch dependency tags unless Cargo resolution fails.

## Task 2: Refresh Dependency Resolution

**Files:**

- Modify: `Cargo.lock`

- [ ] **Step 1: Resolve the new SP1 dependency graph**

Run this from the repository root.

```bash
cargo update \
  -p sp1-core-machine --precise 6.2.0 \
  -p sp1-core-executor --precise 6.2.0 \
  -p sp1-sdk --precise 6.2.0 \
  -p sp1-primitives --precise 6.2.0 \
  -p sp1-prover --precise 6.2.0 \
  -p sp1-zkvm --precise 6.2.0 \
  -p sp1-cli --precise 6.2.0
```

Expected result: Cargo updates `Cargo.lock` and keeps `sp1-sdk 5.2.2` for legacy v5 support.

- [ ] **Step 2: Handle ambiguous package names if Cargo asks for disambiguation**

If Cargo reports multiple packages with the same name, rerun with package specs that include
the exact package ID shown by Cargo.
Keep the intended result the same: SP1 v6 packages at `6.2.0`, legacy `sp1-sdk 5.2.2` unchanged.

- [ ] **Step 3: Inspect the lockfile result**

Run read-only checks.

```bash
git diff -- Cargo.toml Cargo.lock crates/pessimistic-proof-program/Cargo.toml crates/pessimistic-proof-test-suite/aggchain-proof-ecdsa-example/program/Cargo.toml
```

Expected result: direct SP1 v6 pins and resolved SP1 v6 lockfile entries move to `6.2.0`.
No unrelated dependency families should change unless pulled by SP1 6.2.0.

## Task 3: Compile Affected Crates

**Files:**

- Read/modify Rust source only if compilation fails because of SP1 6.2.0 API changes.

- [ ] **Step 1: Check the SP1 facade crate**

Run:

```bash
cargo check -p agglayer-sp1
```

Expected result: PASS.
If it fails due to renamed SP1 types or serialization API changes, update the failing imports or
type names in `crates/agglayer-sp1/src/` with the smallest compatible change.

- [ ] **Step 2: Check certificate proving caller**

Run:

```bash
cargo check -p agglayer-aggregator-notifier
```

Expected result: PASS.
If it fails around `ProverClient`, `EnvProver`, `SP1Stdin`, `SP1ProofWithPublicValues`, or
`SP1VerificationError`, update `crates/agglayer-aggregator-notifier/src/certifier/mod.rs` only.

- [ ] **Step 3: Check proof libraries**

Run:

```bash
cargo check -p pessimistic-proof -p pessimistic-proof-core -p pessimistic-proof-test-suite
```

Expected result: PASS.
If it fails in test-suite helper code, keep changes local to the helper or test-suite crate.

- [ ] **Step 4: Check node integration build surface**

Run:

```bash
cargo check -p agglayer-node
```

Expected result: PASS.

## Task 4: Update Mandatory Test Fixtures Only If Required

**Files:**

- Modify: `crates/agglayer-types/src/certificate/testutils.rs` only if compiler or tests require it.
- Modify: `crates/agglayer-sp1/tests/policy.rs` only if compiler or tests require it.
- Modify: `crates/agglayer-sp1/tests/version.rs` only if compiler or tests require it.
- Modify: `crates/agglayer-grpc-types/src/compat/v1/tests.rs` only if compiler or tests require it.

- [ ] **Step 1: Run focused SP1 tests**

Run:

```bash
cargo test -p agglayer-sp1
```

Expected result: PASS.

- [ ] **Step 2: Update generated mock metadata only on failure**

If tests fail because newly generated v6 mock proofs now carry or require `v6.2.0`, update helper
calls that create new v6 SP1 proof metadata from this:

```rust
let mut sp1 = v6_sp1_stark_with_context(proof.as_ref(), &vkey, "v6.1.0").unwrap();
```

to this:

```rust
let mut sp1 = v6_sp1_stark_with_context(proof.as_ref(), &vkey, "v6.2.0").unwrap();
```

Do not update tests whose purpose is to prove that `v6.1.0` remains classified as SP1 v6.

- [ ] **Step 3: Re-run focused SP1 tests**

Run:

```bash
cargo test -p agglayer-sp1
```

Expected result: PASS.

## Task 5: Final Verification

**Files:**

- No planned source changes.

- [ ] **Step 1: Run targeted workspace checks**

Run:

```bash
cargo check -p agglayer-sp1 -p agglayer-aggregator-notifier -p pessimistic-proof -p pessimistic-proof-core -p agglayer-node
```

Expected result: PASS.

- [ ] **Step 2: Run focused tests**

Run:

```bash
cargo test -p agglayer-sp1
```

Expected result: PASS.

- [ ] **Step 3: Build the knowledge base**

Run:

```bash
mdbook build docs/knowledge-base/
```

Expected result: PASS.

- [ ] **Step 4: Review the final diff**

Run:

```bash
git diff --stat
git diff -- Cargo.toml Cargo.lock crates/pessimistic-proof-program/Cargo.toml crates/pessimistic-proof-test-suite/aggchain-proof-ecdsa-example/program/Cargo.toml
```

Expected result: changes are limited to mandatory SP1 6.2.0 upgrade files plus any compiler-required
Rust compatibility edits.

## Task 6: Commit Only With Explicit Approval

**Files:**

- No file changes.

- [ ] **Step 1: Ask before staging**

This repository's `AGENTS.md` prohibits non-read-only git operations without explicit approval.
Do not run `git add`, `git commit`, or other write operations unless the user explicitly requests it.

- [ ] **Step 2: If approved, use the repository commit workflow**

Load the repository `commit` skill before committing.
Use the final verified diff to create a concise commit message focused on the SP1 6.2.0 upgrade.
