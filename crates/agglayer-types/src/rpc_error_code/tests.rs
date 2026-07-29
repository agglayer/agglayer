use super::*;

#[test]
fn display_and_tag_are_pinned() {
    let cases = [
        (
            RpcErrorCode::SendCertificate,
            "certificate submission failed",
            "send-certificate",
            -10006,
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

#[test]
fn serialization_matches_tag() {
    let codes = [
        RpcErrorCode::SendCertificate,
        RpcErrorCode::NotFound,
        RpcErrorCode::MethodDisabled,
        RpcErrorCode::AlreadyCompleted,
        RpcErrorCode::NotCompleted,
        RpcErrorCode::NoLiveTask,
        RpcErrorCode::TaskStillLive,
        RpcErrorCode::Unavailable,
    ];

    for code in codes {
        assert_eq!(
            serde_json::to_value(code).unwrap(),
            serde_json::json!(code.tag())
        );
    }
}
