//! OpenTelemetry logging configuration and initialization.
//!
//! This module provides a flexible and configurable logging system that integrates
//! OpenTelemetry tracing and metrics. It offers both programmatic configuration
//! through a builder pattern and environment variable-based configuration.
//!
//! # Features
//!
//! - Builder-style configuration API
//! - Environment variable support (with "env" feature)
//! - Multiple log formats (compact, pretty, json)
//! - Configurable sampling and metrics collection
//! - Custom resource attributes
//! - Optional console output
//! - Optional file output
//!
//! # Quick Start
//!
//! ```rust
//! use tracing_otel_extra::Logger;
//! use tracing::Level;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Basic initialization with console output
//!     let guard = Logger::new("my-service").init()?;
//!
//!     // Your application code here...
//!
//!     // The guard will automatically clean up when dropped
//!     Ok(())
//! }
//! ```
//!
//! # Advanced Configuration
//!
//! ```rust
//! use tracing_otel_extra::Logger;
//! use tracing::Level;
//! use opentelemetry::KeyValue;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let guard = Logger::new("my-service")
//!         .with_level(Level::DEBUG)
//!         .with_sample_ratio(0.5)
//!         .with_metrics_interval_secs(60)
//!         .with_console_enabled(true)  // Enable console output
//!         .with_attributes(vec![
//!             KeyValue::new("environment", "production"),
//!             KeyValue::new("version", "1.0.0"),
//!         ])
//!         .init()?;
//!
//!     // Your application code here...
//!
//!     Ok(())
//! }
//! ```
//!
//! # Output Configuration
//!
//! The logger supports flexible output configuration:
//!
//! ## Console Only (Default)
//! ```rust,no_run
//! use tracing_otel_extra::Logger;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let guard = Logger::new("my-service")
//!         .with_console_enabled(true)
//!         .init()?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## File Only
//! ```rust,no_run
//! use tracing_otel_extra::{Logger, LoggerFileAppender, LogFormat, LogRollingRotation};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//! let file_appender = LoggerFileAppender {
//!     enable: true,
//!     non_blocking: false,
//!     level: None,
//!     ansi: false,
//!     format: Some(LogFormat::Json),
//!     rotation: LogRollingRotation::Daily,
//!     dir: Some("/var/log".to_string()),
//!     filename_prefix: Some("myapp".to_string()),
//!     filename_suffix: Some("log".to_string()),
//!     max_log_files: 10,
//! };
//!
//! let guard = Logger::new("my-service")
//!     .with_console_enabled(false)
//!     .with_file_appender(Some(file_appender))
//!     .init()?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Both Console and File
//! ```rust,no_run
//! use tracing_otel_extra::{Logger, LoggerFileAppender, LogFormat, LogRollingRotation};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let file_appender = LoggerFileAppender {
//!         enable: true,
//!         non_blocking: false,
//!         level: None,
//!         ansi: false,
//!         format: Some(LogFormat::Json),
//!         rotation: LogRollingRotation::Daily,
//!         dir: Some("/var/log".to_string()),
//!         filename_prefix: Some("myapp".to_string()),
//!         filename_suffix: Some("log".to_string()),
//!         max_log_files: 10,
//!     };
//!
//!     let guard = Logger::new("my-service")
//!     .with_console_enabled(true)
//!     .with_file_appender(Some(file_appender))
//!     .init()?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Environment Variables
//!
//! When using the "env" feature, you can configure the logger through environment variables:
//!
//! ```rust
//! use tracing_otel_extra::Logger;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     #[cfg(feature = "env")]
//!     {
//!         // Using default prefix "LOG_"
//!         // let guard = Logger::from_env(None)?.init()?;
//!         // Or with custom prefix
//!         let guard = Logger::from_env(Some("MY_APP_"))?.init()?;
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Available Environment Variables
//!
//! | Variable | Description | Default |
//! |----------|-------------|---------|
//! | `LOG_SERVICE_NAME` | Service name | Crate name |
//! | `LOG_FORMAT` | Log format (`compact`, `pretty`, `json`) | `compact` |
//! | `LOG_SPAN_EVENTS` | Span events (`FMT::NEW`, `FMT::ENTER`, `FMT::EXIT`, `FMT::CLOSE`, `FMT::NONE`, `FMT::ACTIVE`, `FMT::FULL`) | `FMT::NEW | FMT::CLOSE` |
//! | `LOG_ANSI` | Enable ANSI colors | `true` |
//! | `LOG_LEVEL` | Log level | `info` |
//! | `LOG_SAMPLE_RATIO` | Sampling ratio (0.0-1.0) | `1.0` |
//! | `LOG_METRICS_INTERVAL_SECS` | Metrics collection interval | `30` |
//! | `LOG_ATTRIBUTES` | Additional attributes (`key=value,key2=value2`) | - |
//! | `LOG_CONSOLE_ENABLED` | Enable console output | `true` |
//!
//! ### File Logging Environment Variables
//!
//! | Variable | Description | Default |
//! |----------|-------------|---------|
//! | `LOG_FILE_ENABLE` | Enable file logging | `false` |
//! | `LOG_FILE_NON_BLOCKING` | Enable non-blocking file logging | `false` |
//! | `LOG_FILE_LEVEL` | File log level | `info` |
//! | `LOG_FILE_FORMAT` | File log format (`compact`, `pretty`, `json`) | `compact` |
//! | `LOG_FILE_ROTATION` | File rotation (`minutely`, `hourly`, `daily`, `never`) | `hourly` |
//! | `LOG_FILE_DIR` | Log directory | `./logs` |
//! | `LOG_FILE_FILENAME_PREFIX` | Log filename prefix | `app` |
//! | `LOG_FILE_FILENAME_SUFFIX` | Log filename suffix | `log` |
//! | `LOG_FILE_MAX_LOG_FILES` | Maximum number of log files to keep | `5` |
//!
//! # Examples
//!
//! ## Basic Configuration
//! ```bash
//! LOG_SERVICE_NAME=my-service
//! LOG_LEVEL=debug
//! LOG_CONSOLE_ENABLED=true
//! ```
//!
//! ## Advanced Configuration
//! ```bash
//! LOG_FORMAT=json
//! LOG_ANSI=false
//! LOG_SAMPLE_RATIO=0.5
//! LOG_METRICS_INTERVAL_SECS=60
//! LOG_ATTRIBUTES=environment=prod,region=us-west
//! LOG_CONSOLE_ENABLED=true
//! ```
//!
//! ## File Logging Configuration
//! ```bash
//! # Enable file logging
//! LOG_FILE_ENABLE=true
//! LOG_FILE_FORMAT=json
//! LOG_FILE_ROTATION=daily
//! LOG_FILE_DIR=/var/log/myapp
//! LOG_FILE_FILENAME_PREFIX=myapp
//! LOG_FILE_FILENAME_SUFFIX=log
//! LOG_FILE_MAX_LOG_FILES=10
//! ```
//!
//! ## Console Only Configuration
//! ```bash
//! LOG_CONSOLE_ENABLED=true
//! # No file logging
//! ```
//!
//! ## File Only Configuration
//! ```bash
//! LOG_CONSOLE_ENABLED=false
//! LOG_FILE_ENABLE=true
//! LOG_FILE_FORMAT=json
//! ```
use crate::{
    logs::{
        create_output_layers,
        layer::{deserialize_attributes, deserialize_log_format, LogFormat, LogRollingRotation},
        subscriber::setup_tracing,
    },
    otel::OtelGuard,
};
use anyhow::{Context, Result};
use opentelemetry::KeyValue;
use serde::Deserialize;
use tracing::Level;
use tracing_appender::rolling::Rotation;
use tracing_subscriber::fmt::format::FmtSpan;

