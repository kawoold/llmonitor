# Components — llmonitor LLM Proxy

## Project Structure

Single Rust crate (`src/`) with module folders per domain. React frontend built separately and embedded via `rust-embed`.

```
llmonitor/
├── src/
│   ├── main.rs                  # Entry point, server bootstrap
│   ├── config/                  # Configuration service + DB-backed config store
│   ├── proxy/                   # OpenAI-compatible HTTP handlers + request pipeline
│   ├── providers/               # Provider enum + concrete adapters
│   │   └── claude/              # Anthropic Claude adapter
│   ├── tracking/                # Usage record types, mpsc writer, session engine
│   ├── stats/                   # Statistics queries + aggregation
│   ├── metrics/                 # Prometheus metrics registry
│   ├── auth/                    # Basic auth middleware
│   ├── middleware/              # HTTP middleware (security headers, request ID, rate limit)
│   └── db/                      # SQLite pool setup, migrations
├── frontend/                    # TypeScript + React SPA (separate build)
│   ├── src/
│   ├── package.json
│   └── vite.config.ts
├── migrations/                  # sqlx migration files
├── Cargo.toml
├── Dockerfile
└── docker-compose.yml
```

---

## Component Catalog

### C-01: HttpServer

**Module**: `src/main.rs` + `src/middleware/`

**Purpose**: Entry point and HTTP server lifecycle. Assembles the Axum router, attaches middleware, binds to configured port, and manages graceful shutdown.

**Responsibilities**:
- Bootstrap Tokio runtime and start Axum HTTP server
- Configure the full middleware stack (request ID injection, rate limiting, security headers, CORS)
- Mount the proxy router (`/v1/*`), management API router (`/api/*`), metrics endpoint (`/metrics`), and frontend asset handler (`/`)
- Handle graceful shutdown on SIGTERM/SIGINT
- Initialize shared application state (DB pool, provider registry, tracking channel, metrics registry)

---

### C-02: ProxyHandler

**Module**: `src/proxy/`

**Purpose**: Receives OpenAI-compatible client requests, resolves the target provider, dispatches the request, captures the response, triggers usage tracking, and returns the OpenAI-format response.

**Responsibilities**:
- Parse and validate incoming OpenAI Chat Completions requests
- Extract the `X-Provider` header and resolve the `Provider` enum variant
- Dispatch to the appropriate Provider for both streaming and non-streaming modes
- Extract the `UsageRecord` from provider responses
- Send the `UsageRecord` to the TrackingService channel (non-blocking)
- Return OpenAI-format responses to the client
- Handle and translate provider errors into OpenAI-compatible error responses

---

### C-03: Provider (Enum)

**Module**: `src/providers/mod.rs`

**Purpose**: Central dispatch enum over all supported provider implementations. Matches provider name from the `X-Provider` header to a concrete adapter.

**Responsibilities**:
- Define the `Provider` enum with one variant per supported provider (`Provider::Claude`)
- Expose a unified async interface for non-streaming and streaming chat completions
- Resolve provider name strings to `Provider` variants (case-insensitive)
- Return a descriptive error for unknown provider names
- Carry the provider configuration (API key, endpoint, model defaults) as enum variant fields

**Note**: Adding a new provider requires adding an enum variant and a new adapter module — no changes to the core proxy pipeline.

---

### C-04: ClaudeAdapter

**Module**: `src/providers/claude/`

**Purpose**: Concrete implementation of the Claude (Anthropic) provider. Handles all translation between OpenAI format and Anthropic Messages API format, including prompt cache metadata.

**Responsibilities**:
- Translate `OpenAiChatRequest` → `AnthropicMessagesRequest` (including system prompt extraction, role mapping, model mapping)
- Translate `AnthropicMessagesResponse` → `OpenAiChatResponse` + `UsageRecord` (including cache hit/miss token extraction from `x-anthropic-*` response headers)
- Translate streaming `AnthropicStreamChunk` → `OpenAiStreamChunk` (SSE passthrough with format conversion)
- Execute HTTP requests to the Anthropic API (`https://api.anthropic.com/v1/messages`) using `reqwest`
- Handle Anthropic-specific error codes and translate to `ProviderError`
- Extract and forward Anthropic cache control directives from OpenAI extension fields

---

### C-05: SessionEngine

**Module**: `src/tracking/session.rs`

**Purpose**: Derives a stable, deterministic conversation session ID from the message history in the request body, allowing requests that share the same conversational context to be grouped.

