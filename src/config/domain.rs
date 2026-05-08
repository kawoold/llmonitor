use serde::{Deserialize, Serialize};
use crate::config::types::ApiKey;

#[derive(Debug, Clone)]
pub struct ConfigSnapshot {
    pub anthropic_api_key: ApiKey,
    pub request_timeout_secs: u64,
    pub rate_limit_capacity: u32,
    pub rate_limit_refill_per_sec: u32,
}

impl Default for ConfigSnapshot {
    fn default() -> Self {
        Self {
            anthropic_api_key: ApiKey::default(),
            request_timeout_secs: 30,
            rate_limit_capacity: 100,
            rate_limit_refill_per_sec: 20,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RedactedConfigResponse {
    pub anthropic_api_key: String,
    pub request_timeout_secs: u64,
    pub rate_limit_capacity: u32,
    pub rate_limit_refill_per_sec: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSettingsRequest {
    pub anthropic_api_key: Option<String>,
    pub request_timeout_secs: Option<u64>,
    pub rate_limit_capacity: Option<u32>,
    pub rate_limit_refill_per_sec: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateCredentialsRequest {
    pub new_password: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Unknown config key: {0}")]
    UnknownKey(String),
    #[error("API key cannot be empty")]
    EmptyApiKey,
    #[error("Database error: {0}")]
    Db(#[from] sqlx::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Database error")]
    Db,
    #[error("Internal error")]
    Internal,
}

#[derive(Debug, thiserror::Error)]
pub enum AdminError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid username format")]
    InvalidUsername,
    #[error("Password too short (minimum 8 characters)")]
    PasswordTooShort,
    #[error("Database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("Internal error")]
    Internal,
}

#[derive(Debug, thiserror::Error)]
pub enum BootstrapError {
    #[error("No admin users found and LLMONITOR_ADMIN_USER/LLMONITOR_ADMIN_PASSWORD env vars are not set")]
    MissingEnvVars,
    #[error("Admin creation failed: {0}")]
    AdminError(#[from] AdminError),
    #[error("Database error: {0}")]
    Db(#[from] sqlx::Error),
}

pub const KNOWN_CONFIG_KEYS: &[&str] = &[
    "anthropic_api_key",
    "request_timeout_secs",
    "rate_limit_capacity",
    "rate_limit_refill_per_sec",
];
