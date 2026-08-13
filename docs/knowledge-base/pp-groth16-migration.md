# Moving the pessimistic proof from PLONK to Groth16

**Status:** proposed, not yet executed.
**Scope:** the pessimistic proof only, and only chains verified through `AgglayerGateway`.
The aggchain proof is unaffected.

Groth16 wrapping is faster to produce and cheaper to verify on L1 than PLONK.
This document describes everything needed to make the switch,
and how to go back if we ever have to.

## Three words you need

- **Wrapping** — the final format of the proof we send to L1. Today PLONK, moving to Groth16.
- **Route** — an entry on L1 that says: *for this 4-byte tag, use this verifier contract and
  this program key*. Once added, a route can never be edited, only frozen.
- **Selector** — that 4-byte tag. The node puts it at the front of every proof.

A PLONK proof and a Groth16 proof need **different verifier contracts**,
so they need different routes, so they need different selectors.
The program key is the same for both.

## What actually changes

The program itself does not change, so its key does not change either.
Only the wrapping around it does. That means the whole migration is:

| | Selector | Verifier | Program key |
|---|---|---|---|
| Today, stays as the way back | `0x0000000e` | `SP1VerifierPlonk` | `0x00d14f97…` |
| **Add this** | `0x0100000e` | `SP1VerifierGroth16` | `0x00d14f97…` (same) |

**One new route.** The existing route is untouched and becomes the rollback target for free.
Selectors carry the wrapping in the high byte and the program major in the low three, so the
new one differs from the old one without the program version moving.

## Migration process

1. **Check no chain uses the old direct-verifier setup.**
   Those chains (`VerifierType::Pessimistic`) have no route and no selector,
   so nothing in this document protects them, and a config rollback cannot help them either.
   If even one live chain uses it, stop.
2. **Confirm the program key really is unchanged**: `cargo make pp-check-vkey-change`.
   The whole plan rests on the new route reusing today's key.
   If it did change, see *When the program changes* below.
3. **Release the prover library** with Groth16 support, and tag it.
   Nothing changes yet — nobody asks for Groth16 until step 4.
4. **Build the node**, with both proof toolkits in the Docker image
   so either wrapping can be produced.
5. **Check a Groth16 verifier contract exists** on every target network.
   It is not in our contracts repo, so it comes from Succinct or we deploy it.
   Confirm any candidate address with `VERIFIER_HASH()` rather than trusting a list.
6. **Add the Groth16 route on L1**, before touching the node.
   Adding a route changes nothing for the running node, so this is safe to do early.
7. **Stop new certificates and let the queue drain.**
   Note that `admin_disableNetwork` only *reports* a network as disabled — it does not block
   anything — so this has to be agreed with whoever runs the aggsenders.
8. **Deploy the node with `proof-wrapping = "groth16"`.**
   Deploying alone changes nothing — PLONK is the default — so this key is the switch.
9. **Turn the networks back on** and confirm a certificate settles.
10. **Leave the PLONK route alone.** Do not freeze it. It is the way back.

## Rollback runbook

**Use when:** Groth16 turns out to be broken, or we stop trusting it, for any reason.

**What you need:** nothing on L1. The PLONK route is the one that was already there.

1. Stop new certificates and let in-flight ones finish, as in migration step 7.
2. In the node config, set:

   ```toml
   proof-wrapping = "plonk"
   ```

   Or drop the key entirely — PLONK is the default.
   The selector follows the wrapping, so there is no second value to keep in step.
3. Restart the node.
4. Turn the networks back on.
5. Confirm one certificate settles, and check the transaction used the PLONK route.

### Good to know

- Certificates that already settled are untouched.
  The selector is not stored against them, and settled certificates are never reprocessed.
- A certificate whose transaction was already built keeps the selector it was built with,
  and settles on that route. This is why neither route should be frozen.

## When the program changes

Bump the program major only when the ELF changes — that is what the version means.
A new program has a new key, so **both** of its routes must be registered:
`0x000000NN` for PLONK and `0x010000NN` for Groth16, both carrying the new key.
Register the PLONK one at the same time as the Groth16 one, not during an incident:
`addPessimisticVKeyRoute` needs the route-admin role and can sit up to 3 days behind the
timelock, which would strand the rollback exactly when it is needed.

---

## Reference

