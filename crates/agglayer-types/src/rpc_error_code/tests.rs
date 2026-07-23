use super::*;

#[test]
fn display_and_tag_are_pinned() {
    let cases = [
        (
            RpcErrorCode::RollupNotRegistered,
            "rollup not registered",
            "rollup-not-registered",
            -10001,
        ),
        (
            RpcErrorCode::SignatureMismatch,
            "signature mismatch",
            "signature-mismatch",
            -10002,
        ),
        (
            RpcErrorCode::ValidationFailure,
            "validation failure",
            "validation-failure",
            -10003,
        ),
        (
            RpcErrorCode::SettlementError,
            "settlement error",
            "settlement-error",
            -10004,
        ),
        (
            RpcErrorCode::StatusError,
            "status error",
            "status-error",
            -10005,
        ),
        (
            RpcErrorCode::SendCertificate,
            "certificate submission failed",
            "send-certificate",
            -10006,
        ),
        (
            RpcErrorCode::RateLimited,
            "rate limited",
            "rate-limited",
            -10007,
        ),
        (RpcErrorCode::NotFound, "not found", "not-found", -10008),
        (
            RpcErrorCode::MethodDisabled,
            "method disabled",
            "method-disabled",
            -10009,
        ),
        (
            RpcErrorCode::AlreadyCompleted,
            "already completed",
            "already-completed",
            -10010,
        ),
        (
            RpcErrorCode::NotCompleted,
            "not completed",
            "not-completed",
            -10011,
        ),
        (
            RpcErrorCode::NoLiveTask,
            "no live task",
            "no-live-task",
            -10012,
        ),
        (
            RpcErrorCode::TaskStillLive,
            "task still live",
            "task-still-live",
            -10013,
        ),
        (
            RpcErrorCode::Unavailable,
            "unavailable",
            "unavailable",
            -10014,
        ),
    ];

    for (code, expected_display, expected_tag, expected_code) in cases {
        assert_eq!(code.to_string(), expected_display);
        assert_eq!(code.tag(), expected_tag);
        assert_eq!(code.code(), expected_code);
    }
}
