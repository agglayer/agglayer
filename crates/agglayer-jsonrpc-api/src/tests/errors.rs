//! Tests for rendering RPC errors

use agglayer_types::{CertificateId, Digest};
use jsonrpsee::types::ErrorObjectOwned;

use crate::Error;

#[rstest::rstest]
#[case(
    "cert_notfound",
    agglayer_rpc::CertificateRetrievalError::NotFound { certificate_id: CertificateId::new(Digest([0x51; 32])) }
)]
#[case(
    "method_disabled",
    Error::MethodDisabled { method: "interop_sendTx" }
)]
fn rpc_error_rendering(#[case] name: &str, #[case] err: impl Into<Error>) {
    let err: Error = err.into();
    let debug_str = format!("{err:?}");
    let err_obj = ErrorObjectOwned::from(err);
    let err_json_string = {
        // Going through an extra encode/decode here helps normalize the output.
        let json_string = serde_json::to_string(&err_obj).unwrap();
        let json = serde_json::from_str::<serde_json::Value>(&json_string).unwrap();
        serde_json::to_string_pretty(&json).unwrap()
    };

    insta::assert_snapshot!(name, err_json_string, &debug_str);
}