/// Configuration for the OpenTelemetry tracing and logging system.
///
/// This struct provides a builder-style API for configuring various aspects of
/// the tracing system. It supports both programmatic configuration and
/// environment variable-based configuration (with the "env" feature).
///
/// # Examples
///
/// ```rust
/// use tracing_otel_extra::{Logger, FmtSpan};
/// use tracing::Level;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Create with default settings (console enabled)
///     // let guard = Logger::new("my-service").init()?;
///
///     // Create with custom settings
///     let guard = Logger::new("my-service")
///         .with_level(Level::DEBUG)
///         .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
///         .with_sample_ratio(0.5)
///         .with_console_enabled(true)  // Enable console output
///         .init()?;
///
///     // Your application code here...
///
///     Ok(())
/// }
/// ```
///
/// # Output Configuration
///
/// The logger supports flexible output configuration:
///
/// - **Console Only (Default)**: `console_enabled = true`, no file appender
/// - **File Only**: `console_enabled = false`, with file appender
/// - **Both Console and File**: `console_enabled = true`, with file appender
///
/// # Environment Variables
///
/// When using the "env" feature, you can configure the logger through environment variables.
/// See the module-level documentation for a complete list of available variables.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Logger {
    /// The name of the service being traced.
    /// Defaults to the crate name if not specified.
    #[serde(default = "default::service_name")]
    pub service_name: String,

    /// The format to use for log output.
    /// Supported formats: compact, pretty, json.
    #[serde(
        deserialize_with = "deserialize_log_format",
        default = "LogFormat::default"
    )]
    pub format: LogFormat,

    /// The span events to include in the output.
    #[serde(
        default = "default::span_events",
        deserialize_with = "deserialize_span_events"
    )]
    pub span_events: FmtSpan,

    /// Whether to use ANSI colors in the output.
    /// Defaults to true.
    #[serde(default)]
    pub ansi: bool,

    /// The minimum log level to record.
    /// Defaults to INFO.
    #[serde(
        deserialize_with = "deserialize_level_required",
        default = "default::log_level"
    )]
    pub level: Level,

    /// The ratio of traces to sample (0.0 to 1.0).
    /// Defaults to 1.0 (sample all traces).
    #[serde(default = "default::sample_ratio")]
    pub sample_ratio: f64,

    /// The interval in seconds between metrics collection.
    /// Defaults to 30 seconds.
    #[serde(default = "default::metrics_interval_secs")]
    pub metrics_interval_secs: u64,

    /// Additional attributes to add to the resource.
    /// These will be included in all traces and metrics.
    #[serde(default, deserialize_with = "deserialize_attributes")]
    pub attributes: Vec<KeyValue>,

    /// Whether to enable console output.
    /// Defaults to true.
    #[serde(default = "default::console_enabled")]
    pub console_enabled: bool,

    /// Set this if you want to write log to file
    #[serde(default)]
    pub file_appender: Option<LoggerFileAppender>,

    /// Set this if you want to write log to OpenTelemetry
    #[serde(default)]
    pub otel_logs_enabled: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct LoggerFileAppender {
    /// Enable logger file appender
    pub enable: bool,

    /// Enable write log to file non-blocking
    #[serde(default)]
    pub non_blocking: bool,

    /// The minimum log level to record.
    /// If not set, will use the level from Logger
    #[serde(default, deserialize_with = "deserialize_level_optional")]
    pub level: Option<Level>,

    /// Set the logger file appender ansi.
    #[serde(default)]
    pub ansi: bool,

    /// Set the logger file appender format.
    /// If not set, will use the format from Logger
    ///
    /// * options: `compact` | `pretty` | `json`
    #[serde(default, deserialize_with = "deserialize_log_format_optional")]
    pub format: Option<LogFormat>,

    /// Set the logger file appender rotation.
    #[serde(default = "default::rotation")]
    pub rotation: LogRollingRotation,

    /// Set the logger file appender dir
    ///
    /// default is `./logs`
    #[serde(default)]
    pub dir: Option<String>,

    /// Set log filename prefix
    #[serde(default)]
    pub filename_prefix: Option<String>,

    /// Set log filename suffix
    #[serde(default)]
    pub filename_suffix: Option<String>,

    /// Set the logger file appender keep max log files.
    #[serde(default = "default::max_log_files")]
    pub max_log_files: usize,
}

