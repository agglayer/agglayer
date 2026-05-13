## Database Migration Guide

This document describes how to use the database migration API provided by the `Builder` type.

Database migration happens during storage initialization and may take some time, depending on the amount of data. The migration system ensures safe schema evolution while maintaining data integrity and supporting recovery from interruptions.

### Basic Usage

The `Builder` type provides a fluent API for defining and executing database migrations:

```rust
use agglayer_storage::{
    storage::DB,
    schema::ColumnDescriptor,
};

// Define column descriptors for the migration
const ADDED_CFS: &[ColumnDescriptor] = &[
    ColumnDescriptor::new::<NewCf1Column>(),
    ColumnDescriptor::new::<NewCf2Column>(),
];

const DROPPED_CFS: &[ColumnDescriptor] = &[
    ColumnDescriptor::new::<OldCf1Column>(),
    ColumnDescriptor::new::<OldCf2Column>(),
];

// Initialize builder with path and initial schema
let db = DB::builder(db_path, initial_schema)?
    // Apply migration steps
    .add_cfs(ADDED_CFS, |db| {
        // Migration logic here to populate new CFs
        Ok(())
    })?
    .drop_cfs(DROPPED_CFS)?
    // Finalize with expected final schema
    .finalize(final_schema)?;
```

For concrete examples, see the `test::sample` module implementations such as `sample_migrate_v0_v1` and `sample_migrate_v1_v2`.

### Initial Schema

The DB builder is initialized with the very first version schema. This represents the baseline state from which all migrations are applied:

```rust
const INITIAL_SCHEMA: &[ColumnDescriptor] = &[
    ColumnDescriptor::new::<CfV0_1Column>(),
    ColumnDescriptor::new::<CfV0_2Column>(),
];

DB::builder(db_path, INITIAL_SCHEMA)
```

### Migration Steps

Migrations consist of a sequence of two operation types:

- `.add_cfs(&[ColumnDescriptor], |db| { ... })`: Create new column families and populate them with data
- `.drop_cfs(&[ColumnDescriptor])`: Remove old column families

Each step is tracked individually, allowing recovery from partial migrations.

### Step Immutability

**Once the software is released, migration steps must not be modified.** For any subsequent schema updates, new steps must be added at the end of the sequence.

This ensures that existing databases can migrate forward correctly, as the system tracks which steps have already been completed.

### Idempotency Requirement

**Each migration step must be idempotent at each point of its execution.** This ensures that if a step is interrupted mid-execution, it can be safely restarted and will operate correctly.

### Recommended Pattern

Follow this pattern to ensure idempotency:

1. **Add** new column families and populate them (read from old, write to new)
2. **Drop** old column families

Avoid modifying values in place, as this makes recovery from interruptions difficult.

Example:

```rust
const DATA_V1: &[ColumnDescriptor] = &[ColumnDescriptor::new::<DataColumn>()];
const DATA_V2: &[ColumnDescriptor] = &[ColumnDescriptor::new::<DataV2Column>()];

DB::builder(db_path, DATA_V1)
    // Step 1: Create new CF and migrate data
    .add_cfs(DATA_V2, |db| {
        // Read from "data", write to "data_v2"
        for key in db.keys::<DataColumn>()? {
            let key = key?;
            let value = db.get::<DataColumn>(&key)?;
            let (new_key, new_value) = transform(key, value);
            db.put::<DataV2Column>(&new_key, &new_value)?;
        }
        Ok(())
    })?
    // Step 2: Remove old CF
    .drop_cfs(DATA_V1)?
    .finalize(DATA_V2)?
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
3. **Write idempotent migrations** following the add-drop pattern
4. **Never modify released steps** - always append new steps
5. **Use final schema validation** to catch mistakes early
6. **Write only to new CFs** in the migration function - read from old, write to new
