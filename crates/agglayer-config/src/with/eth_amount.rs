use alloy_primitives::utils::{parse_units, ParseUnits, Unit, UnitsError};
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
        const GWEI: u128 = 10_u128.pow(Unit::GWEI.get() as u32);
        let gwei_amount = amount / GWEI;
        if gwei_amount * GWEI == *amount {
            Self(format!("{gwei_amount}gwei"))
        } else {
            Self(format!("{amount}wei"))
        }
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
    // Edge cases: zero values (used in production)
    #[case("0gwei", Ok(0))]
    #[case("0wei", Ok(0))]
    #[case("0eth", Ok(0))]
    // Edge cases: empty string and invalid inputs
    #[case("", Err(()))]
    // Edge cases: very large numbers
    #[case("340282366920938463463374607431768211455wei", Ok(u128::MAX))]
    // Edge cases: invalid units
    #[case("100xyz", Err(()))]
    #[case("1invalid", Err(()))]
    // Edge cases: leading spaces
    #[case(" 1gwei", Err(()))]
    #[case(" 100wei", Err(()))]
    // Edge cases: multiple spaces (alloy's parse_units handles this)
    #[case("100  gwei", Ok(100_000_000_000))]
    #[case("1  eth", Ok(1_000_000_000_000_000_000))]
    fn deserialize(#[case] amount_str: &str, #[case] amount: Result<u128, ()>) {
        let toml_val: toml::Value = toml!(amount = amount_str).into();
        let config = TestConfig::from_toml(toml_val);
        let expected = amount.map(TestConfig::from_wei);
        assert_eq!(config.as_ref().ok(), expected.as_ref().ok());

        if let Ok(config) = config {
            // One more roundtrip and check it is still the same value
            let toml_val: toml::Value = config.to_toml();
            let config_roundtrip = TestConfig::from_toml(toml_val).unwrap();
            assert_eq!(config_roundtrip, config);
        }
    }
}
