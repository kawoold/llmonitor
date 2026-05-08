pub mod app_state;
pub mod auth;
pub mod config;
pub mod db;
pub mod metrics;
pub mod middleware;
pub mod providers;
pub mod proxy;
pub mod stats;
pub mod tracking;

use axum::{middleware as axum_middleware, routing::get, Router};
use tower::ServiceBuilder;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};

use app_state::AppState;
use auth::basic_auth_middleware;
use config::{
    handlers::{get_config, update_config, update_credentials},
    Config,
};
use middleware::{rate_limit::RateLimitLayer, security_headers::security_headers_middleware};
use proxy::{
    frontend::{serve_asset, serve_index},
    handlers::chat_completion,
    health::health_handler,
    models::get_models,
    passthrough::messages_passthrough,
};
use stats::handlers::{get_cache, get_sessions, get_summary, get_usage};

pub fn build_router(state: AppState, config: &Config) -> Router {
    let rate_limit = RateLimitLayer::new(
        config.rate_limit_capacity,
        config.rate_limit_refill_per_sec,
    );

    let proxy_routes = Router::new()
        .route("/chat/completions", axum::routing::post(chat_completion))
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            basic_auth_middleware,
        ))
        .layer(rate_limit);

    let management_routes = Router::new()
        .route("/config", get(get_config).put(update_config))
        .route("/config/credentials", axum::routing::put(update_credentials))
        .route("/models", get(get_models))
        .route("/stats/summary", get(get_summary))
        .route("/stats/usage", get(get_usage))
        .route("/stats/sessions", get(get_sessions))
        .route("/stats/cache", get(get_cache))
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            basic_auth_middleware,
        ));

    let metrics_route = Router::new().route(
        "/metrics",
        get({
            let state = state.clone();
            move || {
                let rendered = state.metrics.render();
                async move { rendered }
            }
        }),
    );

    Router::new()
        .nest("/v1", proxy_routes)
        .route("/v1/messages", axum::routing::post(messages_passthrough))
        .nest("/api", management_routes)
        .merge(metrics_route)
        .route("/health", get(health_handler))
        .route("/assets/*path", get(serve_asset))
        .fallback(serve_index)
        .layer(axum_middleware::from_fn(security_headers_middleware))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
                .layer(PropagateRequestIdLayer::x_request_id()),
        )
        .with_state(state)
}

pub mod test_helpers {
    use std::sync::Arc;
    use sqlx::SqlitePool;

    use crate::config::Config;
    use crate::tracking::{TrackingService, UsageRecord, CHANNEL_CAPACITY};

    pub fn test_config() -> Config {
        Config::default()
    }

    pub async fn test_db() -> SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("in-memory SQLite failed");

        sqlx::query("PRAGMA journal_mode=WAL").execute(&pool).await.unwrap();
        sqlx::query("PRAGMA foreign_keys=ON").execute(&pool).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        pool
    }

    /// Creates a TrackingService backed by a drain-only channel for use in tests.
    pub fn test_tracking_service() -> TrackingService {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<UsageRecord>(CHANNEL_CAPACITY);
        tokio::spawn(async move { while rx.recv().await.is_some() {} });
        TrackingService::new(Arc::new(tx))
    }
}
