# NFR Requirements — Unit 1: Proxy Core

## Performance

### PERF-01: Proxy Latency Overhead
- The proxy layer (routing, validation, middleware) MUST add less than 10ms overhead (p99) on top of upstream provider latency
- Measurement excludes network I/O to the upstream provider

### PERF-02: Concurrent Connections
- The server MUST handle at least 100 concurrent in-flight upstream requests without degradation
- Achieved via Tokio async I/O — no thread-per-request model

### PERF-03: Upstream Request Timeout
- Default: **30 seconds**
- Configurable via the configuration API (`PUT /api/config`, field: `upstream_timeout_secs`)
- On timeout: return 504 Gateway Timeout with OpenAI error format
- The timeout applies per-request from dispatch to final byte received from upstream

### PERF-04: Rate Limit Defaults
- Capacity: **100 tokens** (burst headroom)
- Refill rate: **20 tokens/second** (sustains ~1,200 RPM)
- Both configurable via the configuration API (`rate_limit_capacity`, `rate_limit_refill_per_sec`)
- Scope: global (single bucket across all clients)

---

## Security (SECURITY-01 through SECURITY-15 enforced)

### SEC-01: No Auth on Proxy Endpoint (by design)
- `POST /v1/chat/completions` requires no authentication — trusted internal deployment model
- Rate limiting (PERF-04) and body size limits serve as abuse mitigations in lieu of auth

### SEC-02: HTTP Security Headers (SECURITY-04)
- All responses from HTML-serving endpoints MUST include:
  - `Content-Security-Policy: default-src 'self'`
  - `Strict-Transport-Security: max-age=31536000; includeSubDomains`
  - `X-Content-Type-Options: nosniff`
  - `X-Frame-Options: DENY`
  - `Referrer-Policy: strict-origin-when-cross-origin`

### SEC-03: Input Validation (SECURITY-05)
- Request body size limit: **10 MB** default, configurable via `max_body_size_bytes`
- All request fields validated via serde deserialization + business rule checks (BR-01 through BR-05)
- No raw string concatenation into any query or command

### SEC-04: Error Hardening (SECURITY-09)
- All error responses use the OpenAI error envelope; internal details never exposed
- Global error handler catches any unhandled panics/errors and returns generic 500

### SEC-05: Request Tracing (SECURITY-03)
- Every request receives a UUID v4 request ID
- All log entries for a request include the request ID, timestamp, log level, and message
- Upstream provider API keys MUST NOT appear in any log output

### SEC-06: Supply Chain (SECURITY-10)
- `Cargo.lock` committed to version control
- All dependency versions pinned in `Cargo.toml`
- No `latest` tags permitted

---

## Reliability

### REL-01: Graceful Shutdown
- On SIGTERM/SIGINT: stop accepting new connections, allow in-flight requests to complete (drain period), then exit
- Drain timeout: 30 seconds maximum before forced exit

### REL-02: Fail-Fast Startup
- Server MUST NOT start accepting connections if DB connection, migrations, or admin bootstrap fail
- Exit code non-zero on any startup failure with a descriptive log message

### REL-03: Global Error Handler
- A top-level Axum error handler catches any unhandled panics and returns 500 with generic message
- Panic details logged at ERROR level with request ID (server-side only)

---

## Observability

### OBS-01: Structured Logging
- Log format: JSON in production (controlled by `RUST_LOG` environment variable)
- Default log level: INFO
- Log level configurable at startup only via `RUST_LOG` (not runtime-configurable)
- Required fields per log entry: `timestamp`, `level`, `request_id` (where applicable), `message`
- Sensitive values (API keys, passwords) MUST NOT appear in any log output

### OBS-02: Request Logging
- Every completed request logged at INFO with: method, path, status code, duration_ms, request_id
- Upstream provider errors logged at WARN with provider name and sanitised error (no key details)
- Internal errors logged at ERROR with request_id and error chain (no secrets)

---

## Maintainability

### MAINT-01: Dependency Management
- Rust edition: 2021
- `Cargo.lock` committed; `cargo audit` recommended in CI
- No unused dependencies

### MAINT-02: Test Coverage
- Unit tests for all business rules (BR-01 through BR-11)
- Property-based tests (per PBT partial extension) for request deserialisation round-trips
- Integration test: full startup → health check → proxy request flow (using mock upstream)
