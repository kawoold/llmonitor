use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

async fn build_test_app() -> axum::Router {
    let config = llmonitor::test_helpers::test_config();
    let db = llmonitor::test_helpers::test_db().await;
    let tracking = llmonitor::test_helpers::test_tracking_service();
    let state = llmonitor::app_state::AppState::new(db, config.clone(), tracking);
    llmonitor::build_router(state, &config)
}

#[tokio::test]
async fn health_returns_ok() {
    let app = build_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}
