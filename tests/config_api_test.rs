use axum::{body::Body, http::{Request, StatusCode}};
use base64::{engine::general_purpose::STANDARD, Engine};
use tower::ServiceExt;

async fn build_app_with_admin() -> axum::Router {
    let config = llmonitor::test_helpers::test_config();
    let db = llmonitor::test_helpers::test_db().await;

    let admin_svc = llmonitor::config::admin::AdminService::new(db.clone());
    admin_svc.create("admin", "password123").await.unwrap();

    let tracking = llmonitor::test_helpers::test_tracking_service();
    let state = llmonitor::app_state::AppState::new(db, config.clone(), tracking);
    llmonitor::build_router(state, &config)
}

fn basic_auth(user: &str, pass: &str) -> String {
    format!("Basic {}", STANDARD.encode(format!("{}:{}", user, pass)))
}

#[tokio::test]
async fn get_config_returns_redacted_response() {
    let app = build_app_with_admin().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/config")
                .header("Authorization", basic_auth("admin", "password123"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json.get("anthropic_api_key").is_some());
    assert!(json.get("request_timeout_secs").is_some());
}

#[tokio::test]
async fn put_config_stores_api_key() {
    let app = build_app_with_admin().await;
    let body = serde_json::json!({"anthropic_api_key": "sk-ant-test-1234567890abcdef"});
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/config")
                .header("Authorization", basic_auth("admin", "password123"))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let masked = json["anthropic_api_key"].as_str().unwrap();
    assert!(masked.contains("***...***"), "key should be masked, got: {masked}");
    assert!(!masked.contains("sk-ant-test-1234567890abcdef"), "plaintext key must not appear");
}

#[tokio::test]
async fn put_config_rejects_empty_api_key() {
    let app = build_app_with_admin().await;
    let body = serde_json::json!({"anthropic_api_key": ""});
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/config")
                .header("Authorization", basic_auth("admin", "password123"))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
