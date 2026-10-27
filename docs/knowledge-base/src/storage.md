# Storage

Agglayer storage is implemented on RocksDB with a strict separation
between physical database mechanics and logical domain stores.

## Database topology

Storage configuration exposes multiple paths,
either inferred from a common `db-path` or configured explicitly.
At runtime,
the node currently opens pending/state/epochs databases,
and optionally debug storage in debug mode.

| Database | Default subpath | Primary purpose |
|---|---|---|
| Pending DB | `pending/` | Pending queue and proof material |
| State DB | `state/` | Canonical per-network state |
| Epochs DB root | `epochs/` | Root directory for per-epoch RocksDB instances |
| Debug DB | `debug/` | Debug-only certificate traces (opened only in debug mode) |

See `crates/agglayer-config/src/storage.rs` for configuration details.

Note:

- `metadata_db_path` exists in config today,
  but node startup currently does not open a dedicated metadata RocksDB.
  Metadata is stored via the `metadata_cf` column family in the state DB.

## Physical vs logical layers

- **Physical layer** (`crates/agglayer-storage/src/storage/`):
  typed column-family access,
  serialization codecs,
  batched writes,
  iterators,
  and RocksDB open/migration mechanics.
- **Logical layer** (`crates/agglayer-storage/src/stores/`):
  domain stores with business-oriented APIs (`StateStore`, `PendingStore`,
  `EpochStore`, `DebugStore`).

Keep domain policy in logical stores.
Keep encoding and persistence mechanics in the physical layer.

## Column families by store

State DB (`stores/state/cf_definitions.rs`):

- `certificate_header_cf`
- `certificate_per_network_cf`
- `latest_settled_certificate_per_network_cf`
- `metadata_cf`
- `local_exit_tree_per_network_cf`
- `balance_tree_per_network_cf`
- `nullifier_tree_per_network_cf`
- `network_info_cf`
- `disabled_networks_cf`

Pending DB (`stores/pending/cf_definitions.rs`):

- `latest_proven_certificate_per_network_cf`
- `latest_pending_certificate_per_network_cf`
- `pending_queue_cf`
- `proof_per_certificate_cf`

Per-epoch DB (`stores/per_epoch/cf_definitions.rs`):

- `per_epoch_certificates_cf`
- `per_epoch_metadata_cf`
- `per_epoch_proofs_cf`
- `per_epoch_start_checkpoint_cf`
- `per_epoch_end_checkpoint_cf`

Debug DB (`stores/debug/cf_definitions.rs`):

- `debug_certificates`

Migration bookkeeping also uses a dedicated migration column family.

## Migrations, backups, and safety

- Migration logic lives under `crates/agglayer-storage/src/storage/migration/`
  and includes checks for unexpected/default column-family content.
- Storage protobuf schemas under `proto/agglayer/storage/v0/`
  define compatibility boundaries for stored structures.
- Backups are managed via storage backup configuration
  and CLI backup commands.

Backups are requested by the write paths themselves,
so a restored database stays usable without operator action.
The state and pending DBs are backed up together when:

- A certificate is accepted as pending,
  which is when its header is inserted with the `Pending` status.
  The acceptance path writes the certificate body to the pending DB
  before inserting the header, so the snapshot contains both.
  This backup is what keeps a submitted but unprocessed certificate
  recoverable, since nothing outside the live databases references it
  until the orchestrator picks it up.
- A certificate is proven.
  Settlement is submitted from a spawned task shortly after,
  so this is the last write still ordered ahead of the certificate reaching L1.
  Its generated proof, which lives in the pending DB, is already persisted.
- A settlement tx hash is recorded on or removed from a certificate header.
- A local network state is written, which happens on settlement.

An epoch DB is backed up separately when that epoch is packed.
Requests never block the write that triggered them,
and travel on two separate queues so state requests cannot crowd out epoch requests:

- State+pending requests coalesce into a single queue slot.
  Extra requests are dropped, which is safe because a backup snapshots
  the databases when it runs, not when it is requested,
  so the queued backup also covers the write that got its request dropped.
- Epoch requests are queued unbounded and never dropped:
  an epoch is packed exactly once,
  so a dropped request would mean that epoch is never backed up.

On shutdown, the backup engine drains both queues before exiting,
so requests that were already queued still produce backups.

Request volume, queue wait and backup duration are exported as metrics;
see [Observability](./observability.md).

When changing storage schemas or keys:

1. Define the migration path up front.
2. Keep reads backward-compatible where possible.
3. Add tests covering upgrade and rollback behavior.

## Network info backfill

`network_info_cf` caches each network's latest certificate pointers, its network type, and its latest settled claim.
The legacy latest-certificate indexes hold the same pointers:
`latest_settled_certificate_per_network_cf` in the state DB,
and the latest pending and proven indexes in the pending DB.
Until the readers move to `network_info_cf`, the writers are meant to keep both up to date.
Submission
and explicit pending/proven overrides write both representations through the [shared certificate updater][certificate-updater].
The state store serializes normal submissions through the height check and both database writes.
One storage-owned submission mutex covers this short sequence across networks;
L1 requests, initial replacement validation, debug writes, and orchestrator notifications run outside it.
The existing height check remains inside the pending write.
Admin operations require exclusive operational access and do not take the submission mutex.
Proof completion uses the sequential network task and a storage async adapter,
without a submission lock.
Changing an arbitrary header's status to `Proven` does not advance the proven pointer.
Settlement will also write the network type; pending and proven writes leave it untouched.

