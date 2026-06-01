use super::{options::ColumnOptions, Codec};

pub trait ColumnSchema {
    type Key: Codec;
    type Value: Codec;

    const COLUMN_FAMILY_NAME: &'static str;

    const COLUMN_OPTIONS: ColumnOptions = ColumnOptions::DEFAULT;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ColumnDescriptor {
    name: &'static str,
    options: &'static ColumnOptions,
}

impl ColumnDescriptor {
    pub const fn new<C: ColumnSchema>() -> Self {
        Self {
            name: C::COLUMN_FAMILY_NAME,
            options: &C::COLUMN_OPTIONS,
        }
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub const fn options(&self) -> &'static ColumnOptions {
        self.options
    }
}
