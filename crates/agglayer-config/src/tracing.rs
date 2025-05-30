use std::{fmt::Display, path::PathBuf};

use serde::{Deserialize, Deserializer, Serialize};
use tracing_subscriber::{fmt::writer::BoxMakeWriter, EnvFilter};

/// The tracing configuration.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Tracing {
    /// The `RUST_LOG` environment variable will take precedence over the
    /// configuration tracing level.
    #[serde(default)]
    pub level: TracingLevel,
    #[serde(default)]
    pub outputs: Vec<TracingOutput>,
    #[serde(default)]
    pub format: TracingFormat,
    /// Socket of the open telemetry agent endpoint.
    /// If not provided open telemetry will not be used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub otlp_agent: Option<String>,
    /// Otlp service name.
    /// If not provided open telemetry will not be used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub otlp_service_name: Option<String>,
}

/// The log format.
#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TracingFormat {
    #[default]
    Pretty,
    Json,
}

/// The log level.
#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TracingLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
    Fatal,
}

impl Display for TracingLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let level = match self {
            TracingLevel::Trace => "trace",
            TracingLevel::Debug => "debug",
            TracingLevel::Info => "info",
            TracingLevel::Warn => "warn",
            TracingLevel::Error => "error",
            TracingLevel::Fatal => "fatal",
        };

        write!(f, "{level}")
    }
}

impl From<TracingLevel> for EnvFilter {
    fn from(value: TracingLevel) -> Self {
        EnvFilter::new(format!("warn,agglayer={value},pessimistic_proof={value}"))
    }
}

/// The log output.
///
/// This can be either `stdout`, `stderr`, or a file path.
///
/// The [`Deserialize`] implementation allows for the configuration file to
/// specify the output location as a string, which is then parsed into the
/// appropriate enum variant. If the string is not recognized to be either
/// `stdout` or `stderr`, it is assumed to be a file path.
#[derive(Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub enum TracingOutput {
    #[default]
    Stdout,
    Stderr,
    File(PathBuf),
    Otlp,
}

impl<'de> Deserialize<'de> for TracingOutput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // If the string is not recognized to be either `stdout` or `stderr`,
        // it is assumed to be a file path.
        match s.as_str() {
            "stdout" => Ok(TracingOutput::Stdout),
            "stderr" => Ok(TracingOutput::Stderr),
            "otlp" => Ok(TracingOutput::Otlp),
            _ => Ok(TracingOutput::File(PathBuf::from(s))),
        }
    }
}

impl TracingOutput {
    /// Get a [`BoxMakeWriter`] for the tracing output.
    ///
    /// This can be used to plug the tracing output into the tracing subscriber.
    pub fn as_make_writer(&self) -> BoxMakeWriter {
        match self {
            TracingOutput::Stdout => BoxMakeWriter::new(std::io::stdout),
            TracingOutput::Stderr => BoxMakeWriter::new(std::io::stderr),
            TracingOutput::Otlp => BoxMakeWriter::new(std::io::stdout),
            TracingOutput::File(path) => {
                let appender = tracing_appender::rolling::never(".", path);
                BoxMakeWriter::new(appender)
            }
        }
    }
}
