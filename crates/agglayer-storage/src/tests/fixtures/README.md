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
- **Generation date**: 2025-12-02
- **Networks**: 3
- **Certificates per network**: 20
- **Total certificates**: 60
- **Seed**: 42 (for reproducibility)
- **Proofs**: Included (mock proofs for testing)
- **Size**: ~111 KB compressed

#### Column Families Included

The database contains data across **18 column families**:

**State Database:**
- `certificate_header_cf` (60 entries) - Headers with epoch/index mapping
- `certificate_per_network_cf` (60 entries) - Network-height to cert ID mapping
- `latest_settled_certificate_per_network_cf` (3 entries) - Latest settled certificate per network
- `disabled_networks_cf` (1 entry) - Networks that have been disabled (~30% randomly)
- `metadata_cf` (1 entry) - Global metadata (latest settled epoch)
- `local_exit_tree_per_network_cf` (105 entries) - LET leaves (2-4 per network), leaf count, and 32 frontier nodes per network (properly computed)
- `balance_tree_per_network_cf` (24 entries) - Sparse merkle tree nodes with proper keccak256-based hash derivation
- `nullifier_tree_per_network_cf` (24 entries) - Nullifier tree nodes with proper cryptographic structure
- `network_info_cf` (12 entries) - Network type, settled cert info, claims, pending cert info

**Pending Database:**
- `latest_pending_certificate_per_network_cf` (3 entries) - Latest pending cert per network
- `latest_proven_certificate_per_network_cf` (3 entries) - Latest proven cert per network
- `pending_queue_cf` (6 entries) - Pending certificates queue
- `proof_per_certificate_cf` (1 entry) - Proofs for certificates

**Epochs Database:**
- `per_epoch_certificates_cf` (60 entries) - Certificates indexed by epoch
- `per_epoch_metadata_cf` (42 entries) - Per-epoch metadata (settlement tx hash, packed status)
- `per_epoch_proofs_cf` (60 entries) - Proofs indexed by epoch

**Debug Database:**
- `debug_certificates` (60 entries) - Full certificate data for debugging

**Total**: 525 entries across all databases

#### Data Characteristics

All data is generated using a seeded random number generator (seed: 42) for reproducibility:

- **Certificates**: Diverse parameters (0-4 bridge exits, varying signature versions, multiple aggchain data types)
- **Network Types**: Varied across Ecdsa, Generic, MultisigOnly, MultisigAndAggchainProof
- **Network Info**: Realistic random hashes for pp_root, LER, global indexes, bridge exit hashes
- **Disabled Networks**: ~30% randomly disabled with Admin/Unspecified reasons and random timestamps
- **Trees**: Cryptographically correct sparse merkle trees with keccak256-derived node hashes and proper frontier computation for Local Exit Trees
- **Epochs**: 3 certificates per epoch (configurable via `CERTIFICATES_PER_EPOCH` constant)

## Regenerating Artifacts

If you need to regenerate these artifacts (e.g., after intentional serialization format changes):

```bash
# Generate in temp directory first for testing
cargo run -p agglayer-storage --bin generate-test-db --features "testutils,cli" -- \
  --output-dir ./temp/reference_db_v1 \
  --num-networks 3 \
  --certificates-per-network 20 \
  --seed 42 \
  --tarball \
  --tarball-name reference_db_v1.tar.gz

# After validation, move to fixtures directory
mv temp/reference_db_v1.tar.gz crates/agglayer-storage/tests/fixtures/
```

**Important Notes**:
- Always use the same seed (42) to ensure reproducibility!
- The data generation uses seeded RNG for deterministic output
- Review the metadata.json to verify expected statistics
- Test the new artifact with regression tests before committing

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

Each database archive has an accompanying `reference_db_v1_metadata.json` file in this directory for quick reference.

The extracted database directory also includes a `metadata.json` file with:
- Generation timestamp
- Configuration used (networks, certificates, seed, proofs)
- Statistics (total networks, total certificates, entries per column family)
- Database paths (state, pending, epochs, debug)

This metadata is used by regression tests to validate the database contents match expectations.

