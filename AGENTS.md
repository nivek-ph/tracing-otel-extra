# tracing-otel-extra

This document provides context and guidelines for AI agents working with the `tracing-otel-extra` codebase.

## Project Overview

**tracing-otel-extra** is a comprehensive Rust library for tracing, logging, and OpenTelemetry integration. It focuses on providing production-ready observability solutions for Axum web applications with minimal boilerplate.

### Key Goals

- Simplify OpenTelemetry setup for Rust applications
- Provide opinionated, production-oriented defaults
- Integrate tracing, metrics, and logging cohesively
- Support the Axum web framework with first-class middleware

## Repository Structure

```
tracing-otel-extra/
├── crates/
│   ├── axum-otel/           # Axum middleware for HTTP tracing
│   ├── tracing-otel/        # Core logging and tracing utilities
│   └── tracing-opentelemetry/ # OpenTelemetry integration layer
├── examples/
│   ├── otel/                # Basic OpenTelemetry example
│   └── microservices/       # Multi-service distributed tracing demo
├── Cargo.toml               # Workspace configuration
└── docker-compose.yml       # Development infrastructure
```

### Crate Dependencies

```
axum-otel
    └── tracing-otel-extra (tracing-otel)
            └── tracing-opentelemetry-extra (tracing-opentelemetry)
```

## Coding Conventions

### Rust Edition & Toolchain

- **Edition**: Rust 2024 (`edition = "2024"`)
- **Minimum Rust Version**: 1.92.0
- **Resolver**: Cargo resolver v2

### Code Style

1. **Lints**: The codebase uses strict linting (see `axum-otel/src/lib.rs` for reference):
   ```rust
   #![deny(unsafe_code)]
   #![warn(
       missing_docs,
       missing_debug_implementations,
       missing_copy_implementations,
       trivial_casts,
       trivial_numeric_casts,
       unused_import_braces,
       unused_qualifications
   )]
   ```

2. **Documentation**: All public APIs must have doc comments with examples where appropriate.

3. **Error Handling**: Use `anyhow::Result` for application-level errors. Library code should define specific error types when appropriate.

4. **Builder Pattern**: Configuration structs use the builder pattern with `with_*` methods:
   ```rust
   Logger::new("my-service")
       .with_format(LogFormat::Json)
       .with_level(Level::DEBUG)
       .init()
   ```

5. **Imports**: Prefer explicit imports over glob imports. Group imports by:
   - Standard library
   - External crates
   - Internal modules

### Feature Flags

The `tracing-otel-extra` crate uses feature flags extensively:

| Feature   | Description                      |
| --------- | -------------------------------- |
| `otel`    | OpenTelemetry integration        |
| `logger`  | Basic logging functionality      |
| `env`     | Environment-based configuration  |
| `context` | Trace context utilities          |
| `fields`  | Common tracing fields/attributes |
| `http`    | HTTP request/response tracing    |
| `span`    | Span creation utilities          |
| `macros`  | Tracing macros                   |

When adding new functionality, consider whether it should be gated behind a feature flag.

## Key Patterns

### 1. Resource Management

OpenTelemetry providers are managed via guard patterns that clean up on drop:

```rust
let _guard = Logger::new("service").init()?;
// Providers are automatically flushed and shut down when _guard is dropped
```

### 2. Tower Integration

The `axum-otel` crate integrates with `tower-http::TraceLayer`:

```rust
TraceLayer::new_for_http()
    .make_span_with(AxumOtelSpanCreator::new().level(Level::INFO))
    .on_response(AxumOtelOnResponse::new())
    .on_failure(AxumOtelOnFailure::new())
```

### 3. OpenTelemetry Context Propagation

The `set_otel_parent` function extracts trace context from HTTP headers:

```rust
pub fn set_otel_parent(headers: &http::HeaderMap, span: &tracing::Span) {
    let remote_context = extract_context_from_headers(headers);
    span.set_parent(remote_context);
    // Record trace_id for logging
}
```

### 4. Dynamic Span Creation

Use the `dyn_span!` macro for runtime-configurable log levels:

```rust
let span = dyn_span!(
    self.level,
    "request",
    http.method = ?method,
    http.route = route,
    trace_id = Empty
);
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p tracing-otel-extra

# Run tests with specific features
cargo test -p tracing-otel-extra --features "context,http"
```

### Test Requirements

