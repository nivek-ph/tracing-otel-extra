# tracing-otel-extra

[![Crates.io](https://img.shields.io/crates/v/tracing-otel-extra.svg)](https://crates.io/crates/tracing-otel-extra)
[![Documentation](https://docs.rs/tracing-otel-extra/badge.svg)](https://docs.rs/tracing-otel-extra)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../LICENSE)

A tracing and OpenTelemetry integration utility library for Rust applications, providing easy-to-use configuration and initialization capabilities.

## Features

- **Easy to Use** - Simple configuration of tracing and OpenTelemetry through Builder pattern
- **Multiple Output Formats** - Support for Compact, Pretty, and JSON formats
- **Flexible Configuration** - Configurable sampling rates, log levels, metrics collection intervals, etc.
- **Automatic Resource Cleanup** - Automatic management of TracerProvider and MeterProvider through RAII pattern
- **Built-in Metrics Support** - Integrated OpenTelemetry metrics collection and export
- **Environment Detection** - Automatic detection of operating system and process information
- **OTLP Export** - Built-in OTLP protocol support, can directly export to Jaeger, OTEL Collector, etc.

## Quick Start

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
tracing-otel-extra = "0.1.0"
tracing = "0.1"
tokio = { version = "1.0", features = ["full"] }
```

### Basic Usage

```rust
use tracing_otel_extra::Logger;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize with default configuration
    let _guard = Logger::default().init()?;
    
    info!("Application started");
    warn!("This is a warning message");
    
    // _guard automatically cleans up resources when it goes out of scope
    Ok(())
}
```

### Advanced Configuration

```rust
use tracing_otel_extra::{Logger, LogFormat};
use opentelemetry::KeyValue;
use tracing::Level;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = Logger::new("my-awesome-service")
        .with_format(LogFormat::Json)           // Use JSON format
        .with_level(Level::DEBUG)               // Set log level
        .with_ansi(false)                       // Disable ANSI colors
        .with_sample_ratio(0.1)                 // 10% sampling rate
        .with_metrics_interval(60)              // 60-second metrics collection interval
        .with_stdout_metrics(false)             // Disable console metrics output
        .with_attributes(vec![                  // Add custom attributes
            KeyValue::new("environment", "production"),
            KeyValue::new("version", "1.2.3"),
        ])
        .init()?;
    
    // Your application code
    tracing::info!(
        user_id = 12345,
        action = "login",
        "User logged in successfully"
    );
    
    Ok(())
}
```

### Legacy API (Backward Compatibility)

```rust
use tracing_otel_extra::init_logging;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = init_logging("legacy-service")?;
    
    tracing::info!("Initialized using legacy API");
    
    Ok(())
}
```

## Configuration Options

| Option                  | Type            | Default    | Description                                            |
| ----------------------- | --------------- | ---------- | ------------------------------------------------------ |
| `service_name`          | `String`        | Crate name | Service name for OpenTelemetry resource identification |
| `format`                | `LogFormat`     | `Compact`  | Log output format: `Compact`, `Pretty`, `Json`         |
| `ansi`                  | `bool`          | `true`     | Whether to enable ANSI color output                    |
| `level`                 | `Level`         | `INFO`     | Log level filtering                                    |
| `sample_ratio`          | `f64`           | `1.0`      | Trace sampling ratio (0.0-1.0)                         |
| `metrics_interval_secs` | `u64`           | `30`       | Metrics collection and export interval (seconds)       |
| `attributes`            | `Vec<KeyValue>` | `[]`       | Custom OpenTelemetry attributes                        |
| `otel_logs_enabled`     | `bool`          | `false`    | Whether to enable OpenTelemetry logs export            |

## Environment Variable Configuration

This library supports standard OpenTelemetry environment variables:

```bash
# OTLP export endpoint
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
export OTEL_EXPORTER_OTLP_PROTOCOL=grpc
# HTTP OTLP options:
# export OTEL_EXPORTER_OTLP_PROTOCOL=http/protobuf
# export OTEL_EXPORTER_OTLP_PROTOCOL=http/json
#
# Specify Protocol for Traces and Metrics:
# export OTEL_EXPORTER_OTLP_TRACES_PROTOCOL=http/protobuf
# export OTEL_EXPORTER_OTLP_METRICS_PROTOCOL=http/json
#
# Default behavior: when no protocol env vars are set, both traces and metrics use grpc.

# Log level (takes precedence over code configuration)
export RUST_LOG=debug

# Resource attributes
export OTEL_RESOURCE_ATTRIBUTES=service.name=my-service,service.version=1.0.0
```

## Integration with Axum

Use with `axum-otel` to achieve complete web service observability:

```rust
use axum::{routing::get, Router};
use axum_otel::{AxumOtelSpanCreator, AxumOtelOnResponse, Level};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing_otel_extra::{Logger, LogFormat};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    let _guard = Logger::new("web-service")
        .with_format(LogFormat::Json)
        .init()?;
    
    // Build Axum application
    let app = Router::new()
        .route("/api/health", get(health_check))
        .layer(
            ServiceBuilder::new()
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(AxumOtelSpanCreator::new().level(Level::INFO))
                        .on_response(AxumOtelOnResponse::new().level(Level::INFO))
                )
        );
    
    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}
```

## Resource Cleanup

`ProviderGuard` implements the RAII pattern and automatically cleans up OpenTelemetry resources when the guard goes out of scope:

```rust
{
    let _guard = Logger::new("temp-service").init()?;
    // Use tracing
    tracing::info!("Temporary service started");
} // <- guard automatically cleans up resources here

// Manual cleanup is also possible
let guard = Logger::new("manual-cleanup").init()?;
// ... use tracing
guard.shutdown()?; // Manual cleanup
```

## Requirements

- **Rust Version**: 1.70+
- **Tokio Runtime**: Requires tokio async runtime
- **OTLP Receiver**: Need to configure an OTLP-compatible receiver (such as Jaeger, OTEL Collector)

## Troubleshooting

### Common Issues

1. **Failed to connect to OTLP receiver**
   ```bash
   # Check endpoint configuration
   export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
   
   # Quickly start Jaeger using Docker
   docker run -d --name jaeger \
     -p 16686:16686 \
     -p 14250:14250 \
     -p 4317:4317 \
     jaegertracing/all-in-one:latest
   ```

2. **Log level filtering not working**
   ```bash
   # Ensure environment variable is set correctly
   export RUST_LOG=debug
   # Or explicitly set level in code
   .with_level(Level::DEBUG)
   ```

3. **Metrics collection issues**
   ```rust
   // Adjust metrics collection interval
   Logger::new("service")
       .with_metrics_interval(10) // Collect every 10 seconds
       .with_stdout_metrics(true) // Enable console output for debugging
   ```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Issues and Pull Requests are welcome! Please ensure:

1. Code passes all tests
2. Add appropriate documentation
3. Follow the project's code style

## Related Projects

- [axum-otel](../axum-otel) - OpenTelemetry middleware for Axum Web framework
- [OpenTelemetry Rust](https://github.com/open-telemetry/opentelemetry-rust) - OpenTelemetry Rust implementation
- [tracing](https://github.com/tokio-rs/tracing) - Structured logging framework for Rust
