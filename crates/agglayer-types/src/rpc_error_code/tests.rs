use super::*;

#[test]
fn display_and_tag_are_pinned() {
    let cases = [
        (RpcErrorCode::NotFound, "not found", "not-found"),
        (
            RpcErrorCode::AlreadyCompleted,
            "already completed",
            "already-completed",
        ),
        (RpcErrorCode::NotCompleted, "not completed", "not-completed"),
        (RpcErrorCode::NoLiveTask, "no live task", "no-live-task"),
        (
            RpcErrorCode::TaskStillLive,
            "task still live",
            "task-still-live",
        ),
        (RpcErrorCode::Unavailable, "unavailable", "unavailable"),
    ];

    for (code, expected_display, expected_tag) in cases {
        assert_eq!(code.to_string(), expected_display);
        assert_eq!(code.tag(), expected_tag);
    }
}
