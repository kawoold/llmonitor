pub mod claude;

use rand_core::{OsRng, RngCore};
use tracing::{error, warn};

use crate::config::types::ApiKey;
use crate::proxy::error::ProxyError;
use crate::proxy::types::{OpenAiChatRequest, OpenAiChatResponse, RawUsage};
use axum::body::Body;
use axum::response::Response;
use claude::ClaudeAdapter;

#[derive(Debug, Clone)]
pub enum Provider {
    Claude(ClaudeAdapter),
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("Not implemented")]
    NotImplemented,
    #[error("Authentication failed")]
    AuthFailed,
    #[error("Rate limited by upstream")]
    RateLimited,
    #[error("Upstream server error: {0}")]
    ServerError(String),
    #[error("Request timed out")]
    Timeout,
    #[error("Client error: {0}")]
    ClientError(String),
}

impl Provider {
    pub fn from_name(name: &str, api_key: ApiKey) -> Result<Self, ProxyError> {
        match name.to_lowercase().as_str() {
            "claude" | "anthropic" => {
                if api_key.is_empty() {
                    return Err(ProxyError::ProviderNotConfigured(name.to_string()));
                }
                Ok(Provider::Claude(ClaudeAdapter::new(api_key)))
            }
            _ => Err(ProxyError::ProviderUnknown(name.to_string())),
        }
    }

    pub fn is_known_name(name: &str) -> bool {
        matches!(name.to_lowercase().as_str(), "claude" | "anthropic")
    }

    pub fn name(&self) -> &'static str {
        match self {
            Provider::Claude(_) => "claude",
        }
    }

    /// All accepted model names: both canonical IDs and their short aliases.
    pub fn accepted_models(&self) -> &'static [&'static str] {
        match self {
            Provider::Claude(_) => &[
                // Canonical IDs
                "claude-opus-4-7",
                "claude-sonnet-4-6",
                "claude-haiku-4-5-20251001",
                "claude-3-5-sonnet-20241022",
                "claude-3-5-haiku-20241022",
                "claude-3-opus-20240229",
                "claude-3-sonnet-20240229",
                "claude-3-haiku-20240307",
                // Short aliases resolved during translation
                "claude-opus-4",
                "claude-sonnet-4",
                "claude-haiku-4",
                "claude-3-5-sonnet",
                "claude-3-5-haiku",
                "claude-3-opus",
                "claude-3-sonnet",
                "claude-3-haiku",
            ],
        }
    }

    pub fn accepts_model(&self, model: &str) -> bool {
        self.accepted_models().contains(&model)
    }

    pub async fn chat_completion(
        &self,
        request: OpenAiChatRequest,
    ) -> Result<(OpenAiChatResponse, RawUsage), ProviderError> {
        match self {
            Provider::Claude(adapter) => {
                let anthropic_req = ClaudeAdapter::translate_request(&request)?;

                const BASE_DELAYS_MS: [u64; 3] = [500, 1000, 2000];
                let mut last_err = ProviderError::NotImplemented;

                for attempt in 1..=3usize {
                    match adapter.send(anthropic_req.clone(), 30).await {
                        Ok(resp) => return Ok(ClaudeAdapter::translate_response(resp)),
                        Err(e) => {
                            let retryable = matches!(
                                e,
                                ProviderError::ServerError(_) | ProviderError::RateLimited
                            );

                            if !retryable || attempt >= 3 {
                                if attempt >= 3 && retryable {
                                    error!(attempt, error = %e, "all retries exhausted");
                                }
                                return Err(e);
                            }

                            let delay_ms =
                                (BASE_DELAYS_MS[attempt - 1] as f64 * jitter_factor()) as u64;
                            warn!(attempt, delay_ms, error = %e, "retryable error, backing off");
                            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                            last_err = e;
                        }
                    }
                }

                Err(last_err)
            }
        }
    }

    pub async fn chat_completion_stream(
        &self,
        _request: OpenAiChatRequest,
    ) -> Result<Response<Body>, ProviderError> {
        Err(ProviderError::NotImplemented)
    }
}

fn jitter_factor() -> f64 {
    let raw = OsRng.next_u32() as f64 / u32::MAX as f64;
    0.9 + raw * 0.2
}

impl From<ProviderError> for ProxyError {
    fn from(err: ProviderError) -> Self {
        match err {
            ProviderError::NotImplemented => {
                ProxyError::InternalError("Provider not yet implemented".to_string())
            }
            ProviderError::AuthFailed => ProxyError::UpstreamAuthError,
            ProviderError::RateLimited => ProxyError::UpstreamRateLimit,
            ProviderError::ServerError(msg) => ProxyError::UpstreamServerError(msg),
            ProviderError::Timeout => ProxyError::UpstreamTimeout,
            ProviderError::ClientError(msg) => ProxyError::UpstreamClientError(msg),
        }
    }
}
