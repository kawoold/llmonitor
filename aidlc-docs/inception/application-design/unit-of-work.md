# Units of Work — llmonitor LLM Proxy

## Build Sequence

**Foundation-first**: Unit 1 → Unit 5 → Unit 2 → Unit 3 → Unit 4 → Unit 6

```
Unit 1: Proxy Core          (foundation — server, types, DB pool, migration runner)
    │
    ▼
Unit 5: Config Management   (admin auth, config store — required before API keys available)
    │
    ▼
Unit 2: Claude Adapter      (reads API key from config; end-to-end proxy now testable)
    │
    ▼
Unit 3: Token Tracking      (request_logs schema; hooks into proxy pipeline)
    │
    ▼
Unit 4: Statistics & Metrics (queries request_logs; Prometheus counters populated by pipeline)
    │
    ▼
Unit 6: React Frontend      (built last against stable API; docker-compose added here)
```

---

## Unit 1: Proxy Core

**Build Order**: 1st

**Purpose**: Foundation of the entire system. Establishes the Axum HTTP server, OpenAI API schema types, the Provider enum scaffold, the request/response pipeline, and all cross-cutting middleware. Also sets up the SQLite connection pool and migration runner (migrations themselves are added by Units 5 and 3).

**Components**:
- C-01 HttpServer (`src/main.rs`, `src/middleware/`)
- C-02 ProxyHandler (`src/proxy/handlers.rs`, `src/proxy/routing.rs`, `src/proxy/error.rs`)
- C-03 Provider enum scaffold (`src/providers/mod.rs`) — stub, no real provider yet
- C-12 FrontendAssets (`src/proxy/frontend.rs`) — placeholder until Unit 6 frontend is built
- DB pool setup (`src/db/mod.rs`) — SqlitePool, WAL mode, migration runner

**Infrastructure** (Unit 1 owns this):
- `Dockerfile` — multi-stage build (Rust builder + Node builder → minimal runtime image)
- Environment variables: `LLMONITOR_DB_PATH`, `LLMONITOR_PORT`, `LLMONITOR_ADMIN_USER`, `LLMONITOR_ADMIN_PASSWORD`
- Port exposure: configurable proxy port (default 8080)

**Deliverables**:
- Working Axum server that starts, runs migrations, and serves a `200 OK` on `GET /health`
- OpenAI request/response types (serde, validated)
- Middleware stack: request ID, rate limit (configurable), security headers (SECURITY-04)
- `POST /v1/chat/completions` endpoint wired up (returns 501 until Unit 2 adds real provider)
- `Dockerfile` with multi-stage build

**Definition of Done**:
- `cargo build` succeeds
- Server starts and passes `GET /health`
- `POST /v1/chat/completions` with unknown X-Provider header returns structured 400 error
- Docker image builds successfully

---

## Unit 5: Configuration Management

**Build Order**: 2nd

**Purpose**: DB-backed configuration store, Basic Auth enforcement on `/api/*` routes, Argon2id credential management, and admin bootstrap on first run. Must be built before Unit 2 because the Claude API key is read from the config store.

**Components**:
- C-10 ConfigService (`src/config/service.rs`)
- C-11 AuthMiddleware (`src/auth/middleware.rs`)
- C-13 AdminBootstrapService (`src/config/bootstrap.rs`)

**Database**:
- Migration `0001_config.sql` — creates the `config` table

**API Endpoints Added**:
- `GET /api/config` — return redacted current config (Basic Auth required)
- `PUT /api/config` — update provider API keys and proxy settings (Basic Auth required)
- `PUT /api/config/credentials` — update admin username/password (Basic Auth required)

**Deliverables**:
- Config table schema and migration
- ConfigService CRUD with Argon2id password hashing
- Basic Auth middleware applied to all `/api/*` routes
- Admin bootstrap: reads env vars on first start, hashes + stores credentials, refuses to start without them
- Brute-force protection on auth failures (progressive delay)

**Definition of Done**:
- Server refuses to start if no admin exists and env vars absent
- Admin credentials initialised from env vars on fresh DB
- `PUT /api/config` with Claude API key persists correctly
- `GET /api/config` returns redacted response (key masked)
- Invalid Basic Auth returns 401 with `WWW-Authenticate` header

---

## Unit 2: Claude Provider Adapter

**Build Order**: 3rd

**Purpose**: Concrete implementation of the Claude (Anthropic) provider. Reads the API key from ConfigService (populated in Unit 5), translates OpenAI requests to Anthropic format, handles prompt cache metadata, and supports both batch and streaming modes.

**Components**:
- C-04 ClaudeAdapter (`src/providers/claude/`)
- C-03 Provider enum — `Provider::Claude` variant wired in (replaces stub from Unit 1)

**Deliverables**:
- `AnthropicRequest` / `AnthropicResponse` / `AnthropicStreamEvent` types
- Full OpenAI→Anthropic request translation (messages, system prompt, model mapping)
- Full Anthropic→OpenAI response translation (content, stop reason, usage)
- Cache hit/miss token extraction from Anthropic usage fields
- Streaming SSE passthrough (lower priority — batch must work first)
- `Provider::from_name("claude")` resolves correctly, reads API key from ConfigService

