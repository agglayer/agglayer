## Database Migration Guide

This document describes how to use the database migration API provided by the `Builder` type.

Database migration happens during storage initialization and may take some time, depending on the amount of data. The migration system ensures safe schema evolution while maintaining data integrity and supporting recovery from interruptions.

### Basic Usage

The `Builder` type provides a fluent API for defining and executing database migrations:

```rust
use agglayer_storage::storage::DB;

// Initialize builder with path and initial schema
let db = DB::builder(db_path, initial_schema)?
    // Apply migration steps
    .create_cfs(&["new_cf_1", "new_cf_2"])?
    .migrate(|db| {
        // Migration logic here
        Ok(())
    })?
    .drop_cfs(&["old_cf_1", "old_cf_2"])?
    // Finalize with expected final schema
    .finalize(final_schema)?;
```

For concrete examples, see the `test::sample` module implementations such as `sample_migrate_v0_v1` and `sample_migrate_v1_v2`.

### Initial Schema

The DB builder is initialized with the very first version schema. This represents the baseline state from which all migrations are applied:

```rust
DB::builder(db_path, &["cf_v0_1", "cf_v0_2"])
```

### Migration Steps

Migrations consist of a sequence of three operation types:

- `.create_cfs(&[...cfs])`: Create new column families
- `.migrate(|db| { ... })`: Perform data transformation
- `.drop_cfs(&[...cfs])`: Remove old column families

These steps are tracked individually, allowing recovery from partial migrations.

### Step Immutability

**Once the software is released, migration steps must not be modified.** For any subsequent schema updates, new steps must be added at the end of the sequence.

This ensures that existing databases can migrate forward correctly, as the system tracks which steps have already been completed.

### Idempotency Requirement

**Each migration step must be idempotent at each point of its execution.** This ensures that if a step is interrupted mid-execution, it can be safely restarted and will operate correctly.

### Recommended Pattern

Follow this pattern to ensure idempotency:

1. **Create** new column families
2. **Migrate** by writing only to the new column families (read from old, write to new)
3. **Drop** old column families

Avoid modifying values in place, as this makes recovery from interruptions difficult.

Example:

```rust
DB::builder(db_path, &["data"])
    // Step 1: Create new CF with transformed schema
    .create_cfs(&["data_v2"])?
    // Step 2: Migrate data (read from "data", write to "data_v2")
    .migrate(|db| {
        for entry in db.iter::<OldColumn>() {
            let (key, value) = entry?;
            let transformed = transform(value);
            db.put::<NewColumn>(&key, &transformed)?;
        }
        Ok(())
    })?
    // Step 3: Remove old CF
    .drop_cfs(&["data"])?
    .finalize(&["data_v2"])?
```

### Final Schema Validation

The `final_schema` parameter passed to `.finalize()` is not strictly necessary but serves as a useful sanity check. It verifies that the database schema matches expectations after all migration steps complete.

If the actual schema doesn't match the expected schema, initialization will fail early with a clear error.

### Migration Records

The system automatically tracks completed migration steps in an internal column family. When reopening a database:

1. Previously completed steps are skipped
2. Only new or incomplete steps are executed
3. Gaps in the migration record trigger an error to prevent data corruption

### Best Practices

1. **Test migrations thoroughly** with representative data before release
2. **Keep steps small** to minimize interruption impact
3. **Write idempotent migrations** following the create-migrate-drop pattern
4. **Never modify released steps** - always append new steps
5. **Use final schema validation** to catch mistakes early