impl LoggerFileAppender {
    /// Merge configuration from Logger, using LoggerFileAppender values if set,
    /// otherwise fall back to Logger values
    pub fn merge_with_logger(&self, logger: &Logger) -> LoggerFileAppender {
        LoggerFileAppender {
            enable: self.enable,
            ansi: self.ansi,
            non_blocking: self.non_blocking,
            level: self.level.or(Some(logger.level)),
            format: self.format.clone().or(Some(logger.format.clone())),
            rotation: self.rotation.clone(),
            dir: self.dir.clone().or(Some(default::dir())),
            filename_prefix: self
                .filename_prefix
                .clone()
                .or(Some(default::filename_prefix())),
            filename_suffix: self
                .filename_suffix
                .clone()
                .or(Some(default::filename_suffix())),
            max_log_files: self.max_log_files,
        }
    }

    pub fn dir_or_default(&self) -> String {
        self.dir.clone().unwrap_or_else(default::dir)
    }

    pub fn filename_prefix_or_default(&self) -> String {
        self.filename_prefix
            .clone()
            .unwrap_or_else(default::filename_prefix)
    }

    pub fn filename_suffix_or_default(&self) -> String {
        self.filename_suffix
            .clone()
            .unwrap_or_else(default::filename_suffix)
    }

