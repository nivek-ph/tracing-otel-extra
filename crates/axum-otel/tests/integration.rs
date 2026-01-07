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
use tower::ServiceExt;
use tower_http::trace::TraceLayer;
use tracing::instrument;
use tracing_subscriber::{Registry, layer::SubscriberExt, util::SubscriberInitExt};

#[instrument]
async fn hello() -> &'static str {
    "Hello, world!"
}

fn app() -> Router<()> {
    Router::new().route("/", get(hello)).layer(
        TraceLayer::new_for_http()
            .make_span_with(AxumOtelSpanCreator::new().level(Level::INFO))
            .on_response(AxumOtelOnResponse::new().level(Level::INFO))
            .on_failure(AxumOtelOnFailure::new()),
    )
}

#[tokio::test]
async fn test_axum_otel_middleware() {
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
    Registry::default()
        .with(otel_layer)
        .try_init()
        .expect("Failed to initialize tracing subscriber");

    let app = app();

    // Send request using oneshot
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

    // Check http.status_code attribute
    let status_code_attr = request_span
        .attributes
        .iter()
        .find(|kv| kv.key.as_str() == "http.status_code")
        .map(|kv| kv.value.to_string());

    assert_eq!(
        status_code_attr,
        Some("200".to_string()),
        "Expected http.status_code to be 200"
    );

    provider
        .shutdown()
        .expect("Failed to shutdown tracer provider");
}
