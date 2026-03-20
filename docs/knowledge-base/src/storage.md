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

When changing storage schemas or keys:

1. Define the migration path up front.
2. Keep reads backward-compatible where possible.
3. Add tests covering upgrade and rollback behavior.
