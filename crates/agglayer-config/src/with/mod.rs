//! Helper (de)serializers to be used with `#[serde(with)]` and `#[serde_as]`.

mod eth_amount;
mod human_duration;
mod human_size;

/// A config-friendly Ethereum amount ([u128] in wei).
///
/// Specified as a string with unit suffix such as `"1eth"`, `"100gwei"`, etc.
pub use eth_amount::EthAmount;
/// A config-friendly [std::time::Duration].
///
/// Can be specified as either human-readable string, such as `"1h"` or
/// `"15min"`, or as an integer interpreted as the number of seconds.
pub use human_duration::HumanDuration;
/// A config-friendly byte size ([usize]).
///
/// Can be specified as either human-readable string, such as `"64MiB"` or
/// `"1GiB"`, or as an integer interpreted as the number of bytes.
pub use human_size::HumanSize;
