use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub enum TimeWindow {
    OneHour,
    OneDay,
    SevenDays,
    ThirtyDays,
}

impl TimeWindow {
    pub fn label(self) -> &'static str {
        match self {
            TimeWindow::OneHour => "1h",
            TimeWindow::OneDay => "24h",
            TimeWindow::SevenDays => "7d",
            TimeWindow::ThirtyDays => "30d",
        }
    }
}

impl TryFrom<&str> for TimeWindow {
    type Error = StatsError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "1h" => Ok(TimeWindow::OneHour),
            "24h" => Ok(TimeWindow::OneDay),
            "7d" => Ok(TimeWindow::SevenDays),
            "30d" => Ok(TimeWindow::ThirtyDays),
            other => Err(StatsError::InvalidWindow(other.to_string())),
        }
    }
}

#[derive(Debug, Error)]
pub enum StatsError {
    #[error("database error: {0}")]
    DatabaseError(String),
    #[error("unknown window '{0}'; valid values are 1h, 24h, 7d, 30d")]
    InvalidWindow(String),
}

impl From<sqlx::Error> for StatsError {
    fn from(e: sqlx::Error) -> Self {
        StatsError::DatabaseError(e.to_string())
    }
}

impl IntoResponse for StatsError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            StatsError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database_error"),
            StatsError::InvalidWindow(_) => (StatusCode::BAD_REQUEST, "invalid_window"),
        };
        (
            status,
            Json(serde_json::json!({
                "error": {
                    "code": code,
                    "message": self.to_string()
                }
            })),
        )
            .into_response()
    }
}

#[derive(Debug, Serialize)]
pub struct StatsSummary {
    pub window: String,
    pub total_requests: i64,
    pub total_tokens: i64,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub by_provider: Vec<ProviderSummary>,
}

#[derive(Debug, Serialize)]
pub struct ProviderSummary {
    pub provider: String,
    pub model: String,
    pub request_count: i64,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
}

#[derive(Debug, Serialize)]
pub struct UsageDataPoint {
    pub bucket: String,
    pub request_count: i64,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
}

#[derive(Debug, Serialize)]
pub struct SessionStats {
    pub session_id: String,
    pub request_count: i64,
    pub total_tokens: i64,
    pub first_seen: String,
    pub last_seen: String,
}

#[derive(Debug, Serialize)]
pub struct CacheStats {
    pub window: String,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub total_requests: i64,
    pub cache_hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_window_parses_all_valid_values() {
        assert!(matches!(TimeWindow::try_from("1h"), Ok(TimeWindow::OneHour)));
        assert!(matches!(TimeWindow::try_from("24h"), Ok(TimeWindow::OneDay)));
        assert!(matches!(TimeWindow::try_from("7d"), Ok(TimeWindow::SevenDays)));
        assert!(matches!(TimeWindow::try_from("30d"), Ok(TimeWindow::ThirtyDays)));
    }

    #[test]
    fn time_window_rejects_unknown_value() {
        let err = TimeWindow::try_from("1w").unwrap_err();
        assert!(matches!(err, StatsError::InvalidWindow(_)));
        assert!(err.to_string().contains("1w"));
    }

    #[test]
    fn time_window_labels_round_trip() {
        for (input, expected) in [("1h", "1h"), ("24h", "24h"), ("7d", "7d"), ("30d", "30d")] {
            let w = TimeWindow::try_from(input).unwrap();
            assert_eq!(w.label(), expected);
        }
    }
}
