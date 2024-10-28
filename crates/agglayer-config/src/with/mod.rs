//! Helper (de)serializers to be used with `#[serde(with)]` and `#[serde_as]`.

mod human_duration;

/// A config-friendly [std::time::Duration].
///
/// Can be specified as either human-readable string, such as `"1h"` or
/// `"15min"`, or as an integer interpreted as the number of seconds.
pub use human_duration::HumanDuration;
