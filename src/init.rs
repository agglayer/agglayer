use tracing_subscriber::{fmt::format::FmtSpan, prelude::*, util::SubscriberInitExt, EnvFilter};

pub(crate) fn tracing() {
    tracing_subscriber::Registry::default()
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_span_events(FmtSpan::CLOSE)
                .with_filter(EnvFilter::from_default_env()),
        )
        .init();
}
