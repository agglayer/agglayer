/// Stable RPC error codes, usable both as wire values and as `eyre` tags.
///
/// The `Display` strings are readable fragments — they render inside error
/// chains ("failed to abort task: no live task: …"), so keep them lowercase
/// prose, not mnemonics. Each variant corresponds to one distinct operator
/// reaction; do not add a variant without one. The `-10001..=-10009` range
/// predates this rule: those variants mirror the historical public-API codes
/// verbatim for wire compatibility.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, thiserror::Error)]
#[serde(rename_all = "kebab-case")]
#[repr(i32)]
pub enum RpcErrorCode {
    /// Rollup is not registered.
    #[error("rollup not registered")]
    RollupNotRegistered = -10001,

    /// Rollup signature verification failed.
    #[error("signature mismatch")]
    SignatureMismatch = -10002,

    /// Proof or state validation failed.
    #[error("validation failure")]
    ValidationFailure = -10003,

    /// L1 settlement failed.
    #[error("settlement error")]
    SettlementError = -10004,

    /// Transaction status retrieval failed.
    #[error("status error")]
    StatusError = -10005,

    /// Certificate submission failed.
    #[error("certificate submission failed")]
    SendCertificate = -10006,

    /// Transaction settlement was rate limited.
    #[error("rate limited")]
    RateLimited = -10007,

    /// The referenced resource does not exist — check the id. Covers both
    /// the admin task-control API (job, attempt, attempt result, L1 tx) and
    /// the public API (certificate header).
    #[error("not found")]
    NotFound = -10008,

    /// Method is permanently disabled.
    #[error("method disabled")]
    MethodDisabled = -10009,

    /// The job already has a terminal result and the operation was not
    /// forced — pass force, or stop.
    #[error("already completed")]
    AlreadyCompleted = -10010,

    /// The job has no terminal result but the operation requires one.
    #[error("not completed")]
    NotCompleted = -10011,

    /// No in-memory task exists for this pending job. In the current stack
    /// the recovery is a node restart (startup recovery respawns pending
    /// jobs).
    #[error("no live task")]
    NoLiveTask = -10012,

    /// The operation requires the job's task to be gone, but one is live —
    /// wait for completion or abort first.
    #[error("task still live")]
    TaskStillLive = -10013,

    /// Transient condition (task command queue full, L1 RPC unreachable) —
    /// retry later.
    #[error("unavailable")]
    Unavailable = -10014,
}

impl RpcErrorCode {
    /// Stable numeric JSON-RPC error code.
    pub const fn code(self) -> i32 {
        self as i32
    }

    /// Stable machine-readable tag for wire payloads.
    pub const fn tag(&self) -> &'static str {
        match self {
            Self::RollupNotRegistered => "rollup-not-registered",
            Self::SignatureMismatch => "signature-mismatch",
            Self::ValidationFailure => "validation-failure",
            Self::SettlementError => "settlement-error",
            Self::StatusError => "status-error",
            Self::SendCertificate => "send-certificate",
            Self::RateLimited => "rate-limited",
            Self::NotFound => "not-found",
            Self::MethodDisabled => "method-disabled",
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
