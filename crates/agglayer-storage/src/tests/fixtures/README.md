# Test Fixtures for Regression Testing

This directory contains RocksDB database artifacts used for regression testing across version upgrades.

## Purpose

These fixtures ensure that:
1. Database deserialization remains backward compatible across dependency upgrades (e.g., alloy 0.14 → 1.0)
2. All column families can be read correctly
3. Data integrity is maintained during serialization format changes

## Artifacts

### `reference_db_v1.tar.gz`

Reference database generated with the following configuration:

- **Version**: 1.0.0
- **Networks**: 3
- **Certificates per network**: 10
- **Total certificates**: 30
- **Seed**: 42 (for reproducibility)
- **Proofs**: Included
- **Size**: ~60 KB compressed, ~564 KB uncompressed

#### Column Families Included

The database contains data across 16 column families:

**State Database:**
- `certificate_header_cf` (30 entries)
- `certificate_per_network_cf` (30 entries)
- `latest_settled_certificate_per_network_cf` (3 entries)
- `metadata_cf` (1 entry)
- `local_exit_tree_per_network_cf` (18 entries)
- `balance_tree_per_network_cf` (15 entries)
- `nullifier_tree_per_network_cf` (3 entries)
- `network_info_cf` (12 entries)

**Pending Database:**
- `latest_pending_certificate_per_network_cf` (3 entries)
- `latest_proven_certificate_per_network_cf` (3 entries)
- `pending_queue_cf` (6 entries)
- `proof_per_certificate_cf` (1 entry)

**Epochs Database:**
- `per_epoch_certificates_cf` (30 entries)
- `per_epoch_metadata_cf` (24 entries)
- `per_epoch_proofs_cf` (30 entries)

**Debug Database:**
- `debug_certificates` (30 entries)

**Total**: 239 entries across all databases

## Regenerating Artifacts

If you need to regenerate these artifacts (e.g., after intentional serialization format changes):

```bash
cargo run --bin generate-test-db --features testutils,cli -p agglayer-storage -- \
  --output-dir ./crates/agglayer-storage/tests/fixtures/reference_db_v1 \
  --num-networks 3 \
  --certificates-per-network 10 \
  --seed 42 \
  --tarball \
  --tarball-name reference_db_v1.tar.gz
```

**Important**: Always use the same seed (42) to ensure reproducibility!

## Using in Tests

The regression tests automatically:
1. Extract the tarball to a temporary location
2. Open all databases in read-only mode
3. Iterate through all column families
4. Verify deserialization of all entries
5. Compare against the metadata expectations

See `tests/regression_db_deserialization.rs` for implementation details.

## Versioning

When creating new artifacts for major changes:
- Use semantic versioning: `reference_db_v2.tar.gz`, etc.
- Keep old versions to test backward compatibility
- Update this README with new artifact details
- Include the metadata.json file for reference

## Metadata

Each database directory includes a `metadata.json` file with:
- Generation timestamp
- Configuration used
- Statistics (networks, certificates, entries per CF)
- Database paths

This metadata is used by regression tests to validate the database contents.

