use crate::{
    dyn_span,
    extract::{context, fields},
};
use http::Request;
use tracing::{field::Empty, Level, Span};

/// Creates a new [`Span`] for the given request.
/// you can use this span to record the request and response
///
/// # Example
///
/// ```rust
/// use tracing_otel_extra::extract::span::make_request_span;
/// use tracing::Level;
/// use http::Request;
///
/// let request = http::Request::builder()
///     .method("GET")
///     .uri("https://example.com")
///     .body(())
///     .unwrap();
/// let span = make_request_span(Level::INFO, &request);
/// span.record("http.method", "GET");
/// span.record("http.route", "GET /");
/// span.record("http.status", 200);
/// span.record("http.user_agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36");
/// span.record("otel.name", "request");
/// span.record("otel.kind", "server");
/// span.record("otel.status", "ok");
/// span.record("request_id", "1234567890");
/// ```
pub fn make_request_span<B>(level: Level, request: &Request<B>) -> Span {
    let span = dyn_span!(
        level,
        "request",
        // HTTP fields
        http.version = ?fields::extract_http_version(request),
        http.host = ?fields::extract_host(request),
        http.method = ?fields::extract_http_method(request),
        http.route = Empty,
        http.scheme = ?fields::extract_http_scheme(request).map(debug),
        http.status = Empty,
        http.target = ?fields::extract_http_target(request),
        http.user_agent = ?fields::extract_user_agent(request),
        // OpenTelemetry fields
        otel.name = Empty,
        otel.kind = ?Empty,
        otel.status = Empty,
        // Request tracking
        request_id = %fields::extract_request_id(request),
        trace_id = Empty
    );
    context::set_otel_parent(request.headers(), &span);
    span
}
