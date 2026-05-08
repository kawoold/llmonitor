use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    response::Json,
};

use crate::app_state::AppState;

use super::types::{CacheStats, SessionStats, StatsError, StatsSummary, TimeWindow, UsageDataPoint};

fn resolve_window(params: &HashMap<String, String>) -> Result<TimeWindow, StatsError> {
    match params.get("window") {
        Some(w) => TimeWindow::try_from(w.as_str()),
        None => Ok(TimeWindow::OneDay),
    }
}

pub async fn get_summary(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<StatsSummary>, StatsError> {
    let window = resolve_window(&params)?;
    Ok(Json(state.stats.summary(window).await?))
}

pub async fn get_usage(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<UsageDataPoint>>, StatsError> {
    let window = resolve_window(&params)?;
    Ok(Json(state.stats.usage_timeseries(window).await?))
}

pub async fn get_sessions(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<SessionStats>>, StatsError> {
    let window = resolve_window(&params)?;
    Ok(Json(state.stats.session_stats(window).await?))
}

pub async fn get_cache(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<CacheStats>, StatsError> {
    let window = resolve_window(&params)?;
    Ok(Json(state.stats.cache_stats(window).await?))
}
