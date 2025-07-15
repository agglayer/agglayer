#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    derive_more::Display,
    derive_more::From,
    serde::Deserialize,
    serde::Serialize,
)]
#[cfg_attr(feature = "testutils", derive(arbitrary::Arbitrary))]
#[serde(transparent)]
pub struct Height(u64);

impl Height {
    pub const ZERO: Height = Height::new(0);

    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    pub const fn new(height: u64) -> Height {
        Height(height)
    }

    #[must_use = "The value of the next height is returned but not used"]
    pub const fn next(&self) -> Height {
        Height(self.0.checked_add(1).expect("Height overflow"))
    }

    pub const fn increment(&mut self) {
        *self = self.next();
    }

    pub const fn distance_since(&self, o: &Height) -> u64 {
        self.0
            .checked_sub(o.0)
            .expect("Subtracting to negative values")
    }
}

