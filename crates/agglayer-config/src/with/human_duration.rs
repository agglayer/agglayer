use std::time::Duration;

use serde_with::serde_conv;

serde_conv!(pub HumanDuration, Duration, HumanDurationImpl::new, HumanDurationImpl::get);

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
#[serde(untagged)]
enum HumanDurationImpl {
    Secs(u64),
    Human(#[serde(with = "humantime_serde")] Duration),
}

impl HumanDurationImpl {
    fn new(value: &Duration) -> Self {
        Self::Human(*value)
    }

    pub fn get(self) -> Result<Duration, std::convert::Infallible> {
        match self {
            Self::Secs(secs) => Ok(Duration::from_secs(secs)),
            Self::Human(duration) => Ok(duration),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use toml::toml;

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
    struct TestConfig {
        #[serde(with = "super::HumanDuration")]
        time: Duration,
    }

    impl TestConfig {
        fn from_secs(secs: u64) -> Self {
            let time = Duration::from_secs(secs);
            Self { time }
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
            (TestConfig::from_secs(10)).to_toml(),
            toml!(time = "10s").into(),
        );
        assert_eq!(
            (TestConfig::from_secs(60)).to_toml(),
            toml!(time = "1m").into(),
        );
        assert_eq!(
            (TestConfig::from_secs(600)).to_toml(),
            toml!(time = "10m").into(),
        );
        assert_eq!(
            (TestConfig::from_secs(601)).to_toml(),
            toml!(time = "10m 1s").into(),
        );
        assert_eq!(
            (TestConfig::from_secs(3600)).to_toml(),
            toml!(time = "1h").into(),
        );
    }

    #[test]
    fn deserialize() {
        assert_eq!(
            TestConfig::from_secs(10),
            TestConfig::from_toml(toml!(time = 10).into()).unwrap(),
        );
        assert_eq!(
            TestConfig::from_secs(10),
            TestConfig::from_toml(toml!(time = "10s").into()).unwrap(),
        );
        assert_eq!(
            TestConfig::from_toml(toml!(time = 70).into()).unwrap(),
            TestConfig::from_toml(toml!(time = "1min 10s").into()).unwrap(),
        );
        assert!(TestConfig::from_toml("10s".into()).is_err());
        assert!(TestConfig::from_toml(10.into()).is_err());
    }
}
