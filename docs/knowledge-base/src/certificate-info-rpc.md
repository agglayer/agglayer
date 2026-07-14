# Certificate info RPC

`interop_getCertificateInfo(network_id, height, include_claims?)` is a public JSON-RPC
read method binding a certificate to the bridge exits and claims it carries.
It exists so that external tooling (benchmarks, monitoring scripts) can correlate
a bridge deposit with the certificate that settles it.

## Response

```json
{
  "certificate_id": "0x…",
  "network_id": 19,
  "height": 142,
  "status": "Settled",
  "exit_count": 3,
  "leaf_range": { "start": 1204, "end": 1207 },
  "claims": ["0x10000000000000065"]
}
```

- `exit_count` is the number of bridge exits in the certificate,
  i.e. the number of leaves it appends to the network's local exit tree (LET).
- `leaf_range` is the absolute LET index range covered by those exits,
  `end` exclusive.
  A deposit with leaf index `L` (its deposit count on the origin network)
  is covered by the certificate where `start <= L < end`.
  Zero-exit certificates report an empty range (`start == end`).
- `claims` lists the global indexes of the imported bridge exits,
  and is only present when `include_claims` (third parameter, default `false`) is set.
- Any `status` other than `Settled` means the binding is **provisional**:
  a certificate that ends up `InError` can be replaced at the same height
  by a different certificate with different exits.

## Why requests are guarded

The goal of this endpoint is to watch **recent** bridge activity:
find which certificate carries a deposit, and follow it until it settles.
It is not a historical query API.

In-flight certificates come from the pending store, which is always open.
Settled certificates live in one database per epoch,
and reading an old epoch opens its database on demand.
A public endpoint doing that for any requested height would let anyone
force the node to open old databases at will — a cheap denial of service.

The rule is therefore:
the node only reads epoch storage for the **latest epoch with a settlement**.
A certificate settled in an older epoch is rejected with an `InvalidArgument`
error ("settled in a past epoch") before any epoch storage is touched.
This costs nothing for the stated goal:
a tool following a live deposit only ever needs the in-flight certificate
and the ones that just settled.

Remaining trade-offs, none of which matter for that goal:

- A non-`Settled` binding is provisional:
  an `InError` certificate can be replaced at the same height.
  The `status` field says so; consumers re-check instead of caching.
- Stage timestamps are not returned:
  the node does not persist transition times, so consumers time the `status`
  changes they observe while polling.

## Consumer recipe

A tool that bridged a deposit knows `(network_id, deposit_count)`
from the bridge event as soon as its transaction is mined.
On each tick of its poll loop:

1. `interop_getLatestSettledCertificateHeader(network_id)`
   gives the latest settled height `S`.
2. `interop_getCertificateInfo(network_id, S + 1)` is the in-flight certificate.
   Not found means nothing is being processed yet — retry.
3. If `deposit_count` falls in `leaf_range`,
   this is the covering certificate.
   Keep polling that height:
   each new `status` value (`Pending`, `Proven`, `Candidate`, `Settled`)
   is a stage transition, timed at poll granularity.
   A `deposit_count` beyond the range belongs to a later certificate — keep waiting.
   Below the range, it already settled — check the heights just under `S`.
4. Once settled, the exact on-chain time comes from the header's
   `settlement_tx_hash` (`interop_getCertificateHeader`)
   via `eth_getTransactionReceipt` and `eth_getBlockByNumber` on L1.
