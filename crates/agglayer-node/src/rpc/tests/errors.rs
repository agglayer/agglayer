//! Tests for rendering RPC errors

use std::time::Duration;

use agglayer_contracts::polygon_rollup_manager::PolygonRollupManagerErrors;
use agglayer_types::Digest;
use alloy::{primitives::SignatureError as AlloySignatureError, signers::k256};
use ethers::{
    providers::ProviderError,
    types::{Bytes, SignatureError as EthSignatureError, H160, H256},
};
use jsonrpsee::types::ErrorObjectOwned;

use crate::{
    kernel::{self, ZkevmNodeVerificationError},
    rate_limiting::{self, component, Component},
    rpc::Error,
    service,
};

type RpcProvider = ethers::providers::Provider<ethers::providers::Http>;
type CheckTxStatusError = kernel::CheckTxStatusError<RpcProvider>;
type ContractError = ethers::contract::ContractError<RpcProvider>;
type SendTxError = service::SendTxError<RpcProvider>;
type SettlementError = kernel::SettlementError<RpcProvider>;
type SignatureError = kernel::SignatureVerificationError<RpcProvider>;
type TxStatusError = service::TxStatusError<RpcProvider>;
type WallClockLimitedInfo = <component::SendTx as Component>::LimitedInfo;

#[rstest::rstest]
#[case("rollup_not_reg", SendTxError::RollupNotRegistered { rollup_id: 1337 })]
#[case(
    "sig_invalid_len",
    SendTxError::SignatureError(SignatureError::CouldNotRecoverTxSigner(
        EthSignatureError::InvalidLength(42)
    ))
)]
#[case("sig_verif", SendTxError::SignatureError(SignatureError::CouldNotRecoverTxSigner(
    EthSignatureError::VerificationError(H160([0x11; 20]), H160([0x22; 20]))
)))]
#[case(
    "sig_recov",
    SendTxError::SignatureError(SignatureError::CouldNotRecoverTxSigner(
        EthSignatureError::RecoveryError
    ))
)]
#[case(
    "cert_sig",
    SendTxError::SignatureError(SignatureError::CouldNotRecoverCertSigner(
        AlloySignatureError::K256(k256::ecdsa::Error::new())
    ))
)]
#[case(
    "sig_signer",
    SendTxError::SignatureError(SignatureError::InvalidSigner{
        signer: H160([0x33; 20]),
        trusted_sequencer: H160([0x44; 20]),
    })
)]
#[case("sig_contract", SendTxError::SignatureError(ContractError::ContractNotDeployed.into()))]
#[case("dry_run_rollup_man", SendTxError::DryRunRollupManager(
        PolygonRollupManagerErrors::RevertString("Reverting".into())
))]
#[case(
    "root_bad_rollup",
    SendTxError::RootVerification(ZkevmNodeVerificationError::InvalidRollupId(13))
)]
#[case(
    "root_rpc",
    SendTxError::RootVerification(ZkevmNodeVerificationError::RpcError(
        jsonrpsee::core::client::Error::Custom("Node smells too much".into())
    ))
)]
#[case(
    "root_state",
    SendTxError::RootVerification(ZkevmNodeVerificationError::InvalidStateRoot {
        expected: H256([0x55; 32]),
        got: H256([0x66; 32]),
    })
)]
#[case(
    "root_exit",
    SendTxError::RootVerification(ZkevmNodeVerificationError::InvalidExitRoot {
        expected: H256([0x77; 32]),
        got: H256([0x88; 32]),
    })
)]
#[case("settle_receipt", SendTxError::Settlement(SettlementError::NoReceipt))]
#[case(
    "settle_io",
    SendTxError::Settlement(SettlementError::ProviderError(ProviderError::UnsupportedRPC))
)]
#[case(
    "settle_contract",
    SendTxError::Settlement(SettlementError::ContractError(ContractError::Revert(
        Bytes::from_static(b"foo")
    )))
)]
#[case(
    "settle_l1_timeout",
    SendTxError::Settlement(SettlementError::Timeout(Duration::from_secs(30 * 60)))
)]
#[case(
    "rate_disallowed",
    SendTxError::RateLimited(rate_limiting::RateLimited::SendTxDiabled {})
)]
#[case(
    "rate_sendtx",
    SendTxError::RateLimited(rate_limiting::RateLimited::SendTxRateLimited(WallClockLimitedInfo {
        max_per_interval: 3,
        time_interval: Duration::from_secs(30 * 60),
        until_next: Some(Duration::from_secs(123)),
    }))
)]
#[case(
    "rate_sendtx_nonext",
    SendTxError::RateLimited(rate_limiting::RateLimited::SendTxRateLimited(WallClockLimitedInfo {
        max_per_interval: 4,
        time_interval: Duration::from_secs(40 * 60),
        until_next: None,
    }))
)]
#[case(
    "txstatus_notfound",
    TxStatusError::TxNotFound { hash: H256([0x97; 32]) }
)]
#[case(
    "txstatus_check",
    TxStatusError::StatusCheck(CheckTxStatusError::ProviderError(
        ProviderError::SignerUnavailable
    ))
)]
#[case(
    "cert_notfound",
    service::CertificateRetrievalError::NotFound { certificate_id: Digest([0x51; 32]) }
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
