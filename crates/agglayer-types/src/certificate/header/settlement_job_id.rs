use ulid::Ulid;

#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    derive_more::AsRef,
    derive_more::Display,
    derive_more::From,
    derive_more::Into,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(transparent)]
#[derive(Hash)]
pub struct SettlementJobId(Ulid);

impl SettlementJobId {
    pub const fn new(ulid: Ulid) -> Self {
        SettlementJobId(ulid)
    }

    pub const fn as_ulid(&self) -> Ulid {
        self.0
    }
}
