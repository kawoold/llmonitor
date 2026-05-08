# Requirements — llmonitor LLM Proxy

## Intent Analysis

| Field | Value |
|---|---|
| **User Request** | Build an LLM proxy that tracks token usage and model usage, provides statistics, uses the OpenAI API format, with provider-specific feature support. First provider: Claude (Anthropic). |
| **Request Type** | New Project (Greenfield) |
| **Scope Estimate** | System-wide — multi-component: Rust proxy service, React frontend, SQLite database |
| **Complexity Estimate** | Complex — provider abstraction layer, OpenAI compatibility shim, persistence, analytics, management UI |

---

## Functional Requirements

### FR-01: OpenAI-Compatible Proxy API

- The system MUST expose an HTTP API that is compatible with the OpenAI Chat Completions API (`POST /v1/chat/completions`)
- The proxy MUST accept requests in OpenAI request format and translate them to the appropriate upstream provider format
- The proxy MUST translate upstream provider responses back into OpenAI-compatible response format
- Clients MUST be able to use standard OpenAI SDK clients against this proxy with no modification beyond the base URL
- The proxy MUST forward all standard OpenAI fields (model, messages, temperature, max_tokens, stream, etc.)

### FR-02: Provider Routing

- The proxy MUST route requests to the appropriate backend provider based on a custom HTTP request header: `X-Provider: <provider-name>` (e.g. `X-Provider: claude`)
- Requests without an `X-Provider` header MUST receive a clear error response (400 Bad Request) indicating the required header
- Provider names MUST be case-insensitive
- Invalid or unsupported provider names MUST return a clear error response

### FR-03: Claude (Anthropic) Provider Integration

- The proxy MUST support routing requests to the Anthropic Claude API
- The proxy MUST translate OpenAI-format requests to Anthropic Messages API format and back
- The proxy MUST support Anthropic **prompt caching**:
  - Clients can pass cache control directives in a provider-specific extension field or header
  - The proxy forwards cache control metadata to Anthropic's API
  - The proxy tracks cache hit/miss status from Anthropic's response headers/metadata
  - Cache hit/miss rates are recorded per request and available in statistics
- The proxy MUST correctly map common OpenAI model aliases to Claude model IDs (e.g. routing requests for a "claude" model directly to Claude)

### FR-04: Streaming Support

- The proxy MUST support streaming responses using Server-Sent Events (SSE)
- Streaming support applies to both the client-facing OpenAI-compatible layer and the upstream provider connection
- Streaming is a secondary priority — non-streaming (batch) responses MUST work first
- Token counts for streaming responses MUST still be tracked (from the final `usage` chunk or inferred)

### FR-05: Token Usage Tracking

- The proxy MUST track the following fields for every request:
  - Timestamp (UTC)
  - Provider name
  - Model name (as returned by the provider)
  - Input token count (prompt tokens)
  - Output token count (completion tokens)
  - Total token count
  - Cache hit tokens (Anthropic prompt caching — tokens served from cache)
  - Cache miss tokens (Anthropic prompt caching — tokens not cached)
  - Request duration (milliseconds)
  - HTTP status code of upstream response
  - Conversation/session thread ID (derived from message history — see FR-06)
- All usage data MUST be persisted to SQLite

### FR-06: Session / Conversation Tracking

- The proxy MUST derive a conversation thread ID from the request body
- The thread ID is a deterministic hash of the conversation history (the `messages` array content, excluding the latest user message)
- Requests that share the same conversation history prefix MUST be grouped under the same thread ID
- The thread ID MUST be stored with each request record in the database
- Statistics endpoints MUST support filtering and aggregation by thread ID

### FR-07: Statistics REST API

- The proxy MUST expose a statistics REST API:
  - `GET /api/stats/summary` — overall totals (requests, tokens, by provider, by model)
  - `GET /api/stats/usage` — time-series usage data with rolling window filter
  - `GET /api/stats/sessions` — per-session aggregates (thread ID, request count, tokens)
  - `GET /api/stats/cache` — Claude prompt cache hit/miss rates and token savings
- Rolling window filters MUST support: `1h`, `24h`, `7d`, `30d`
- All statistics endpoints MUST return JSON

### FR-08: Prometheus Metrics

- The proxy MUST expose a Prometheus-compatible metrics endpoint at `GET /metrics`
- Metrics MUST include:
  - `llmonitor_requests_total` (counter, labels: provider, model, status)
  - `llmonitor_tokens_total` (counter, labels: provider, model, type=[input,output])
  - `llmonitor_request_duration_seconds` (histogram, labels: provider, model)
  - `llmonitor_cache_tokens_total` (counter, labels: type=[hit,miss]) — Anthropic only
  - `llmonitor_active_sessions` (gauge)

### FR-09: React Frontend (Configuration & Analytics Dashboard)

- The system MUST include a TypeScript + React web frontend
- The frontend MUST be served by the Rust backend (as static assets)
- The frontend MUST provide:
  - **Analytics Dashboard**: Visualize token usage over time, by provider, by model, by session; display rolling window statistics
  - **Cache Analytics**: Display Anthropic prompt cache hit/miss rates and token savings
  - **Configuration Management**: UI to view and modify proxy configuration (see FR-10)
- The frontend MUST authenticate to the management API using basic auth credentials

### FR-10: Configuration Management API

