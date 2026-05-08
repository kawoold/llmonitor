pub mod admin;
pub mod bootstrap;
pub mod domain;
pub mod handlers;
pub mod service;
pub mod types;

#[cfg(test)]
mod tests;

pub use service::ConfigService;

use std::time::Duration;
use types::ApiKey;

#[derive(Debug, Clone)]
pub struct Config {
    pub anthropic_api_key: ApiKey,
    pub database_url: String,
    pub listen_addr: String,
    pub request_timeout: Duration,
    pub rate_limit_capacity: u32,
    pub rate_limit_refill_per_sec: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            anthropic_api_key: ApiKey::default(),
            database_url: "llmonitor.db".to_string(),
            listen_addr: "0.0.0.0:8080".to_string(),
            request_timeout: Duration::from_secs(30),
            rate_limit_capacity: 100,
            rate_limit_refill_per_sec: 20,
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        let mut cfg = Config::default();

        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            cfg.anthropic_api_key = ApiKey::new(key);
        }
        if let Ok(db) = std::env::var("DATABASE_URL") {
            cfg.database_url = db;
        }
        if let Ok(addr) = std::env::var("LISTEN_ADDR") {
            cfg.listen_addr = addr;
        }
        if let Ok(timeout) = std::env::var("REQUEST_TIMEOUT_SECS") {
            if let Ok(secs) = timeout.parse::<u64>() {
                cfg.request_timeout = Duration::from_secs(secs);
            }
        }

        cfg
    }
}
