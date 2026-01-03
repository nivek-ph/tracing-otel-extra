use crate::{
    logs::{LogFormat, Logger},
    otel::{
        get_resource, init_logger_provider, init_meter_provider, init_tracer_provider,
        init_tracing_subscriber, opentelemetry::KeyValue, OtelGuard,
    },
};
use anyhow::{anyhow, Context, Result};
use std::sync::OnceLock;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_opentelemetry_extra::BoxLayer;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan, MakeWriter},
    EnvFilter, Layer, Registry,
};

// Keep non-blocking appender worker guard to prevent log loss
static NONBLOCKING_APPENDER_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

pub fn set_nonblocking_appender_guard(guard: WorkerGuard) -> Result<()> {
    NONBLOCKING_APPENDER_GUARD
        .set(guard)
        .map_err(|_| anyhow!("cannot lock for appender"))
}

/// Creates an environment filter for tracing based on the given level.
///
/// This function attempts to create a filter from environment variables first,
/// falling back to the provided level if no environment configuration is found.
///
/// # Arguments
///
/// * `level` - The default tracing level to use if no environment configuration is found
///
/// # Examples
///
/// ```rust
/// use tracing_otel_extra::logs::init_env_filter;
/// use tracing::Level;
///
/// let filter = init_env_filter(&Level::INFO);
/// ```
pub fn init_env_filter(level: &Level) -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level.to_string()))
}

/// Apply the specified format to a tracing layer
fn apply_layer_format<N, W>(
    layer: fmt::Layer<Registry, N, fmt::format::Format, W>,
    format: &LogFormat,
) -> Box<dyn Layer<Registry> + Sync + Send>
where
    N: for<'writer> fmt::format::FormatFields<'writer> + Sync + Send + 'static,
    W: for<'writer> MakeWriter<'writer> + Sync + Send + 'static,
{
    match format {
        LogFormat::Compact => layer.compact().boxed(),
        LogFormat::Pretty => layer.pretty().boxed(),
        LogFormat::Json => layer
            .event_format(fmt::format().json().flatten_event(true))
            .fmt_fields(fmt::format::JsonFields::new())
            .boxed(),
    }
}

/// Initialize a format layer with the given writer and format
pub fn init_layer<W2>(
    writer: W2,
    format: &LogFormat,
    span_events: FmtSpan,
    ansi: bool,
) -> Box<dyn Layer<Registry> + Sync + Send>
where
    W2: for<'writer> MakeWriter<'writer> + Sync + Send + 'static,
{
    let layer = fmt::Layer::new()
        .with_writer(writer)
        .with_ansi(ansi)
        .with_span_events(span_events);
    apply_layer_format(layer, format)
}

/// Create output layers based on configuration.
///
/// This function creates output layers based on the provided configuration.
///
/// # Arguments
///
/// * `console_enabled` - Whether to enable console output
pub fn create_output_layers(logger: &Logger) -> Result<Vec<BoxLayer>> {
    let mut layers: Vec<BoxLayer> = vec![];

    // Add console layer if enabled
    if logger.console_enabled {
        let stdout_layer = init_layer(
            std::io::stdout,
            &logger.format,
            logger.span_events.clone(),
            logger.ansi,
        );
        layers.push(stdout_layer);
    }
    // Add file layer if configured and enabled
    if let Some(config) = &logger.file_appender {
        if config.enable {
            let rolling_builder = tracing_appender::rolling::Builder::new()
                .max_log_files(config.max_log_files)
                .rotation(config.get_rolling_rotation());

            let file_appender = rolling_builder
                .filename_prefix(config.filename_prefix_or_default())
                .filename_suffix(config.filename_suffix_or_default())
                .build(config.dir_or_default())
                .context("Failed to build file appender")?;

            let file_appender_layer = if config.non_blocking {
                let (non_blocking_file_appender, work_guard) =
                    tracing_appender::non_blocking(file_appender);
                set_nonblocking_appender_guard(work_guard)?;
                init_layer(
                    non_blocking_file_appender,
                    &config.format_or_default(),
                    logger.span_events.clone(),
                    config.ansi,
                )
            } else {
                init_layer(
                    file_appender,
                    &config.format_or_default(),
                    logger.span_events.clone(),
                    config.ansi,
                )
            };
            layers.push(file_appender_layer);
        }
    }
    Ok(layers)
}

/// Initializes the complete tracing stack with OpenTelemetry integration.
///
/// This function sets up the entire tracing infrastructure, including:
/// - OpenTelemetry tracing
/// - Metrics collection
/// - Logs collection (when enable_otel_logs is true)
/// - Log formatting
/// - Environment filtering
///
/// # Arguments
///
/// * `service_name` - The name of your service
/// * `attributes` - Additional key-value pairs to include in the resource
/// * `sample_ratio` - The ratio of traces to sample (0.0 to 1.0)
/// * `metrics_interval_secs` - The interval in seconds between metric collections
/// * `level` - The default tracing level
/// * `layers` - A vector of formatting layers for the tracing output
/// * `enable_otel_logs` - Whether to enable OpenTelemetry logs export
///
/// # Returns
///
/// Returns a `Result` containing the configured `OtelGuard`,
/// or an error if initialization fails.
///
/// # Examples
///
/// ```rust
/// use tracing_otel_extra::logs::setup_tracing;
/// use opentelemetry::KeyValue;
/// use tracing::Level;
/// use tracing_subscriber::fmt;
/// use tracing_subscriber::fmt::Layer;
/// use tracing_opentelemetry_extra::BoxLayer;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let layers: Vec<BoxLayer> = vec![Box::new(fmt::Layer::new().compact())];
///     let guard = setup_tracing(
///         "my-service",
///         &[KeyValue::new("environment", "production")],
///         1.0,
///         30,
///         Level::INFO,
///         layers,
///         true, // enable OTel logs
///     )?;
///
///     // Your application code here...
///
///     // Cleanup when done
///     guard.shutdown()?;
///     Ok(())
/// }
/// ```
pub fn setup_tracing(
    service_name: &str,
    attributes: &[KeyValue],
    sample_ratio: f64,
    metrics_interval_secs: u64,
    level: Level,
    layers: Vec<BoxLayer>,
    otel_logs_enabled: bool,
) -> Result<OtelGuard> {
    let env_filter = init_env_filter(&level);
    let resource = get_resource(service_name, attributes);
    let tracer_provider = init_tracer_provider(&resource, sample_ratio)?;
    let meter_provider = init_meter_provider(&resource, metrics_interval_secs)?;
    let logger_provider = if otel_logs_enabled {
        Some(init_logger_provider(&resource)?)
    } else {
        None
    };

    let guard = init_tracing_subscriber(
        service_name,
        env_filter,
        layers,
        tracer_provider,
        meter_provider,
        logger_provider,
    )?;

    Ok(guard)
}