    pub fn format_or_default(&self) -> LogFormat {
        self.format.clone().unwrap_or(LogFormat::Compact)
    }

    pub fn get_rolling_rotation(&self) -> Rotation {
        match self.rotation {
            LogRollingRotation::Minutely => Rotation::MINUTELY,
            LogRollingRotation::Hourly => Rotation::HOURLY,
            LogRollingRotation::Daily => Rotation::DAILY,
            LogRollingRotation::Never => Rotation::NEVER,
        }
    }
}

fn deserialize_span_events<'de, D>(deserializer: D) -> Result<FmtSpan, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let s = s.trim();

    if s.is_empty() {
        return Ok(FmtSpan::NONE);
    }

    let mut result = FmtSpan::NONE;

    for part in s.split('|').map(|p| p.trim()) {
        let span = match part {
            "FMT::NEW" | "FmtSpan::NEW" => FmtSpan::NEW,
            "FMT::ENTER" | "FmtSpan::ENTER" => FmtSpan::ENTER,
            "FMT::EXIT" | "FmtSpan::EXIT" => FmtSpan::EXIT,
            "FMT::CLOSE" | "FmtSpan::CLOSE" => FmtSpan::CLOSE,
            "FMT::ACTIVE" | "FmtSpan::ACTIVE" => FmtSpan::ACTIVE,
            "FMT::FULL" | "FmtSpan::FULL" => return Ok(FmtSpan::FULL),
            _ => {
                return Err(serde::de::Error::custom(format!(
                    "Invalid span events: '{part}'. Valid options: FMT::NEW, FMT::ENTER, FMT::EXIT, FMT::CLOSE, FMT::NONE, FMT::ACTIVE, FMT::FULL"
                )));
            }
        };
        result |= span;
    }

    Ok(result)
}

fn deserialize_level_required<'de, D>(deserializer: D) -> Result<Level, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

fn deserialize_level_optional<'de, D>(deserializer: D) -> Result<Option<Level>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.trim().is_empty() {
        return Ok(None);
    }
    s.parse().map(Some).map_err(serde::de::Error::custom)
}

fn deserialize_log_format_optional<'de, D>(deserializer: D) -> Result<Option<LogFormat>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.trim().is_empty() {
        return Ok(None);
    }
    match s.to_lowercase().as_str().trim() {
        "compact" => Ok(Some(LogFormat::Compact)),
        "pretty" => Ok(Some(LogFormat::Pretty)),
        "json" => Ok(Some(LogFormat::Json)),
        _ => Err(serde::de::Error::custom(format!(
            "Invalid log format: '{s}'"
        ))),
    }
}

pub mod default {
    use crate::logs::layer::LogRollingRotation;
    use tracing::Level;
    use tracing_subscriber::fmt::format::FmtSpan;

    /// Default service name: crate name
    pub fn service_name() -> String {
        env!("CARGO_CRATE_NAME").to_string()
    }

    /// Default max log files: 5
    pub fn max_log_files() -> usize {
        5
    }

    /// Default log level: INFO
    pub fn log_level() -> Level {
        Level::INFO
    }

    /// Default log dir: ./logs
    pub fn dir() -> String {
        "./logs".to_string()
    }

    /// Default filename prefix: app
    pub fn filename_prefix() -> String {
        "combine".to_string()
    }

    /// Default filename suffix: log
    pub fn filename_suffix() -> String {
        "log".to_string()
    }

    pub fn span_events() -> FmtSpan {
        FmtSpan::NEW | FmtSpan::CLOSE
    }

    pub fn sample_ratio() -> f64 {
        1.0
    }

