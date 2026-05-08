# Logical Components — Unit 5: Config Management

## LC-U5-01: ConfigService

**Module**: `src/config/service.rs`

**Role**: Thin async wrapper over the `config` key-value table. All config reads and writes go through this service.

**Struct**:
```rust
pub struct ConfigService {
    db: SqlitePool,
}
```

**Key behaviours**:
- `get_all()` → `ConfigSnapshot`: SELECT all config rows; apply defaults for missing keys
- `get_anthropic_api_key()` → `ApiKey`: Direct DB read (no in-memory cache — Q1=B)
- `update_settings()` → `RedactedConfigResponse`: Validates then runs atomic transaction
- `get_redacted()` → `RedactedConfigResponse`: Calls `get_all()`, applies masking algorithm

**Lives in**: `AppState.config_service: ConfigService`

---

## LC-U5-02: AdminService

**Module**: `src/config/admin.rs`

**Role**: Admin user CRUD + Argon2id credential verification.

**Struct**:
```rust
pub struct AdminService {
    db: SqlitePool,
}
```

**Key behaviours**:
- `verify(username, password)`: DB lookup + Argon2id verify; delays 500ms on any failure (Pattern 2)
- `create(username, password)`: Validates format/length, hashes password, inserts row
- `update_password(username, new_password)`: Validates length, hashes, updates row
- `any_exists()`: `SELECT COUNT(*) FROM admin_users` — used by bootstrap

**Lives in**: `AppState.admin_service: AdminService`

---

## LC-U5-03: BasicAuthMiddleware

**Module**: `src/auth/middleware.rs`

**Role**: Axum `from_fn_with_state` middleware applied to all `/api/*` routes via `route_layer`. Extracts and validates HTTP Basic Auth credentials using `AdminService`.

**Signature**:
```rust
pub async fn basic_auth_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response
```

**Pipeline** (Pattern 1 — fail-closed):
1. Extract `Authorization` header → missing → 401
2. Strip `Basic ` prefix, base64-decode → malformed → 401
3. Split on first `:` → `(username, password)` → malformed → 401
4. `state.admin_service.verify(username, password).await` → Err → 401 (delay already applied)
5. `next.run(request).await`

**Applied via**: `router.route_layer(from_fn_with_state(state, basic_auth_middleware))`

---

## LC-U5-04: AdminBootstrapService

**Module**: `src/config/bootstrap.rs` (replaces Unit 1 stub)

**Role**: Startup function (not a long-lived struct). Checks for existing admin users and creates the first admin from env vars if absent.

**Signature**:
```rust
pub async fn bootstrap(db: &SqlitePool) -> Result<(), BootstrapError>
```

**Sequence**:
1. `AdminService::any_exists(db)` → true → return Ok
2. Read `LLMONITOR_ADMIN_USER` + `LLMONITOR_ADMIN_PASSWORD` env vars
3. Missing either → return `Err(BootstrapError::MissingEnvVars)` → caller calls `process::exit(1)`
4. `AdminService::create(username, password)` → hash + insert
5. `tracing::info!("Admin user created from environment variables")`
6. Return `Ok(())`

---

## LC-U5-05: API Key Masking Function

**Module**: `src/config/service.rs` (private helper)

**Role**: Converts an `ApiKey` to its masked string representation for HTTP responses (BR-U5-08).

```rust
fn mask_api_key(key: &str) -> String {
    if key.is_empty() {
        return String::new();
    }
    if key.len() <= 12 {
        return "***...***".to_string();
    }
    format!("{}***...***{}", &key[..8], &key[key.len()-4..])
}
```

**Unit tested** with: empty key, short key (≤12 chars), normal key (>12 chars).

---

## Component Interactions

```
HTTP Request → /api/*
    │
    ▼
LC-U5-03: BasicAuthMiddleware
    │  extracts credentials
    │  calls AdminService::verify()
    │       │  queries admin_users table (LC-U5-02)
    │       │  Argon2id::verify_password()
    │       ▼
    │  Ok → next handler
    │  Err → 401 (after 500ms delay)
    │
    ▼
Handler (e.g., get_config, update_config)
    │  calls ConfigService (LC-U5-01)
    │       │  queries / updates config table
    │       │  applies masking (LC-U5-05) for responses
    │       ▼
    │  RedactedConfigResponse → 200 JSON
    │
    ▼
Response
```
