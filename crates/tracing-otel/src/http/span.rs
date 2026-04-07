//! HTTP request span creation utilities.
//!
//! [`make_request_span`] sets attributes that match [OpenTelemetry HTTP server spans](https://opentelemetry.io/docs/specs/semconv/http/http-spans/)
//! (for example `server.address`, `user_agent.original`, `url.path`, `url.scheme`, `network.protocol.*`).
//! See the [`axum_otel` crate](https://docs.rs/axum-otel) documentation for a migration table from older attribute names.

use crate::{
    dyn_span,
    http::{context, fields},
};
use http::Request;
use tracing::{Level, Span, field::Empty};

/// Creates a new [`Span`] for the given request.
pub fn make_request_span<B>(level: Level, request: &Request<B>) -> Span {
    let span = dyn_span!(
        level,
        "request",
        // HTTP fields
        http.request.method = %fields::extract_http_method(request),
        http.route = Empty,
        http.response.status_code = Empty,
        network.protocol.name = fields::extract_network_protocol_name(request),
        network.protocol.version = Empty,
        // OpenTelemetry fields
        otel.name = Empty,
        otel.kind = ?Empty,
        otel.status_code = Empty,
        otel.status_description = Empty,
        // Request tracking
        request_id = Empty,
        server.address = Empty,
        trace_id = Empty,
        url.path = fields::extract_url_path(request),
        url.query = Empty,
        url.scheme = Empty,
        user_agent.original = Empty
    );

    if let Some(host) = fields::extract_host(request) {
        span.record("server.address", host);
    }
    if let Some(user_agent) = fields::extract_user_agent(request) {
        span.record("user_agent.original", user_agent);
    }
    if let Some(version) = fields::extract_network_protocol_version(request) {
        span.record("network.protocol.version", version);
    }
    if let Some(request_id) = fields::extract_request_id(request) {
        span.record("request_id", request_id);
    }
    if let Some(query) = fields::extract_url_query(request) {
        span.record("url.query", query);
    }
    if let Some(scheme) = fields::extract_url_scheme(request) {
        span.record("url.scheme", scheme);
    }

    context::set_otel_parent(request.headers(), &span);
    span
}
