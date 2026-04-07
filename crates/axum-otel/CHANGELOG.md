# Changelog

## [Unreleased]


## [0.31.7](https://github.com/iamnivekx/tracing-otel-extra/compare/axum-otel-v0.31.6...axum-otel-v0.31.7)




### Fixed

- Align HTTP spans with latest OpenTelemetry semantics ([#19](https://github.com/iamnivekx/tracing-otel-extra/pull/19)) - ([c5acf3f](https://github.com/iamnivekx/tracing-otel-extra/commit/c5acf3f3c0ad4584431733264acef245506a51bc))


### ⚠️ Breaking Changes

- HTTP span attributes were renamed to align with [OpenTelemetry HTTP semantic conventions](https://opentelemetry.io/docs/specs/semconv/http/http-spans/): `http.host` → `server.address`, `http.user_agent` → `user_agent.original`, `http.client_ip` → `client.address` (when `[ConnectInfo](https://docs.rs/axum/latest/axum/extract/struct.ConnectInfo.html)` is present). Update dashboards, alerts, and sampling rules that referenced the old attribute keys.
- `http.response.status_code` is recorded as an integer (OpenTelemetry `int`), not a string.

### 📚 Documentation

- Expand crate-level documentation with attribute migration and links to the OpenTelemetry HTTP spec.

## [0.31.5](https://github.com/iamnivekx/tracing-otel-extra/compare/axum-otel-v0.31.4...axum-otel-v0.31.5)

### 🚜 Refactor

- Reorganize imports and simplify shutdown logic in tracing modules - ([0f95108](https://github.com/iamnivekx/tracing-otel-extra/commit/0f951082ae571380fef1c626855271d1ab74794a))

### ⚙️ Miscellaneous Tasks

- Update workspace dependencies and enhance CI configuration - ([244742d](https://github.com/iamnivekx/tracing-otel-extra/commit/244742d220816d3750abfd67175be04bacd057da))

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). 