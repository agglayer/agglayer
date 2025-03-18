use super::{
    limiter,
    state::{WallClockLimitedInfo, WallClockState},
    ConfigurableResource, Resource,
};

pub enum SendTxSettlement {}

impl Resource for SendTxSettlement {
    type Instant = tokio::time::Instant;

    type State = WallClockState;

    type LimitedInfo = SendTxRateLimited;
}

#[derive(PartialEq, Eq, Clone, Debug, serde::Serialize, thiserror::Error)]
#[serde(rename_all = "kebab-case")]
pub enum SendTxRateLimited {
    #[error("The `sendTx` settlement has been limited")]
    SendTxRateLimited(#[source] WallClockLimitedInfo),

    #[error("The `sendTx` settlement disabled by rate limiter")]
    SendTxDiabled {},
}

impl From<limiter::RateLimited<WallClockLimitedInfo>> for SendTxRateLimited {
    fn from(info: limiter::RateLimited<WallClockLimitedInfo>) -> Self {
        match info {
            limiter::RateLimited::Disabled {} => Self::SendTxDiabled {},
            limiter::RateLimited::Inner(err) => Self::SendTxRateLimited(err),
        }
    }
}

impl ConfigurableResource for SendTxSettlement {}
