use axum::{Router, routing::get};
use axum_otel::{AxumOtelOnFailure, AxumOtelOnResponse, AxumOtelSpanCreator, Level};

use tower_http::trace::TraceLayer;

#[tokio::test]
async fn test_axum_otel_middleware() {
    // Basic test to ensure the middleware compiles and runs without panicking
    // Ideally we would verify spans are exported, but that requires setting up a full in-memory tracer
    // For now, we verify the app handles requests correctly with the middleware active.

    async fn handler() -> &'static str {
        "Hello, world!"
    }

    let app = Router::new().route("/", get(handler)).layer(
        TraceLayer::new_for_http()
            .make_span_with(AxumOtelSpanCreator::new().level(Level::INFO))
            .on_response(AxumOtelOnResponse::new().level(Level::INFO))
            .on_failure(AxumOtelOnFailure::new()),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{}/", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    assert_eq!(resp.text().await.unwrap(), "Hello, world!");
}
