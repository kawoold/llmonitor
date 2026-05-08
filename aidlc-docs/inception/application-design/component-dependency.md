# Component Dependencies — llmonitor LLM Proxy

## Dependency Matrix

| Component | Depends On | Communication Pattern |
|---|---|---|
| C-01 HttpServer | All components (assembly) | Direct construction, Arc sharing |
| C-02 ProxyHandler | C-03 Provider, C-05 SessionEngine, C-06 TrackingService, C-09 MetricsRegistry, C-10 ConfigService | Function calls via Arc<AppState> |
| C-03 Provider (enum) | C-04 ClaudeAdapter, C-10 ConfigService (API key) | Enum variant construction, direct call |
| C-04 ClaudeAdapter | reqwest (external HTTP) | Async HTTP |
| C-05 SessionEngine | None (pure function) | Direct call |
| C-06 TrackingService | mpsc::Sender<UsageRecord> | Channel send (non-blocking) |
| C-07 TrackingWriterService | mpsc::Receiver<UsageRecord>, SQLite (via sqlx) | Channel recv + DB insert |
| C-08 StatsService | SQLite (via sqlx) | Async DB queries |
| C-09 MetricsRegistry | prometheus crate | In-process counter mutation |
| C-10 ConfigService | SQLite (via sqlx), Argon2 | Async DB queries + crypto |
| C-11 AuthMiddleware | C-10 ConfigService | Async call per request |
| C-12 FrontendAssets | rust-embed (compile-time) | Static bytes, no runtime deps |
| C-13 AdminBootstrapService | C-10 ConfigService, env vars, sqlx migrations | Startup-time only |

---

## Dependency Graph (Text)

```
main.rs (C-01 HttpServer)
├── runs once: C-13 AdminBootstrapService
│       └── C-10 ConfigService → SQLite
│
├── spawns: C-07 TrackingWriterService (background task)
│       └── SQLite
│
└── builds router with Arc<AppState> containing:
        ├── C-10 ConfigService → SQLite
        ├── C-06 TrackingService → mpsc::Sender
        ├── C-08 StatsService → SQLite
        └── C-09 MetricsRegistry (in-process)

/v1/* routes:
    C-02 ProxyHandler
        ├── C-10 ConfigService (API key lookup)
        ├── C-03 Provider enum
        │       └── C-04 ClaudeAdapter → Anthropic API (HTTPS)
        ├── C-05 SessionEngine (pure, no deps)
        ├── C-06 TrackingService (mpsc send)
        └── C-09 MetricsRegistry (counter update)

/api/* routes:
    C-11 AuthMiddleware → C-10 ConfigService
        ├── C-08 StatsService → SQLite (stats endpoints)
        └── C-10 ConfigService → SQLite (config endpoints)

/metrics:
    C-09 MetricsRegistry (render)

/ (frontend):
    C-12 FrontendAssets (embedded bytes)
```

---

## Data Flow — Non-Streaming Request

```
Client
  │  POST /v1/chat/completions
  │  X-Provider: claude
  │  Authorization: (none — proxy is unauthenticated)
  ▼
HttpServer (Axum router)
  │  middleware: request-id injection, rate limit check
  ▼
ProxyHandler::chat_completion()
  │  1. Deserialize OpenAiChatRequest
  │  2. Provider::from_name("claude") → Provider::Claude(adapter)
  │  3. SessionEngine::derive_session_id(&messages) → SessionId
  │  4. Provider::chat_completion(request) →
  │       ClaudeAdapter::translate_request()
  │       ClaudeAdapter::send() → POST api.anthropic.com/v1/messages
  │       ClaudeAdapter::translate_response() → (OpenAiChatResponse, RawUsage)
  │  5. Build UsageRecord { session_id, ..RawUsage, timestamp, duration_ms }
  │  6. TrackingService::record(record) → mpsc::send (non-blocking)
  │  7. MetricsRegistry::record_request(...)
  │  8. Return OpenAiChatResponse as JSON
  ▼
Client receives OpenAI-format response

(Background)
TrackingWriterService::run()
  └── recv UsageRecord from mpsc channel
  └── sqlx INSERT INTO request_logs → SQLite
```

---

## Data Flow — Statistics Query

```
React Frontend
  │  GET /api/stats/summary
  │  Authorization: Basic <base64>
  ▼
AuthMiddleware
  │  ConfigService::verify_admin_password() → Argon2id verify
  ▼
StatsService::summary()
  │  sqlx SELECT aggregates FROM request_logs → SQLite
  ▼
JSON response to React Frontend
```

---

## SQLite Access Patterns

| Table | Writers | Readers |
|---|---|---|
| `request_logs` | TrackingWriterService (batch insert) | StatsService (aggregation queries) |
| `config` | ConfigService (update, bootstrap) | ConfigService (get), AuthMiddleware (via ConfigService) |

SQLite WAL mode enabled — concurrent reads with single writer are safe.

---

## External Dependencies

| Dependency | Component | Protocol |
|---|---|---|
| Anthropic API (`api.anthropic.com`) | C-04 ClaudeAdapter | HTTPS/TLS |
| SQLite file (local disk) | C-07, C-08, C-10, C-13 | sqlx async driver |
| Docker volume (SQLite file path) | C-01 (env var config) | Filesystem |
