mod interfaces;

pub use interfaces::{
    reader::{
        network_info_reader::NetworkInfoReader, DebugReader, EpochStoreReader, MetadataReader,
        PendingCertificateReader, PerEpochReader, StateReader,
    },
    writer::{
        DebugWriter, EpochStoreWriter, MetadataWriter, PendingCertificateWriter, PerEpochWriter,
        StateWriter, UpdateEvenIfAlreadyPresent,
    },
};

pub mod debug;
pub mod epochs;
pub mod pending;
pub mod per_epoch;
pub mod state;

macro_rules! try_digest {
    ($value:expr, $err_msg:expr) => {
        agglayer_types::Digest::try_from(&*$value).map_err(|_| {
            $crate::error::Error::Unexpected(format!(
                "Unable to deserialize {} into a Digest",
                $err_msg
            ))
        })
    };
}

macro_rules! expected_type_or_fail {
    ($value:expr, $pattern:pat, $return_value:expr, $err_msg:expr) => {
        match $value {
            Some($crate::types::network_info::v0::NetworkInfoValue {
                value: Some($pattern),
            }) => Ok(Some($return_value)),
            None => Ok(None),
            _ => Err($crate::error::Error::Unexpected($err_msg.to_string())),
        }
    };
}

pub(crate) use expected_type_or_fail;
pub(crate) use try_digest;
