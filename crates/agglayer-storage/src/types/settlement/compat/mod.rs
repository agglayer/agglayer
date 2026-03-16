macro_rules! required_field {
    ($from:expr, $field:ident => $converter:expr) => {
        $from
            .$field
            .ok_or_else(|| {
                $crate::types::settlement::compat::Error::missing_field(stringify!($field))
            })
            .and_then(|value| {
                ($converter)(value).map_err(|error: $crate::types::settlement::compat::Error| {
                    error.inside_field(stringify!($field))
                })
            })?
    };
}

mod client_error;
mod contract_call;
mod error;
mod primitives;
mod settlement_attempt;
mod settlement_job;
mod tx_result;

pub use error::Error;
