use std::time::Duration;

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, Algorithm, Params, Version,
};
use rand_core::OsRng;
use sqlx::SqlitePool;
use tracing::warn;

use crate::config::domain::{AdminError, AuthError};

#[derive(Debug, Clone)]
pub struct AdminService {
    db: SqlitePool,
}

impl AdminService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    pub async fn verify(&self, username: &str, password: &str) -> Result<(), AuthError> {
        let result = self.verify_inner(username, password).await;
        if result.is_err() {
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        result
    }

    async fn verify_inner(&self, username: &str, password: &str) -> Result<(), AuthError> {
        let row = sqlx::query_as::<_, (String,)>(
            "SELECT password_hash FROM admin_users WHERE username = ?",
        )
        .bind(username)
        .fetch_optional(&self.db)
        .await
        .map_err(|_| AuthError::Db)?;

        let hash_str = match row {
            Some((h,)) => h,
            None => {
                warn!(username, "Auth failed: unknown username");
                return Err(AuthError::InvalidCredentials);
            }
        };

        let parsed = PasswordHash::new(&hash_str).map_err(|_| AuthError::Internal)?;
        let argon2 = make_argon2();

        argon2
            .verify_password(password.as_bytes(), &parsed)
            .map_err(|_| {
                warn!(username, "Auth failed: wrong password");
                AuthError::InvalidCredentials
            })
    }

    pub async fn create(&self, username: &str, password: &str) -> Result<(), AdminError> {
        validate_username(username)?;
        validate_password(password)?;

        let hash = hash_password(password).map_err(|_| AdminError::Internal)?;
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO admin_users (username, password_hash, created_at, updated_at) VALUES (?, ?, ?, ?)",
        )
        .bind(username)
        .bind(&hash)
        .bind(&now)
        .bind(&now)
        .execute(&self.db)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE") {
                AdminError::UserAlreadyExists
            } else {
                AdminError::Db(e)
            }
        })?;

        Ok(())
    }

    pub async fn update_password(
        &self,
        username: &str,
        new_password: &str,
    ) -> Result<(), AdminError> {
        validate_password(new_password)?;

        let hash = hash_password(new_password).map_err(|_| AdminError::Internal)?;
        let now = chrono::Utc::now().to_rfc3339();

        let result = sqlx::query(
            "UPDATE admin_users SET password_hash = ?, updated_at = ? WHERE username = ?",
        )
        .bind(&hash)
        .bind(&now)
        .bind(username)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AdminError::UserNotFound);
        }
        Ok(())
    }

    pub async fn any_exists(&self) -> Result<bool, sqlx::Error> {
        let (count,) =
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM admin_users")
                .fetch_one(&self.db)
                .await?;
        Ok(count > 0)
    }
}

fn make_argon2() -> Argon2<'static> {
    let params = Params::new(65536, 3, 4, None).expect("valid argon2 params");
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
}

fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = make_argon2();
    Ok(argon2.hash_password(password.as_bytes(), &salt)?.to_string())
}

fn validate_username(username: &str) -> Result<(), AdminError> {
    if username.is_empty() || username.len() > 64 {
        return Err(AdminError::InvalidUsername);
    }
    if !username
        .chars()
        .all(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '.'))
    {
        return Err(AdminError::InvalidUsername);
    }
    Ok(())
}

fn validate_password(password: &str) -> Result<(), AdminError> {
    if password.len() < 8 {
        return Err(AdminError::PasswordTooShort);
    }
    Ok(())
}
