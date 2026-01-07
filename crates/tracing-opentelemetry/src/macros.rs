//! Macros for building OpenTelemetry exporters.
//!
//! This module provides macros and utilities for building OTLP exporters
//! with protocol detection from environment variables.

use opentelemetry_otlp::{OTEL_EXPORTER_OTLP_PROTOCOL, Protocol};

/// Parse an OTLP protocol value.
///
/// # Arguments
///
/// * `value` - The protocol value to parse.
///
/// # Returns
///
/// The parsed protocol, or `None` if the value is invalid.
fn parse_protocol(value: &str) -> Option<Protocol> {
    match value.trim().to_ascii_lowercase().as_str() {
        "grpc" => Some(Protocol::Grpc),
        "http/protobuf" | "http/proto" => Some(Protocol::HttpBinary),
        "http/json" => Some(Protocol::HttpJson),
        _ => None,
    }
}

/// Get an OTLP protocol from an environment variable.
///
/// # Arguments
///
/// * `key` - The environment variable key.
///
/// # Returns
///
/// The parsed protocol, or `None` if the variable is unset or invalid.
fn protocol_from_env(key: &str) -> Option<Protocol> {
    std::env::var(key)
        .ok()
        .and_then(|value| parse_protocol(&value))
}

/// Resolve the OTLP protocol for a signal, with a global fallback.
///
/// # Arguments
///
/// * `signal_env` - The signal-specific environment variable key.
///
/// # Returns
///
/// The resolved protocol, defaulting to gRPC.
pub fn protocol_for_signal(signal_env: &str) -> Protocol {
    protocol_from_env(signal_env)
        .or_else(|| protocol_from_env(OTEL_EXPORTER_OTLP_PROTOCOL))
        .unwrap_or(Protocol::Grpc)
}

/// Build the exporter based on the configured protocol.
///
/// This macro creates an OTLP exporter using either gRPC (tonic) or HTTP transport
/// based on the protocol configuration from environment variables.
///
/// # Arguments
///
/// * `$builder` - The exporter builder (e.g., `SpanExporter::builder()`)
/// * `$protocol_env` - The signal-specific environment variable for protocol override
/// * `$msg` - Error message prefix for build failures
/// * `$config` - Optional closure to configure the builder before building
///
/// # Example
///
/// ```ignore
/// use crate::macros::build_exporter;
///
/// let exporter = build_exporter!(
///     opentelemetry_otlp::SpanExporter::builder(),
///     "OTEL_EXPORTER_OTLP_TRACES_PROTOCOL",
///     "Failed to build span exporter"
/// )?;
/// ```
macro_rules! build_exporter {
    ($builder:expr, $protocol_env:expr, $msg:literal) => {
        build_exporter!($builder, $protocol_env, $msg, |b| b)
    };
    ($builder:expr, $protocol_env:expr, $msg:literal, |$binder:ident| $config:expr) => {{
        use ::anyhow::Context as _;
        use ::opentelemetry_otlp::Protocol;
        use ::opentelemetry_otlp::WithExportConfig as _;

        let protocol = $crate::macros::protocol_for_signal($protocol_env);
        match protocol {
            Protocol::Grpc => {
                let $binder = $builder.with_tonic();
                let builder = $config;
                builder.build().context(format!("{} (gRPC)", $msg))
            }
            _ => {
                let $binder = $builder.with_http().with_protocol(protocol);
                let builder = $config;
                builder.build().context(format!("{} (HTTP)", $msg))
            }
        }
    }};
}

pub(crate) use build_exporter;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_protocol() {
        assert_eq!(parse_protocol("grpc"), Some(Protocol::Grpc));
        assert_eq!(parse_protocol("GRPC"), Some(Protocol::Grpc));
        assert_eq!(parse_protocol("http/protobuf"), Some(Protocol::HttpBinary));
        assert_eq!(parse_protocol("http/proto"), Some(Protocol::HttpBinary));
        assert_eq!(parse_protocol("http/json"), Some(Protocol::HttpJson));
        assert_eq!(parse_protocol("invalid"), None);
    }
}
