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
async fn missing_auth_returns_401() {
    let app = build_app_with_admin().await;
    let response = app
        .oneshot(Request::builder().uri("/api/config").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn wrong_credentials_returns_401() {
    let app = build_app_with_admin().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/config")
                .header("Authorization", basic_auth("admin", "wrongpass"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn correct_credentials_returns_200() {
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
}