**Definition of Done**:
- `POST /v1/chat/completions` with `X-Provider: claude` and valid API key in config returns a real Claude response
- Prompt cache hit/miss tokens extracted correctly from Anthropic response
- Anthropic API errors translated to structured OpenAI error format
- Streaming responses work end-to-end (SSE)

---

## Unit 3: Token Tracking & Session Engine

**Build Order**: 4th

**Purpose**: Persistent token usage recording. Hooks into the proxy pipeline (C-02 ProxyHandler) to capture `UsageRecord` per request via non-blocking mpsc channel, derives conversation session IDs, and writes to SQLite asynchronously.

**Components**:
- C-05 SessionEngine (`src/tracking/session.rs`)
- C-06 TrackingService (`src/tracking/service.rs`)
- C-07 TrackingWriterService (`src/tracking/writer.rs`)
- Shared types (`src/tracking/types.rs`)

**Database**:
- Migration `0002_request_logs.sql` — creates the `request_logs` table

**Deliverables**:
- `request_logs` schema and migration
- `SessionEngine::derive_session_id()` — SHA-256 hash of prior messages (property-based tested)
- `TrackingService` — Arc-wrapped mpsc sender, non-blocking `record()` method
- `TrackingWriterService` — background Tokio task, batch inserts (batch_size=50, flush_interval=500ms)
- ProxyHandler wired to call `TrackingService::record()` after each response
- Graceful drain on shutdown

**Definition of Done**:
- Requests to `/v1/chat/completions` result in `request_logs` rows in SQLite
- Session IDs are identical for requests sharing the same prior message history
- Single-turn requests produce a consistent nil session ID
- Writer drains remaining records on graceful shutdown
- Channel-full backpressure logged but request still completes

---

## Unit 4: Statistics & Prometheus Metrics

**Build Order**: 5th

**Purpose**: Analytics layer over the `request_logs` table. Exposes rolling-window aggregation queries via REST API and maintains in-process Prometheus counters updated by the proxy pipeline.

**Components**:
- C-08 StatsService (`src/stats/service.rs`)
- C-09 MetricsRegistry (`src/metrics/registry.rs`)

**API Endpoints Added**:
- `GET /api/stats/summary` — totals by provider and model (Basic Auth required)
- `GET /api/stats/usage` — time-series data points for rolling window (Basic Auth required)
- `GET /api/stats/sessions` — per-session aggregates (Basic Auth required)
- `GET /api/stats/cache` — Claude cache hit/miss rates and token savings (Basic Auth required)
- `GET /metrics` — Prometheus text exposition (no auth)

**Deliverables**:
- `StatsService` with `TimeWindow` enum and all four aggregation query methods
- `MetricsRegistry` with all `llmonitor_*` Prometheus metrics registered
- ProxyHandler wired to call `MetricsRegistry::record_request()` and `record_tokens()` after each response
- Rolling window filter: `1h`, `24h`, `7d`, `30d`
- All stats endpoints return correct JSON

**Definition of Done**:
- After proxying requests, `GET /api/stats/summary` reflects correct token totals
- `GET /metrics` returns valid Prometheus text format with all metric families
- Cache hit/miss rates reflect Anthropic response data
- All four rolling windows return plausible data

---

## Unit 6: React Frontend

**Build Order**: 6th (last)

**Purpose**: TypeScript + React SPA providing the analytics dashboard and configuration management UI. Built after all Rust API endpoints are stable. Compiled output embedded into the Rust binary via rust-embed. Also adds docker-compose and completes infrastructure documentation.

**Components**:
- React SPA (`frontend/` directory)
- C-12 FrontendAssets updated with real compiled output (replaces placeholder from Unit 1)

**Infrastructure** (Unit 6 adds):
- `docker-compose.yml` — local development stack (proxy + volume mount for SQLite)
- `.env.example` — full environment variable documentation
- `DEPLOYMENT.md` (or README section) — deployment instructions

**Frontend Features**:
- **Analytics Dashboard**: Token usage charts (by provider, model, rolling windows), request volume over time
- **Cache Analytics**: Anthropic prompt cache hit/miss rates, token savings visualisation
- **Configuration UI**: View and update provider API keys (masked), proxy settings, admin credentials
- **Auth**: Basic Auth credentials entered in UI; stored in browser session for API calls

**Deliverables**:
- Vite + React + TypeScript project scaffolded under `frontend/`
- API client layer (typed fetch wrappers for all `/api/*` endpoints)
- Analytics Dashboard with rolling window selector (1h/24h/7d/30d)
- Cache Analytics panel (Claude-specific)
- Configuration management forms
- `npm run build` produces `frontend/dist/` which is embedded into the Rust binary
- `docker-compose.yml` for local development
- `.env.example` with all required variables documented

**Definition of Done**:
- `npm run build` succeeds and `cargo build` embeds the frontend
- Dashboard shows real data from `GET /api/stats/*` endpoints
- Configuration can be read and updated via the UI
- UI enforces Basic Auth (prompts for credentials if not set)
- docker-compose brings up a working local stack
