use tracing_subscriber::{prelude::*, util::SubscriberInitExt, EnvFilter};

pub(crate) fn tracing(config: &crate::config::Log) {
    // TODO: Support multiple outputs.
    let writer = config.outputs.first().cloned().unwrap_or_default();

    tracing_subscriber::Registry::default()
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_writer(writer.as_make_writer())
                .with_filter(
                    EnvFilter::try_from_default_env().unwrap_or_else(|_| config.level.into()),
                ),
        )
        .init();
}
