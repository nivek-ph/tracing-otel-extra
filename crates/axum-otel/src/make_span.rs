use axum::{
    extract::{ConnectInfo, MatchedPath},
    http,
};
use opentelemetry::trace::SpanKind;
use std::net::SocketAddr;
use tower_http::trace::MakeSpan;
use tracing::{Level, field::Empty};
use tracing_otel_extra::{
    dyn_span,
    extract::{context, fields},
};

/// An implementor of [`MakeSpan`] which creates `tracing` spans populated with information about
/// the request received by an `axum` web server.
///
/// Original implementation from [tower-http](https://github.com/tower-rs/tower-http/blob/main/tower-http/src/trace/make_span.rs).
///
/// This span creator automatically adds the following attributes to each span:
///
/// - `http.request.method`: The HTTP method
/// - `http.route`: The matched route
/// - `http.response.status_code`: The response status code (recorded when the response is sent)
/// - `server.address`: The `Host` header (OpenTelemetry [`server.address`](https://opentelemetry.io/docs/specs/semconv/registry/attributes/server/))
/// - `client.address`: The client IP when [`ConnectInfo`] is available (OpenTelemetry [`client.address`](https://opentelemetry.io/docs/specs/semconv/registry/attributes/client/))
/// - `network.protocol.name`: The network protocol name
/// - `network.protocol.version`: The network protocol version
/// - `url.path`: The request path
/// - `url.query`: The request query string
/// - `url.scheme`: The request scheme
/// - `user_agent.original`: The `User-Agent` header
/// - `request_id`: A unique request identifier
/// - `trace_id`: The OpenTelemetry trace ID
///
/// # Example
///
/// ```rust
/// use axum_otel::{AxumOtelSpanCreator, Level};
/// use tower_http::trace::TraceLayer;
///
/// let layer = TraceLayer::new_for_http()
///     .make_span_with(AxumOtelSpanCreator::new().level(Level::INFO));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct AxumOtelSpanCreator {
    level: Level,
}

impl AxumOtelSpanCreator {
    /// Create a new `AxumOtelSpanCreator`.
    pub fn new() -> Self {
        Self {
            level: Level::TRACE,
        }
    }

    /// Set the [`Level`] used for [tracing events].
    ///
    /// Defaults to [`Level::TRACE`].
    ///
    /// [tracing events]: https://docs.rs/tracing/latest/tracing/#events
    pub fn level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }
}

impl Default for AxumOtelSpanCreator {
    fn default() -> Self {
        Self::new()
    }
}

impl<B> MakeSpan<B> for AxumOtelSpanCreator {
    fn make_span(&mut self, request: &http::Request<B>) -> tracing::Span {
        let http_method = request.method().as_str();
        let http_route = request
            .extensions()
            .get::<MatchedPath>()
            .map(|p| p.as_str());

        let peer = request
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ConnectInfo(addr)| *addr);

        let span_name = http_route.as_ref().map_or_else(
            || http_method.to_string(),
            |route| format!("{http_method} {route}"),
        );

        let span = dyn_span!(
            self.level,
            "request",
            client.address = Empty,
            http.request.method = %fields::extract_http_method(request),
            http.route = Empty,
            http.response.status_code = Empty,
            network.protocol.name = fields::extract_network_protocol_name(request),
            network.protocol.version = Empty,
            otel.name = span_name,
            otel.kind = ?SpanKind::Server,
            otel.status_code = Empty,
            otel.status_description = Empty,
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
        if let Some(route) = http_route {
            span.record("http.route", route);
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
        if let Some(peer) = peer {
            span.record("client.address", tracing::field::display(peer.ip()));
        }

        context::set_otel_parent(request.headers(), &span);
        span
    }
}
