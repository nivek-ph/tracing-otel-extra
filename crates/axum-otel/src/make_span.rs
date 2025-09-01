use crate::{get_request_id, set_otel_parent};
use axum::{
    extract::{ConnectInfo, MatchedPath},
    http,
};
use opentelemetry::trace::SpanKind;
use std::net::SocketAddr;
use tower_http::trace::MakeSpan;
use tracing::{
    field::{debug, Empty},
    Level,
};

/// An implementor of [`MakeSpan`] which creates `tracing` spans populated with information about
/// the request received by an `axum` web server.
///
/// Original implementation from [tower-http](https://github.com/tower-rs/tower-http/blob/main/tower-http/src/trace/make_span.rs).
///
/// This span creator automatically adds the following attributes to each span:
///
/// - `http.method`: The HTTP method
/// - `http.route`: The matched route
/// - `http.client_ip`: The client's IP address
/// - `http.host`: The Host header
/// - `http.user_agent`: The User-Agent header
/// - `request_id`: A unique request identifier
/// - `trace_id`: The OpenTelemetry trace ID
///
/// # Example
///
/// ```rust
/// use axum_otel::AxumOtelSpanCreator;
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

        let user_agent = request
            .headers()
            .get(http::header::USER_AGENT)
            .and_then(|header| header.to_str().ok());

        let host = request
            .headers()
            .get(http::header::HOST)
            .and_then(|header| header.to_str().ok());

        let client_ip = request
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ConnectInfo(ip)| debug(ip));

        let span_name = http_route.as_ref().map_or_else(
            || http_method.to_string(),
            |route| format!("{} {}", http_method, route),
        );
        macro_rules! make_span {
            ($level:expr) => {
                tracing::span!(
                    $level,
                    "request",
                    http.client_ip = client_ip,
                    http.versions = ?request.version(),
                    http.host = host,
                    http.method = ?request.method(),
                    http.route = http_route,
                    http.scheme = request.uri().scheme().map(debug),
                    http.status_code = Empty,
                    http.target = request.uri().path_and_query().map(|p| p.as_str()),
                    http.user_agent = user_agent,
                    otel.name = span_name,
                    otel.kind = ?SpanKind::Server,
                    otel.status_code = Empty,
                    request_id = ?get_request_id(request.headers()),
                    trace_id = Empty,
                )
            }
        }
        let span = match self.level {
            Level::ERROR => make_span!(Level::ERROR),
            Level::WARN => make_span!(Level::WARN),
            Level::INFO => make_span!(Level::INFO),
            Level::DEBUG => make_span!(Level::DEBUG),
            Level::TRACE => make_span!(Level::TRACE),
        };
        set_otel_parent(request.headers(), &span);
        span
    }
}
