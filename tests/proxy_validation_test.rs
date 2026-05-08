use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use base64::{engine::general_purpose::STANDARD, Engine};
use tower::ServiceExt;

async fn build_test_app() -> axum::Router {
    let config = llmonitor::test_helpers::test_config();
    let db = llmonitor::test_helpers::test_db().await;

    let admin_svc = llmonitor::config::admin::AdminService::new(db.clone());
    admin_svc.create("testuser", "testpass").await.unwrap();

    let tracking = llmonitor::test_helpers::test_tracking_service();
    let state = llmonitor::app_state::AppState::new(db, config.clone(), tracking);
    llmonitor::build_router(state, &config)
}

fn basic_auth(user: &str, pass: &str) -> String {
    format!("Basic {}", STANDARD.encode(format!("{}:{}", user, pass)))
}

#[tokio::test]
async fn missing_auth_on_proxy_returns_401() {
    let app = build_test_app().await;

    let body = serde_json::json!({
        "model": "claude-sonnet-4-6",
        "messages": [{"role": "user", "content": "hello"}]
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("content-type", "application/json")
                .header("x-provider", "anthropic")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn missing_provider_header_returns_400() {
    let app = build_test_app().await;

    let body = serde_json::json!({
        "model": "claude-sonnet-4-6",
        "messages": [{"role": "user", "content": "hello"}]
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("content-type", "application/json")
                .header("Authorization", basic_auth("testuser", "testpass"))
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["error"]["code"], "provider_header_missing");
}

#[tokio::test]
async fn unknown_provider_returns_400() {
    let app = build_test_app().await;

    let body = serde_json::json!({
        "model": "claude-sonnet-4-6",
        "messages": [{"role": "user", "content": "hello"}]
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/chat/completions")
                .header("content-type", "application/json")
                .header("Authorization", basic_auth("testuser", "testpass"))
                .header("x-provider", "openai")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["error"]["code"], "provider_unknown");
}
