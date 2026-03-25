use serde_with::serde_conv;

serde_conv!(pub HumanSize, usize, HumanSizeImpl::new, HumanSizeImpl::get);

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
#[serde(untagged)]
enum HumanSizeImpl {
    Bytes(usize),
    Human(byte_unit::Byte),
}

impl HumanSizeImpl {
    fn new(value: &usize) -> Self {
        Self::Human(byte_unit::Byte::from_u64(*value as u64))
    }

    pub fn get(self) -> Result<usize, std::convert::Infallible> {
        match self {
            Self::Bytes(bytes) => Ok(bytes),
            Self::Human(byte) => Ok(byte.as_u64() as usize),
        }
    }
}

#[cfg(test)]
mod tests {
    use toml::toml;

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
    struct TestConfig {
        #[serde(with = "super::HumanSize")]
        size: usize,
    }

    impl TestConfig {
        fn from_bytes(bytes: usize) -> Self {
            Self { size: bytes }
        }

        fn from_toml(value: toml::Value) -> Result<Self, toml::de::Error> {
            value.try_into()
        }

        fn to_toml(&self) -> toml::Value {
            toml::Value::try_from(self).unwrap()
        }
    }

    #[test]
    fn serialize() {
        assert_eq!(
            TestConfig::from_bytes(1024).to_toml(),
            toml!(size = "1 KiB").into(),
        );
        assert_eq!(
            TestConfig::from_bytes(1024 * 1024).to_toml(),
            toml!(size = "1 MiB").into(),
        );
        assert_eq!(
            TestConfig::from_bytes(64 * 1024 * 1024).to_toml(),
            toml!(size = "64 MiB").into(),
        );
    }

    #[test]
    fn deserialize() {
        assert_eq!(
            TestConfig::from_bytes(1024),
            TestConfig::from_toml(toml!(size = 1024).into()).unwrap(),
        );
        assert_eq!(
            TestConfig::from_bytes(1024),
            TestConfig::from_toml(toml!(size = "1KiB").into()).unwrap(),
        );
        assert_eq!(
            TestConfig::from_bytes(64 * 1024 * 1024),
            TestConfig::from_toml(toml!(size = "64MiB").into()).unwrap(),
        );
        assert_eq!(
            TestConfig::from_bytes(64 * 1024 * 1024),
            TestConfig::from_toml(toml!(size = "64 MiB").into()).unwrap(),
        );
    }
}
