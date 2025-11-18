# tracing-opentelemetry-extra

**Reference:** This crate is mainly organized based on the [official tracing-opentelemetry OTLP example](https://github.com/tokio-rs/tracing-opentelemetry/blob/v0.1.x/examples/opentelemetry-otlp.rs).

This crate provides enhanced OpenTelemetry integration for tracing applications. It's based on the [tracing-opentelemetry examples](https://github.com/tokio-rs/tracing-opentelemetry/blob/v0.1.x/examples/opentelemetry-otlp.rs) and provides a clean, easy-to-use API for setting up OpenTelemetry tracing and metrics.

## Features

- Easy OpenTelemetry initialization with OTLP exporter
- Configurable sampling and resource attributes
- Automatic cleanup with guard pattern
- Support for both tracing and metrics
- Clean separation of concerns from other tracing libraries

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
tracing-opentelemetry-extra = "0.31.x"
```

## Quick Start

### Basic Usage

```rust
use opentelemetry::KeyValue;
use tracing_opentelemetry_extra::{get_resource, init_tracer_provider, init_meter_provider, OtelGuard};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create resource with service name and attributes
    let resource = get_resource(
        "my-service",
        &[
            KeyValue::new("environment", "production"),
            KeyValue::new("version", "1.0.0"),
        ],
    );

    // Initialize providers
    let tracer_provider = init_tracer_provider(&resource, 1.0)?;
    let meter_provider = init_meter_provider(&resource, 30)?;

    // initialize tracing subscriber with otel layers

    // Create guard for automatic cleanup
    let _guard = OtelGuard::new(Some(tracer_provider), Some(meter_provider));

    // Your application code here...
    tracing::info!("Application started");

    // Cleanup is handled automatically when the guard is dropped
    Ok(())
}
```

### With Tracing Subscriber

```rust
use opentelemetry::KeyValue;
use tracing_opentelemetry_extra::{get_resource, init_tracer_provider, init_meter_provider, init_tracing_subscriber, OtelGuard};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create resource
    let service_name = "my-service";
    let resource = get_resource(service_name, &[KeyValue::new("environment", "production")]);
    
    // Initialize providers
    let tracer_provider = init_tracer_provider(&resource, 1.0)?;
    let meter_provider = init_meter_provider(&resource, 30)?;

    // Set up tracing subscriber
    let env_filter = init_env_filter(&Level::INFO);
    
    // Create guard for cleanup
    let _guard = init_tracing_subscriber(
        service_name,
        env_filter,
        vec![Box::new(tracing_subscriber::fmt::layer())],
        tracer_provider,
        meter_provider,
    )?;

    // Your application code here...
    tracing::info!("Application started with OpenTelemetry");

    Ok(())
}
```

## Configuration

### Sampling

Control the ratio of traces to sample (0.0 to 1.0):

```rust
// Sample 50% of traces
let tracer_provider = init_tracer_provider(&resource, 0.5)?;

// Sample all traces
let tracer_provider = init_tracer_provider(&resource, 1.0)?;
```

### Metrics Collection

Configure the interval for metrics collection:

```rust
// Collect metrics every 60 seconds
let meter_provider = init_meter_provider(&resource, 60)?;
```

### Resource Attributes

Add custom attributes to your service:

```rust
let resource = get_resource(
    "my-service",
    &[
        KeyValue::new("environment", "production"),
        KeyValue::new("version", "1.0.0"),
        KeyValue::new("region", "us-west-2"),
    ],
);
```

## Features

- `subscriber` (default): Enables tracing-subscriber integration

## Examples

See the [examples directory](../../examples/) for more detailed usage examples.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](../../LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](../../LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

## Related Crates

- [tracing-otel-extra](../tracing-otel/) - HTTP, context, fields, and span utilities
- [axum-otel](../axum-otel/) - Axum web framework integration 