    pub fn metrics_interval_secs() -> u64 {
        30
    }

    pub fn rotation() -> LogRollingRotation {
        LogRollingRotation::Hourly
    }

    /// Default console enabled: true
    pub fn console_enabled() -> bool {
        true
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self {
            service_name: default::service_name(),
            format: LogFormat::default(),
            span_events: default::span_events(),
            ansi: true,
            level: default::log_level(),
            sample_ratio: default::sample_ratio(),
            metrics_interval_secs: default::metrics_interval_secs(),
            attributes: vec![],
            console_enabled: default::console_enabled(),
            file_appender: None,
            otel_logs_enabled: false,
        }
    }
}

impl Logger {
    /// Create a new configuration with the given service name.
    ///
    /// # Arguments
    ///
    /// * `service_name` - The name of the service being traced
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            ..Default::default()
        }
    }

    /// Set the service name.
    pub fn with_service_name(mut self, service_name: impl Into<String>) -> Self {
        self.service_name = service_name.into();
        self
    }

    /// Set the log format (compact, pretty, or json).
    pub fn with_format(mut self, format: LogFormat) -> Self {
        self.format = format;
        self
    }

    /// Set the span events to include in the output.
    pub fn with_span_events(mut self, span_events: FmtSpan) -> Self {
        self.span_events = span_events;
        self
    }

    /// Set whether to use ANSI colors in the output.
    pub fn with_ansi(mut self, ansi: bool) -> Self {
        self.ansi = ansi;
        self
    }

    /// Set the minimum log level to record.
    pub fn with_level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    /// Set the ratio of traces to sample (0.0 to 1.0).
    pub fn with_sample_ratio(mut self, ratio: f64) -> Self {
        self.sample_ratio = ratio;
        self
    }

    /// Set the interval in seconds between metrics collection.
    pub fn with_metrics_interval_secs(mut self, secs: u64) -> Self {
        self.metrics_interval_secs = secs;
        self
    }

    /// Add custom attributes to the resource.
    pub fn with_attributes(mut self, attributes: Vec<KeyValue>) -> Self {
        self.attributes = attributes;
        self
    }

    /// Set whether to enable console output.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enable console output. Defaults to true.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tracing_otel_extra::Logger;
    ///
    /// // Disable console output (useful when only using file logging)
    /// let logger = Logger::new("my-service")
    ///     .with_console_enabled(false);
    ///
    /// // Enable console output (default behavior)
    /// let logger = Logger::new("my-service")
    ///     .with_console_enabled(true);
    /// ```
    pub fn with_console_enabled(mut self, enabled: bool) -> Self {
        self.console_enabled = enabled;
        self
    }

    /// Set file appender configuration.
    pub fn with_file_appender(mut self, file_appender: Option<LoggerFileAppender>) -> Self {
        self.file_appender = file_appender;
        self
    }

    /// Initialize tracing with this configuration.
    ///
    /// This method will:
    /// 1. Set up the global tracing subscriber
    /// 2. Configure the OpenTelemetry tracer and meter providers
    /// 3. Configure output layers based on console_enabled and file_appender settings
    /// 4. Return a guard that ensures proper cleanup
    ///
    /// # Output Configuration
    ///
    /// The initialization will configure output layers based on:
    /// - `console_enabled`: If true, adds a console formatting layer
    /// - `file_appender`: If configured and enabled, adds a file formatting layer
    /// - At least one output layer must be configured (console or file)
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `ProviderGuard` that will automatically
    /// clean up the tracing providers when dropped.
    ///
    /// # Examples
    ///
    /// Basic usage with console output:
    /// ```rust
    /// use tracing_otel_extra::Logger;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     // Create with default settings (console enabled)
    ///     let guard = Logger::new("my-service").init()?;
    ///     
    ///     // Use tracing...
    ///     tracing::info!("Hello, world!");
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// File-only logging:
    /// ```rust
    /// use tracing_otel_extra::{Logger, LoggerFileAppender, LogFormat, LogRollingRotation};
    /// use tracing::Level;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let file_appender = LoggerFileAppender {
    ///         enable: true,
    ///         non_blocking: false,
    ///         level: Some(Level::INFO),
    ///         ansi: false,
    ///         format: Some(LogFormat::Json),
    ///         rotation: LogRollingRotation::Daily,
    ///         dir: Some("./logs".to_string()),
    ///         filename_prefix: Some("app".to_string()),
    ///         filename_suffix: Some("log".to_string()),
    ///         max_log_files: 5,
    ///     };
    ///
    ///     let guard = Logger::new("my-service")
    ///         .with_console_enabled(false)  // Disable console
    ///         .with_file_appender(Some(file_appender))
    ///         .init()?;
    ///     
    ///     // Use tracing (outputs to file only)
    ///     tracing::info!("This will only go to the log file");
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// Both console and file logging:
    /// ```rust
    /// use tracing_otel_extra::{Logger, LoggerFileAppender, LogFormat, LogRollingRotation};
    /// use tracing::Level;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let file_appender = LoggerFileAppender {
    ///         enable: true,
    ///         non_blocking: false,
    ///         level: Some(Level::INFO),
    ///         ansi: false,
    ///         format: Some(LogFormat::Json),
    ///         rotation: LogRollingRotation::Daily,
    ///         dir: Some("./logs".to_string()),
    ///         filename_prefix: Some("app".to_string()),
    ///         filename_suffix: Some("log".to_string()),
    ///         max_log_files: 5,
    ///     };
    ///
    ///     let guard = Logger::new("my-service")
    ///         .with_console_enabled(true)   // Enable console
    ///         .with_file_appender(Some(file_appender))
    ///         .init()?;
    ///     
    ///     // Use tracing (outputs to both console and file)
    ///     tracing::info!("This will go to both console and file");
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to initialize the tracing subscriber
    /// - Failed to set up OpenTelemetry providers
    /// - Failed to configure the environment filter
    /// - No output layers are configured (both console and file are disabled)
    pub fn init(self) -> Result<OtelGuard> {
        init_tracing_from_logger(self)
    }

    /// Initialize the logger from environment variables.
    ///
    /// This method requires the "env" feature to be enabled.
    ///
    /// # Arguments
    ///
    /// * `prefix` - Optional prefix for environment variables. If None, "LOG_" is used.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tracing_otel_extra::Logger;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     #[cfg(feature = "env")]
    ///     {
    ///         let guard = Logger::from_env(None)?.init()?;
    ///     }
    ///     Ok(())
    /// }
    /// ```
    #[cfg(feature = "env")]
    pub fn from_env(prefix: Option<&str>) -> Result<Self> {
        init_logger_from_env(prefix)
    }
}

