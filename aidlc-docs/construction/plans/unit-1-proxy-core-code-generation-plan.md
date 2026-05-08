# Code Generation Plan — Unit 1: Proxy Core

## Unit Context

- **Workspace root**: `/home/kawoold/Development/rust/llmonitor`
- **Project type**: Greenfield, single Rust crate with domain modules
- **Code location**: `src/` in workspace root (never `aidlc-docs/`)
- **Requirements covered**: FR-01, FR-02, FR-11, NFR-PERF-01/02, NFR-OBS-01, NFR-SEC-02–06, NFR-DEPLOY-01

**Dependencies on other units**: None — Unit 1 is the foundation  
**Provides to other units**: `AppState`, `SqlitePool`, Axum router extension points, `Provider` enum scaffold, all shared types

**Outcome**: A compilable, runnable Rust binary that starts the Axum server, passes health checks, enforces middleware, and returns structured errors for unimplemented providers (stub). All subsequent units extend this skeleton.

---

## Generation Steps

### Step 1: Project Structure & Cargo.toml
- [x] Create `Cargo.toml` with all Unit 1 dependencies (axum, tokio, tower, tower-http, serde, serde_json, sqlx, tracing, tracing-subscriber, thiserror, anyhow, uuid, chrono, rust-embed, validator, tokio-util, futures)
- [x] Create `.gitignore` (target/, *.db, .env)
- [x] Create `.dockerignore`
- [x] Create `migrations/.gitkeep` (directory placeholder; real migrations added in Units 5 and 3)
- [x] Create `frontend/dist/.gitkeep` (placeholder so rust-embed compiles before frontend is built)

### Step 2: Shared Types & Error Layer
- [x] Create `src/proxy/types.rs` — `OpenAiChatRequest`, `OpenAiMessage`, `MessageContent`, `OpenAiChatResponse`, `Choice`, `OpenAiUsage`, `OpenAiStreamChunk`, `StreamChoice`, `MessageDelta`
- [x] Create `src/proxy/error.rs` — `ProxyError` enum (all variants from domain-entities.md), `impl IntoResponse for ProxyError` (OpenAI error envelope), `OpenAiErrorBody`, `OpenAiErrorDetail`

### Step 3: Provider Scaffold
- [x] Create `src/providers/mod.rs` — `Provider` enum (`Claude(ClaudeAdapter)` stub), `Provider::from_name()`, `Provider::name()`, `Provider::accepted_models()`, `Provider::chat_completion()` (returns `ProviderError::NotImplemented` stub), `Provider::chat_completion_stream()` (stub)
- [x] Create `src/providers/claude/mod.rs` — `ClaudeAdapter` struct (fields only, no impl yet — implemented in Unit 2)

### Step 4: Stub Modules (compile-time placeholders for Units 2–5)
- [x] Create `src/tracking/mod.rs` — `TrackingService` stub (no-op `record()`)
- [x] Create `src/stats/mod.rs` — `StatsService` stub (returns empty responses)
- [x] Create `src/metrics/mod.rs` — `MetricsRegistry` stub (no-op methods)
- [x] Create `src/config/mod.rs` — `Config` struct (all fields with defaults), `ConfigService` stub
- [x] Create `src/config/bootstrap.rs` — `bootstrap()` stub (reads env vars, logs, returns Ok)
- [x] Create `src/auth/mod.rs` — `AuthLayer` stub (passes all requests through — real auth in Unit 5)

### Step 5: Database Setup
- [x] Create `src/db/mod.rs` — `create_pool(db_path: &str) -> SqlitePool`, WAL mode pragmas (`journal_mode`, `synchronous`, `foreign_keys`, `busy_timeout`), pool options (max 10 connections)

### Step 6: Middleware
- [x] Create `src/middleware/mod.rs` — module declarations
- [x] Create `src/middleware/rate_limit.rs` — `TokenBucket` struct, `RateLimitLayer` Tower middleware, token-bucket algorithm (LC-01), `Retry-After` header on 429

### Step 7: Application State
- [x] Create `src/app_state.rs` — `AppState` struct with all fields (`db`, `tracking`, `metrics`, `config`, `stats`, `started_at`), `impl AppState` constructor

