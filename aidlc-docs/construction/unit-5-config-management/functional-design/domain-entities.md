# Domain Entities — Unit 5: Config Management

## Database Schema

### `config` table (key-value store)
```sql
CREATE TABLE IF NOT EXISTS config (
    key        TEXT PRIMARY KEY NOT NULL,
    value      TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
```

### `admin_users` table
```sql
CREATE TABLE IF NOT EXISTS admin_users (
    username      TEXT PRIMARY KEY NOT NULL,
    password_hash TEXT NOT NULL,
    created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
```

---

## Rust Types

### ConfigSnapshot
```rust
// Internal typed view of all config table entries
pub struct ConfigSnapshot {
    pub anthropic_api_key: ApiKey,     // from config/types.rs (LC-07 redaction)
    pub request_timeout_secs: u64,
    pub rate_limit_capacity: u32,
    pub rate_limit_refill_per_sec: u32,
}
```

### RedactedConfigResponse
```rust
// Returned by GET /api/config — API key is masked
#[derive(Serialize)]
pub struct RedactedConfigResponse {
    pub anthropic_api_key: String,    // masked: "sk-ant-***...***" or ""
    pub request_timeout_secs: u64,
    pub rate_limit_capacity: u32,
    pub rate_limit_refill_per_sec: u32,
}
```

### UpdateSettingsRequest
```rust
// Body for PUT /api/config
// All fields are optional — absent = "leave unchanged"
#[derive(Deserialize)]
pub struct UpdateSettingsRequest {
    pub anthropic_api_key: Option<String>,    // must be non-empty if present (BR-U5-06)
    pub request_timeout_secs: Option<u64>,
    pub rate_limit_capacity: Option<u32>,
    pub rate_limit_refill_per_sec: Option<u32>,
}
```

### UpdateCredentialsRequest
```rust
// Body for PUT /api/config/credentials
#[derive(Deserialize)]
pub struct UpdateCredentialsRequest {
    pub new_password: String,    // min 8 chars (BR-U5-10)
}
```

### AdminUser (internal)
```rust
pub struct AdminUser {
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### ConfigError
```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Unknown config key: {0}")]
    UnknownKey(String),
    #[error("API key cannot be empty")]
    EmptyApiKey,
    #[error("Database error: {0}")]
    Db(#[from] sqlx::Error),
}
```

### AuthError
```rust
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("Internal error")]
    Internal,
}
```

### AdminError
```rust
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
```

### BootstrapError
```rust
#[derive(Debug, thiserror::Error)]
pub enum BootstrapError {
    #[error("No admin users found and LLMONITOR_ADMIN_USER/LLMONITOR_ADMIN_PASSWORD env vars are not set")]
    MissingEnvVars,
    #[error("Admin creation failed: {0}")]
    AdminError(#[from] AdminError),
    #[error("Database error: {0}")]
    Db(#[from] sqlx::Error),
}
```

---

## Known Config Keys (Closed Set — BR-U5-07)

| Key | Rust type | Default |
|---|---|---|
| `anthropic_api_key` | `ApiKey` | `ApiKey("")` |
| `request_timeout_secs` | `u64` | `30` |
| `rate_limit_capacity` | `u32` | `100` |
| `rate_limit_refill_per_sec` | `u32` | `20` |

---

## Service Interfaces

### ConfigService
```rust
pub struct ConfigService {
    db: SqlitePool,
}

impl ConfigService {
    pub async fn get_all(&self) -> Result<ConfigSnapshot, ConfigError>;
    pub async fn get_redacted(&self) -> Result<RedactedConfigResponse, ConfigError>;
    pub async fn update_settings(&self, req: UpdateSettingsRequest) -> Result<RedactedConfigResponse, ConfigError>;
    pub async fn get_anthropic_api_key(&self) -> Result<ApiKey, ConfigError>;
}
```

### AdminService
```rust
pub struct AdminService {
    db: SqlitePool,
}

impl AdminService {
    pub async fn verify(&self, username: &str, password: &str) -> Result<(), AuthError>;
    pub async fn create(&self, username: &str, password: &str) -> Result<(), AdminError>;
    pub async fn update_password(&self, username: &str, new_password: &str) -> Result<(), AdminError>;
    pub async fn any_exists(&self) -> Result<bool, sqlx::Error>;
}
```

### AdminBootstrapService
```rust
pub async fn bootstrap(db: &SqlitePool) -> Result<(), BootstrapError>;
```

---

## API Key Masking Algorithm (BR-U5-08)

```
fn mask_api_key(key: &str) -> String:
    if key.is_empty():
        return ""
    if key.len() <= 12:
        return "***...***"
    return key[..8] + "***...***" + key[key.len()-4..]
```

Example: `"sk-ant-api03-abcdefghij1234"` → `"sk-ant-a***...***1234"`
