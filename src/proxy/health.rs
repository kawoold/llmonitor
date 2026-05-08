use axum::{extract::State, response::Json};
use serde::Serialize;

use crate::app_state::AppState;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub uptime_secs: u64,
    pub db: String,
}

pub async fn health_handler(State(state): State<AppState>) -> Json<HealthResponse> {
    let db_status = match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => "ok",
        Err(_) => "error",
    };

    let uptime = state.started_at.elapsed().as_secs();

    Json(HealthResponse {
        status: if db_status == "ok" { "ok" } else { "degraded" }.to_string(),
        uptime_secs: uptime,
        db: db_status.to_string(),
    })
}