// Initialize tracing from logger
pub fn init_tracing_from_logger(logger: Logger) -> Result<OtelGuard> {
    let layers = create_output_layers(&logger)?;

    let guard = setup_tracing(
        &logger.service_name,
        &logger.attributes,
        logger.sample_ratio,
        logger.metrics_interval_secs,
        logger.level,
        layers,
        logger.otel_logs_enabled,
    )
    .context("Failed to initialize tracing")?;
    Ok(guard)
}

/// Convenience function to initialize tracing with default settings
pub fn init_logging(service_name: &str) -> Result<OtelGuard> {
    let logger = Logger::new(service_name);
    init_tracing_from_logger(logger)
}

#[cfg(feature = "env")]
pub fn init_logger_from_env(prefix: Option<&str>) -> Result<Logger> {
    let prefix = prefix.unwrap_or("LOG_");
    let file_prefix = format!("{prefix}FILE_");
    // file appender from env
    let file_appender: Option<LoggerFileAppender> = envy::prefixed(&file_prefix).from_env().ok();
    // logger from env
    let mut logger: Logger = envy::prefixed(prefix)
        .from_env()
        .context("Failed to deserialize environment variables")?;

    if let Some(file_appender) = file_appender {
        let merged_file_appender = file_appender.merge_with_logger(&logger);
        logger = logger.with_file_appender(Some(merged_file_appender));
    }
    Ok(logger)
}