**Responsibilities**:
- Accept the `messages` array from the OpenAI request
- Exclude the latest user message (the current turn); hash the remaining prior conversation history
- Produce a `SessionId` (hex-encoded SHA-256 digest of the serialised prior messages)
- Return a stable nil/zero session ID for single-turn or first-turn requests with no prior history

---

### C-06: TrackingService

**Module**: `src/tracking/service.rs`

**Purpose**: Non-blocking interface for sending `UsageRecord` values to the background writer. Wraps the `mpsc::Sender` end of the write channel.

**Responsibilities**:
- Expose a `record(UsageRecord)` method that sends to the bounded mpsc channel
- Return a non-blocking error if the channel is full (backpressure signal — logged, request still succeeds)
- Hold the `mpsc::Sender<UsageRecord>` as shared state (wrapped in `Arc`)

---

### C-07: TrackingWriterService

**Module**: `src/tracking/writer.rs`

**Purpose**: Long-running background Tokio task that consumes `UsageRecord` values from the mpsc channel and writes them to SQLite.

**Responsibilities**:
- Receive `UsageRecord` items from the `mpsc::Receiver<UsageRecord>`
- Write each record to the `request_logs` SQLite table via sqlx
- Implement a simple batching strategy (flush on N records or T milliseconds, whichever comes first)
- Log write errors without crashing (continue processing remaining records)
- Drain the channel gracefully on shutdown signal

---

### C-08: StatsService

**Module**: `src/stats/`

**Purpose**: Query layer over the `request_logs` table, providing rolling-window aggregations for the statistics REST API.

**Responsibilities**:
- Execute rolling-window aggregation queries for summary, time-series, session, and cache statistics
- Support time windows: `1h`, `24h`, `7d`, `30d`
- Group statistics by provider, model, and session ID
- Return typed result structs suitable for JSON serialisation

---

### C-09: MetricsRegistry

**Module**: `src/metrics/`

**Purpose**: Maintains in-process Prometheus counters, histograms, and gauges. Renders the `/metrics` endpoint in Prometheus text exposition format.

**Responsibilities**:
- Define and register all `llmonitor_*` Prometheus metrics at startup
- Expose mutation methods called by the ProxyHandler after each completed request
- Render the full metrics snapshot as a Prometheus text string on demand
- Keep metrics in sync with DB-stored records (metrics are in-memory; DB is the source of truth for historical data)

---

### C-10: ConfigService

**Module**: `src/config/`

**Purpose**: DB-backed configuration store. All runtime configuration (provider API keys, proxy settings, admin credentials) is persisted in the SQLite `config` table.

**Responsibilities**:
- Read the current configuration from the `config` table (returning a typed `Config` struct)
- Write updated configuration entries
- Store and verify admin credentials (Argon2id hashing for passwords)
- Expose redacted configuration responses (API keys masked) for the GET config API
- Support atomic config updates via SQLite transactions

---

### C-11: AuthMiddleware

**Module**: `src/auth/`

**Purpose**: Axum middleware layer that enforces HTTP Basic Authentication on all `/api/*` routes. Proxy routes (`/v1/*`) and metrics (`/metrics`) bypass this layer.

**Responsibilities**:
- Extract and decode the `Authorization: Basic <base64>` header
- Delegate credential verification to `ConfigService`
- Return `401 Unauthorized` with `WWW-Authenticate` header on auth failure
- Implement progressive delay on repeated failures (brute-force mitigation)
- Pass request through on successful authentication

---

### C-12: FrontendAssets

**Module**: `src/proxy/frontend.rs` (or inline in router)

**Purpose**: Serves the compiled React frontend (TypeScript/Vite build output) as embedded static assets within the Rust binary.

**Responsibilities**:
- Embed the `frontend/dist/` directory into the binary at compile time via `rust-embed`
- Serve individual asset files by path (`/assets/*`, `/favicon.ico`, etc.)
- Serve `index.html` as a fallback for all non-asset, non-API routes (SPA client-side routing support)
- Set correct `Content-Type` headers per file extension

---

### C-13: AdminBootstrapService

**Module**: `src/config/bootstrap.rs`

**Purpose**: Startup routine that detects a fresh install (no admin record in DB) and initialises the admin credentials from environment variables.

**Responsibilities**:
- On startup, query the `config` table for an existing admin credential record
- If none found: read `LLMONITOR_ADMIN_USER` and `LLMONITOR_ADMIN_PASSWORD` from environment
- Hash the password with Argon2id and persist the credential record
- Abort startup with a clear error message if no admin exists AND the env vars are not set
