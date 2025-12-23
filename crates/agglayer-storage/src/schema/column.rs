use super::Codec;

pub trait ColumnSchema {
    type Key: Codec;
    type Value: Codec;

    const COLUMN_FAMILY_NAME: &'static str;
}
