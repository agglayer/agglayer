// Helper macro used by the rest of this module
macro_rules! required_field {
    ($from:expr, $field:ident) => {
        $from
            .$field
            .ok_or(Error::MissingField(stringify!($field)))?
            .try_into()
            .map_err(|e| Error::ParsingField(stringify!($field), Box::new(e)))?
    };
}

mod address;
mod aggchain_proof;
mod bridge_exit;
mod bytes;
mod certificate;
mod certificate_header;
mod certificate_id;
mod claim;
mod digest;
mod epoch_configuration;
mod error;
mod global_index;
mod imported_bridge_exit;
mod l1_info_tree_leaf;
mod merkle_proof;
mod token_info;
mod u256;

pub use error::Error;