- Tests requiring OpenTelemetry exporters need a collector running:
  ```bash
  docker run -d -p 4317:4317 otel/opentelemetry-collector
  ```

- Integration tests may use `#[tokio::test]` for async context

### Test Patterns

```rust
#[cfg(test)]
#[cfg(feature = "context")]
mod tests {
    use super::*;

    fn init_tracing() {
        // Setup test tracing subscriber
    }

    #[tokio::test]
    async fn test_feature() {
        init_tracing();
        // Test implementation
    }
}
```

## Development Workflow

### Adding New Features

1. Determine if the feature needs a new feature flag
2. Add appropriate workspace dependencies to root `Cargo.toml`
3. Implement with full documentation
4. Add tests with appropriate feature gates
5. Update the crate's `CHANGELOG.md`

### Common Tasks

#### Adding a new HTTP header extraction

1. Add the field constant to `crates/tracing-otel/src/trace/fields.rs`
2. Implement extraction function following existing patterns
3. Use in span creation via `AxumOtelSpanCreator`

#### Adding new log format support

1. Extend `LogFormat` enum in `crates/tracing-otel/src/logs/logger.rs`
2. Implement the format layer in `init_subscriber_*` functions
3. Add environment variable mapping if using `env` feature

#### Modifying span attributes

1. Update `AxumOtelSpanCreator::make_span` in `crates/axum-otel/src/make_span.rs`
2. Ensure compliance with [OpenTelemetry HTTP traces](https://opentelemetry.io/docs/specs/semconv/http/http-spans/) (see `axum_otel` crate docs on docs.rs for the attribute migration table and `CHANGELOG.md` for breaking renames)
3. Keep `tracing-otel-extra` `make_request_span` (`crates/tracing-otel/src/http/span.rs`) in sync when changing shared HTTP field names
4. Update documentation with new attributes

## Dependencies

### Key External Dependencies

| Crate                          | Purpose                         |
| ------------------------------ | ------------------------------- |
| `opentelemetry` (0.31)         | Core OpenTelemetry APIs         |
| `opentelemetry-otlp` (0.31)    | OTLP exporter                   |
| `tracing` (0.1)                | Rust tracing framework          |
| `tracing-subscriber` (0.3)     | Subscriber implementations      |
| `tracing-opentelemetry` (0.32) | Bridge between tracing and OTel |
| `axum` (0.8)                   | Web framework                   |
| `tower-http` (0.6)             | HTTP middleware utilities       |

### Version Compatibility Notes

- `reqwest-tracing` doesn't yet support opentelemetry 0.31; using `opentelemetry_0_30` feature for backward compatibility
- Always check compatibility when upgrading OpenTelemetry crates as they often have breaking changes

## Environment Variables

### Standard OpenTelemetry Variables

```bash
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
OTEL_EXPORTER_OTLP_PROTOCOL=grpc
OTEL_RESOURCE_ATTRIBUTES='service.name=my-service,service.version=1.0.0'
RUST_LOG=debug
```

### Logger Configuration (with `env` feature)

```bash
LOG_SERVICE_NAME=my-service
LOG_FORMAT=json              # compact, pretty, json
LOG_LEVEL=info
LOG_SAMPLE_RATIO=0.1
LOG_ANSI=false
LOG_CONSOLE_ENABLED=true
LOG_FILE_ENABLE=true
LOG_FILE_DIR=/var/log
LOG_FILE_PREFIX=myapp
LOG_FILE_ROTATION=daily      # daily, hourly, minutely, never
```

## Architecture Decisions

### Why builder pattern over struct initialization?

The builder pattern allows:

- Default values without requiring all fields
- Method chaining for clean configuration
- Future extensibility without breaking changes
- Compile-time validation of required fields

## Release Process

This project uses `release-plz` for automated releases. See `release-plz.toml` for configuration.

### Versioning

- All workspace crates share the same version
- Follow SemVer strictly
- Document all changes in per-crate `CHANGELOG.md` files

## Resources

- [OpenTelemetry Rust](https://github.com/open-telemetry/opentelemetry-rust)
- [Tracing Crate](https://docs.rs/tracing)
- [Axum Framework](https://github.com/tokio-rs/axum)
- [Tower HTTP](https://github.com/tower-rs/tower-http)
- [OpenTelemetry Semantic Conventions](https://opentelemetry.io/docs/concepts/semantic-conventions/)

