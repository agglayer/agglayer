# SP1 6.2.0 Upgrade Design

## Goal

Upgrade the mandatory SP1 v6 dependency surface from `6.1.0` to `6.2.0`.
Keep the change focused on build and runtime compatibility for Agglayer certificate proving.

## Scope

The upgrade covers SP1 v6 crates that are pinned directly in the workspace and standalone
SP1 program crates.
The legacy `sp1-sdk-v5` dependency remains pinned to `5.2.2` because it supports
existing v5 proof compatibility.

The implementation should update all mandatory `6.1.0` pins that participate in building,
executing, verifying, or proving SP1 v6 artifacts.
It should not replace compatibility tests that intentionally check generic v6 version handling
unless compilation or test behavior requires it.

## Affected Areas

- Root workspace SP1 dependencies in `Cargo.toml`.
- Standalone SP1 program manifests under `crates/pessimistic-proof-program/` and
  `crates/pessimistic-proof-test-suite/aggchain-proof-ecdsa-example/program/`.
- `Cargo.lock`, after dependency resolution.
- Test helpers or expected metadata only when `6.2.0` compatibility requires it.

## Approach

Use the mandatory-only approach.
Update direct SP1 v6 pins to `=6.2.0`, refresh the lockfile, and run targeted checks.
If the compiler or tests identify API changes, make the smallest code changes needed to restore
the current behavior.

Avoid broad textual replacement of every `v6.1.0` string.
Some literal strings are compatibility fixtures that prove the code accepts v6 proof metadata,
not runtime dependency declarations.

## Verification

Run targeted Rust checks for crates that depend on SP1 v6.
At minimum, verify the workspace dependency graph resolves and affected crates compile.
Run focused `agglayer-sp1` and certificate proving tests if compilation succeeds.

## Risks

SP1 upgrades can affect proof encoding, verifier behavior, and prover-service compatibility.
The blast radius includes certificate proving, local SP1 execution, proof serialization,
and e2e settlement flows that depend on generated proofs.

The deployment image may also need an SP1 toolchain update.
The root manifest already calls this out, so implementation should preserve or update that
operator guidance if required by `6.2.0`.
