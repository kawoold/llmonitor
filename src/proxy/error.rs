use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProxyError {
    #[error("X-Provider header is required")]
    ProviderHeaderMissing,

    #[error("Unknown provider: {0}")]
    ProviderUnknown(String),

    #[error("Provider {0} is not configured (missing API key)")]
    ProviderNotConfigured(String),

    #[error("Model {model} is not accepted by {provider}. Accepted models: {accepted}")]
    ModelNotAccepted {
        model: String,
        provider: String,
        accepted: String,
    },

    #[error("Failed to parse request: {0}")]
    RequestParseError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Upstream client error: {0}")]
    UpstreamClientError(String),

    #[error("Upstream authentication error")]
    UpstreamAuthError,

    #[error("Upstream rate limit exceeded")]
    UpstreamRateLimit,

    #[error("Upstream server error: {0}")]
    UpstreamServerError(String),

    #[error("Upstream request timed out")]
    UpstreamTimeout,

    #[error("Streaming is not supported; use batch mode")]
    StreamingNotSupported,

    #[error("Internal error: {0}")]
    InternalError(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiErrorBody {
    pub error: OpenAiErrorDetail,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiErrorDetail {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
}

impl IntoResponse for ProxyError {
    fn into_response(self) -> Response {
        let (status, error_type, code, message) = match &self {
            ProxyError::ProviderHeaderMissing => (
                StatusCode::BAD_REQUEST,
                "invalid_request_error",
                "provider_header_missing",
                self.to_string(),
            ),
            ProxyError::ProviderUnknown(_) => (
                StatusCode::BAD_REQUEST,
                "invalid_request_error",
                "provider_unknown",
                self.to_string(),
            ),
            ProxyError::ProviderNotConfigured(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "api_error",
                "provider_not_configured",
                self.to_string(),
            ),
            ProxyError::ModelNotAccepted { .. } => (
                StatusCode::BAD_REQUEST,
                "invalid_request_error",
                "model_not_accepted",
                self.to_string(),
            ),
            ProxyError::RequestParseError(_) => (
                StatusCode::BAD_REQUEST,
                "invalid_request_error",
                "invalid_request",
                self.to_string(),
            ),
            ProxyError::RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limit_error",
                "rate_limit_exceeded",
                self.to_string(),
            ),
            ProxyError::UpstreamClientError(_) => (
                StatusCode::BAD_REQUEST,
                "invalid_request_error",
                "upstream_client_error",
                self.to_string(),
            ),
            ProxyError::UpstreamAuthError => (
                StatusCode::UNAUTHORIZED,
                "authentication_error",
                "upstream_auth_error",
                self.to_string(),
            ),
            ProxyError::UpstreamRateLimit => (
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limit_error",
                "upstream_rate_limit",
                self.to_string(),
            ),
            ProxyError::UpstreamServerError(_) => (
                StatusCode::BAD_GATEWAY,
                "api_error",
                "upstream_server_error",
                self.to_string(),
            ),
            ProxyError::UpstreamTimeout => (
                StatusCode::GATEWAY_TIMEOUT,
                "api_error",
                "upstream_timeout",
                self.to_string(),
            ),
            ProxyError::StreamingNotSupported => (
                StatusCode::NOT_IMPLEMENTED,
                "api_error",
                "streaming_not_supported",
                self.to_string(),
            ),
            ProxyError::InternalError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "api_error",
                "internal_error",
                "An internal error occurred".to_string(),
            ),
        };

        let body = OpenAiErrorBody {
            error: OpenAiErrorDetail {
                message,
                error_type: error_type.to_string(),
                code: code.to_string(),
                param: None,
            },
        };

        (status, Json(body)).into_response()
    }
}
