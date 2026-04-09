# Certificate Proof Versioning Design

## Problem

Agglayer certificate storage currently mixes two different concerns under a
single outer storage version.
The certificate envelope version stayed at `1`, while the embedded aggchain
proof payload changed shape across SP1 upgrades.
That makes old stored certificates fail to deserialize on newer nodes when the
proof bytes are no longer compatible with the current SP1 Rust types.

This is not only a test-fixture issue.
It affects upgrade safety for persisted certificates already written to RocksDB.

## Goals

- Keep existing on-disk `CertificateV1` rows readable after upgrade.
- Make new writes use an explicit `CertificateV2` storage format.
- Keep `AggchainData` canonical in `agglayer-interop-types`, not duplicated in
  Agglayer.
- Preserve the exact SP1 proof `version` string end-to-end.
- Allow the node to decide which SP1 proof versions are readable,
  executable, and writable.
- Avoid protobuf schema changes.

## Non-Goals

- Re-proving old SP1 proofs into new SP1 proof objects.
- Making old readable SP1 proofs executable by the SP1-v6 certifier.
- Migrating all old rows eagerly during node startup.

## Recommendation

Freeze `CertificateV1` as a decode-only on-disk format.
Introduce `CertificateV2` as the only storage write format.
Move the version-aware aggchain proof representation into
`agglayer-interop-types`, then keep Agglayer-specific acceptance policy in the
Agglayer node.

In other words:

- `interop` owns the shared shape.
- Agglayer owns read/execute/write policy.
- storage owns versioned envelope adapters.

## Canonical Data Model

`agglayer-interop-types` remains the single source of truth for:

- `AggchainData`
- `AggchainProof`
- `Proof`

The SP1 branch of that shared model becomes a versioned byte envelope rather
than a direct Rust SP1 proof struct.

Conceptually:

```rust
enum Proof {
    Sp1(Sp1ProofEnvelope),
}

struct Sp1ProofEnvelope {
    version: String,
    proof: Vec<u8>,
    vkey: Vec<u8>,
}
```

The exact `version` string is preserved for storage, gRPC round-trip, and
observability.
Agglayer parses that string into an internal policy classification when it needs
to make a decision.

## Proof Version Policy

The Agglayer node defines proof acceptance policy.
This policy does not belong in `agglayer-interop-types`.

The node exposes logic equivalent to:

- readable versions
- executable versions
- writable versions

For the current upgrade, the policy is:

- readable: SP1 circuit versions in the historical `v4.*` family and the
  current `v6.*` family
- executable: `v6.*` only
- writable: `v6.*` only

This keeps old rows readable without pretending they are valid SP1-v6 execution
inputs.

## Storage Model

### CertificateV1

`CertificateV1` stays as a decode-only historical wrapper.
It is never written by the upgraded node.

Two historical variants must be supported while decoding storage version `1`:

- `CertificateV1` with interop 0.14 typed SP1-v6 proof payloads
- `CertificateV1` with interop 0.13 typed SP1-v4 proof payloads

Both decode paths map into the canonical in-memory certificate shape by
producing the shared SP1 proof envelope.

### CertificateV2

`CertificateV2` is the only format written by the Store.
Its outer version byte is `2`.
Its aggchain proof payload is already the shared versioned SP1 envelope.

Store behavior becomes:

- decode `V0` -> current in-memory certificate
- decode `V1` -> current in-memory certificate
- decode `V2` -> current in-memory certificate
- encode current in-memory certificate -> `V2` only

## Store API Contract

The Store API only accepts and returns the current in-memory certificate type.
It does not expose `CertificateV1`.

Operationally:

1. read raw bytes from RocksDB
2. decode according to storage version
3. return the current certificate shape
4. write back only as `CertificateV2`

If the current certificate contains an SP1 proof version that is readable but
not writable, storage encode fails explicitly.

That means old certificates can be read, but they are not silently rewritten as
`V2` unless their proof version is writable.

## gRPC / Protobuf

No protobuf schema change is required.

`agglayer.interop.types.v1.SP1StarkProof` already carries:

- `version`
- `proof`
- `vkey`

The conversion layer should stop deserializing those bytes directly into the
current SP1 Rust types as part of parsing.
Instead, it should map them into the shared SP1 proof envelope and let Agglayer
policy decide whether the version is readable, executable, or writable.

This keeps the wire contract stable while making the version field meaningful.

## Runtime Behavior

Code paths that only need proof metadata, such as vkey hashing, should operate
from the proof envelope and version policy.

Code paths that need actual SP1 execution should:

1. require an executable SP1 version
2. deserialize the proof bytes into the current SP1 Rust types only at that
   point

If a certificate contains an old readable proof version, the node returns a
clear error for execution rather than failing during generic decode.

## Migration Strategy

This design does not require an eager migration.

Old `V1` rows remain readable in place.
New writes are `V2`.

Future read-repair or background migration can be added later, but only for
proof versions that are considered writable.
Old readable-but-non-writable proofs should not be silently rewritten.

## Testing Requirements

- historical `regression_01.hex` decodes successfully
- newly written certificates encode with version byte `2`
- readable old SP1 versions are rejected on storage write
- readable old SP1 versions round-trip through gRPC without protobuf changes
- executable paths reject non-executable SP1 versions explicitly

## Risks

The main risk is conceptual drift if Agglayer and interop each define their own
`AggchainData` long-term.
This design avoids that by keeping the canonical proof envelope in
`agglayer-interop-types`.

Another risk is overly broad version parsing.
The policy should remain explicit and conservative, and it should classify only
the proof version families that the node has intentionally validated.
