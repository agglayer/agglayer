# Pessimistic Proof

Agglayer's safety model relies on pessimistic proofs to validate
cross-network state transitions.
This chapter explains crate responsibilities,
operational invariants, and the development workflow.

## Architecture

The proof stack is intentionally split across three crates:

- `pessimistic-proof-core` contains transition logic,
  input/output structures, and proof-domain errors.
- `pessimistic-proof-program` is the SP1 guest program.
  It runs inside the zkVM and commits public outputs.
- `pessimistic-proof` is the host-side wrapper that embeds the ELF,
  drives proving/verifying, and maps errors for callers.

At runtime,
Agglayer prepares witness inputs from certificate and network state,
executes or requests proving,
and verifies that outputs match expected commitments
before allowing settlement.

## Safety invariants

When changing proof-related code,
treat these invariants as non-negotiable:

- **Determinism:** identical inputs must produce identical public outputs.
- **Commitment compatibility:** commitment version semantics
  (V2 through V5) must remain consistent with verifier expectations.
- **State continuity:** previous/new roots in proof outputs
  must correspond to storage transitions.
- **Verifier identity stability:** vkey changes are explicit protocol events,
  not incidental refactors.

For the full validity-check matrix,
including proof and signature combinations,
see `docs/validity_checks.md`.

## Development workflow

Use the dedicated make tasks for proof changes.

```bash
cargo make pp-elf
```

This workflow builds the ELF and runs vkey/cycle-tracker checks.
For targeted checks:

```bash
cargo make pp-check-vkey-change
```

If a vkey change is intentional,
acceptance must be explicit and reviewed:

```bash
cargo make pp-accept-vkey-change
```

Guidelines for safe edits:

- Prefer minimal, locally justified proof changes.
- Include proof-focused tests in `pessimistic-proof-test-suite` when behavior changes.
- Document any semantic changes to public values or commitments.
