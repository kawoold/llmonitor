# Tech Stack Decisions — Unit 5: Config Management

## Inherited from Unit 1 (no change)

| Decision | Choice | Reason |
|---|---|---|
| Async runtime | `tokio` | Already in use |
| HTTP framework | `axum 0.7` | Already in use |
| Database | `sqlx 0.7` + SQLite WAL | Already in use |
| Error handling | `thiserror` + `anyhow` | Already in use |
| Structured logging | `tracing` + `tracing-subscriber` | Already in use |

## New in Unit 5

| Decision | Choice | Reason |
|---|---|---|
| Password hashing | `argon2 = "0.5"` (already in Cargo.toml) | Argon2id support; already declared |
| Base64 decoding (Basic Auth) | `std` / manual decode via `base64` crate (from existing deps) | Basic Auth header is standard base64; no new dep needed — use `axum`'s `TypedHeader` or manual parse |
| Config hot-reload | None (Q1=B — query SQLite per request) | Simplicity; ~1ms WAL read acceptable |
| Auth middleware | `axum::middleware::from_fn` (already pattern in Unit 1) | Consistent with `security_headers_middleware` pattern |
| Rate limiting on /api/ | None (Q2=B) | Auth + 500ms delay sufficient |

## Base64 Decoding for Basic Auth

The `Authorization: Basic <token>` header contains `base64(username:password)`. Decoding options:

- **Option A**: Use the `base64` crate (already a transitive dependency via `reqwest`/`argon2`) — `base64::engine::general_purpose::STANDARD.decode(token)`
- **Option B**: Use `axum-extra`'s `TypedHeader<Authorization<Basic>>` — adds a new crate

**Decision**: Option A — use `base64` crate directly. No new dependency; `base64` is already in the lockfile as a transitive dep. If not available as a direct dep, add `base64 = "0.21"` to Cargo.toml.

## Migration File

Migration `0001_config.sql` creates both `config` and `admin_users` tables. Executed by `sqlx::migrate!()` in `main.rs` (already wired in Unit 1). No changes to migration runner needed.

## Argon2id Parameters

```rust
use argon2::{Argon2, Algorithm, Version, Params};

let params = Params::new(65536, 3, 4, None)?; // memory_kib, iterations, parallelism
let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
```

Password verification uses `argon2::PasswordVerifier::verify_password()`. Password hashing uses `argon2::PasswordHasher::hash_password()` with a random salt from `argon2::password_hash::SaltString::generate()`.
