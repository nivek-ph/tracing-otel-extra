//! OpenTelemetry integration for tracing.
//!
//! This module provides utilities for initializing and configuring OpenTelemetry
//! tracing and metrics in your application. It includes functions for:
//!
//! - Configuring resource attributes
//! - Initializing tracer and meter providers
use crate::macros::build_exporter;
use anyhow::Result;
use opentelemetry::global;
use opentelemetry_sdk::{
    Resource,
    logs::SdkLoggerProvider,
    metrics::{MeterProviderBuilder, PeriodicReader, SdkMeterProvider, Temporality},
    propagation::TraceContextPropagator,
    trace::{RandomIdGenerator, Sampler, SdkTracerProvider},
};
use std::{env::var, time::Duration};

/// Global OTLP endpoint environment variable.
const OTEL_EXPORTER_OTLP_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_ENDPOINT";
/// Environment variable for signal-specific traces protocol override.
const OTEL_EXPORTER_OTLP_TRACES_PROTOCOL: &str = "OTEL_EXPORTER_OTLP_TRACES_PROTOCOL";
/// Environment variable for signal-specific traces endpoint override.
const OTEL_EXPORTER_OTLP_TRACES_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT";
/// Environment variable for signal-specific metrics protocol override.
const OTEL_EXPORTER_OTLP_METRICS_PROTOCOL: &str = "OTEL_EXPORTER_OTLP_METRICS_PROTOCOL";
/// Environment variable for signal-specific metrics endpoint override.
const OTEL_EXPORTER_OTLP_METRICS_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_METRICS_ENDPOINT";
/// Environment variable for signal-specific logs protocol override.
const OTEL_EXPORTER_OTLP_LOGS_PROTOCOL: &str = "OTEL_EXPORTER_OTLP_LOGS_PROTOCOL";
/// Environment variable for signal-specific logs endpoint override.
const OTEL_EXPORTER_OTLP_LOGS_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_LOGS_ENDPOINT";

/// Check whether an OTLP exporter should be enabled for a signal.
///
/// A signal-specific endpoint enables only that signal, while
/// `OTEL_EXPORTER_OTLP_ENDPOINT` enables all signals. Empty or whitespace-only
/// endpoint values are treated as unset.
///
/// # Arguments
///
/// * `endpoint_env` - The signal-specific endpoint environment variable to
///   check.
///
/// # Returns
///
/// `true` if either the signal-specific endpoint or the global OTLP endpoint is
/// configured with a non-empty value.
fn exporter_enabled(endpoint_env: &str) -> bool {
    endpoint_configured(endpoint_env) || endpoint_configured(OTEL_EXPORTER_OTLP_ENDPOINT)
}

/// Check whether an endpoint environment variable has a usable value.
///
/// Empty or whitespace-only values are treated as unset so templated
/// deployments can leave endpoint variables empty without accidentally enabling
/// OTLP exporters.
///
/// # Arguments
///
/// * `endpoint_env` - The endpoint environment variable to check.
///
/// # Returns
///
/// `true` if the environment variable exists and contains a non-empty value.
fn endpoint_configured(endpoint_env: &str) -> bool {
    var(endpoint_env)
        .ok()
        .is_some_and(|value| !value.trim().is_empty())
}

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
/// Returns an error if the span exporter cannot be built. When no OTLP endpoint
/// is configured, or the endpoint env var is empty, the provider is still
/// initialized without an exporter.
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

    let builder = SdkTracerProvider::builder()
        .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
            sample_ratio,
        ))))
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(resource.clone());

    let tracer_provider = if exporter_enabled(OTEL_EXPORTER_OTLP_TRACES_ENDPOINT) {
        let exporter = build_span_exporter()?;
        builder.with_batch_exporter(exporter).build()
    } else {
        builder.build()
    };

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
/// Returns an error if the metric exporter cannot be built. When no OTLP
/// endpoint is configured, or the endpoint env var is empty, the provider is
/// still initialized without a periodic exporter.
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
    let builder = MeterProviderBuilder::default().with_resource(resource.clone());

    let meter_provider = if exporter_enabled(OTEL_EXPORTER_OTLP_METRICS_ENDPOINT) {
        let exporter = build_metric_exporter()?;
        let reader = PeriodicReader::builder(exporter)
            .with_interval(Duration::from_secs(metrics_interval_secs))
            .build();

        builder.with_reader(reader).build()
    } else {
        builder.build()
    };

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
/// Returns an error if the log exporter cannot be built. When no OTLP endpoint
/// is configured, or the endpoint env var is empty, the provider is still
/// initialized without an exporter.
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
    let builder = SdkLoggerProvider::builder().with_resource(resource.clone());

    let logger_provider = if exporter_enabled(OTEL_EXPORTER_OTLP_LOGS_ENDPOINT) {
        let exporter = build_log_exporter()?;
        builder.with_batch_exporter(exporter).build()
    } else {
        builder.build()
    };

    Ok(logger_provider)
}

