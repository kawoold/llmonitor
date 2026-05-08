# Services — llmonitor LLM Proxy

Services represent cross-cutting orchestration concerns that coordinate multiple components.

---

## S-01: RequestPipelineService

**Purpose**: Orchestrates the end-to-end request lifecycle for each proxied LLM request. Not a standalone struct — this logic lives inside the ProxyHandler, using shared `AppState`.

**Flow**:

```
1. Parse & validate OpenAI request body (serde + validator)
2. Extract X-Provider header → resolve Provider enum variant
3. Derive SessionId from messages array (SessionEngine)
4. Dispatch to Provider::chat_completion / chat_completion_stream
5. On response:
   a. Extract RawUsage from provider response
   b. Build UsageRecord (attach session_id, timestamp, duration, status)
   c. Send UsageRecord to TrackingService (non-blocking)
   d. Update MetricsRegistry (non-blocking, in-process)
6. Return OpenAI-format response to client
7. On any error: translate to OpenAI error format, still attempt tracking if usage is known
```

**Streaming variant**:
- Steps 1-3 identical
- Provider returns a Stream of OpenAiStreamChunks
- ProxyHandler wraps stream as SSE response
- Usage record is built from the final `usage` chunk (streamed last by Anthropic)
- TrackingService.record() is called inside the stream's completion future

---

## S-02: TrackingWriterService (Background Task)

**Purpose**: Long-running async task that owns the consumer end of the usage record mpsc channel and drains it to SQLite. Runs in its own `tokio::spawn` task for the lifetime of the server.

**Lifecycle**:

```
Startup:
  1. Spawned by main.rs after DB pool is ready
  2. Holds mpsc::Receiver<UsageRecord> and SqlitePool

Loop:
  1. Select on: recv_many(batch, batch_size) OR timeout(flush_interval)
  2. On batch ready OR timeout elapsed: execute bulk INSERT into request_logs
  3. Log errors without panicking; drain continues
  4. Repeat until sender side is dropped

Shutdown:
  1. Server receives SIGTERM → drops TrackingService (closes sender)
  2. Writer drains remaining records in channel
  3. Task exits cleanly
```

**Configuration** (from DB config):
- `tracking_batch_size`: default 50 records
- `tracking_flush_interval_ms`: default 500ms

---

## S-03: AdminBootstrapService (Startup Task)

**Purpose**: One-shot startup routine ensuring the system cannot run without valid admin credentials. Runs synchronously before the HTTP server starts accepting connections.

**Flow**:

```
1. Run DB migrations (sqlx::migrate!())
2. Check ConfigService::admin_exists()
3a. If admin exists: continue startup
3b. If no admin:
    - Read LLMONITOR_ADMIN_USER from env (error if absent)
    - Read LLMONITOR_ADMIN_PASSWORD from env (error if absent)
    - Validate: username non-empty, password min 8 chars
    - Hash password with Argon2id
    - Persist admin record via ConfigService
    - Log "Admin credentials initialised from environment"
    - Clear env vars from process memory after reading (best effort)
4. Continue server startup
```

---

## S-04: AppState (Shared State Container)

**Purpose**: Axum shared state (`Arc<AppState>`) injected into all handlers. Acts as the service locator / dependency container.

```rust
pub struct AppState {
    pub db: SqlitePool,
    pub tracking: Arc<TrackingService>,
    pub metrics: Arc<MetricsRegistry>,
    pub config: Arc<ConfigService>,
    pub stats: Arc<StatsService>,
}
```

All handlers receive `State(Arc<AppState>)` via Axum's state extraction.

---

## Service Interaction Summary

```
HTTP Request
    │
    ▼
AuthMiddleware ──── (only /api/* routes) ──── ConfigService.verify_admin_password()
    │
    ▼
ProxyHandler (RequestPipelineService logic)
    ├── Provider::from_name() ──────────────── Config (API key lookup)
    ├── SessionEngine::derive_session_id()
    ├── Provider::chat_completion()
    │       └── ClaudeAdapter.send() ─────────── Anthropic API (external)
    ├── TrackingService::record() ────────────── mpsc channel (non-blocking)
    │       └── TrackingWriterService (bg) ───── SQLite insert
    └── MetricsRegistry::record_request()
    │
    ▼
OpenAI Response to client

/api/stats/* ──── StatsService ─────────────── SQLite queries
/metrics     ──── MetricsRegistry.render() ─── in-process counters
/api/config  ──── ConfigService ────────────── SQLite config table
/            ──── FrontendAssets ──────────── rust-embed (binary assets)
```
