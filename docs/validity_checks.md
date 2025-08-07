# Proof and signature verification

Chains provide aggchain proofs (stark) and signatures on commitments which evolved over time.
These are verified either on the agglayer, or in the PP, or both.

This document summarizes what is verified and where.

The PP has a public input called `aggchain_hash`, whose composition depends on the consensus type.

## Consensus type 0

| #   | Case       | Verified in Agglayer | Verified in PP | Commitment Version                 |
| --- | ---------- | -------------------- | -------------- | ---------------------------------- |
| 1   | ECDSA only | ECDSA                | ECDSA          | currently V2 or V3, must become V5 |

## Consensus type 1

| #   | Case             | Verified in Agglayer | Verified in PP   | Commitment Version             |
| --- | ---------------- | -------------------- | ---------------- | ------------------------------ |
| 1   | Multisig only    | Multisig             | Multisig         | V5                             |
| 2   | STARK + ECDSA    | STARK + ECDSA        | STARK            | (currently V4, must become V5) |
| 3   | STARK + Multisig | STARK + Multisig     | STARK + Multisig | V5                             |

Notes:

- Aggchain proof alone isn’t accepted by the Agglayer; it must be accompanied by at least one ECDSA signature from the trusted sequencer (see #[issue to whitelist submitter in case of sp1 issue]).
- Cases 2 and 3 can be merged (single ECDSA treated as 1-of-1 multisig) only if the permissionless chain has a registered multisig committee that includes the trusted sequencer; otherwise we need to keep them separated because the PP cannot provide a valid `aggchain_hash` for the trusted sequencer as single signer without a registered multisig committee.

## Commitments

This section outlines the commitment versions.

## V2

```diff
commitment_v2 = keccak256_combine([
    certificate.new_local_exit_root,
    commit_imported_bridge_exits: certificate.claim_hash(), // defined below
]);

/// keccak(ib[0].global_index # .. # ib[n].global_index)
fn claim_hash() -> Digest {
    keccak256_combine(self.imported_bridge_exits.iter().map(|ibe| {
        [
            ibe.global_index.as_le_slice(),
        ]
        .concat()
    }))
}
```

## V3

- Add the `height`
- Add the bridge exit hash on the `commit_imported_bridge_exits`

```diff
commitment_v3 = keccak256_combine([
    certificate.new_local_exit_root,
    commit_imported_bridge_exits: certificate.claim_hash(), // defined below
+   certificate.height,
]);

/// keccak(ib[0].global_index # ib[0].bridge_exit_hash # .. # ib[n].global_index # ib[n].bridge_exit_hash)
fn claim_hash() -> Digest {
    keccak256_combine(self.imported_bridge_exits.iter().map(|ibe| {
        [
            ibe.global_index.as_le_slice(),
+           ibe.bridge_exit_hash.as_slice(),
        ]
        .concat()
    }))
}
```

## V4

- Add the aggchain params

```diff
commitment_v4 = keccak256_combine([
    certificate.new_local_exit_root,
    commit_imported_bridge_exits: certificate.claim_hash(), // defined below
    certificate.height,
+   certificate.aggchain_params,
]);

/// keccak(ib[0].global_index # ib[0].bridge_exit_hash # .. # ib[n].global_index # ib[n].bridge_exit_hash)
fn claim_hash() -> Digest {
    keccak256_combine(self.imported_bridge_exits.iter().map(|ibe| {
        [
            ibe.global_index.as_le_slice(),
            ibe.bridge_exit_hash.as_slice(),
        ]
        .concat()
    }))
}
```

## V5

- Add the certificate id

```diff
commitment_v5 = keccak256_combine([
    certificate.new_local_exit_root,
    commit_imported_bridge_exits: certificate.claim_hash(), // defined below
    certificate.height,
    certificate.aggchain_params, // nil value if no aggchain proof
+   certificate_id // re-computed only on the agglayer, but free input for the PP
]);

/// keccak(ib[0].global_index # ib[0].bridge_exit_hash # .. # ib[n].global_index # ib[n].bridge_exit_hash)
fn claim_hash() -> Digest {
    keccak256_combine(self.imported_bridge_exits.iter().map(|ibe| {
        [
            ibe.global_index.as_le_slice(),
            ibe.bridge_exit_hash.as_slice(),
        ]
        .concat()
    }))
}
```
