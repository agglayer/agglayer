macro_rules! required_field {
    ($from:expr, $field:ident => try_into::<$to:ty>) => {
        $from
            .$field
            .ok_or_else(|| {
                $crate::types::settlement::compat::Error::missing_field(stringify!($field))
            })
            .and_then(|value| {
                ::core::convert::TryInto::<$to>::try_into(value).map_err(|error| {
                    let error: $crate::types::settlement::compat::Error = error.into();
                    error.inside_field(stringify!($field))
                })
            })?
    };
    ($from:expr, $field:ident => into::<$to:ty>) => {
        $from
            .$field
            .ok_or_else(|| {
                $crate::types::settlement::compat::Error::missing_field(stringify!($field))
            })
            .map(<$to>::from)?
    };
}

mod client_error;
mod contract_call;
mod error;
mod settlement_attempt;
mod settlement_job;
mod tx_result;

pub use error::Error;