Submission reuses the pending store's height check and rejects changes to settled certificates.
Persisted header identity and proof-completion inputs are trusted.
It uses the existing insertion order: legacy pending pointer, certificate body, then header
and cached pending pointer together in one state DB batch.
That batch creates a missing header or preserves its metadata while updating its status.
The state store requests any submission backup after the batch succeeds.
A concurrent legacy reader can briefly observe a pointer before its body or header is available.
Explicit overrides retain the ability to move a pointer backwards.
Admin force-push uses the same submission updater,
including initial header creation for a certificate that has no header yet.
Status, error, height, epoch, index, and local exit root remain certificate-header facts.
Only pending height/id and proven id are written to their respective cache rows.

The state and pending databases cannot share a transaction.
Explicit pointer overrides write the cache before the legacy pointer.
Success means all writes for the requested operation completed.
The updater returns database errors immediately as `eyre::Error` with write-stage context.
It does not retry, roll back, or repair a partial publication on a later write.
A failure between writes can leave the legacy and cached pointers out of sync;
operators must repair that state before relying on it.
Crash recovery between consecutive writes is outside this writer's scope.
On proof completion, the network task logs a publication error and continues processing.
Normal RPC submission retains its original one-time certificate replacement validation,
including settlement jobs and L1 receipts.
It does not revalidate the storage snapshot after L1/signature validation;
concurrent same-height requests retain the preexisting replacement race.
Admin force-push bypasses normal replacement validation but still refuses changes to settled certificates.
The standalone header-insertion helper is available only to tests and fixtures.

`getNetworkInfo` determines network existence from cached data and bounded cache pointer reads.
Legacy pointers skipped as junk by backfill do not make a network known.
A missing network type does not make a pending-only or proven-only network unknown,
and a dangling cached pointer still establishes existence.
Cache read failures propagate as errors.
Unknown networks retain the existing JSON-RPC error conversion and gRPC `NotFound` mapping.

[certificate-updater]: ../../../crates/agglayer-storage/src/stores/certificate_updater.rs

A settled row's claim is complete by construction.
A settled row without a claim row means that no imported bridge exit
was settled through that certificate.
No completion flag records this.

The third state DB migration step fills `network_info_cf` from the legacy indexes,
exactly once.
It drops and recreates the column first,
because rows written before that step may lack the claim history behind them.
It reads everything before writing anything,
and it writes the whole backfill in one batch before the step is recorded.
A failure leaves the step unrecorded,
and the next start recreates the column and runs the whole backfill again.
After the step is recorded, nothing reads the legacy indexes to repair the column:
only the runtime writers update it.

The node opens the state DB, and so runs the step,
before the clock, settlement, the orchestrator, and the RPC servers start.
The first start opens the per-epoch databases of settled certificates:
each settled network's latest epoch, for its certificate and proof,
and one per certificate the claim scan reads,
which can take a while for a network whose latest claim is far back.
It also loads the nullifier tree of each settled network
whose latest certificate carries no claim.
The health endpoint is not bound yet while it runs,
so relax liveness and startup probes for the first start:
a probe that kills it makes every restart run the whole backfill again.
Opening a database that holds certificates without the pending database and the epoch store
panics in the migration step, so the step can never be recorded with an empty column by mistake.
The first open by a binary with this step logs a migration-framework warning
that it drops the existing column, even on a fresh database: this is expected.

For each network in the legacy indexes:

- A pending or proven pointer belongs to the network it is stored under;
  the network id repeated in a proven value is not trusted.
  A pointer whose header is missing, belongs to another network,
  or sits at another height is skipped with a warning.
- The settled row is rebuilt with its claim.
  The latest settled certificate's own claim is used when it has one.
  Otherwise a stored empty nullifier tree proves that no claim was ever settled,
  so no history is read.
  Otherwise, including when no tree is stored because the network last settled
  before local state was persisted, the settled history is scanned backwards,
  one epoch per certificate, until a certificate with an imported bridge exit.
- The network type comes from the latest settled certificate, read from its epoch:
  a network can change type, and only settlement shows a type L1 accepted.
  A network that never settled gets no type.
- The settled row always stores the settled certificate's local exit root.
  Its pessimistic root stays unset when the proof is gone
  or the current codec cannot decode it,
  and its leaf count stays unset when no exit tree is stored
  or the stored one already moved past the settled root.

The settled pointer and claim history are checked strictly.
A settled pointer whose header is missing or does not match,
a settled certificate without an epoch assignment, missing from its epoch, or replaced there,
a malformed nullifier tree, an unreadable epoch or local exit tree,
or a gap in the history the scan reads fails the migration,
and the node does not start.
The previous binary still opens the database after a failed backfill,
so redeploying it is the immediate recovery.
Then fix the data offline, or restore the state and pending backups as a pair,
and restart.

Once recorded, the step is not reversible.
A binary that declares fewer migration steps refuses to open the database,
so rolling back needs a pre-upgrade backup.
Backup rotation soon deletes the pre-upgrade backups,
so copy them aside before upgrading.