- All proxy configuration MUST be managed via a REST API and persisted in the SQLite database — no config files or environment variables for runtime configuration (except initial admin credentials)
- Configuration items include:
  - Upstream provider API keys (one per supported provider)
  - Admin credentials (username + password hash)
  - Provider-specific settings (e.g. default Claude model, cache control defaults)
  - Proxy behavior settings (e.g. request timeout, max body size)
- Configuration endpoints:
  - `GET /api/config` — retrieve current configuration (sensitive values redacted)
  - `PUT /api/config` — update configuration
  - `PUT /api/config/credentials` — update admin credentials
- All configuration API endpoints MUST require basic authentication (see NFR-SEC-01)

### FR-11: Provider Extensibility

- The provider abstraction layer MUST be designed to allow additional providers to be added without modifying the core proxy logic
- Claude is the only formally supported provider in the initial release; future providers (e.g. OpenAI direct, Google Gemini, AWS Bedrock, Ollama) can be added ad-hoc
- Each provider implementation MUST implement a common `Provider` trait covering: request translation, response translation, streaming, and usage extraction

---

## Non-Functional Requirements

### NFR-PERF-01: Proxy Latency Overhead
- The proxy MUST add less than 10ms of overhead (p99) to upstream provider latency under normal load
- Token tracking and persistence MUST be asynchronous — database writes MUST NOT block the response path

### NFR-PERF-02: Concurrency
- The proxy MUST handle concurrent requests using async I/O (Tokio runtime)
- The proxy MUST support at least 100 concurrent upstream connections without degradation

### NFR-STORE-01: SQLite Storage
- All persistent data (usage records, configuration, session data) MUST be stored in a single SQLite database file
- The SQLite file location MUST be configurable via environment variable at startup
- Database migrations MUST be applied automatically on startup using a migration framework (e.g. sqlx migrate)

### NFR-DEPLOY-01: Docker Container
- The system MUST be packaged as a Docker container
- The Dockerfile MUST produce a minimal production image (multi-stage build)
- The container MUST expose the proxy port and metrics port as configurable environment variables
- The SQLite database file MUST be mountable as a Docker volume for persistence

### NFR-OBS-01: Structured Logging
- All log output MUST be structured (JSON format in production)
- Every log entry MUST include: timestamp, request ID, log level, and message
- Secrets, API keys, and upstream provider credentials MUST NEVER appear in logs

---

## Security Requirements

### NFR-SEC-01: Management API Authentication
- The configuration and statistics management REST API endpoints (`/api/*`) MUST require HTTP Basic Authentication
- Admin credentials (username + password) MUST be configured on first startup
- Passwords MUST be stored as hashed values using an adaptive hashing algorithm (Argon2 or bcrypt)
- The management API MUST implement brute-force protection (rate limiting or progressive delay on failed auth)

### NFR-SEC-02: Proxy API (No Auth)
- The OpenAI-compatible proxy endpoint (`/v1/*`) does NOT require client authentication — it operates in trusted internal mode
- Rate limiting MUST still be applied to the proxy endpoint to prevent abuse

### NFR-SEC-03: Transport Security
- The proxy SHOULD support TLS termination (or be deployed behind a TLS-terminating reverse proxy)
- HSTS headers MUST be set when serving the frontend over HTTPS

### NFR-SEC-04: HTTP Security Headers
- All responses from the frontend-serving endpoints MUST include the required security headers:
  - `Content-Security-Policy: default-src 'self'`
  - `Strict-Transport-Security: max-age=31536000; includeSubDomains`
  - `X-Content-Type-Options: nosniff`
  - `X-Frame-Options: DENY`
  - `Referrer-Policy: strict-origin-when-cross-origin`

### NFR-SEC-05: Input Validation
- All API endpoints MUST validate request bodies using typed schema validation (serde + validator)
- Request body size MUST be limited (configurable, default 10MB)
- All database operations MUST use parameterized queries (no string concatenation)

### NFR-SEC-06: Secure Defaults
- No default credentials — admin credentials MUST be set on first run; the proxy MUST refuse to start without them configured
- Error responses MUST NOT expose stack traces, internal paths, or database details
- Upstream provider API keys MUST be redacted from all log output and statistics API responses

### NFR-SEC-07: Supply Chain
- Rust dependencies MUST use Cargo.lock (committed to version control)
- Frontend dependencies MUST use package-lock.json (committed to version control)
- No `latest` tags in the production Dockerfile
- A dependency vulnerability scan step MUST be included in the build instructions

---

## Constraints & Assumptions

- **Language**: Rust (backend) + TypeScript + React (frontend)
- **Framework**: Axum (Rust HTTP server)
- **Database**: SQLite via sqlx
- **Frontend build**: Vite or similar bundler, producing static assets served by Axum
- **Initial provider**: Anthropic Claude only
- **No cost tracking**: Token counts only; USD cost calculation is out of scope
- **Session identification**: Deterministic hash of message history prefix (no user-level auth)
- **Configuration**: Entirely DB-backed; initial admin credentials via environment variable on first startup only

---

## Out of Scope

- User-level authentication (multi-user, API key issuance)
- Cost / billing calculation
- Extended thinking / reasoning token support (deferred)
- Providers beyond Claude (deferred)
- Custom date range queries for statistics (deferred — rolling windows only)
- Multi-region or distributed deployment
