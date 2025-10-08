/// Multiplier is a quantity specifying a scaling factor of some sort.
///
/// It is internally implemented as a `u64` fixed point scaled by 1000.
/// It defaults to scaling by 1.0.
#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, serde::Serialize, serde::Deserialize,
)]
#[serde(try_from = "f32", into = "f32")]
pub struct Multiplier(u64);

#[derive(PartialEq, Eq, Debug, Clone, thiserror::Error)]
pub enum FromF32Error {
    #[error("Multiplier out of range")]
    OutOfRange,
}

impl Multiplier {
    pub const SCALE: u64 = 1000;
    pub const ONE: Self = Self(Self::SCALE);

    pub const fn from_u64_per_1000(x: u64) -> Self {
        Self(x)
    }

    pub fn try_from_f32_round(x: f32) -> Result<Self, FromF32Error> {
        let x = (x * Self::SCALE as f32).round();
        (0.0..=(u64::MAX as f32))
            .contains(&x)
            .then_some(Self(x as u64))
            .ok_or(FromF32Error::OutOfRange)
    }

    pub const fn as_u64_per_1000(self) -> u64 {
        self.0
    }

    pub fn as_f32(self) -> f32 {
        self.0 as f32 / Self::SCALE as f32
    }
}

impl Default for Multiplier {
    fn default() -> Self {
        Self::ONE
    }
}

impl TryFrom<f32> for Multiplier {
    type Error = FromF32Error;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        Self::try_from_f32_round(value)
    }
}

impl From<Multiplier> for f32 {
    fn from(value: Multiplier) -> Self {
        value.as_f32()
    }
}
