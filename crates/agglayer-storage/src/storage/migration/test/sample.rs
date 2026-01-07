// Sample migration testing schema and data.

use std::path::Path;

use agglayer_types::{Height, NetworkId};

use crate::{
    schema::{ColumnDescriptor, ColumnSchema},
    storage::{Builder, DBMigrationErrorDetails, DBOpenError},
};

pub type KeyV0 = NetworkId;

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NetworkInfoV0 {
    pub height: Height,
    pub num_beans: u32,
    pub num_failures: u32,
}

pub type KeyV1 = NetworkId;

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NetworkInfoV1 {
    pub height: Height,
    pub num_beans: u32,
    pub num_failures: u64,
    pub is_cool: bool,
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct KeyV2 {
    pub network_id: NetworkId,
    pub height: Height,
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NetworkInfoV2Cool {
    num_beans: u32,
    sunglasses: bool,
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NetworkInfoV2Uncool {
    num_beans: u32,
    num_failures: u64,
}

crate::schema::impl_codec_using_bincode_for! {
    NetworkInfoV0,
    NetworkInfoV1,
    KeyV2,
    NetworkInfoV2Cool,
    NetworkInfoV2Uncool,
}

// Version 0 schema
pub struct NetworkInfoV0Column;

impl ColumnSchema for NetworkInfoV0Column {
    type Key = KeyV0;
    type Value = NetworkInfoV0;

    const COLUMN_FAMILY_NAME: &'static str = "network_info_v0";
}

pub const CFS_V0: &[ColumnDescriptor] = &[ColumnDescriptor::new::<NetworkInfoV0Column>()];

// Version 1 schema
pub struct NetworkInfoV1Column;

impl ColumnSchema for NetworkInfoV1Column {
    type Key = KeyV1;
    type Value = NetworkInfoV1;

    const COLUMN_FAMILY_NAME: &'static str = "network_info_v1";
}

pub const CFS_V1: &[ColumnDescriptor] = &[ColumnDescriptor::new::<NetworkInfoV1Column>()];

// Version 2 schema - Cool networks
pub struct NetworkInfoV2CoolColumn;

impl ColumnSchema for NetworkInfoV2CoolColumn {
    type Key = KeyV2;
    type Value = NetworkInfoV2Cool;

    const COLUMN_FAMILY_NAME: &'static str = "network_info_v2_cool";
}

// Version 2 schema - Uncool networks
pub struct NetworkInfoV2UncoolColumn;

impl ColumnSchema for NetworkInfoV2UncoolColumn {
    type Key = KeyV2;
    type Value = NetworkInfoV2Uncool;

    const COLUMN_FAMILY_NAME: &'static str = "network_info_v2_uncool";
}

pub const CFS_V2: &[ColumnDescriptor] = &[
    ColumnDescriptor::new::<NetworkInfoV2CoolColumn>(),
    ColumnDescriptor::new::<NetworkInfoV2UncoolColumn>(),
];

impl Builder<'_> {
    pub fn open_sample(path: &Path) -> Result<Self, DBOpenError> {
        let cfs = [ColumnDescriptor::new::<NetworkInfoV0Column>()];
        Self::open(path, &cfs)
    }

    pub fn sample_migrate_v0_v1(self) -> Result<Self, DBOpenError> {
        // Create and populate the new V1 column family
        self.add_cfs(CFS_V1, |db| {
            // Iterate over all V0 entries
            for key in db.keys::<NetworkInfoV0Column>()? {
                let key = key?;
                migration_failpoint()?;
                if let Some(v0_value) = db.get::<NetworkInfoV0Column>(&key)? {
                    // Transform V0 to V1 (widen num_failures to u64, add is_cool field)
                    let v1_value = NetworkInfoV1 {
                        height: v0_value.height,
                        num_beans: v0_value.num_beans,
                        num_failures: v0_value.num_failures as u64,
                        is_cool: v0_value.num_beans > 100, // Networks with >100 beans are cool
                    };
                    db.put::<NetworkInfoV1Column>(&key, &v1_value)?;
                }
            }
            migration_failpoint()?;
            Ok(())
        })?
        // Drop the old V0 column family
        .drop_cfs(CFS_V0)
    }

    pub fn sample_migrate_v1_v2(self) -> Result<Self, DBOpenError> {
        // Create and populate the new V2 column families (cool and uncool)
        self.add_cfs(CFS_V2, |db| {
            // Iterate over all V1 entries
            for key in db.keys::<NetworkInfoV1Column>()? {
                let network_id = key?;
                migration_failpoint()?;
                if let Some(v1_value) = db.get::<NetworkInfoV1Column>(&network_id)? {
                    // Create a V2 key with network_id and height from V1
                    let v2_key = KeyV2 {
                        network_id,
                        height: v1_value.height,
                    };

                    // Split based on is_cool flag
                    if v1_value.is_cool {
                        let v2_cool = NetworkInfoV2Cool {
                            num_beans: v1_value.num_beans,
                            sunglasses: true, // Cool networks wear sunglasses by default
                        };
                        db.put::<NetworkInfoV2CoolColumn>(&v2_key, &v2_cool)?;
                    } else {
                        let v2_uncool = NetworkInfoV2Uncool {
                            num_beans: v1_value.num_beans,
                            num_failures: v1_value.num_failures,
                        };
                        db.put::<NetworkInfoV2UncoolColumn>(&v2_key, &v2_uncool)?;
                    }
                }
            }
            migration_failpoint()?;
            Ok(())
        })?
        // Drop the old V1 column family
        .drop_cfs(CFS_V1)
    }
}

fn migration_failpoint() -> Result<(), DBMigrationErrorDetails> {
    // Failpoint for testing partial migration recovery
    fail::fail_point!("sample_migrate", |ret| {
        ret.map_or(Ok(()), |s| {
            Err(DBMigrationErrorDetails::Custom(eyre::eyre!(
                "failpoint triggered: {s}"
            )))
        })
    });
    Ok(())
}

pub const DATA_V0: [(KeyV0, NetworkInfoV0); 5] = [
    (
        NetworkId::new(42),
        NetworkInfoV0 {
            height: Height::new(100),
            num_beans: 50,
            num_failures: 2,
        },
    ),
    (
        NetworkId::new(137),
        NetworkInfoV0 {
            height: Height::new(200),
            num_beans: 150,
            num_failures: 5,
        },
    ),
    (
        NetworkId::new(256),
        NetworkInfoV0 {
            height: Height::new(300),
            num_beans: 75,
            num_failures: 10,
        },
    ),
    (
        NetworkId::new(789),
        NetworkInfoV0 {
            height: Height::new(400),
            num_beans: 200,
            num_failures: 0,
        },
    ),
    (
        NetworkId::new(513),
        NetworkInfoV0 {
            height: Height::new(500),
            num_beans: 25,
            num_failures: 15,
        },
    ),
];

pub const DATA_V1: [(KeyV1, NetworkInfoV1); 8] = [
    // Migrated from V0
    (
        NetworkId::new(42),
        NetworkInfoV1 {
            height: Height::new(100),
            num_beans: 50,
            num_failures: 2,
            is_cool: false, // 50 beans <= 100
        },
    ),
    (
        NetworkId::new(137),
        NetworkInfoV1 {
            height: Height::new(200),
            num_beans: 150,
            num_failures: 5,
            is_cool: true, // 150 beans > 100
        },
    ),
    (
        NetworkId::new(256),
        NetworkInfoV1 {
            height: Height::new(300),
            num_beans: 75,
            num_failures: 10,
            is_cool: false, // 75 beans <= 100
        },
    ),
    (
        NetworkId::new(789),
        NetworkInfoV1 {
            height: Height::new(400),
            num_beans: 200,
            num_failures: 0,
            is_cool: true, // 200 beans > 100
        },
    ),
    (
        NetworkId::new(513),
        NetworkInfoV1 {
            height: Height::new(500),
            num_beans: 25,
            num_failures: 15,
            is_cool: false, // 25 beans <= 100
        },
    ),
    // New entries in V1
    (
        NetworkId::new(101),
        NetworkInfoV1 {
            height: Height::new(150),
            num_beans: 120,
            num_failures: 3,
            is_cool: true,
        },
    ),
    (
        NetworkId::new(333),
        NetworkInfoV1 {
            height: Height::new(250),
            num_beans: 80,
            num_failures: 7,
            is_cool: false,
        },
    ),
    (
        NetworkId::new(888),
        NetworkInfoV1 {
            height: Height::new(600),
            num_beans: 500,
            num_failures: 1,
            is_cool: true,
        },
    ),
];

pub const DATA_V1_NEW_START: usize = 5;

pub const DATA_V2_COOL: [(KeyV2, NetworkInfoV2Cool); 6] = [
    // Migrated from V1
    (
        KeyV2 {
            network_id: NetworkId::new(137),
            height: Height::new(200),
        },
        NetworkInfoV2Cool {
            num_beans: 150,
            sunglasses: true,
        },
    ),
    (
        KeyV2 {
            network_id: NetworkId::new(789),
            height: Height::new(400),
        },
        NetworkInfoV2Cool {
            num_beans: 200,
            sunglasses: true,
        },
    ),
    (
        KeyV2 {
            network_id: NetworkId::new(101),
            height: Height::new(150),
        },
        NetworkInfoV2Cool {
            num_beans: 120,
            sunglasses: true,
        },
    ),
    (
        KeyV2 {
            network_id: NetworkId::new(888),
            height: Height::new(600),
        },
        NetworkInfoV2Cool {
            num_beans: 500,
            sunglasses: true,
        },
    ),
    // New entries in V2
    (
        KeyV2 {
            network_id: NetworkId::new(666),
            height: Height::new(350),
        },
        NetworkInfoV2Cool {
            num_beans: 300,
            sunglasses: true,
        },
    ),
    (
        KeyV2 {
            network_id: NetworkId::new(42),
            height: Height::new(750),
        },
        NetworkInfoV2Cool {
            num_beans: 180,
            sunglasses: true,
        },
    ),
];

pub const DATA_V2_COOL_NEW_START: usize = 4;

pub const DATA_V2_UNCOOL: [(KeyV2, NetworkInfoV2Uncool); 6] = [
    // Migrated from V1
    (
        KeyV2 {
            network_id: NetworkId::new(42),
            height: Height::new(100),
        },
        NetworkInfoV2Uncool {
            num_beans: 50,
            num_failures: 2,
        },
    ),
    (
        KeyV2 {
            network_id: NetworkId::new(256),
            height: Height::new(300),
        },
        NetworkInfoV2Uncool {
            num_beans: 75,
            num_failures: 10,
        },
    ),
    (
        KeyV2 {
            network_id: NetworkId::new(513),
            height: Height::new(500),
        },
        NetworkInfoV2Uncool {
            num_beans: 25,
            num_failures: 15,
        },
    ),
    (
        KeyV2 {
            network_id: NetworkId::new(333),
            height: Height::new(250),
        },
        NetworkInfoV2Uncool {
            num_beans: 80,
            num_failures: 7,
        },
    ),
    // New entries in V2
    (
        KeyV2 {
            network_id: NetworkId::new(222),
            height: Height::new(180),
        },
        NetworkInfoV2Uncool {
            num_beans: 60,
            num_failures: 20,
        },
    ),
    (
        KeyV2 {
            network_id: NetworkId::new(777),
            height: Height::new(420),
        },
        NetworkInfoV2Uncool {
            num_beans: 90,
            num_failures: 4,
        },
    ),
];

pub const DATA_V2_UNCOOL_NEW_START: usize = 4;
