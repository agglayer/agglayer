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

    pub fn to_rocksdb_compression_type(self) -> rocksdb::DBCompressionType {
        match self {
            Self::None => rocksdb::DBCompressionType::None,
            Self::Lz4 => rocksdb::DBCompressionType::Lz4,
        }
    }
}

impl From<ColumnCompressionType> for rocksdb::DBCompressionType {
    fn from(value: ColumnCompressionType) -> Self {
        value.to_rocksdb_compression_type()
    }
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

    pub fn apply_to_rocksdb_options(self, opts: &mut rocksdb::Options) {
        match self {
            Self::Default => {}
            Self::Fixed { size } => {
                opts.set_prefix_extractor(rocksdb::SliceTransform::create_fixed_prefix(size));
            }
        }
    }
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

    pub fn to_rocksdb_options(&self) -> rocksdb::Options {
        let mut opts = rocksdb::Options::default();
        opts.set_compression_type(self.compression.into());
        self.prefix_extractor.apply_to_rocksdb_options(&mut opts);
        opts
    }
}

impl From<&ColumnOptions> for rocksdb::Options {
    fn from(value: &ColumnOptions) -> Self {
        value.to_rocksdb_options()
    }
}
