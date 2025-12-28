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

This crate (`tracing-otel-extra` / `axum-otel`) provides a more comprehensive solution compared to `axum-tracing-opentelemetry`. Here are the key differences:

| Feature | `axum-otel` | `axum-tracing-opentelemetry` |
|---------|-------------|------------------------------|
| **Metrics Collection** | ✅ Built-in metrics collection and OTLP export | ❌ Tracing only |
| **Log Formats** | ✅ Multiple formats (Compact, Pretty, JSON) | ❌ Limited |
| **File Logging** | ✅ Built-in file appender with rotation | ❌ Not available |
| **Configuration** | ✅ Builder pattern + environment variables | ⚠️ Manual setup required |
| **Resource Management** | ✅ RAII automatic cleanup | ⚠️ Manual cleanup needed |
| **HTTP Attributes** | ✅ Comprehensive (method, route, client_ip, host, user_agent, request_id, trace_id) | ⚠️ Basic attributes |
| **OpenTelemetry Context** | ✅ Automatic parent context propagation | ⚠️ Manual setup |
| **Microservices Support** | ✅ Complete observability stack examples | ⚠️ Basic integration |
| **Sampling Configuration** | ✅ Easy sampling ratio configuration | ⚠️ Manual configuration |

### Why Choose `axum-otel`?

1. **Out-of-the-box Metrics**: Built-in metrics collection and export capabilities without additional setup
2. **Production Ready**: File logging with rotation, environment-based configuration, and automatic resource management
3. **Better Developer Experience**: Simple builder pattern API that reduces boilerplate code
4. **Complete Observability**: Full-stack solution with examples for microservices architectures
5. **Rich HTTP Instrumentation**: Automatically captures comprehensive HTTP request/response attributes

If you need a simple tracing-only solution, `axum-tracing-opentelemetry` might suffice. However, for production applications requiring metrics, structured logging, and comprehensive observability, `axum-otel` provides significantly more value out of the box.

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
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Server listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
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