### How a proof reaches L1

```
proof[0:4]   our selector    -> AgglayerGateway.pessimisticVKeyRoutes[sel] = {verifier, ppVKey, frozen}
proof[4:8]   SP1's own tag   -> identifies the wrapping; written by the SDK, never by us
proof[8:]    the proof
```

`AgglayerGateway` looks up the route by our selector and calls
`ISP1Verifier(route.verifier).verifyProof(route.pessimisticVKey, publicValues, proof[4:])`.
The verifier then checks `proof[4:8]` against its own wrapping and reverts if it disagrees.
That last check is why a Groth16 proof cannot be sent to a PLONK route.

No contract change is needed: `SP1VerifierPlonk` and `SP1VerifierGroth16` implement the same
`ISP1Verifier.verifyProof`, so the gateway does not care which one sits behind a route.
Registering the route is the only on-chain step.

This repo currently emits selector `0x0000000e` (program version 14).
Which selector a given network's route uses depends on the node version deployed there,
so read `pessimisticVKeyRoutes` on that network rather than assuming.
On mainnet the PP route points at `SP1VerifierPlonk` v6.1.0
(`0x0459d576A6223fEeA177Fb3DF53C9c77BF84C459`).
That is a plain verifier, not a gateway that dispatches on `proof[4:8]`,
so one route cannot serve both wrappings.

Gateways: mainnet `0x046Bb8bb98Db4ceCbB2929542686B74b516274b3`,
Cardona `0xaA8103640A6C92af48A97D720168011E9f3Ec697`.

### Selector scheme

The high byte is the wrapping, the low three bytes are the program version.

```
0x000000NN   PLONK     (what every historical selector already is: 0x06 .. 0x0e)
0x010000NN   Groth16   (new)
```

The two ranges can never collide, which is the point.
Taking "the next free number" instead would eventually clash: a route parked on `0x0000000f`
takes the number the next program version needs, and routes cannot be edited afterwards,
so that clash would be permanent.

Only two selectors exist for a given program, `PP_SELECTOR_PLONK` and `PP_SELECTOR_GROTH16`,
and the wrapping picks between them. There is nothing else to configure, so the selector and
the wrapping cannot disagree. `proof-wrapping` defaults to PLONK, so a deployment that never
sets it keeps settling exactly as it does today.
See [pessimistic-proof-elf.md](pessimistic-proof-elf.md) for how the version drives the ELF
and the vkey snapshot.

Headroom: about 16.7 million versions, and 256 wrappings with 2 in use.
At the historical 7–13 version bumps a year, this is not a limit.

### The program key is the same for both wrappings

`vk.bytes32()` is the hex form of `vk.hash_bn254()`,
and both hash the program's verifying key — nothing about the wrapping enters.
So both routes carry the **same** `ppVKey`; only the verifier address differs.
For this repo's current ELF that value is
`0x00d14f977a6ec393014f300ad78d0761dc29435d3fa1e2626fa466bd3343578e`
(`crates/pessimistic-proof-test-suite/tests/snapshots/vkey_selector__vkey_snapshot.snap`).

Do not confuse it with the *circuit* key hash, `sha256(circuit_vk)[..4]`,
which **is** per wrapping and travels inside the proof as `proof[4:8]`.
The names collide; they are different values with different jobs.

### Storage needs no migration

Proofs are stored as bincode, and the proof enum already has a Groth16 case at a fixed
position (`Core 0, Compressed 1, Plonk 2, Groth16 3`).
New proofs write the Groth16 tag; old PLONK rows still decode.
Mixed rows are normal and expected, not a transitional state to clean up.

### Still to confirm

- Is the program key genuinely unchanged? Run `cargo make pp-check-vkey-change`, which
  rebuilds the ELF through Docker. The new route reuses today's key, so this is the one
  assumption the plan rests on.
- Does a Groth16 verifier v6.1.0 exist on each network, or must we deploy one?
- Who holds `AL_ADD_PP_ROUTE_ROLE`, and is it behind the timelock?
  It does not block this migration, but it sets the cost of registering routes for a future
  program version.

Already settled: Succinct fulfils Groth16 requests for SP1 `=6.2.2`,
and the live verifier is a plain `SP1VerifierPlonk`,
so there is no shortcut through a dispatching gateway.
