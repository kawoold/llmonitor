use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
};

use crate::app_state::AppState;
use crate::config::domain::{ConfigError, UpdateCredentialsRequest, UpdateSettingsRequest};

pub async fn get_config(State(state): State<AppState>) -> Response {
    match state.config_service.get_redacted().await {
        Ok(cfg) => Json(cfg).into_response(),
        Err(e) => internal_error(e),
    }
}

pub async fn update_config(
    State(state): State<AppState>,
    Json(req): Json<UpdateSettingsRequest>,
) -> Response {
    match state.config_service.update_settings(req).await {
        Ok(cfg) => Json(cfg).into_response(),
        Err(ConfigError::EmptyApiKey) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({
                "error": {
                    "message": "anthropic_api_key cannot be empty",
                    "type": "invalid_request_error",
                    "code": "empty_api_key"
                }
            })),
        )
            .into_response(),
        Err(e) => internal_error(e),
    }
}

pub async fn update_credentials(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<UpdateCredentialsRequest>,
) -> Response {
    let username = match extract_basic_auth_username(&headers) {
        Some(u) => u,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                [(header::WWW_AUTHENTICATE, "Basic realm=\"llmonitor\"")],
            )
                .into_response()
        }
    };

    match state.admin_service.update_password(&username, &req.new_password).await {
        Ok(()) => Json(serde_json::json!({"message": "Password updated successfully"})).into_response(),
        Err(crate::config::domain::AdminError::PasswordTooShort) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({
                "error": {
                    "message": "Password must be at least 8 characters",
                    "type": "invalid_request_error",
                    "code": "password_too_short"
                }
            })),
        )
            .into_response(),
        Err(e) => internal_error(e),
    }
}

fn extract_basic_auth_username(headers: &HeaderMap) -> Option<String> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let encoded = headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Basic ")?;
    let decoded = String::from_utf8(STANDARD.decode(encoded).ok()?).ok()?;
    decoded.splitn(2, ':').next().map(String::from)
}

fn internal_error(e: impl std::fmt::Display) -> Response {
    tracing::error!(error = %e, "Internal config error");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
            "error": {
                "message": "An internal error occurred",
                "type": "api_error",
                "code": "internal_error"
            }
        })),
    )
        .into_response()
}
