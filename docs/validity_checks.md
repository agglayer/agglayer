# Proof and signature verification

Chains provide aggchain proofs (stark) and signatures on commitments which evolved over time.
These are verified either on the agglayer, or in the PP, or both.

This document summarizes what is verified and where.

The PP has a public input called `aggchain_hash`. As of version 0.3.5, all paths are now `consensus type 1`.

Each case corresponds to what the chain may submit to the agglayer.

| #   | Case             | Verified in Agglayer | Verified in PP   | Commitment Version           |
| --- | ---------------- | -------------------- | ---------------- | ---------------------------- |
| 1   | Legacy ECDSA     | ECDSA                | Multisig 1-of-1  | V2, post-migration: V3 or V5 |
| 2   | Multisig only    | Multisig             | Multisig         | V5                           |
| 3   | STARK + Multisig | STARK + Multisig     | STARK + Multisig | V5                           |

Notes:

- Case 1 corresponds to all chains with a simple ECDSA performed by the trusted sequencer.
  - For each of them, the signer is registered in the L1 as a multisig 1-of-1.
- Katana fits in case 3
  - Single signer is registered in the L1 as a multisig 1-of-1.
  - Agglayer and PP verify this multisig alongside the FEP aggchain proof.

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
