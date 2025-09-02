// Helper macro used by the rest of this module
macro_rules! required_field {
    ($from:expr, $field:ident) => {
        $from
            .$field
            .ok_or(Error::missing_field(stringify!($field)))?
            .try_into()
            .map_err(|e: Error| e.inside_field(stringify!($field)))?
    };
}

mod certificate;
mod certificate_header;
mod certificate_id;
mod epoch_configuration;
mod error_kinds;
mod network_state;

pub use agglayer_interop::grpc::compat::v1::{Error, ErrorKind};

#[cfg(test)]
pub mod tests;
