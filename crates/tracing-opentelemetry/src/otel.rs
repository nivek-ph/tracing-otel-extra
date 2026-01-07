//! OpenTelemetry integration for tracing.
//!
//! This module provides utilities for initializing and configuring OpenTelemetry
//! tracing and metrics in your application. It includes functions for:
//!
//! - Configuring resource attributes
//! - Initializing tracer and meter providers
use crate::macros::build_exporter;
use anyhow::{Context, Result};
use opentelemetry::global;
use opentelemetry_sdk::{
    Resource,
    logs::SdkLoggerProvider,
    metrics::{MeterProviderBuilder, PeriodicReader, SdkMeterProvider, Temporality},
    propagation::TraceContextPropagator,
    trace::{RandomIdGenerator, Sampler, SdkTracerProvider},
};
use std::time::Duration;

/// Environment variable for signal-specific traces protocol override.
const OTEL_EXPORTER_OTLP_TRACES_PROTOCOL: &str = "OTEL_EXPORTER_OTLP_TRACES_PROTOCOL";
/// Environment variable for signal-specific metrics protocol override.
const OTEL_EXPORTER_OTLP_METRICS_PROTOCOL: &str = "OTEL_EXPORTER_OTLP_METRICS_PROTOCOL";
/// Environment variable for signal-specific logs protocol override.
const OTEL_EXPORTER_OTLP_LOGS_PROTOCOL: &str = "OTEL_EXPORTER_OTLP_LOGS_PROTOCOL";

/// Build the span exporter based on the configured protocol.
///
/// # Environment
///
/// Resolution order:
/// 1. `OTEL_EXPORTER_OTLP_TRACES_PROTOCOL`
/// 2. `OTEL_EXPORTER_OTLP_PROTOCOL`
/// 3. Falls back to gRPC
///
/// # Errors
///
/// Returns an error if the exporter cannot be built.
fn build_span_exporter() -> Result<opentelemetry_otlp::SpanExporter> {
    build_exporter!(
        opentelemetry_otlp::SpanExporter::builder(),
        OTEL_EXPORTER_OTLP_TRACES_PROTOCOL,
        "Failed to build OTLP span exporter"
    )
}

/// Build the metric exporter based on the configured protocol.
///
/// # Environment
///
/// Resolution order:
/// 1. `OTEL_EXPORTER_OTLP_METRICS_PROTOCOL`
/// 2. `OTEL_EXPORTER_OTLP_PROTOCOL`
/// 3. Falls back to gRPC
///
/// # Errors
///
/// Returns an error if the exporter cannot be built.
fn build_metric_exporter() -> Result<opentelemetry_otlp::MetricExporter> {
    build_exporter!(
        opentelemetry_otlp::MetricExporter::builder(),
        OTEL_EXPORTER_OTLP_METRICS_PROTOCOL,
        "Failed to build OTLP metric exporter",
        |b| b.with_temporality(Temporality::default())
    )
}

/// Build the log exporter based on the configured protocol.
///
/// # Environment
///
/// Resolution order:
/// 1. `OTEL_EXPORTER_OTLP_LOGS_PROTOCOL`
/// 2. `OTEL_EXPORTER_OTLP_PROTOCOL`
/// 3. Falls back to gRPC
///
/// # Errors
///
/// Returns an error if the exporter cannot be built.
fn build_log_exporter() -> Result<opentelemetry_otlp::LogExporter> {
    build_exporter!(
        opentelemetry_otlp::LogExporter::builder(),
        OTEL_EXPORTER_OTLP_LOGS_PROTOCOL,
        "Failed to build OTLP log exporter"
    )
}

/// Initialize a tracer provider for OpenTelemetry tracing.
///
/// # Arguments
///
/// * `resource` - The OpenTelemetry resource to use.
/// * `sample_ratio` - The ratio of traces to sample (0.0 to 1.0).
///
/// # Errors
///
/// Returns an error if the span exporter cannot be built.
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

    let exporter = build_span_exporter().context("Failed to build OTLP span exporter")?;

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

/// Initialize a meter provider for OpenTelemetry metrics.
///
/// # Arguments
///
/// * `resource` - The OpenTelemetry resource to use.
/// * `metrics_interval_secs` - The interval in seconds between metric collections.
///
/// # Errors
///
/// Returns an error if the metric exporter cannot be built.
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

    let meter_provider = MeterProviderBuilder::default()
        .with_resource(resource.clone())
        .with_reader(reader)
        .build();
    global::set_meter_provider(meter_provider.clone());

    Ok(meter_provider)
}

/// Initialize a logger provider for OpenTelemetry logs.
///
/// # Arguments
///
/// * `resource` - The OpenTelemetry resource to use.
///
/// # Errors
///
/// Returns an error if the log exporter cannot be built.
///
/// # Examples
///
/// ```rust
/// use tracing_opentelemetry_extra::{get_resource, init_logger_provider};
/// use opentelemetry::KeyValue;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let resource = get_resource("my-service", &[]);
///     let logger_provider = init_logger_provider(&resource)?;
///     Ok(())
/// }
/// ```
pub fn init_logger_provider(resource: &Resource) -> Result<SdkLoggerProvider> {
    let exporter = build_log_exporter().context("Failed to build OTLP log exporter")?;

    let logger_provider = SdkLoggerProvider::builder()
        .with_resource(resource.clone())
        .with_batch_exporter(exporter)
        .build();

    Ok(logger_provider)
}
