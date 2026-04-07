use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode},
    routing::get,
};
use axum_otel::{AxumOtelOnFailure, AxumOtelOnResponse, AxumOtelSpanCreator, Level};
use http_body_util::BodyExt;
use opentelemetry::{global, trace::TracerProvider};
use opentelemetry_sdk::{
    Resource,
    trace::{InMemorySpanExporter, RandomIdGenerator, Sampler, SdkTracerProvider},
};
use std::sync::{Mutex, OnceLock};
use tower::ServiceExt;
use tower_http::trace::TraceLayer;
use tracing::instrument;
use tracing_subscriber::{Registry, layer::SubscriberExt};

fn test_lock() -> &'static Mutex<()> {
    static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    TEST_LOCK.get_or_init(|| Mutex::new(()))
}

#[instrument]
async fn hello() -> &'static str {
    "Hello, world!"
}

fn span_attr(span: &opentelemetry_sdk::trace::SpanData, key: &str) -> Option<String> {
    span.attributes
        .iter()
        .find(|kv| kv.key.as_str() == key)
        .map(|kv| {
            let s = kv.value.to_string();
            s.strip_prefix('"')
                .and_then(|unquoted| unquoted.strip_suffix('"'))
                .unwrap_or(&s)
                .to_string()
        })
}

fn app() -> Router<()> {
    Router::new().route("/", get(hello)).layer(
        TraceLayer::new_for_http()
            .make_span_with(AxumOtelSpanCreator::new().level(Level::INFO))
            .on_response(AxumOtelOnResponse::new().level(Level::INFO))
            .on_failure(AxumOtelOnFailure::new()),
    )
}

#[tokio::test(flavor = "current_thread")]
async fn test_axum_otel_middleware() {
    let _test_guard = test_lock().lock().expect("test lock poisoned");

    // Set up in-memory exporter for testing
    let exporter = InMemorySpanExporter::default();
    let provider: SdkTracerProvider = SdkTracerProvider::builder()
        .with_sampler(Sampler::AlwaysOn)
        .with_id_generator(RandomIdGenerator::default())
        .with_simple_exporter(exporter.clone())
        .with_resource(Resource::builder().build())
        .build();

    global::set_tracer_provider(provider.clone());

    // Set up tracing subscriber with OpenTelemetry layer
    let tracer = provider.tracer("axum-otel-test".to_string());
    let otel_layer = tracing_opentelemetry::OpenTelemetryLayer::new(tracer);
    let subscriber = Registry::default().with(otel_layer);
    let _guard = tracing::subscriber::set_default(subscriber);

    let app = app();

    // Send request using oneshot
    let response = app
        .oneshot(
            Request::builder()
                .uri("/?foo=bar")
                .header("host", "example.com")
                .header("user-agent", "integration-test")
                .header("x-forwarded-proto", "https")
                .method(Method::GET)
                .body(Body::empty())
                .expect("Failed to build request"),
        )
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);

    let body = response
        .into_body()
        .collect()
        .await
        .expect("Failed to read body");

    assert_eq!(body.to_bytes(), "Hello, world!".as_bytes());

    // Force flush to ensure spans are exported
    let _ = provider.force_flush();

    // Verify spans were created
    let spans = exporter
        .get_finished_spans()
        .expect("Failed to get finished spans");

    assert!(
        !spans.is_empty(),
        "Expected at least one span to be created"
    );

    // With oneshot(), there's no MatchedPath, so span name is just "GET"
    // When using a real server, it would be "GET /"
    let request_span = spans
        .iter()
        .find(|s| s.name == "GET" || s.name == "GET /")
        .expect("Request span not found");

    let hello_span = spans
        .iter()
        .find(|s| s.name == "hello")
        .expect("Handler span not found");

    assert_eq!(
        hello_span.parent_span_id,
        request_span.span_context.span_id(),
        "Handler span should be a child of the request span"
    );

    assert_eq!(
        span_attr(request_span, "server.address"),
        Some("example.com".to_string()),
        "Expected server.address to be example.com"
    );
    assert_eq!(
        span_attr(request_span, "http.request.method"),
        Some("GET".to_string()),
        "Expected http.request.method to be GET"
    );
    assert_eq!(
        span_attr(request_span, "user_agent.original"),
        Some("integration-test".to_string()),
        "Expected user_agent.original to be integration-test"
    );
    assert_eq!(
        span_attr(request_span, "http.response.status_code"),
        Some("200".to_string()),
        "Expected http.response.status_code to be 200"
    );
    assert_eq!(
        span_attr(request_span, "url.path"),
        Some("/".to_string()),
        "Expected url.path to be /"
    );
    assert_eq!(
        span_attr(request_span, "url.scheme"),
        Some("https".to_string()),
        "Expected url.scheme to be https"
    );
    assert_eq!(
        span_attr(request_span, "url.query"),
        Some("foo=bar".to_string()),
        "Expected url.query to be foo=bar"
    );
    assert_eq!(
        span_attr(request_span, "network.protocol.name"),
        Some("http".to_string()),
        "Expected network.protocol.name to be http"
    );
    assert_eq!(
        span_attr(request_span, "network.protocol.version"),
        Some("1.1".to_string()),
        "Expected network.protocol.version to be 1.1"
    );
    assert_eq!(
        span_attr(request_span, "http.method"),
        None,
        "Expected deprecated http.method to be absent"
    );
    assert_eq!(
        span_attr(request_span, "http.status_code"),
        None,
        "Expected deprecated http.status_code to be absent"
    );
    assert_eq!(
        span_attr(request_span, "http.target"),
        None,
        "Expected deprecated http.target to be absent"
    );
    assert_eq!(
        span_attr(request_span, "http.host"),
        None,
        "Expected deprecated http.host to be absent"
    );
    assert_eq!(
        span_attr(request_span, "http.user_agent"),
        None,
        "Expected deprecated http.user_agent to be absent"
    );

    provider
        .shutdown()
        .expect("Failed to shutdown tracer provider");
}

