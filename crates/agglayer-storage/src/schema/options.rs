/// Column compression
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Default)]
pub enum ColumnCompressionType {
    /// No compression.
    None,

    /// Lz4 compression.
    #[default]
    Lz4,
}

impl ColumnCompressionType {
    pub const DEFAULT: Self = Self::Lz4;
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Default)]
pub enum PrefixExtractor {
    /// Rocksdb default.
    #[default]
    Default,

    /// Fixed size prefix extractor.
    Fixed { size: usize },
}

impl PrefixExtractor {
    pub const DEFAULT: Self = Self::Default;
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Default)]
pub struct ColumnOptions {
    pub compression: ColumnCompressionType,
    pub prefix_extractor: PrefixExtractor,
}

impl ColumnOptions {
    pub const DEFAULT: Self = Self {
        compression: ColumnCompressionType::DEFAULT,
        prefix_extractor: PrefixExtractor::DEFAULT,
    };
}
