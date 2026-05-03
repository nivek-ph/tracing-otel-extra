# Changelog

## [Unreleased]


## [0.31.8](https://github.com/nivek-ph/tracing-otel-extra/compare/tracing-otel-extra-v0.31.7...tracing-otel-extra-v0.31.8)




### Fixed


- *(opentelemetry)* Skip OTLP exporters when no endpoint is configured ([#21](https://github.com/nivek-ph/tracing-otel-extra/pull/21)) - ([4e614fe](https://github.com/nivek-ph/tracing-otel-extra/commit/4e614fe45568da3682ce2e4db4eb32a7317b3dd8))


## [0.31.7](https://github.com/nivek-ph/tracing-otel-extra/compare/tracing-otel-extra-v0.31.6...tracing-otel-extra-v0.31.7)




### Fixed

- Align HTTP spans with latest OpenTelemetry semantics ([#19](https://github.com/nivek-ph/tracing-otel-extra/pull/19)) - ([c5acf3f](https://github.com/nivek-ph/tracing-otel-extra/commit/c5acf3f3c0ad4584431733264acef245506a51bc))


### ⚠️ Breaking Changes

- [`make_request_span`](https://docs.rs/tracing-otel-extra/latest/tracing_otel_extra/http/span/fn.make_request_span.html) emits the same OpenTelemetry-aligned HTTP attribute names as `axum-otel` (for example `server.address` and `user_agent.original` instead of `http.host` and `http.user_agent`). Update any code or tests that depend on the previous field names.

### 📚 Documentation

- Document HTTP span attributes in `http::span` and link to the OpenTelemetry HTTP spec and `axum-otel` migration notes; README mentions `make_request_span`.

### 🐛 Bug Fixes

- Allow logger initialization without an OTLP endpoint, or with empty endpoint env vars, by using local OpenTelemetry providers with no exporters.

- Change default environment variable prefix from `LOG_` to `LOG` to avoid double underscores in variable names (e.g., `LOG__SERVICE_NAME` → `LOG_SERVICE_NAME`)

## [0.31.5](https://github.com/nivek-ph/tracing-otel-extra/compare/tracing-otel-extra-v0.31.4...tracing-otel-extra-v0.31.5)


### 🚜 Refactor

- Reorganize imports and simplify shutdown logic in tracing modules - ([0f95108](https://github.com/nivek-ph/tracing-otel-extra/commit/0f951082ae571380fef1c626855271d1ab74794a))

- Reorder imports and enhance test safety in logger.rs - ([171c58d](https://github.com/nivek-ph/tracing-otel-extra/commit/171c58d07ba727b60cbead34d92804871cc52c2f))


### 📚 Documentation

- Add comprehensive AGENTS.md for tracing-otel-extra project - ([630e018](https://github.com/nivek-ph/tracing-otel-extra/commit/630e0182880560076c271134920c7f2529461947))


### ⚙️ Miscellaneous Tasks

- Update workspace dependencies and enhance CI configuration - ([244742d](https://github.com/nivek-ph/tracing-otel-extra/commit/244742d220816d3750abfd67175be04bacd057da))

- Update dependencies and refactor configuration handling - ([576ba88](https://github.com/nivek-ph/tracing-otel-extra/commit/576ba887424fc684aaea33a92cfc60debe36a521))

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). 
