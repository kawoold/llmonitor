# Tech Stack Decisions — Unit 1: Proxy Core

## Runtime & Async

| Decision | Choice | Rationale |
|---|---|---|
| Async runtime | `tokio` (multi-thread, default worker count) | Industry standard for Axum; handles 100+ concurrent connections natively |
| HTTP server | `axum` 0.7.x | Ergonomic Tower-based framework; native SSE support; type-safe extractors |
| HTTP client (for upstream) | `reqwest` with `tokio` feature | Async, widely used, supports streaming; used in Unit 2 for Anthropic calls |

## Serialisation

| Decision | Choice | Rationale |
|---|---|---|
| JSON | `serde` + `serde_json` | De-facto standard; zero-copy deserialisation via `&str` borrows |
| Schema validation | `validator` crate + custom guards | Lightweight field-level validation integrated with serde |

## Middleware & Tower Layers

| Decision | Choice | Rationale |
|---|---|---|
| Request ID | `tower-http` `SetRequestIdLayer` | Built-in Tower layer; integrates with tracing span |
| Security headers | `tower-http` `SetResponseHeaderLayer` | Composable; applied once at router level |
| Body size limit | `axum::extract::DefaultBodyLimit` | Built into Axum; configurable per-router |
| Rate limiting | Custom token-bucket middleware (`src/middleware/rate_limit.rs`) | Global bucket with `Arc<Mutex<TokenBucket>>`; `tower_governor` skipped (per-IP only, not global) |
| Compression | `tower-http` `CompressionLayer` | Optional, disabled by default; can be enabled for stats API responses |

## Logging & Tracing

| Decision | Choice | Rationale |
|---|---|---|
| Logging framework | `tracing` | Structured, async-safe; native Axum/Tower integration |
| Log subscriber | `tracing-subscriber` with `EnvFilter` + JSON formatter | JSON output controlled by `RUST_LOG`; `tracing-subscriber::fmt::json()` |
| Request tracing | `tower-http` `TraceLayer` | Auto-logs request start/end with status and duration at INFO level |

## Database

| Decision | Choice | Rationale |
|---|---|---|
| SQLite driver | `sqlx` with `sqlite` feature + `runtime-tokio-rustls` | Async, compile-time query checking, migration support |
| Connection pool | `sqlx::SqlitePool` | Built-in; WAL mode set on first connection |
| Migrations | `sqlx::migrate!()` macro | Embedded migrations from `migrations/` directory; run at startup |

## Embedding Frontend

| Decision | Choice | Rationale |
|---|---|---|
| Static asset embedding | `rust-embed` crate | Embeds `frontend/dist/` at compile time; zero runtime I/O for asset serving |
| SPA fallback | Custom handler using `rust-embed` | Serves `index.html` for unknown paths that aren't `/api/*`, `/v1/*`, or `/metrics` |

## Error Handling

| Decision | Choice | Rationale |
|---|---|---|
| Internal error type | `thiserror` for `ProxyError` enum | Ergonomic `Display` + `From` impls; no runtime cost |
| Application-level error propagation | `anyhow` for startup/bootstrap code | Convenient chained errors during init; not used in request handlers |

## Configuration & Environment

| Decision | Choice | Rationale |
|---|---|---|
| Env var reading | Standard `std::env::var` | No additional crate needed; only a handful of startup vars |
| Config store | SQLite via `ConfigService` (Unit 5) | All runtime config is DB-backed; env vars only for bootstrap |

## Key `Cargo.toml` Dependencies (Unit 1 scope)

```toml
[dependencies]
tokio            = { version = "1", features = ["full"] }
axum             = { version = "0.7", features = ["macros"] }
tower            = "0.4"
tower-http       = { version = "0.5", features = ["trace", "set-request-id", "set-response-header", "compression-gzip"] }
serde            = { version = "1", features = ["derive"] }
serde_json       = "1"
validator        = { version = "0.18", features = ["derive"] }
sqlx             = { version = "0.7", features = ["sqlite", "runtime-tokio-rustls", "migrate", "uuid", "chrono"] }
tracing          = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
thiserror        = "1"
anyhow           = "1"
uuid             = { version = "1", features = ["v4", "serde"] }
chrono           = { version = "0.4", features = ["serde"] }
rust-embed       = "8"
tokio-util       = "0.7"   # for streaming body utilities
futures          = "0.3"   # Stream trait
```
