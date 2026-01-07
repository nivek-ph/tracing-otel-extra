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
        // Valid protocols - lowercase
        assert_eq!(parse_protocol("grpc"), Some(Protocol::Grpc));
        assert_eq!(parse_protocol("http/protobuf"), Some(Protocol::HttpBinary));
        assert_eq!(parse_protocol("http/proto"), Some(Protocol::HttpBinary));
        assert_eq!(parse_protocol("http/json"), Some(Protocol::HttpJson));

        // Valid protocols - uppercase
        assert_eq!(parse_protocol("GRPC"), Some(Protocol::Grpc));
        assert_eq!(parse_protocol("HTTP/PROTOBUF"), Some(Protocol::HttpBinary));
        assert_eq!(parse_protocol("HTTP/PROTO"), Some(Protocol::HttpBinary));
        assert_eq!(parse_protocol("HTTP/JSON"), Some(Protocol::HttpJson));

        // Valid protocols - mixed case
        assert_eq!(parse_protocol("Grpc"), Some(Protocol::Grpc));
        assert_eq!(parse_protocol("Http/Protobuf"), Some(Protocol::HttpBinary));
        assert_eq!(parse_protocol("Http/Proto"), Some(Protocol::HttpBinary));
        assert_eq!(parse_protocol("Http/Json"), Some(Protocol::HttpJson));

        // Valid protocols - with whitespace
        assert_eq!(parse_protocol(" grpc "), Some(Protocol::Grpc));
        assert_eq!(
            parse_protocol("  http/protobuf  "),
            Some(Protocol::HttpBinary)
        );
        assert_eq!(parse_protocol("\thttp/proto\n"), Some(Protocol::HttpBinary));
        assert_eq!(parse_protocol(" http/json "), Some(Protocol::HttpJson));

        // Invalid protocols
        assert_eq!(parse_protocol("invalid"), None);
        assert_eq!(parse_protocol(""), None);
        assert_eq!(parse_protocol("http"), None);
        assert_eq!(parse_protocol("grpc/http"), None);
        assert_eq!(parse_protocol("json"), None);
    }

    #[test]
    fn test_protocol_from_env() {
        // Test with unset environment variable
        assert_eq!(protocol_from_env("NONEXISTENT_VAR_12345"), None);

        // Test with valid protocol values
        unsafe {
            std::env::set_var("TEST_PROTOCOL_GRPC", "grpc");
        }
        assert_eq!(
            protocol_from_env("TEST_PROTOCOL_GRPC"),
            Some(Protocol::Grpc)
        );
        unsafe {
            std::env::remove_var("TEST_PROTOCOL_GRPC");
        }

        unsafe {
            std::env::set_var("TEST_PROTOCOL_HTTP_BINARY", "http/protobuf");
        }
        assert_eq!(
            protocol_from_env("TEST_PROTOCOL_HTTP_BINARY"),
            Some(Protocol::HttpBinary)
        );
        unsafe {
            std::env::remove_var("TEST_PROTOCOL_HTTP_BINARY");
        }

        unsafe {
            std::env::set_var("TEST_PROTOCOL_HTTP_JSON", "http/json");
        }
        assert_eq!(
            protocol_from_env("TEST_PROTOCOL_HTTP_JSON"),
            Some(Protocol::HttpJson)
        );
        unsafe {
            std::env::remove_var("TEST_PROTOCOL_HTTP_JSON");
        }

        // Test with invalid protocol value
        unsafe {
            std::env::set_var("TEST_PROTOCOL_INVALID", "invalid");
        }
        assert_eq!(protocol_from_env("TEST_PROTOCOL_INVALID"), None);
        unsafe {
            std::env::remove_var("TEST_PROTOCOL_INVALID");
        }

        // Test with whitespace (should be trimmed)
        unsafe {
            std::env::set_var("TEST_PROTOCOL_WHITESPACE", " grpc ");
        }
        assert_eq!(
            protocol_from_env("TEST_PROTOCOL_WHITESPACE"),
            Some(Protocol::Grpc)
        );
        unsafe {
            std::env::remove_var("TEST_PROTOCOL_WHITESPACE");
        }
    }

    #[test]
    fn test_protocol_for_signal() {
        // Test default fallback to gRPC when no env vars are set
        // Clean up any existing env vars first
        unsafe {
            std::env::remove_var("TEST_SIGNAL_SPECIFIC");
            std::env::remove_var(OTEL_EXPORTER_OTLP_PROTOCOL);
        }
        assert_eq!(protocol_for_signal("TEST_SIGNAL_SPECIFIC"), Protocol::Grpc);

        // Test signal-specific override
        unsafe {
            std::env::set_var("TEST_SIGNAL_SPECIFIC", "http/json");
        }
        assert_eq!(
            protocol_for_signal("TEST_SIGNAL_SPECIFIC"),
            Protocol::HttpJson
        );
        unsafe {
            std::env::remove_var("TEST_SIGNAL_SPECIFIC");
        }

        // Test global fallback when signal-specific is not set
        unsafe {
            std::env::set_var(OTEL_EXPORTER_OTLP_PROTOCOL, "http/protobuf");
        }
        assert_eq!(
            protocol_for_signal("TEST_SIGNAL_SPECIFIC"),
            Protocol::HttpBinary
        );
        unsafe {
            std::env::remove_var(OTEL_EXPORTER_OTLP_PROTOCOL);
        }

        // Test signal-specific takes precedence over global
        unsafe {
            std::env::set_var("TEST_SIGNAL_SPECIFIC", "http/json");
            std::env::set_var(OTEL_EXPORTER_OTLP_PROTOCOL, "http/protobuf");
        }
        assert_eq!(
            protocol_for_signal("TEST_SIGNAL_SPECIFIC"),
            Protocol::HttpJson
        );
        unsafe {
            std::env::remove_var("TEST_SIGNAL_SPECIFIC");
            std::env::remove_var(OTEL_EXPORTER_OTLP_PROTOCOL);
        }

        // Test invalid signal-specific falls back to global
        unsafe {
            std::env::set_var("TEST_SIGNAL_SPECIFIC", "invalid");
            std::env::set_var(OTEL_EXPORTER_OTLP_PROTOCOL, "grpc");
        }
        assert_eq!(protocol_for_signal("TEST_SIGNAL_SPECIFIC"), Protocol::Grpc);
        unsafe {
            std::env::remove_var("TEST_SIGNAL_SPECIFIC");
            std::env::remove_var(OTEL_EXPORTER_OTLP_PROTOCOL);
        }

        // Test invalid signal-specific and invalid global falls back to default
        unsafe {
            std::env::set_var("TEST_SIGNAL_SPECIFIC", "invalid");
            std::env::set_var(OTEL_EXPORTER_OTLP_PROTOCOL, "also_invalid");
        }
        assert_eq!(protocol_for_signal("TEST_SIGNAL_SPECIFIC"), Protocol::Grpc);
        unsafe {
            std::env::remove_var("TEST_SIGNAL_SPECIFIC");
            std::env::remove_var(OTEL_EXPORTER_OTLP_PROTOCOL);
        }
    }
}