#[cfg(test)]
mod tests {
    use super::{
        OTEL_EXPORTER_OTLP_ENDPOINT, OTEL_EXPORTER_OTLP_LOGS_ENDPOINT,
        OTEL_EXPORTER_OTLP_METRICS_ENDPOINT, OTEL_EXPORTER_OTLP_TRACES_ENDPOINT, exporter_enabled,
    };
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn clear_endpoint_envs() {
        unsafe {
            std::env::remove_var(OTEL_EXPORTER_OTLP_ENDPOINT);
            std::env::remove_var(OTEL_EXPORTER_OTLP_TRACES_ENDPOINT);
            std::env::remove_var(OTEL_EXPORTER_OTLP_METRICS_ENDPOINT);
            std::env::remove_var(OTEL_EXPORTER_OTLP_LOGS_ENDPOINT);
        }
    }

    #[test]
    fn exporter_is_disabled_without_global_or_signal_endpoint() {
        let _guard = ENV_LOCK
            .lock()
            .expect("environment lock should not be poisoned");
        clear_endpoint_envs();

        assert!(!exporter_enabled(OTEL_EXPORTER_OTLP_TRACES_ENDPOINT));
        assert!(!exporter_enabled(OTEL_EXPORTER_OTLP_METRICS_ENDPOINT));
        assert!(!exporter_enabled(OTEL_EXPORTER_OTLP_LOGS_ENDPOINT));
    }

    #[test]
    fn exporter_is_disabled_with_empty_endpoint() {
        let _guard = ENV_LOCK
            .lock()
            .expect("environment lock should not be poisoned");
        clear_endpoint_envs();
        unsafe {
            std::env::set_var(OTEL_EXPORTER_OTLP_ENDPOINT, " ");
            std::env::set_var(OTEL_EXPORTER_OTLP_TRACES_ENDPOINT, "");
        }

        assert!(!exporter_enabled(OTEL_EXPORTER_OTLP_TRACES_ENDPOINT));

        clear_endpoint_envs();
    }

    #[test]
    fn exporter_is_enabled_with_global_endpoint() {
        let _guard = ENV_LOCK
            .lock()
            .expect("environment lock should not be poisoned");
        clear_endpoint_envs();
        unsafe {
            std::env::set_var(OTEL_EXPORTER_OTLP_ENDPOINT, "http://localhost:4317");
        }

        assert!(exporter_enabled(OTEL_EXPORTER_OTLP_TRACES_ENDPOINT));
        assert!(exporter_enabled(OTEL_EXPORTER_OTLP_METRICS_ENDPOINT));
        assert!(exporter_enabled(OTEL_EXPORTER_OTLP_LOGS_ENDPOINT));

        clear_endpoint_envs();
    }

    #[test]
    fn exporter_is_enabled_with_signal_endpoint() {
        let _guard = ENV_LOCK
            .lock()
            .expect("environment lock should not be poisoned");
        clear_endpoint_envs();
        unsafe {
            std::env::set_var(
                OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
                "http://localhost:4318/v1/traces",
            );
        }

        assert!(exporter_enabled(OTEL_EXPORTER_OTLP_TRACES_ENDPOINT));
        assert!(!exporter_enabled(OTEL_EXPORTER_OTLP_METRICS_ENDPOINT));
        assert!(!exporter_enabled(OTEL_EXPORTER_OTLP_LOGS_ENDPOINT));

        clear_endpoint_envs();
    }
}
