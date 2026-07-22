/// Semantic RPC error codes, tagged into `eyre` error chains at the point
/// where a failure is classified, and mapped to wire error codes once,
/// generically, at the RPC boundary (`report.downcast_ref::<RpcErrorCode>()`).
///
/// The `Display` strings are readable fragments — they render inside error
/// chains ("failed to abort task: no live task: …"), so keep them lowercase
/// prose, not mnemonics. Each variant corresponds to one distinct operator
/// reaction; do not add a variant without one.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum RpcErrorCode {
    /// The referenced resource (job, attempt, attempt result, L1 tx) does
    /// not exist — check the id.
    #[error("not found")]
    NotFound,

    /// The job already has a terminal result and the operation was not
    /// forced — pass force, or stop.
    #[error("already completed")]
    AlreadyCompleted,

    /// The job has no terminal result but the operation requires one.
    #[error("not completed")]
    NotCompleted,

    /// No in-memory task exists for this pending job. In the current stack
    /// the recovery is a node restart (startup recovery respawns pending
    /// jobs).
    #[error("no live task")]
    NoLiveTask,

    /// The operation requires the job's task to be gone, but one is live —
    /// wait for completion or abort first.
    #[error("task still live")]
    TaskStillLive,

    /// Transient condition (task command queue full, L1 RPC unreachable) —
    /// retry later.
    #[error("unavailable")]
    Unavailable,
}

impl RpcErrorCode {
    /// Stable machine-readable tag for wire payloads.
    pub fn tag(&self) -> &'static str {
        match self {
            Self::NotFound => "not-found",
            Self::AlreadyCompleted => "already-completed",
            Self::NotCompleted => "not-completed",
            Self::NoLiveTask => "no-live-task",
            Self::TaskStillLive => "task-still-live",
            Self::Unavailable => "unavailable",
        }
    }
}

#[cfg(test)]
mod tests;