### Step 8: Proxy Handlers
- [x] Create `src/proxy/mod.rs` — module declarations
- [x] Create `src/proxy/routing.rs` — `resolve_provider()` (header extraction, lowercase normalisation, Provider::from_name, API key check)
- [x] Create `src/proxy/handlers.rs` — `chat_completion()` handler (full pipeline: deserialize → validate → resolve provider → validate model → dispatch → track stub → return), `chat_completion_stream()` stub (returns 501 until Unit 2)
- [x] Create `src/proxy/health.rs` — `health_handler()` (SQLite ping, uptime calc, `HealthResponse` JSON)
- [x] Create `src/proxy/frontend.rs` — `FrontendAssets` rust-embed struct, `serve_asset()`, `serve_index()` SPA fallback

### Step 9: Main Entry Point & Router
- [x] Create `src/main.rs` — `main()` with full startup sequence (env, DB pool, migrations, bootstrap, AppState, router, bind, serve with graceful shutdown), `build_router()` composing all Tower layers and sub-routers (proxy `/v1/*`, management `/api/*` stub, `/metrics` stub, `/health`, `/*` frontend)

### Step 10: Business Logic Unit Tests
- [x] Create `src/proxy/tests.rs` — tests for:
  - `resolve_provider`: missing header → PROVIDER_HEADER_MISSING
  - `resolve_provider`: unknown provider → PROVIDER_UNKNOWN
  - `resolve_provider`: unconfigured API key → PROVIDER_NOT_CONFIGURED
  - `Provider::from_name`: case-insensitivity ("Claude", "CLAUDE" → Claude variant)
  - `Provider::accepted_models`: returns non-empty slice
  - `ProxyError::into_response`: each variant produces correct HTTP status + OpenAI error body
  - Model validation: unknown model → MODEL_NOT_ACCEPTED with accepted list
- [x] Create `src/middleware/rate_limit_tests.rs` — tests for:
  - Token bucket: full bucket allows requests
  - Token bucket: empty bucket returns 429 with Retry-After
  - Token bucket: refill over time restores tokens
- [x] Create `src/proxy/types_tests.rs` — property-based tests (PBT partial):
  - `OpenAiChatRequest` serialisation round-trip (arbitrary valid inputs)
  - `OpenAiChatResponse` serialisation round-trip

### Step 11: Integration Test
- [x] Create `tests/health_test.rs` — spins up full Axum app with in-memory SQLite, calls `GET /health`, asserts `{"status":"ok"}`
- [x] Create `tests/proxy_validation_test.rs` — calls `POST /v1/chat/completions` with missing `X-Provider`, asserts 400 + OpenAI error body

### Step 12: Dockerfile & Deployment Artifacts
- [x] Create `Dockerfile` — multi-stage build as specified in `deployment-architecture.md`
- [x] Create `src/config/types.rs` — `ApiKey` newtype with redacted `Debug`/`Display` impls (LC-07)

### Step 13: Code Documentation Summary
- [x] Create `aidlc-docs/construction/unit-1-proxy-core/code/code-summary.md` — list of all files created, their purpose, and key design decisions implemented

---

## Story / Requirement Traceability

| Step | Requirements |
|---|---|
| 1 | NFR-DEPLOY-01 (Docker), SECURITY-10 (Cargo.lock) |
| 2 | FR-01 (OpenAI schema), SECURITY-05 (typed validation), SECURITY-09 (error hardening) |
| 3 | FR-02 (provider routing), FR-11 (provider extensibility) |
| 4 | All units (compile-time stubs) |
| 5 | NFR-STORE-01 (SQLite + WAL) |
| 6 | NFR-SEC-02 (rate limiting), SECURITY-11 |
| 7 | NFR-PERF-02 (shared state) |
| 8 | FR-01, FR-02, SECURITY-04 (security headers), SECURITY-15 (error handler) |
| 9 | NFR-PERF-02 (Tokio), SECURITY-03 (structured logging), SECURITY-09 (graceful shutdown) |
| 10 | MAINT-02 (unit tests), PBT partial (serialisation round-trips) |
| 11 | MAINT-02 (integration tests) |
| 12 | NFR-DEPLOY-01 (Dockerfile), SECURITY-09 (non-root user, pinned images) |
| 13 | Documentation |
