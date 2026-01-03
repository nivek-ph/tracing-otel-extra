//! OpenTelemetry integration for tracing.
//!
//! This module provides utilities for initializing and configuring OpenTelemetry
//! tracing and metrics in your application. It includes functions for:
//!
//! - Configuring resource attributes
//! - Initializing tracer and meter providers

use anyhow::{Context, Result};
use opentelemetry::global;
use opentelemetry_otlp::{Protocol, WithExportConfig, OTEL_EXPORTER_OTLP_PROTOCOL};

/// Environment variable for signal-specific traces protocol override.
const OTEL_EXPORTER_OTLP_TRACES_PROTOCOL: &str = "OTEL_EXPORTER_OTLP_TRACES_PROTOCOL";
/// Environment variable for signal-specific metrics protocol override.
const OTEL_EXPORTER_OTLP_METRICS_PROTOCOL: &str = "OTEL_EXPORTER_OTLP_METRICS_PROTOCOL";

use opentelemetry_sdk::{
    metrics::{MeterProviderBuilder, PeriodicReader, SdkMeterProvider},
    propagation::TraceContextPropagator,
    trace::{RandomIdGenerator, Sampler, SdkTracerProvider},
    Resource,
};
use std::time::Duration;

/// Parses the protocol from a string value.
///
/// # Arguments
///
/// * `value` - The value to parse
///
/// # Returns
///
/// Returns the parsed protocol or `None` if the value is invalid.
fn parse_protocol(value: &str) -> Option<Protocol> {
    match value.trim().to_ascii_lowercase().as_str() {
        "grpc" => Some(Protocol::Grpc),
        "http/protobuf" | "http/proto" => Some(Protocol::HttpBinary),
        "http/json" => Some(Protocol::HttpJson),
        _ => None,
    }
}

/// Gets the protocol from an environment variable.
///
/// # Arguments
///
/// * `key` - The environment variable key
///
/// # Returns
///
/// Returns the parsed protocol or `None` if the environment variable is not set or invalid.
fn protocol_from_env(key: &str) -> Option<Protocol> {
    std::env::var(key)
        .ok()
        .and_then(|value| parse_protocol(&value))
}

/// Builds the span exporter based on the configured protocol.
///
/// Reads protocol from environment variables in order:
/// 1. `OTEL_EXPORTER_OTLP_TRACES_PROTOCOL`
/// 2. `OTEL_EXPORTER_OTLP_PROTOCOL`
/// 3. Falls back to gRPC
fn build_span_exporter() -> Result<opentelemetry_otlp::SpanExporter> {
    let protocol = protocol_from_env(OTEL_EXPORTER_OTLP_TRACES_PROTOCOL)
        .or_else(|| protocol_from_env(OTEL_EXPORTER_OTLP_PROTOCOL))
        .unwrap_or(Protocol::Grpc);
    match protocol {
        Protocol::Grpc => opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .build()
            .context("Failed to build OTLP span exporter (gRPC)"),
        _ => opentelemetry_otlp::SpanExporter::builder()
            .with_http()
            .with_protocol(protocol)
            .build()
            .context("Failed to build OTLP span exporter (HTTP)"),
    }
}

/// Builds the metric exporter based on the configured protocol.
///
/// Reads protocol from environment variables in order:
/// 1. `OTEL_EXPORTER_OTLP_METRICS_PROTOCOL`
/// 2. `OTEL_EXPORTER_OTLP_PROTOCOL`
/// 3. Falls back to gRPC
fn build_metric_exporter() -> Result<opentelemetry_otlp::MetricExporter> {
    let protocol = protocol_from_env(OTEL_EXPORTER_OTLP_METRICS_PROTOCOL)
        .or_else(|| protocol_from_env(OTEL_EXPORTER_OTLP_PROTOCOL))
        .unwrap_or(Protocol::Grpc);
    match protocol {
        Protocol::Grpc => opentelemetry_otlp::MetricExporter::builder()
            .with_tonic()
            .with_temporality(opentelemetry_sdk::metrics::Temporality::default())
            .build()
            .context("Failed to build OTLP metric exporter (gRPC)"),
        _ => opentelemetry_otlp::MetricExporter::builder()
            .with_http()
            .with_protocol(protocol)
            .with_temporality(opentelemetry_sdk::metrics::Temporality::default())
            .build()
            .context("Failed to build OTLP metric exporter (HTTP)"),
    }
}

/// Initializes a tracer provider for OpenTelemetry tracing.
///
/// This function sets up a tracer provider with the following features:
/// - Parent-based sampling
/// - Random ID generation
/// - OTLP exporter
/// - Custom resource attributes
///
/// # Arguments
///
/// * `resource` - The OpenTelemetry resource to use
/// * `sample_ratio` - The ratio of traces to sample (0.0 to 1.0)
///
/// # Returns
///
/// Returns a `Result` containing the configured `SdkTracerProvider` or an error
/// if initialization fails.
///
/// # Examples
///
/// ```rust
/// use tracing_opentelemetry_extra::{get_resource, init_tracer_provider};
/// use opentelemetry::KeyValue;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let resource = get_resource("my-service", &[]);
///     let tracer_provider = init_tracer_provider(&resource, 1.0)?;
///     Ok(())
/// }
/// ```
pub fn init_tracer_provider(resource: &Resource, sample_ratio: f64) -> Result<SdkTracerProvider> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let exporter = build_span_exporter().context("Failed to build OTLP exporter")?;

    let tracer_provider = SdkTracerProvider::builder()
        .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
            sample_ratio,
        ))))
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(resource.clone())
        .with_batch_exporter(exporter)
        .build();

    global::set_tracer_provider(tracer_provider.clone());

    Ok(tracer_provider)
}

/// Initializes a meter provider for OpenTelemetry metrics.
///
/// This function sets up a meter provider with the following features:
/// - Periodic metric collection
/// - OTLP exporter
/// - Custom resource attributes
///
/// # Arguments
///
/// * `resource` - The OpenTelemetry resource to use
/// * `metrics_interval_secs` - The interval in seconds between metric collections
///
/// # Returns
///
/// Returns a `Result` containing the configured `SdkMeterProvider` or an error
/// if initialization fails.
///
/// # Examples
///
/// ```rust
/// use tracing_opentelemetry_extra::{get_resource, init_meter_provider};
/// use opentelemetry::KeyValue;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let resource = get_resource("my-service", &[]);
///     let meter_provider = init_meter_provider(&resource, 30)?;
///     Ok(())
/// }
/// ```
pub fn init_meter_provider(
    resource: &Resource,
    metrics_interval_secs: u64,
) -> Result<SdkMeterProvider> {
    let exporter = build_metric_exporter().context("Failed to build OTLP metric exporter")?;

    let reader = PeriodicReader::builder(exporter)
        .with_interval(Duration::from_secs(metrics_interval_secs))
        .build();

    let meter_builder = MeterProviderBuilder::default()
        .with_resource(resource.clone())
        .with_reader(reader);

    let meter_provider = meter_builder.build();
    global::set_meter_provider(meter_provider.clone());

    Ok(meter_provider)
}
