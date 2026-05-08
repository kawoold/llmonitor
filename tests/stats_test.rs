use axum::{
    body::Body,
    http::{Request, StatusCode},
};
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

fn auth_header() -> String {
    format!("Basic {}", STANDARD.encode("admin:password123"))
}

#[tokio::test]
async fn stats_summary_requires_auth() {
    let app = build_app_with_admin().await;
    let response = app
        .oneshot(Request::builder().uri("/api/stats/summary").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn stats_summary_returns_empty_on_fresh_db() {
    let app = build_app_with_admin().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/stats/summary")
                .header("Authorization", auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["total_requests"], 0);
    assert!(json["by_provider"].as_array().unwrap().is_empty());
    assert_eq!(json["window"], "24h");
}

#[tokio::test]
async fn stats_summary_invalid_window_returns_400() {
    let app = build_app_with_admin().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/stats/summary?window=1w")
                .header("Authorization", auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["error"]["code"], "invalid_window");
}

#[tokio::test]
async fn stats_usage_defaults_to_24h_window() {
    let app = build_app_with_admin().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/stats/usage")
                .header("Authorization", auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn stats_sessions_returns_ok() {
    let app = build_app_with_admin().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/stats/sessions?window=7d")
                .header("Authorization", auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn stats_cache_returns_zero_hit_rate_on_empty_db() {
    let app = build_app_with_admin().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/stats/cache?window=30d")
                .header("Authorization", auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["cache_hit_rate"], 0.0);
    assert_eq!(json["window"], "30d");
}
