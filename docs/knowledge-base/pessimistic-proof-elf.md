# Pessimistic proof ELF and verification key

## Overview

The pessimistic-proof program is compiled to a RISC-V ELF binary
that runs inside the SP1 zkVM. The verification key (vkey) is
derived from this ELF. Both are checked into the repository
and guarded by snapshot tests.

## Key files

- **Cached ELF binary**:
  `crates/pessimistic-proof/elf/riscv64im-succinct-zkvm-elf`
- **Vkey snapshot**:
  `crates/pessimistic-proof-test-suite/tests/snapshots/vkey_selector__vkey_snapshot.snap`
- **Build script**: `crates/pessimistic-proof/build.rs`
  delegates to the `agglayer-elf-build` crate.
- **Program source**: `crates/pessimistic-proof-program/`

## Build modes (`AGGLAYER_ELF_BUILD`)

The `AGGLAYER_ELF_BUILD` environment variable controls
how the ELF is obtained at build time:

| Value | Behavior |
|-------|----------|
| unset / `cached` | Uses the checked-in binary. No Docker needed. |
| `build` | Rebuilds from source via Docker. Does not update the cached ELF. |
| `refresh` / `update` | Rebuilds via Docker and overwrites the cached ELF. |

The Docker build always uses a **pinned SP1 image**
(`succinctlabs/sp1` with a fixed tag and digest)
for reproducibility.

## cargo-make tasks

| Task | What it does |
|------|-------------|
| `pp-elf` | Rebuild ELF (`update` mode), update cycle-tracker snapshots, then check vkey. |
| `pp-elf-build` | Rebuild ELF only (`update` mode). |
| `pp-check-vkey-change` | Verify vkey snapshot matches rebuilt ELF (`build` mode, `INSTA_UPDATE=no`). |
| `pp-accept-vkey-change` | Accept a new vkey snapshot (`update` mode, `INSTA_UPDATE=always`). |

Typical workflow after a change that affects the ELF:

```bash
cargo make pp-elf                  # rebuild + update cycle tracker
cargo make pp-accept-vkey-change   # accept new vkey if it changed
```

## When the vkey changes

Any change to the transitive dependency closure of
`pessimistic-proof-program` can change the compiled ELF
and therefore the vkey. This includes Cargo dependency updates
even when the program source is unchanged.

When the vkey changes, the vkey selector must be bumped
unless it has already been bumped since the last release.

## How the selector is derived

The selector is four bytes: the **high byte is the proof wrapping**,
the low three bytes are the **major version** of
`crates/pessimistic-proof-program/Cargo.toml`.

| High byte | Wrapping |
|-----------|----------|
| `0x00`    | PLONK (every selector up to `0x0000000e`) |
| `0x01`    | Groth16 |

The build script `crates/pessimistic-proof-core/build.rs`
reads the program's `Cargo.toml`, extracts the major version,
and generates a `PESSIMISTIC_PROOF_PROGRAM_VERSION: u32`
constant. `PP_SELECTOR_PLONK` and `PP_SELECTOR_GROTH16`
combine it with each wrapping byte; the node picks between
them from its configured wrapping.

The two ranges are kept disjoint so a route registered for one
wrapping can never collide with a future version of the other.
Routes on L1 are immutable, so a collision is unrecoverable.
See [pp-groth16-migration.md](pp-groth16-migration.md).

## How to bump the selector

1. Increment the **major version** in
   `crates/pessimistic-proof-program/Cargo.toml`
   (e.g. `9.0.0` -> `10.0.0`).
   Only the major component matters.

2. Rebuild the ELF and accept the new snapshots:
   ```bash
   cargo make pp-elf
   cargo make pp-accept-vkey-change
   ```

The `pp-check-vkey-change` task (part of `pp-elf`) will fail
if the vkey changed but the selector was not bumped,
serving as a safety net.
