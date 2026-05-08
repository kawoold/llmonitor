use sqlx::SqlitePool;
use tracing::info;

use crate::config::domain::{ConfigError, ConfigSnapshot, RedactedConfigResponse, UpdateSettingsRequest};
use crate::config::types::ApiKey;

#[derive(Debug, Clone)]
pub struct ConfigService {
    db: SqlitePool,
}

impl ConfigService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    pub async fn get_all(&self) -> Result<ConfigSnapshot, ConfigError> {
        let rows = sqlx::query_as::<_, (String, String)>("SELECT key, value FROM config")
            .fetch_all(&self.db)
            .await?;

        let mut snapshot = ConfigSnapshot::default();
        for (key, value) in rows {
            match key.as_str() {
                "anthropic_api_key" => snapshot.anthropic_api_key = ApiKey::new(value),
                "request_timeout_secs" => {
                    if let Ok(v) = value.parse() {
                        snapshot.request_timeout_secs = v;
                    }
                }
                "rate_limit_capacity" => {
                    if let Ok(v) = value.parse() {
                        snapshot.rate_limit_capacity = v;
                    }
                }
                "rate_limit_refill_per_sec" => {
                    if let Ok(v) = value.parse() {
                        snapshot.rate_limit_refill_per_sec = v;
                    }
                }
                _ => {}
            }
        }
        Ok(snapshot)
    }

    pub async fn get_anthropic_api_key(&self) -> Result<ApiKey, ConfigError> {
        let row = sqlx::query_as::<_, (String,)>(
            "SELECT value FROM config WHERE key = 'anthropic_api_key'",
        )
        .fetch_optional(&self.db)
        .await?;
        Ok(row.map(|(v,)| ApiKey::new(v)).unwrap_or_default())
    }

    pub async fn get_redacted(&self) -> Result<RedactedConfigResponse, ConfigError> {
        let snapshot = self.get_all().await?;
        Ok(RedactedConfigResponse {
            anthropic_api_key: mask_api_key(&snapshot.anthropic_api_key.0),
            request_timeout_secs: snapshot.request_timeout_secs,
            rate_limit_capacity: snapshot.rate_limit_capacity,
            rate_limit_refill_per_sec: snapshot.rate_limit_refill_per_sec,
        })
    }

    pub async fn update_settings(
        &self,
        req: UpdateSettingsRequest,
    ) -> Result<RedactedConfigResponse, ConfigError> {
        if let Some(ref key) = req.anthropic_api_key {
            if key.is_empty() {
                return Err(ConfigError::EmptyApiKey);
            }
        }

        let mut tx = self.db.begin().await?;
        let now = chrono::Utc::now().to_rfc3339();

        if let Some(key) = req.anthropic_api_key {
            sqlx::query(
                "INSERT OR REPLACE INTO config (key, value, updated_at) VALUES ('anthropic_api_key', ?, ?)",
            )
            .bind(&key)
            .bind(&now)
            .execute(&mut *tx)
            .await?;
            info!(key = "anthropic_api_key", "Config key updated");
        }
        if let Some(v) = req.request_timeout_secs {
            sqlx::query(
                "INSERT OR REPLACE INTO config (key, value, updated_at) VALUES ('request_timeout_secs', ?, ?)",
            )
            .bind(v.to_string())
            .bind(&now)
            .execute(&mut *tx)
            .await?;
            info!(key = "request_timeout_secs", "Config key updated");
        }
        if let Some(v) = req.rate_limit_capacity {
            sqlx::query(
                "INSERT OR REPLACE INTO config (key, value, updated_at) VALUES ('rate_limit_capacity', ?, ?)",
            )
            .bind(v.to_string())
            .bind(&now)
            .execute(&mut *tx)
            .await?;
            info!(key = "rate_limit_capacity", "Config key updated");
        }
        if let Some(v) = req.rate_limit_refill_per_sec {
            sqlx::query(
                "INSERT OR REPLACE INTO config (key, value, updated_at) VALUES ('rate_limit_refill_per_sec', ?, ?)",
            )
            .bind(v.to_string())
            .bind(&now)
            .execute(&mut *tx)
            .await?;
            info!(key = "rate_limit_refill_per_sec", "Config key updated");
        }

        tx.commit().await?;
        self.get_redacted().await
    }
}

pub fn mask_api_key(key: &str) -> String {
    if key.is_empty() {
        return String::new();
    }
    if key.len() <= 12 {
        return "***...***".to_string();
    }
    format!("{}***...***{}", &key[..8], &key[key.len() - 4..])
}
