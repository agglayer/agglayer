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

mod address;
mod aggchain_data;
mod certificate;
mod certificate_header;
mod certificate_id;
mod epoch_configuration;
mod error;
mod error_kinds;

pub use error::{Error, ErrorKind};

#[cfg(test)]
pub mod tests;
