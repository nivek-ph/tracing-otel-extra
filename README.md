# tracing-otel-extra

[![Crates.io](https://img.shields.io/crates/v/tracing-otel-extra.svg)](https://crates.io/crates/tracing-otel-extra)
[![Documentation](https://docs.rs/tracing-otel-extra/badge.svg)](https://docs.rs/tracing-otel-extra)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

A comprehensive collection of tracing and logging utilities for Rust applications, with special focus on Axum web framework integration and OpenTelemetry observability.

## 🚀 Features

- **Easy OpenTelemetry Integration** - Simple configuration and initialization for tracing and metrics
- **Axum Web Framework Support** - Structured logging middleware with request tracing
- **Multiple Output Formats** - Support for Compact, Pretty, and JSON log formats
- **Distributed Tracing** - Full support for OpenTelemetry distributed tracing
- **Metrics Collection** - Built-in metrics collection and export capabilities
- **Automatic Resource Management** - RAII pattern for automatic cleanup
- **Environment Configuration** - Support for standard OpenTelemetry environment variables
- **Microservices Ready** - Complete observability solution for microservices architectures

## 🔍 Comparison with `axum-tracing-opentelemetry`

Both **axum-otel** and **axum-tracing-opentelemetry** are excellent projects in the Axum + OpenTelemetry ecosystem, and they share the same core goal: **making HTTP tracing easier and more reliable for Axum applications**.

The main difference lies not in quality, but in **scope and philosophy**.

### axum-tracing-opentelemetry

**axum-tracing-opentelemetry** is a well-designed, focused middleware that:

* Provides **clean and lightweight HTTP tracing**
* Emphasizes **simplicity and minimal overhead**
* Integrates naturally with existing `tracing` and `tracing-subscriber` setups
* Is easy to adopt when you only need **request-level spans and context propagation**

For many applications, especially smaller services or teams that already have their own observability stack, **axum-tracing-opentelemetry is a great and perfectly sufficient choice**.

### axum-otel (this crate)

**axum-otel** builds on similar foundations, but targets a slightly different use case:

* It aims to provide a **more opinionated, production-oriented observability setup**
* In addition to tracing, it includes **built-in HTTP metrics instrumentation**
* It offers **structured logging**, file logging with rotation, and unified configuration
* It manages OpenTelemetry providers and exporters with **RAII-based lifecycle handling**

The goal of axum-otel is to **reduce boilerplate and decision-making** when setting up observability for production or microservice-oriented systems.

### Key Features of axum-otel

**axum-otel** provides a comprehensive observability solution with the following key features:

* **Tower Integration** - Built on top of `tower-http::TraceLayer`, seamlessly integrates with Axum's middleware system and follows Tower's service-oriented architecture
* **JSON Logging Support** - Full support for structured JSON logging format, making it easy to integrate with log aggregation systems like Loki, Elasticsearch, or cloud logging services
* **Multiple Log Formats** - Supports Compact, Pretty, and JSON log formats, allowing you to choose the best format for your environment (development vs production)
* **Structured Logging** - Rich structured logging with customizable fields, making logs searchable and analyzable
* **File Logging & Rotation** - Built-in file appender with automatic log rotation, perfect for production deployments
* **Builder Pattern API** - Intuitive builder pattern for configuration, reducing boilerplate code
* **HTTP Semantic Attributes** - Request spans capture method, route, URL parts, protocol, `server.address`, `client.address` (with `ConnectInfo`), `user_agent.original`, `request_id`, and `trace_id`, aligned with [OpenTelemetry HTTP traces](https://opentelemetry.io/docs/specs/semconv/http/http-spans/)
* **Metrics Collection** - Built-in HTTP metrics instrumentation with OTLP export support
* **Environment Configuration** - Support for standard OpenTelemetry environment variables for flexible deployment configuration

### Choosing Between Them

* Choose **axum-tracing-opentelemetry** if:

  * You want a **minimal, tracing-only solution**
  * You prefer to assemble observability components yourself
  * Low overhead and simplicity are your top priorities

* Choose **axum-otel** if:

  * You want **tracing, metrics, and logging to work together out of the box**
  * You are building **production or microservice-based systems**
  * You prefer a **single, cohesive observability setup** with fewer moving parts

Both crates are actively maintained and follow best practices in the Rust and OpenTelemetry ecosystems.
Which one to use ultimately depends on **your application’s complexity and observability needs**.

## 📦 Crates

This workspace contains several specialized crates:

### [axum-otel](./crates/axum-otel/README.md)
OpenTelemetry tracing middleware for Axum web framework
- Structured logging middleware
- Request/response tracing
- Customizable span attributes
- Metrics collection

### [tracing-otel-extra](./crates/tracing-otel/README.md)
OpenTelemetry tracing support for tracing-subscriber
- Easy-to-use configuration through Builder pattern
- Multiple log output formats (Compact, Pretty, JSON)
- Automatic resource cleanup with RAII pattern
- Built-in metrics support
- Environment detection and configuration

### [tracing-opentelemetry-extra](./crates/tracing-opentelemetry/README.md)
Enhanced OpenTelemetry integration utilities
- Clean, easy-to-use API for OpenTelemetry setup
- Configurable sampling and resource attributes
- Automatic cleanup with guard pattern
- Support for both tracing and metrics

## 🛠️ Installation

Add the desired crate to your `Cargo.toml`:

```toml
# For Axum web framework integration
[dependencies]
axum-otel = "0.31"
axum = { version = "0.8", features = ["macros"] }
tower-http = { version = "0.6.6", features = ["trace"] }

# For general OpenTelemetry tracing
tracing-otel-extra = "0.31"
tracing = "0.1"
tokio = { version = "1.0", features = ["full"] }
```

## 🚀 Quick Start

### Basic Axum Integration

```rust
use axum::{routing::get, Router};
use axum_otel::{AxumOtelSpanCreator, AxumOtelOnResponse, AxumOtelOnFailure};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::Level;

async fn handler() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    let _guard = tracing_otel_extra::Logger::new("my-service")
        .with_format(tracing_otel_extra::LogFormat::Json)
        .init()
        .expect("Failed to initialize tracing");

    // Build Axum application with tracing
    let app = Router::new()
        .route("/", get(handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(AxumOtelSpanCreator::new().level(Level::INFO))
                .on_response(AxumOtelOnResponse::new().level(Level::INFO))
                .on_failure(AxumOtelOnFailure::new()),
        );

    // Start server
    let listener = TcpListener::bind("127.0.0.1:3000").await.expect("Failed to bind to port 3000");
    tracing::info!("Server listening on {}", listener.local_addr().expect("Failed to get local address"));
    axum::serve(listener, app.into_make_service()).await.expect("Failed to serve application");
}
```

### Advanced Configuration

```rust
use tracing_otel_extra::{Logger, LogFormat};
use opentelemetry::KeyValue;
use tracing::Level;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = Logger::new("production-service")
        .with_format(LogFormat::Json)
        .with_level(Level::DEBUG)
        .with_sample_ratio(0.1)  // 10% sampling
        .with_metrics_interval(60)
        .with_attributes(vec![
            KeyValue::new("environment", "production"),
            KeyValue::new("version", "1.2.3"),
        ])
        .init()?;

    tracing::info!(
        user_id = 12345,
        action = "login",
        "User logged in successfully"
    );

    Ok(())
}
```

## 📚 Examples

### [OpenTelemetry Integration](./examples/otel/README.md)
Basic OpenTelemetry tracing setup with Jaeger visualization.

**Prerequisites:**
```bash
# Start Jaeger
docker run -d -p6831:6831/udp -p6832:6832/udp -p16686:16686 -p4317:4317 \
  jaegertracing/all-in-one:latest
```

**Run:**
```bash
cargo run --example otel
curl http://localhost:8080/hello
```

### [Microservices Example](./examples/microservices/README.md)
Complete microservices observability with distributed tracing using Docker Compose.

**Services:**
- **users-service** (port 8081) - User management
- **articles-service** (port 8082) - Article management  
- **axum-otel-demo** (port 8080) - Demo application

**Observability:**
- **Log Collection**: Grafana Alloy → Loki
- **Tracing**: OpenTelemetry → Tempo
- **Visualization**: Grafana (Loki + Tempo)

**Quick Start:**
```bash
# Start all services
docker compose up -d

# Test API
curl -X POST http://localhost:8081/users \
  -H "Content-Type: application/json" \
  -d '{"name": "John Doe", "email": "john@example.com"}'
```

**Visualization:**
- **Grafana UI**: ![loki + tempo](./images/loki-tempo.png)
- **Jaeger UI**: ![jaeger](./images/jaeger.png) (alternative)

## 🔧 Configuration

### Environment Variables

```bash
# OTLP export configuration
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
export OTEL_EXPORTER_OTLP_PROTOCOL=grpc

# Log level (overrides code configuration)
export RUST_LOG=debug

# Resource attributes
export OTEL_RESOURCE_ATTRIBUTES='service.name=my-service,service.version=1.0.0'
```

### OpenTelemetry

All observability backends use the standard OTLP (OpenTelemetry Protocol) for data export. You can choose from the following options:

#### 1: dockotlp (Self-Hosted Grafana Stack)

[dockotlp](https://github.com/iamnivekx/dockotlp) provides a complete self-hosted observability stack with Grafana, Prometheus, Loki, Tempo, OpenTelemetry Collector, and more.

Perfect for self-hosted deployments where you want full control over your observability infrastructure.

**Quick Start:**

Follow the [dockotlp Quick Start guide](https://github.com/iamnivekx/dockotlp?tab=readme-ov-file#quick-start) to set up the stack.

**Configure your application:**

```bash
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
export OTEL_EXPORTER_OTLP_PROTOCOL=grpc
```

#### 2: Grafana Cloud

**Setup:**

```bash
# Grafana Cloud OTLP endpoint (replace with your region and instance URL)
export OTEL_EXPORTER_OTLP_ENDPOINT=https://otlp-gateway-prod-<region>.grafana.net/otlp

# Use HTTP/Protobuf protocol for Grafana Cloud
export OTEL_EXPORTER_OTLP_PROTOCOL=http/protobuf

# Authentication header (Basic auth with instance ID and API token)
# Format: Authorization=Basic <base64(instanceId:apiToken)>
export OTEL_EXPORTER_OTLP_HEADERS="Authorization=Basic <your-base64-credentials>"
```

**Getting Grafana Cloud Credentials:**

1. Log in to your Grafana Cloud account
2. Navigate to **Connections** → **OpenTelemetry**
3. Copy the **OTLP Endpoint URL** and **Basic Auth** credentials
4. Set the environment variables as shown above

**Example `.env` file for Grafana Cloud:**

```bash
OTEL_EXPORTER_OTLP_PROTOCOL="http/protobuf"
OTEL_EXPORTER_OTLP_ENDPOINT="https://otlp-gateway-prod-ap-southeast-1.grafana.net/otlp"
OTEL_EXPORTER_OTLP_HEADERS="Authorization=Basic <your-base64-credentials>"
OTEL_RESOURCE_ATTRIBUTES="service.name=my-service,service.version=1.0.0"
RUST_LOG=info
```

**Note:** Replace `<your-base64-credentials>` with your actual Base64-encoded credentials from Grafana Cloud. The format is `Base64(instanceId:apiToken)`.

#### Option 3: Jaeger

[Jaeger](https://www.jaegertracing.io/) is a popular open-source distributed tracing system, originally developed by Uber. It provides:
- **Distributed Tracing** - End-to-end request tracing
- **Jaeger UI** - Web-based trace visualization
- **Multiple Storage Backends** - Supports various storage options (Elasticsearch, Cassandra, etc.)

Great for teams already using Jaeger or preferring its specific features.

**Setup:**

```bash
# Start Jaeger All-in-One (includes collector, query, and UI)
docker run -d \
  -p 6831:6831/udp \
  -p 6832:6832/udp \
  -p 16686:16686 \
  -p 4317:4317 \
  jaegertracing/all-in-one:latest

# Configure your application
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
export OTEL_EXPORTER_OTLP_PROTOCOL=grpc
```

**Access Jaeger UI:** http://localhost:16686

### Sampling Configuration

```rust
// Sample 50% of traces
let _guard = Logger::new("service")
    .with_sample_ratio(0.5)
    .init()?;
```

## 🏗️ Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Axum App      │    │   tracing-otel   │    │  OpenTelemetry  │
│                 │    │                  │    │                 │
│ ┌─────────────┐ │    │ ┌──────────────┐ │    │ ┌─────────────┐ │
│ │axum-otel    │ │◄──►│ │Logger        │ │◄──►│ │Jaeger       │ │
│ │middleware   │ │    │ │Configuration │ │    │ │OTEL Collector│ │
│ └─────────────┘ │    │ └──────────────┘ │    │ └─────────────┘ │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## 📖 API Reference

### axum-otel

- `AxumOtelSpanCreator` - Creates spans for HTTP requests
- `AxumOtelOnResponse` - Handles response logging
- `AxumOtelOnFailure` - Handles error logging

### tracing-otel-extra

- `Logger` - Main configuration builder
- `LogFormat` - Log output format options
- `ProviderGuard` - RAII resource management

### tracing-opentelemetry-extra

- `init_tracer_provider` - Initialize OpenTelemetry tracer
- `init_meter_provider` - Initialize OpenTelemetry meter
- `OtelGuard` - Automatic resource cleanup

## 🤝 Contributing

We welcome contributions! Please see our contributing guidelines:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Setup

```bash
# Clone the repository
git clone https://github.com/iamnivekx/tracing-otel-extra.git
cd tracing-otel-extra

# Run tests
cargo test

# Run examples
cargo run --example otel
```

## 📄 License

This project is licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🔗 Links

- [Documentation](https://docs.rs/tracing-otel-extra/)
- [Crates.io](https://crates.io/crates/tracing-otel-extra)
- [GitHub Repository](https://github.com/iamnivekx/tracing-otel-extra)
- [OpenTelemetry](https://opentelemetry.io/)
- [Axum Framework](https://github.com/tokio-rs/axum)
- [dockotlp](https://github.com/iamnivekx/dockotlp)