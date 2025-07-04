//! Tests for rendering RPC errors

use std::time::Duration;

use agglayer_rate_limiting::{self, component, Component};
use agglayer_rpc::error::SignatureVerificationError;
use agglayer_types::{Address, CertificateId, Digest};
use alloy::{
    contract::Error as ContractError,
    primitives::{SignatureError as AlloySignatureError, B256},
    signers::k256,
};
use jsonrpsee::types::ErrorObjectOwned;

use crate::{
    kernel::{self, ZkevmNodeVerificationError},
    service, Error,
};

// Update type aliases to use alloy types
type CheckTxStatusError = kernel::CheckTxStatusError;
type SendTxError = service::SendTxError;
type SettlementError = kernel::SettlementError;
type TxStatusError = service::TxStatusError;
type WallClockLimitedInfo = <component::SendTx as Component>::LimitedInfo;

#[rstest::rstest]
#[case("rollup_not_reg", SendTxError::RollupNotRegistered { rollup_id: 1337 })]
#[case(
    "sig_invalid_len",
    SendTxError::SignatureError(SignatureVerificationError::CouldNotRecoverTxSigner(
        AlloySignatureError::FromHex(alloy::hex::FromHexError::InvalidStringLength)
    ))
)]
#[case("sig_verif", SendTxError::SignatureError(SignatureVerificationError::InvalidSigner {
    signer: Address::from([0x11; 20]),
    trusted_sequencer: Address::from([0x22; 20]),
}))]
#[case(
    "sig_recov",
    SendTxError::SignatureError(SignatureVerificationError::CouldNotRecoverTxSigner(
        AlloySignatureError::K256(k256::ecdsa::Error::new())
    ))
)]
#[case(
    "cert_sig",
    SendTxError::SignatureError(SignatureVerificationError::CouldNotRecoverCertSigner(
        agglayer_types::SignerError::Recovery(AlloySignatureError::K256(
            k256::ecdsa::Error::new()
        ))
    ))
)]
#[case(
    "sig_signer",
    SendTxError::SignatureError(SignatureVerificationError::InvalidSigner{
        signer: Address::from([0x33; 20]),
        trusted_sequencer: Address::from([0x44; 20]),
    })
)]
#[case(
    "sig_contract",
    SendTxError::SignatureError(SignatureVerificationError::ContractError(
        alloy::contract::Error::ContractNotDeployed
    ))
)]
#[case("dry_run_rollup_man", SendTxError::DryRunRollupManager(
    agglayer_contracts::contracts::PolygonRollupManager::PolygonRollupManagerErrors::FinalNumBatchBelowLastVerifiedBatch(
        agglayer_contracts::contracts::PolygonRollupManager::FinalNumBatchBelowLastVerifiedBatch {}
    )
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
        expected: B256::from([0x55; 32]),
        got: B256::from([0x66; 32]),
    })
)]
#[case(
    "root_exit",
    SendTxError::RootVerification(ZkevmNodeVerificationError::InvalidExitRoot {
        expected: B256::from([0x77; 32]),
        got: B256::from([0x88; 32]),
    })
)]
#[case("settle_receipt", SendTxError::Settlement(SettlementError::NoReceipt))]
#[case(
    "settle_io",
    SendTxError::Settlement(SettlementError::ProviderError(alloy::transports::RpcError::Transport(
        alloy::transports::TransportErrorKind::Custom("Settlement transport error".to_string().into())
    ))
))]
#[case(
    "settle_contract",
    SendTxError::Settlement(SettlementError::ContractError(ContractError::TransportError(
        alloy::transports::RpcError::Transport(
            alloy::transports::TransportErrorKind::Custom("Contract transport error".to_string().into())
        )
    )))
)]
#[case(
    "settle_l1_timeout",
    SendTxError::Settlement(SettlementError::Timeout(Duration::from_secs(30 * 60)))
)]
#[case(
    "rate_disallowed",
    SendTxError::RateLimited(agglayer_rate_limiting::RateLimited::SendTxDiabled {})
)]
#[case(
    "rate_sendtx",
    SendTxError::RateLimited(agglayer_rate_limiting::RateLimited::SendTxRateLimited(WallClockLimitedInfo {
        max_per_interval: 3,
        time_interval: Duration::from_secs(30 * 60),
        until_next: Some(Duration::from_secs(123)),
    }))
)]
#[case(
    "rate_sendtx_nonext",
    SendTxError::RateLimited(agglayer_rate_limiting::RateLimited::SendTxRateLimited(WallClockLimitedInfo {
        max_per_interval: 4,
        time_interval: Duration::from_secs(40 * 60),
        until_next: None,
    }))
)]
#[case(
    "txstatus_notfound",
    TxStatusError::TxNotFound { hash: B256::from([0x97; 32]) }
)]
#[case(
    "txstatus_check",
    TxStatusError::StatusCheck(CheckTxStatusError::ProviderError(
        alloy::transports::RpcError::Transport(
            alloy::transports::TransportErrorKind::Custom("Signer unavailable".to_string().into())
        )
    ))
)]
#[case(
    "cert_notfound",
    agglayer_rpc::CertificateRetrievalError::NotFound { certificate_id: CertificateId::new(Digest([0x51; 32])) }
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
