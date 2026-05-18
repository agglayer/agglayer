# Ultrareview SP1 Fixes Design

## Goal

Implement the confirmed ultrareview fixes for SP1 v6 integration
while preserving bridge safety and operational recovery behavior.

## Storage Compatibility Policy

Pending generated proofs that cannot be decoded by the current node are treated as absent.
A certificate that is marked proven but has unreadable proof bytes must be re-proven
instead of blocking pending DB recovery or settlement progress.

This policy applies to the pending DB proof lookup paths.
New generated proofs are still written in the current `agglayer_types::Proof` format.

## SP1 Proof Handling

Aggchain vkey hash work in certification must not run synchronously on async runtime threads.
The certifier should offload vkey deserialization and hashing
through the existing SP1 blocking helper
and compute both the byte digest and u32 word representation
from one deserialized verifying key.

Executable SP1 proof decoding should accept the active `AcceptancePolicy` explicitly.
Existing certifier flows will pass `AcceptancePolicy::DEFAULT`,
keeping current behavior while making migration and rollback policy visible at call sites.

## Error Handling

SP1 proof envelope creation should report serialization failures as serialization failures.
`ProofError` gets serialization-specific variants for proof bytes and vkey bytes
so operational logs do not mislabel encode failures as decode failures.

`Runner::get_vkey` should return `eyre::Result<SP1VerifyingKey>`
and propagate SP1 setup failures instead of panicking.

## Dependency Boundary

`agglayer-storage` should keep `sp1-sdk-v5` out of runtime dependencies
because it is only used by tests.
Move it to dev-dependencies.

## Testing

Add targeted tests for:

- Pending DB unreadable proof bytes are ignored by `get_proof` and `multi_get_proof`.
- Current generated proof rows still round-trip.
- Policy-aware executable proof API keeps default certifier behavior explicit at call sites.
- Serialization errors use serialization-specific `ProofError` variants where practical.

## Blast Radius

The primary blast radius is pending proof reuse and certifier witness generation.
The changes avoid altering proof verification semantics,
settlement output,
or certificate encoding.
The storage policy intentionally favors re-proving
over trusting unreadable persisted proof bytes.
