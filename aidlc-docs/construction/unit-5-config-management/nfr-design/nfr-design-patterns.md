# NFR Design Patterns — Unit 5: Config Management

## Pattern 1: Fail-Closed Auth

**NFR**: NFR-U5-REL-03 — unexpected auth errors must produce 401, never 500

**Implementation**:
```rust
// In BasicAuthMiddleware
match admin_service.verify(username, password).await {
    Ok(()) => next.run(request).await,
    Err(_) => {
        // Any error — DB failure, hash error, not-found — all produce 401
        // AuthError::InvalidCredentials already slept 500ms inside verify()
        // For other errors (DB down etc.), we also return 401 to fail closed
        unauthorized_response()
    }
}
```

The middleware never propagates internal errors to the caller. A DB failure is indistinguishable from a wrong password from the client's perspective.

---

## Pattern 2: Timing Attack Mitigation (Fixed Delay)

**NFR**: NFR-U5-SEC-03 — 500ms fixed delay on any auth failure

**Implementation**: Delay is applied inside `AdminService::verify()`, not in the middleware. This ensures the delay is consistent regardless of which code path reaches the error:

```rust
pub async fn verify(&self, username: &str, password: &str) -> Result<(), AuthError> {
    let result = self.verify_inner(username, password).await;
    if result.is_err() {
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    result
}

async fn verify_inner(&self, username: &str, password: &str) -> Result<(), AuthError> {
    // actual DB lookup + Argon2id verify
}
```

This prevents timing side-channels: both "username not found" and "wrong password" wait exactly 500ms before returning.

---

## Pattern 3: Parameterized Query Pattern (Inherited)

**NFR**: NFR-U5-SEC-07, SECURITY-05

All SQL statements use sqlx's `?` placeholders. No string interpolation. Same pattern as Unit 1's DB layer:

```rust
sqlx::query!("SELECT password_hash FROM admin_users WHERE username = ?", username)
    .fetch_optional(&self.db)
    .await?
```

---

## Pattern 4: Atomic Config Update (Transaction Pattern)

**NFR**: NFR-U5-SEC-10, BR-U5-13

Multi-field config updates execute inside a SQLite transaction:

```rust
let mut tx = self.db.begin().await?;
for (key, value) in updates {
    sqlx::query!("INSERT OR REPLACE INTO config (key, value, updated_at) VALUES (?, ?, ?)",
        key, value, now)
        .execute(&mut *tx)
        .await?;
}
tx.commit().await?;
```

If any upsert fails, the transaction is rolled back automatically when `tx` is dropped.

---

## Pattern 5: Newtype Redaction (Inherited from Unit 1)

**NFR**: NFR-U5-SEC-06, SECURITY-06

`ApiKey(String)` newtype from `src/config/types.rs` ensures API keys can never appear in logs via `{:?}` or `{}` formatting. `ConfigService` returns `ApiKey` internally; only the redacted string representation appears in HTTP responses (BR-U5-08).

---

## Pattern 6: Middleware State Injection via `from_fn_with_state`

**NFR**: Auth middleware needs access to `SqlitePool` to verify credentials

The auth middleware uses `axum::middleware::from_fn_with_state(state, auth_middleware_fn)` so the `AppState` (containing `SqlitePool`) is available inside the middleware function without extracting from request extensions:

```rust
// In build_router():
let management_routes = Router::new()
    .route("/api/config", get(get_config).put(update_config))
    .route("/api/config/credentials", put(update_credentials))
    .route_layer(axum::middleware::from_fn_with_state(
        state.clone(),
        auth::basic_auth_middleware,
    ));

// Middleware signature:
pub async fn basic_auth_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response { ... }
```

`route_layer` applies the middleware only to the routes in that router, not to the outer router's routes — so `/v1/*`, `/health`, and `/metrics` are unaffected.
