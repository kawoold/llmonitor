use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

use llmonitor::{
    app_state::AppState,
    build_router,
    config::Config,
    db,
    tracking::{TrackingService, TrackingWriterService, CHANNEL_CAPACITY},
};

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if present — silently ignored if absent
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "llmonitor=info,tower_http=info".into()),
        )
        .json()
        .init();

    let config = Config::from_env();

    info!(listen_addr = %config.listen_addr, "Starting llmonitor");

    // Strip sqlite:// prefix if caller included it, so we always hold a plain path
    let db_path = config
        .database_url
        .strip_prefix("sqlite://")
        .unwrap_or(&config.database_url)
        .to_string();

    // SQLite won't create parent directories — do it before connecting
    if let Some(parent) = std::path::Path::new(&db_path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let db_url = format!("sqlite://{}?mode=rwc", db_path);
    let db = db::create_pool(&db_url).await?;

    sqlx::migrate!("./migrations").run(&db).await?;

    llmonitor::config::bootstrap::bootstrap(&db).await?;

    // Set up tracking channel and background writer
    let (tx, rx) = tokio::sync::mpsc::channel(CHANNEL_CAPACITY);
    let tracking_service = TrackingService::new(Arc::new(tx));
    let writer = TrackingWriterService::new(rx, db.clone());
    let writer_handle = writer.spawn();

    let state = AppState::new(db, config.clone(), tracking_service);
    let router = build_router(state, &config);

    let listener = tokio::net::TcpListener::bind(&config.listen_addr).await?;
    info!("Listening on {}", config.listen_addr);

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // Wait for writer to drain remaining records
    let _ = writer_handle.await;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received, draining connections...");
    tokio::time::sleep(Duration::from_secs(1)).await;
}