#[tokio::test(flavor = "current_thread")]
async fn test_axum_otel_omits_missing_optional_fields() {
    let _test_guard = test_lock().lock().expect("test lock poisoned");

    let exporter = InMemorySpanExporter::default();
    let provider: SdkTracerProvider = SdkTracerProvider::builder()
        .with_sampler(Sampler::AlwaysOn)
        .with_id_generator(RandomIdGenerator::default())
        .with_simple_exporter(exporter.clone())
        .with_resource(Resource::builder().build())
        .build();

    global::set_tracer_provider(provider.clone());

    let tracer = provider.tracer("axum-otel-test-optional".to_string());
    let otel_layer = tracing_opentelemetry::OpenTelemetryLayer::new(tracer);
    let subscriber = Registry::default().with(otel_layer);
    let _guard = tracing::subscriber::set_default(subscriber);

    let app = app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .method(Method::GET)
                .body(Body::empty())
                .expect("Failed to build request"),
        )
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);

    let _body = response
        .into_body()
        .collect()
        .await
        .expect("Failed to read body");

    let _ = provider.force_flush();

    let spans = exporter
        .get_finished_spans()
        .expect("Failed to get finished spans");

    let request_span = spans
        .iter()
        .find(|s| s.name == "GET" || s.name == "GET /")
        .expect("Request span not found");

    assert_eq!(
        span_attr(request_span, "server.address"),
        None,
        "Expected server.address to be omitted when missing"
    );
    assert_eq!(
        span_attr(request_span, "user_agent.original"),
        None,
        "Expected user_agent.original to be omitted when missing"
    );
    assert_eq!(
        span_attr(request_span, "url.query"),
        None,
        "Expected url.query to be omitted when missing"
    );
    assert_eq!(
        span_attr(request_span, "url.scheme"),
        None,
        "Expected url.scheme to be omitted when missing"
    );

    provider
        .shutdown()
        .expect("Failed to shutdown tracer provider");
}
