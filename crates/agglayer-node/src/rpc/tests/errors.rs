//! Tests for rendering RPC errors

use std::time::Duration;

use agglayer_types::CertificateStatus;
use ethers::{
    providers::ProviderError,
    types::{Bytes, SignatureError as EthSignatureError, H160, H256},
};
use jsonrpsee::types::ErrorObjectOwned;

use crate::{
    kernel::{self, ZkevmNodeVerificationError},
    rate_limiting::{self, component, Component},
    rpc::Error,
};

type RpcProvider = ethers::providers::Provider<ethers::providers::Http>;
type ContractError = ethers::contract::ContractError<RpcProvider>;
type SignatureError = kernel::SignatureVerificationError<RpcProvider>;
type SettlementError = kernel::SettlementError<RpcProvider>;
type WallClockLimitedInfo = <component::SendTx as Component>::LimitedInfo;
type CertError = agglayer_certificate_orchestrator::InitialCheckError;

#[rstest::rstest]
#[case("rollup_not_reg", Error::rollup_not_registered(1337))]
#[case(
    "sig_invalid_len",
    Error::signature_mismatch(SignatureError::CouldNotRecoverSigner(
        EthSignatureError::InvalidLength(42)
    ))
)]
#[case("sig_verif", Error::signature_mismatch(SignatureError::CouldNotRecoverSigner(
    EthSignatureError::VerificationError(H160([0x11; 20]), H160([0x22; 20]))
)))]
#[case(
    "sig_recov",
    Error::signature_mismatch(SignatureError::CouldNotRecoverSigner(
        EthSignatureError::RecoveryError
    ))
)]
#[case(
    "sig_signer",
    Error::signature_mismatch(SignatureError::InvalidSigner{
        signer: H160([0x33; 20]),
        trusted_sequencer: H160([0x44; 20]),
    })
)]
#[case("sig_contract", Error::signature_mismatch(ContractError::ContractNotDeployed.into()))]
#[case("dry_run", Error::dry_run("Dry run flopped.".into()))]
#[case(
    "root_bad_rollup",
    Error::root_verification(ZkevmNodeVerificationError::InvalidRollupId(13))
)]
#[case(
    "root_rpc",
    Error::root_verification(ZkevmNodeVerificationError::RpcError(
        jsonrpsee::core::client::Error::Custom("Node smells too much".into())
    ))
)]
#[case(
    "root_state",
    Error::root_verification(ZkevmNodeVerificationError::InvalidStateRoot {
        expected: H256([0x55; 32]),
        got: H256([0x66; 32]),
    })
)]
#[case(
    "root_exit",
    Error::root_verification(ZkevmNodeVerificationError::InvalidExitRoot {
        expected: H256([0x77; 32]),
        got: H256([0x88; 32]),
    })
)]
#[case("settle_receipt", Error::settlement(SettlementError::NoReceipt))]
#[case(
    "settle_io",
    Error::settlement(SettlementError::ProviderError(ProviderError::UnsupportedRPC))
)]
#[case(
    "settle_contract",
    Error::settlement(SettlementError::ContractError(ContractError::Revert(
        Bytes::from_static(b"foo")
    )))
)]
#[case(
    "settle_l1_timeout",
    Error::settlement(SettlementError::Timeout(Duration::from_secs(30 * 60)))
)]
#[case(
    "rate_disallowed",
    rate_limiting::RateLimited::SendTxDiabled {}.into()
)]
#[case(
    "rate_sendtx",
    rate_limiting::RateLimited::SendTxRateLimited(WallClockLimitedInfo {
        max_per_interval: 3,
        time_interval: Duration::from_secs(30 * 60),
        until_next: Some(Duration::from_secs(123)),
    }).into()
)]
#[case(
    "rate_sendtx_nonext",
    rate_limiting::RateLimited::SendTxRateLimited(WallClockLimitedInfo {
        max_per_interval: 4,
        time_interval: Duration::from_secs(40 * 60),
        until_next: None,
    }).into()
)]
#[case(
    "cert_submission",
    Error::send_certificate(CertError::CertificateSubmission)
)]
#[case(
    "cert_past",
    Error::send_certificate(CertError::InPast {
        height: 55,
        next_height: 57,
    })
)]
#[case(
    "cert_future",
    Error::send_certificate(CertError::FarFuture {
        height: 153,
        max_height: 95,
    })
)]
#[case(
    "cert_replace_candidate",
    Error::send_certificate(CertError::IllegalReplacement {
        status: CertificateStatus::Candidate,
    })
)]
fn rpc_error_rendering(#[case] name: &str, #[case] err: Error) {
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
