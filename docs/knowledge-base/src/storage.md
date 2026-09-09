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
