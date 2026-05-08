# NFR Design Patterns — Unit 1: Proxy Core

## Pattern 1: Global Token Bucket Rate Limiter

**Addresses**: PERF-04, SECURITY-11, NFR-SEC-02

**Pattern**: Token Bucket (shared global state)

```
Design:
  - Single Arc<Mutex<TokenBucket>> held in AppState
  - TokenBucket fields: tokens (u64), capacity (u64), refill_rate (u64/sec), last_refill (Instant)
  - Middleware layer wraps the /v1/* router only

Per-request algorithm:
  1. Lock bucket
  2. Compute elapsed since last_refill; add (elapsed_secs * refill_rate) tokens, capped at capacity
  3. Update last_refill
  4. If tokens >= 1: subtract 1, release lock, continue
  5. If tokens == 0: release lock, return 429 with Retry-After header
     Retry-After = ceil(1.0 / refill_rate) seconds

Trade-offs:
  - Mutex contention is negligible at this request rate (bucket check is nanoseconds)
  - No external dependency (Redis, etc.) needed for a single-process proxy
  - Global scope means a single misbehaving client can exhaust the bucket; acceptable for trusted internal use
```

---

## Pattern 2: Exponential Backoff Retry

**Addresses**: REL-01 (reliability), PERF-03 (timeout awareness)

**Pattern**: Retry with Exponential Backoff (up to 3 attempts)

```
Applies to: upstream HTTP calls (5xx responses, connection errors, timeouts)
Does NOT apply to: 4xx upstream responses (client errors — retrying won't help)

Attempt schedule:
  Attempt 1: immediate
  Attempt 2: wait 500ms, retry
  Attempt 3: wait 1000ms, retry
  Attempt 4: wait 2000ms, retry
  → After 4th failure: propagate error to client as 502/504

Implementation:
  - Retry loop lives in the Provider dispatch layer (src/providers/mod.rs)
  - Each attempt checks total elapsed time against upstream_timeout_secs
  - If remaining time < next_delay: skip retry, return timeout error immediately
  - Jitter: add ±10% random jitter to each delay to prevent thundering herd

Retryable conditions:
  - HTTP 500, 502, 503, 529 from upstream
  - reqwest::Error (connection refused, reset, DNS failure)
  - Elapsed timeout before first byte received

Non-retryable conditions:
  - HTTP 400, 401, 403, 404, 422 (client errors)
  - HTTP 429 (rate limited by provider — propagate immediately with Retry-After)
  - Timeout during streaming after first byte received (partial response already sent)
```

---

## Pattern 3: Tower Middleware Stack (Layered Concerns)

**Addresses**: SECURITY-02, SECURITY-03, SECURITY-04, SEC-03, OBS-01, OBS-02

**Pattern**: Ordered Tower Layer composition

```
Layer order (outermost → innermost, applied top-to-bottom):

1. TraceLayer (tower-http)
   - Logs request start (method, path, request_id) at DEBUG
   - Logs request end (status, duration_ms) at INFO
   - Attaches tracing span to async task for structured log correlation

2. SetRequestIdLayer (tower-http)
   - Generates UUID v4 if X-Request-Id not present in incoming request
   - Sets X-Request-Id on both request (for handlers) and response (for clients)

3. SetResponseHeaderLayer × 5 (tower-http)
   - Content-Security-Policy: default-src 'self'
   - Strict-Transport-Security: max-age=31536000; includeSubDomains
   - X-Content-Type-Options: nosniff
   - X-Frame-Options: DENY
   - Referrer-Policy: strict-origin-when-cross-origin
   - Applied unconditionally to all responses

4. DefaultBodyLimit (axum)
   - Rejects bodies > max_body_size_bytes before reaching handler
   - Returns 413 Payload Too Large

5. CompressionLayer (tower-http, disabled by default)
   - Opt-in via Accept-Encoding header
   - Useful for stats API responses

Router split (after common layers):
  /v1/*     → RateLimitLayer → ProxyHandler
  /api/*    → AuthLayer      → Management handlers
  /metrics  → MetricsHandler (no extra layer)
  /health   → HealthHandler  (no extra layer)
  /*        → FrontendAssets (no extra layer)
```

---

## Pattern 4: Structured Logging with Span Propagation

**Addresses**: SECURITY-03, OBS-01, OBS-02

**Pattern**: `tracing` spans with `tower-http` TraceLayer

```
Span lifecycle per request:
  1. TraceLayer creates a root span: { request_id, method, path }
  2. All log macros within async handler context inherit this span
  3. Any tokio::spawn within the request context propagates span via .instrument()
  4. On response: span closes, duration and status emitted

Log levels:
  TRACE: internal routing decisions (dev only)
  DEBUG: request/response headers (dev only, never in production due to key exposure risk)
  INFO:  completed requests, startup events, config changes
  WARN:  upstream errors, rate limit hits, retry attempts
  ERROR: internal panics, DB errors, startup failures

Secrets guard:
  - Provider API keys MUST use a wrapper type (ApiKey(String)) with a custom Debug impl
    that always emits "ApiKey([REDACTED])" — prevents accidental logging via {:?}
  - Password hashes never stored in AppState; only in ConfigService (Unit 5)
```

---

## Pattern 5: Fail-Safe Global Error Handler

**Addresses**: SECURITY-15, REL-03

**Pattern**: Axum `HandleErrorLayer` + `std::panic::catch_unwind` at entry point

```
Two-level error safety:

Level 1 — Handler errors (ProxyError):
  - ProxyError implements IntoResponse
  - Converts to appropriate HTTP status + OpenAI error body
  - Request ID header always set even on error responses

Level 2 — Unhandled panics:
  - tokio::task panics are caught by the Tokio runtime per-task
  - Axum's HandleErrorLayer converts tower Service errors to 500 responses
  - Global panic hook set at startup: logs panic info at ERROR level (with request_id if available)
  - Response: 500 with generic {"error": {"type": "api_error", "message": "An internal error occurred."}}
  - No panic details in response body
```

---

## Pattern 6: Graceful Shutdown with Drain

**Addresses**: REL-01

**Pattern**: Tokio signal handling + Tower `ServiceExt::ready` drain

```
Shutdown sequence:
  1. tokio::signal::ctrl_c() or SIGTERM triggers shutdown channel
  2. Axum server stops accepting new TCP connections
  3. In-flight requests continue with a 30s drain deadline
  4. After drain (or deadline): drop AppState
  5. Dropping Arc<TrackingService> closes the mpsc sender
  6. TrackingWriterService detects closed channel, flushes remaining records, exits
  7. SqlitePool drops, closes all connections cleanly
  8. Process exits with code 0
```
