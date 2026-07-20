# Remove direct SP1 SDK dependencies from `agglayer-types`

- **Issue:** [#1524](https://github.com/agglayer/agglayer/issues/1524)
  (parent epic [#1541](https://github.com/agglayer/agglayer/issues/1541),
  "SP1v6 migration - Followup tickets")
- **Date:** 2026-06-08
- **Status:** Approved design, pending implementation plan

## Problem

`agglayer-types` depends directly on four external SP1 SDK crates.
The `agglayer-sp1` facade was introduced precisely so that crates outside the
pessimistic-proof family route all SP1 access through it and avoid direct
`sp1-*` dependencies.
This intent is documented in `agglayer-sp1/src/ext.rs:14-16`:
callers like `agglayer-types` "must not depend on `sp1-sdk` directly".

`agglayer-types` is not yet compliant.
This task makes it compliant without changing any behavior.

### Current SP1-related dependencies in `agglayer-types`

| Dependency | Where used | Disposition |
|------------|------------|-------------|
| `agglayer-sp1` (internal facade) | `aggchain_data/aggchain_proof.rs` (always) and testutils | Keep — it is the abstraction layer |
| `sp1-sdk` | only `certificate/testutils.rs` (v6 mock proof) | Remove (relocate behind facade) |
| `sp1-sdk-v5` (optional) | only `certificate/testutils.rs` (v5 mock proof) | Remove (relocate behind facade) |
| `sp1-core-machine` | no source references | Remove (dead dependency) |
| `sp1-prover` | no source references | Remove (dead dependency) |

No external SP1 type appears in any `agglayer-types` public signature.
The only SP1-family type on its public surface is `agglayer_sp1::ProofError`
(the facade's own type), which is unaffected by this change.

## Goal

`agglayer-types` reaches SP1 functionality only through `agglayer-sp1`.
After this change its manifest contains no `sp1-sdk`, `sp1-sdk-v5`,
`sp1-core-machine`, or `sp1-prover` entry.

## Non-goals

- Full removal of the legacy v5 SDK path.
  That is sibling issue [#1511](https://github.com/agglayer/agglayer/issues/1511);
  here the v5 mock path stays functional, only relocated.
- Any refactor unrelated to SP1 dependency hygiene.
- Changing the public API of `agglayer-types` or `agglayer-sp1`.

## Approach

Move the test-only mock-proof builder that still touches `sp1-sdk` and
`sp1-sdk-v5` into the facade's existing `testutils` module, then re-export it so
every current consumer keeps working unchanged.
This was chosen over keeping orchestration in `agglayer-types` with finer-grained
facade helpers, because the version-classification logic (`version_kind`) already
lives in `agglayer-sp1`, so splitting the builder would be artificial and add
surface area.

### 1. Relocate the mock-proof builder into `agglayer-sp1`

Move two items verbatim from `agglayer-types/src/certificate/testutils.rs` into
`agglayer-sp1/src/testutils.rs`:

- `pub fn dummy_sp1_stark_proof_with_version(version: &str) ->
  agglayer_interop_types::aggchain_proof::Proof`
- its private helper `fn sp1_stark_with_context(...)`,
  used only by the v5 arm.

The private helper stays separate from the existing
`agglayer_sp1::v6_sp1_stark_with_context`, because the latter calls
`ensure_v6_writable` and therefore rejects v5 version strings.

Every dependency the builder needs already exists inside `agglayer-sp1`:
`version_kind` / `Sp1ProofVersion`, `v6_sp1_stark_with_context` /
`V6Sp1StarkProof`, the `EMPTY_ELF` / `EMPTY_ELF_V5` constants, `sp1-sdk`,
`sp1-sdk-v5`, and `agglayer-interop-types` (including `bincode`).
The v5 path's OS-thread-spawn workaround (which avoids nesting a tokio runtime
inside async tests) moves unchanged, so behavior is identical.

The builder lives under the module already gated by
`#[cfg(any(test, feature = "testutils"))]`, so it never enters production builds.

### 2. Rewire `agglayer-types` while preserving the public path

- `certificate/testutils.rs`:
  delete the two moved items;
  `create_dummy_stark_proof()` calls
  `agglayer_sp1::testutils::dummy_sp1_stark_proof_with_version("test")`;
  drop the now-unused `EMPTY_ELF` / `EMPTY_ELF_V5` import and all `sp1_sdk*`
  imports.
- `certificate/mod.rs`:
  re-export the helper from the facade instead of the local module, under the
  `testutils` feature gate, while keeping `compute_signature_info` re-exported
  from the local module.
- `lib.rs`:
  unchanged.
  `crate::certificate::dummy_sp1_stark_proof_with_version` and
  `EMPTY_ELF` / `EMPTY_ELF_V5` still resolve, so the public testutils surface is
  identical.

All current consumers reach the helper through
`agglayer_types::testutils::dummy_sp1_stark_proof_with_version` and the ELF
constants through `agglayer_types::testutils::EMPTY_ELF*`:
`agglayer-storage`, `agglayer-grpc-api`, `agglayer-grpc-types`, and
`agglayer-types`' own tests.
None of them need edits.

### 3. Manifest changes

`agglayer-types/Cargo.toml`:

- remove `sp1-sdk`, `sp1-sdk-v5`, `sp1-core-machine`, and `sp1-prover` from
  `[dependencies]`;
- in `[features].testutils`, drop `dep:sp1-sdk-v5` while keeping
  `agglayer-sp1/testutils`, `dep:arbitrary`, `pessimistic-proof/testutils`, and
  `agglayer-interop-types/testutils`;
- keep `agglayer-sp1.workspace = true`.

`agglayer-sp1/Cargo.toml`:
no change.
It already depends on `sp1-sdk`, `sp1-sdk-v5`, and `agglayer-interop-types`, and
already has a `testutils` feature.

## Verification

- `cargo check -p agglayer-types` with no features,
  proving SP1 SDK crates are not needed in normal builds.
- `cargo check -p agglayer-types --features testutils` and
  `cargo check -p agglayer-sp1 --features testutils`.
- Build and test the downstream consumers:
  `agglayer-storage`, `agglayer-grpc-api`, `agglayer-grpc-types`.
- `cargo machete` to confirm `sp1-core-machine` and `sp1-prover` are gone and no
  new unused dependency was introduced.
- Run the repository Definition-of-Done checks before any commit.

## Blast radius

Low.
The public API of `agglayer-types` is unchanged; only the test-helper location
moves, hidden behind existing re-exports.
Files touched:

- `agglayer-sp1/src/testutils.rs` (gain the builder)
- `agglayer-types/Cargo.toml`
- `agglayer-types/src/certificate/testutils.rs`
- `agglayer-types/src/certificate/mod.rs`