#[cfg(feature = "env")]
pub fn init_logging_from_env(prefix: Option<&str>) -> Result<OtelGuard> {
    let logger = init_logger_from_env(prefix)?;
    init_tracing_from_logger(logger)
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::KeyValue;
    use serde_json;

    #[derive(Debug, serde::Deserialize)]
    struct TestFmtSpan {
        #[serde(deserialize_with = "deserialize_span_events")]
        span_events: FmtSpan,
    }

    #[test]
    fn test_logger_builder() {
        let logger = Logger::new("test-service")
            .with_level(Level::DEBUG)
            .with_sample_ratio(0.5)
            .with_attributes(vec![KeyValue::new("test", "value")]);

        assert_eq!(logger.service_name, "test-service");
        assert_eq!(logger.level, Level::DEBUG);
        assert_eq!(logger.sample_ratio, 0.5);
        assert_eq!(logger.attributes.len(), 1);
    }

    #[test]
    fn test_deserialize_span_events_fmt_format() {
        // Test single values
        let result: TestFmtSpan = serde_json::from_str(r#"{"span_events": "FMT::NEW"}"#).unwrap();
        assert_eq!(result.span_events, FmtSpan::NEW);

        let result: TestFmtSpan = serde_json::from_str(r#"{"span_events": "FMT::CLOSE"}"#).unwrap();
        assert_eq!(result.span_events, FmtSpan::CLOSE);

        // Test combinations
        let result: TestFmtSpan =
            serde_json::from_str(r#"{"span_events": "FMT::NEW|FMT::CLOSE"}"#).unwrap();
        assert_eq!(result.span_events, FmtSpan::NEW | FmtSpan::CLOSE);

        let result: TestFmtSpan =
            serde_json::from_str(r#"{"span_events": "FMT::ENTER|FMT::EXIT"}"#).unwrap();
        assert_eq!(result.span_events, FmtSpan::ENTER | FmtSpan::EXIT);

        // Test predefined combinations
        let result: TestFmtSpan =
            serde_json::from_str(r#"{"span_events": "FMT::ACTIVE"}"#).unwrap();
        assert_eq!(result.span_events, FmtSpan::ACTIVE);

        let result: TestFmtSpan = serde_json::from_str(r#"{"span_events": "FMT::FULL"}"#).unwrap();
        assert_eq!(result.span_events, FmtSpan::FULL);

        // Test with spaces
        let result: TestFmtSpan =
            serde_json::from_str(r#"{"span_events": " FMT::NEW | FMT::CLOSE "}"#).unwrap();
        assert_eq!(result.span_events, FmtSpan::NEW | FmtSpan::CLOSE);
    }

    #[test]
    fn test_deserialize_span_events_empty() {
        let result: TestFmtSpan = serde_json::from_str(r#"{"span_events": ""}"#).unwrap();
        assert_eq!(result.span_events, FmtSpan::NONE);

        let result: TestFmtSpan = serde_json::from_str(r#"{"span_events": "   "}"#).unwrap();
        assert_eq!(result.span_events, FmtSpan::NONE);
    }

    #[test]
    fn test_deserialize_span_events_invalid() {
        let result: Result<TestFmtSpan, _> = serde_json::from_str(r#"{"span_events": "INVALID"}"#);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid span events"));

        let result: Result<TestFmtSpan, _> =
            serde_json::from_str(r#"{"span_events": "FMT::NEW|INVALID"}"#);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid span events"));
    }

    #[test]
    fn test_logger_with_file_appender() {
        let file_appender = LoggerFileAppender {
            enable: true,
            non_blocking: false,
            level: Some(Level::INFO),
            ansi: false,
            format: Some(LogFormat::Json),
            rotation: LogRollingRotation::Daily,
            dir: Some("/var/log/test".to_string()),
            filename_prefix: Some("test".to_string()),
            filename_suffix: Some("log".to_string()),
            max_log_files: 10,
        };

        let logger = Logger::new("test-service")
            .with_level(Level::DEBUG)
            .with_sample_ratio(0.5)
            .with_attributes(vec![KeyValue::new("test", "value")])
            .with_file_appender(Some(file_appender));

        assert_eq!(logger.service_name, "test-service");
        assert_eq!(logger.level, Level::DEBUG);
        assert_eq!(logger.sample_ratio, 0.5);
        assert_eq!(logger.attributes.len(), 1);
        assert!(logger.file_appender.is_some());

        let file_appender = logger.file_appender.unwrap();
        assert!(file_appender.enable);
        assert_eq!(file_appender.level, Some(Level::INFO));
        assert_eq!(file_appender.format, Some(LogFormat::Json));
        assert_eq!(file_appender.rotation, LogRollingRotation::Daily);
        assert_eq!(file_appender.dir, Some("/var/log/test".to_string()));
        assert_eq!(file_appender.filename_prefix, Some("test".to_string()));
        assert_eq!(file_appender.filename_suffix, Some("log".to_string()));
        assert_eq!(file_appender.max_log_files, 10);
    }

    #[test]
    fn test_env_file_appender_parsing() {
        #[cfg(feature = "env")]
        {
            // 设置环境变量
            std::env::set_var("LOG_FILE_ENABLE", "true");
            std::env::set_var("LOG_FILE_FORMAT", "json");
            std::env::set_var("LOG_FILE_DIR", "/var/log/test");
            std::env::set_var("LOG_FILE_FILENAME_PREFIX", "test-app");
            std::env::set_var("LOG_FILE_FILENAME_SUFFIX", "log");
            std::env::set_var("LOG_FILE_MAX_LOG_FILES", "10");

            let result: Result<LoggerFileAppender, _> = envy::prefixed("LOG_FILE_").from_env();
            println!("Parse result: {:?}", result);

            let file_appender: Option<LoggerFileAppender> = result.ok();
            assert!(file_appender.is_some());

            let file_appender = file_appender.unwrap();
            assert!(file_appender.enable);
            assert_eq!(file_appender.format, Some(LogFormat::Json));
            assert_eq!(file_appender.dir, Some("/var/log/test".to_string()));
            assert_eq!(file_appender.filename_prefix, Some("test-app".to_string()));
            assert_eq!(file_appender.filename_suffix, Some("log".to_string()));
            assert_eq!(file_appender.max_log_files, 10);
            assert_eq!(file_appender.rotation, LogRollingRotation::Hourly); // 默认值
        }
    }

    #[test]
    fn test_simple_env_parsing() {
        #[cfg(feature = "env")]
        {
            // 只设置最基本的字段
            std::env::set_var("LOG_FILE_ENABLE", "true");

            println!("Testing simple env parsing...");
            let result: Result<LoggerFileAppender, _> = envy::prefixed("LOG_FILE_").from_env();
            println!("Simple parse result: {:?}", result);

            // 即使只有 enable=true，也应该能解析成功
            let file_appender: Option<LoggerFileAppender> = result.ok();
            if let Some(fa) = file_appender {
                println!(
                    "Parsed file appender: enable={}, format={:?}, level={:?}",
                    fa.enable, fa.format, fa.level
                );
                assert!(fa.enable);
            } else {
                println!("Failed to parse file appender");
                assert!(false, "Should have parsed file appender");
            }
        }
    }

    #[test]
    fn test_logger_console_control() {
        // Test default console enabled
        let logger = Logger::new("test-service");
        assert!(logger.console_enabled);

        // Test disabling console
        let logger = Logger::new("test-service").with_console_enabled(false);
        assert!(!logger.console_enabled);

        // Test enabling console
        let logger = Logger::new("test-service").with_console_enabled(true);
        assert!(logger.console_enabled);
    }

    #[test]
    fn test_logger_console_and_file_combination() {
        let file_appender = LoggerFileAppender {
            enable: true,
            non_blocking: false,
            level: Some(Level::INFO),
            ansi: false,
            format: Some(LogFormat::Json),
            rotation: LogRollingRotation::Daily,
            dir: Some("/var/log/test".to_string()),
            filename_prefix: Some("test".to_string()),
            filename_suffix: Some("log".to_string()),
            max_log_files: 10,
        };

        // Test both console and file enabled
        let logger = Logger::new("test-service")
            .with_console_enabled(true)
            .with_file_appender(Some(file_appender.clone()));
        assert!(logger.console_enabled);
        assert!(logger.file_appender.is_some());

        // Test only file enabled (console disabled)
        let logger = Logger::new("test-service")
            .with_console_enabled(false)
            .with_file_appender(Some(file_appender.clone()));
        assert!(!logger.console_enabled);
        assert!(logger.file_appender.is_some());

        // Test only console enabled (no file appender)
        let logger = Logger::new("test-service")
            .with_console_enabled(true)
            .with_file_appender(None);
        assert!(logger.console_enabled);
        assert!(logger.file_appender.is_none());
    }
}
