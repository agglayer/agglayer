#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to bind metrics server: {0}")]
    UnableToBindMetricsServer(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum MetricsError {
    #[error("Error gathering metrics: {0}")]
    GatheringMetrics(#[from] prometheus::Error),

    #[error("Error formatting metrics: {0}")]
    FormattingMetrics(#[from] std::string::FromUtf8Error),
}
