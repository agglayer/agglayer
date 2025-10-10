/// Multiplier is a quantity specifying a scaling factor of some sort.
///
/// It is internally implemented as a `u64` fixed point scaled by 1000.
/// It defaults to scaling by 1.0.
#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, serde::Serialize, serde::Deserialize,
)]
#[serde(try_from = "f64", into = "f64")]
pub struct Multiplier(u64);

#[derive(PartialEq, Eq, Debug, Clone, thiserror::Error)]
pub enum FromF64Error {
    #[error("Multiplier out of range")]
    OutOfRange,

    #[error("Multiplier supports up to 3 decimal places.")]
    Imprecise,
}

impl Multiplier {
    pub const ONE: Self = Self(Self::SCALE);
    pub const ZERO: Self = Self(0);
    pub const MAX: Self = Self(u64::MAX);

    const FROM_F64_TOLERANCE: f64 = 1e-6;
    const FROM_F64_MAX: u64 = ((1_u64 << f64::MANTISSA_DIGITS) as f64).next_down() as u64;
    const SCALE: u64 = 1000;

    pub const fn from_u64_per_1000(x: u64) -> Self {
        Self(x)
    }

    /// Get a multiplier from `f64`, failing if we are out of range
    /// or if excessive decimals are supplied.
    pub fn try_from_f64_check(x: f64) -> Result<Self, FromF64Error> {
        // We first get the rounded conversion, check the delta against the original
        // value and fail if there is too much precision loss, indicating there were
        // too many decimals in the original floating point number.
        let r = Self::try_from_f64_round(x)?;
        let delta = r.as_u64_per_1000() as f64 - Self::scale_f64(x);

        // We still allow some tolerance to account for the fact that floating point
        // cannot represent base-10 decimals (such as 1.2) exactly.
        (delta.abs() <= Self::FROM_F64_TOLERANCE)
            .then_some(r)
            .ok_or(FromF64Error::Imprecise)
    }

    /// Get a multiplier from `f64`, failing if we are out of range.
    pub fn try_from_f64_round(x: f64) -> Result<Self, FromF64Error> {
        let x = Self::scale_f64(x).round();
        (0.0..=Self::FROM_F64_MAX as f64)
            .contains(&x)
            .then_some(Self(x as u64))
            .ok_or(FromF64Error::OutOfRange)
    }

    pub const fn as_u64_per_1000(self) -> u64 {
        self.0
    }

    pub fn as_f64(self) -> f64 {
        self.0 as f64 / Self::SCALE as f64
    }

    fn scale_f64(x: f64) -> f64 {
        x * Self::SCALE as f64
    }
}

impl Default for Multiplier {
    fn default() -> Self {
        Self::ONE
    }
}

impl TryFrom<f64> for Multiplier {
    type Error = FromF64Error;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::try_from_f64_check(value)
    }
}

impl From<Multiplier> for f64 {
    fn from(value: Multiplier) -> Self {
        value.as_f64()
    }
}

#[cfg(test)]
mod test {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(0.0, 0)]
    #[case(1.0, 1000)]
    #[case(1.5, 1500)]
    #[case(2.0, 2000)]
    #[case(0.001, 1)]
    #[case(0.123, 123)]
    #[case(10.5, 10500)]
    fn test_try_from_f64_check_valid_values(#[case] input: f64, #[case] expected: u64) {
        let result = Multiplier::try_from_f64_check(input).unwrap();
        assert_eq!(result, Multiplier::from_u64_per_1000(expected));
    }

    #[rstest]
    #[case(-1.0)]
    #[case(-0.001)]
    #[case(-100.0)]
    #[case(1.001 * u64::MAX as f64)]
    fn test_try_from_f64_check_out_of_range(#[case] input: f64) {
        assert_eq!(
            Multiplier::try_from_f64_check(input).unwrap_err(),
            FromF64Error::OutOfRange
        );
    }

    #[rstest]
    #[case(1.2345)]
    #[case(0.0001)]
    #[case(2.12345)]
    fn test_try_from_f64_check_imprecise(#[case] input: f64) {
        assert_eq!(
            Multiplier::try_from_f64_check(input).unwrap_err(),
            FromF64Error::Imprecise
        );
    }

    #[rstest]
    #[case(0.0, 0)]
    #[case(1.0, 1000)]
    #[case(1.5, 1500)]
    #[case(2.0, 2000)]
    #[case(1.2345, 1235)]
    #[case(1.2344, 1234)]
    #[case(0.0001, 0)]
    #[case(0.0006, 1)]
    fn test_try_from_f64_round_valid_values(#[case] input: f64, #[case] expected: u64) {
        let result = Multiplier::try_from_f64_round(input).unwrap();
        assert_eq!(result, Multiplier::from_u64_per_1000(expected));
    }

    #[rstest]
    #[case(-1.0)]
    #[case(-0.001)]
    #[case(-100.0)]
    fn test_try_from_f64_round_out_of_range(#[case] input: f64) {
        assert_eq!(
            Multiplier::try_from_f64_round(input).unwrap_err(),
            FromF64Error::OutOfRange
        );
    }

    #[rstest]
    #[case(1000)]
    #[case(1500)]
    #[case(2000)]
    #[case(123)]
    fn test_roundtrip(#[case] value: u64) {
        let original = Multiplier::from_u64_per_1000(value);
        let as_f64 = original.as_f64();
        let back = Multiplier::try_from_f64_check(as_f64).unwrap();
        assert_eq!(original, back);
    }
}
