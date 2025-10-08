use alloy_primitives::utils::{parse_units, ParseUnits, UnitsError};
use serde_with::serde_conv;

serde_conv!(pub EthAmount, u128, EthAmountImpl::new, EthAmountImpl::get);

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Amount must include a unit (e.g. eth, gwei, or wei).")]
    MissingUnit,

    #[error(transparent)]
    UnitParsing(UnitsError),

    #[error("Amount exceeds u128::MAX")]
    Overflow,

    #[error("Negative amounts not supported")]
    NegativeAmount,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct EthAmountImpl(String);

impl EthAmountImpl {
    fn new(amount: &u128) -> Self {
        Self(format!("{amount}wei"))
    }

    fn get(self) -> Result<u128, Error> {
        let amount_str: &str = &self.0;
        // Split into number part and unit
        let unit_pos = amount_str
            .find(|c: char| c.is_ascii_alphabetic())
            .ok_or(Error::MissingUnit)?;
        let (number_part, unit) = amount_str.split_at(unit_pos);
        let number_part = number_part.trim_end();

        // Parse using alloy's parse_units
        let parsed = parse_units(number_part, unit).map_err(Error::UnitParsing)?;

        // Convert ParseUnits to u128
        match parsed {
            ParseUnits::U256(value) => u128::try_from(value).map_err(|_| Error::Overflow),
            ParseUnits::I256(_) => Err(Error::NegativeAmount),
        }
    }
}

#[cfg(test)]
mod tests {
    use toml::toml;

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
    struct TestConfig {
        #[serde(with = "super::EthAmount")]
        amount: u128,
    }

    impl TestConfig {
        fn from_wei(wei: u128) -> Self {
            Self { amount: wei }
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
            TestConfig::from_wei(1_000_000_000).to_toml(),
            toml!(amount = "1000000000wei").into(),
        );
    }

    #[rstest::rstest]
    #[case("1000000000wei", Ok(1_000_000_000))]
    #[case("1000000000 wei", Ok(1_000_000_000))]
    #[case("1000000000", Err(()))]
    #[case("1gwei", Ok(1_000_000_000))]
    #[case("100gwei", Ok(100_000_000_000))]
    #[case("0.5gwei", Ok(500_000_000))]
    #[case("1eth", Ok(1_000_000_000_000_000_000))]
    #[case("0.1eth", Ok(100_000_000_000_000_000))]
    #[case("2.5eth", Ok(2_500_000_000_000_000_000))]
    #[case("1ETH", Ok(1_000_000_000_000_000_000))]
    #[case("100GWEI", Ok(100_000_000_000))]
    #[case("1000WEI", Ok(1_000))]
    fn deserialize(#[case] amount_str: &str, #[case] amount: Result<u128, ()>) {
        let toml_val: toml::Value = toml!(amount = amount_str).into();
        let from_toml = TestConfig::from_toml(toml_val).map_err(|_| ());
        let expected = amount.map(TestConfig::from_wei);
        assert_eq!(from_toml, expected);
    }
}
