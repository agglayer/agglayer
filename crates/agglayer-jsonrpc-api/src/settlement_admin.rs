//! Shared error handling for settlement administration RPC methods.

use agglayer_types::RpcErrorCode;

use crate::error::Error;

/// Turns a settlement-service error report into the private admin RPC error
/// contract.
pub(crate) fn map_admin_error(report: eyre::Report) -> Error {
    match report.downcast_ref::<RpcErrorCode>() {
        Some(&code) => Error::Classified {
            code,
            message: format!("{report:?}"),
        },
        None => {
            tracing::error!(?report, "Admin operation failed with unclassified error");
            Error::internal(format!("{report:?}"))
        }
    }
}
