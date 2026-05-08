# Logical Components — Unit 1: Proxy Core

## LC-01: Global Token Bucket

**Type**: In-process shared state component
**Location**: `src/middleware/rate_limit.rs`

```
TokenBucket {
  tokens:      u64          -- current available tokens
  capacity:    u64          -- max tokens (from Config)
  refill_rate: u64          -- tokens/second (from Config)
  last_refill: Instant
}

Shared as: Arc<Mutex<TokenBucket>> in AppState
Middleware: RateLimitMiddleware wraps /v1/* subrouter only
```

Interactions:
- Reads `rate_limit_capacity` and `rate_limit_refill_per_sec` from `Config` at startup
- Updated on config change requires server restart (simple; no runtime reload needed for rate limits)

---

## LC-02: Retry Executor

**Type**: Logic component (no separate struct; embedded in Provider dispatch)
**Location**: `src/providers/mod.rs` — `Provider::dispatch_with_retry()`

```
RetryConfig {
  max_attempts:    u8         -- 4 (1 initial + 3 retries)
  delays_ms:       [500, 1000, 2000]
  jitter_pct:      10         -- ±10% random jitter
  retryable_codes: [500, 502, 503, 529]
}

State per call:
  attempt: u8
  total_elapsed: Duration
  last_error: ProviderError
```

Interactions:
- Wraps `ClaudeAdapter::send()` (Unit 2)
- Checks remaining timeout budget before each retry delay
- Logs each retry attempt at WARN level with attempt number and delay

---

## LC-03: Tower Middleware Stack

**Type**: Composed Tower layers
**Location**: `src/main.rs` — `build_router()`

```
Stack composition (pseudocode):
  Router::new()
    .layer(TraceLayer::new_for_http())
    .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
    .layer(SetResponseHeaderLayer::if_not_present(CONTENT_SECURITY_POLICY, ...))
    .layer(SetResponseHeaderLayer::if_not_present(STRICT_TRANSPORT_SECURITY, ...))
    .layer(SetResponseHeaderLayer::if_not_present(X_CONTENT_TYPE_OPTIONS, ...))
    .layer(SetResponseHeaderLayer::if_not_present(X_FRAME_OPTIONS, ...))
    .layer(SetResponseHeaderLayer::if_not_present(REFERRER_POLICY, ...))
    .layer(DefaultBodyLimit::max(config.max_body_size_bytes))
```

Sub-router for `/v1/*`:
```
  proxy_router
    .layer(RateLimitMiddlewareLayer::new(Arc::clone(&state.rate_limiter)))
```

Sub-router for `/api/*`:
```
  api_router
    .layer(BasicAuthLayer::new(Arc::clone(&state.config)))
    -- added in Unit 5
```

---

## LC-04: Request ID Propagator

**Type**: Tower layer + request extension
**Location**: tower-http `SetRequestIdLayer`

```
- Generates UUID v4 for each request lacking X-Request-Id
- Stores as request extension: req.extensions().get::<RequestId>()
- Sets X-Request-Id on response headers
- tracing span includes request_id field automatically via TraceLayer
```

---

## LC-05: Structured Log Emitter

**Type**: Process-global subscriber (configured once at startup)
**Location**: `src/main.rs` — init before server start

```
Subscriber stack:
  tracing_subscriber::registry()
    .with(EnvFilter::from_default_env())   -- reads RUST_LOG
    .with(fmt::layer().json())             -- JSON output to stdout
    .init()

All log output goes to stdout (12-factor app); container runtime captures it.
```

Log fields always present:
  `timestamp`, `level`, `target` (module path), `message`

Log fields present when in request context:
  `request_id`, `method`, `path`

---

## LC-06: Graceful Shutdown Coordinator

**Type**: Async task coordination
**Location**: `src/main.rs`

```
Components:
  - shutdown_signal() future: resolves on SIGTERM or ctrl-c
  - axum::serve(...).with_graceful_shutdown(shutdown_signal())
  - 30-second drain enforced by tokio::time::timeout wrapping the serve future

Shutdown propagation chain:
  signal → serve stops accepting → in-flight complete → AppState drop →
  TrackingService drop → mpsc sender close → writer task drains → SqlitePool close → exit 0
```

---

## LC-07: ApiKey Redaction Wrapper

**Type**: Newtype wrapper
**Location**: `src/config/types.rs` (used across units)

```rust
pub struct ApiKey(String);

impl std::fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ApiKey([REDACTED])")
    }
}

impl std::fmt::Display for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[REDACTED]")
    }
}
```

All provider API keys stored as `ApiKey` in `Config`; prevents accidental key logging via `{:?}` or `{}`.

---

## Component Interaction Map

```
Incoming request
    │
    ▼
LC-03 Tower Stack
    ├── LC-04 Request ID  (attach UUID to request + response)
    ├── LC-05 Log Emitter (open tracing span)
    ├── Security headers  (set on all responses)
    └── Body size guard   (reject > max_body_size)
            │
            ▼ (for /v1/* only)
        LC-01 Token Bucket
            │ (token available)
            ▼
        ProxyHandler
            ├── validation (BR-01–BR-05)
            ├── LC-02 Retry Executor → Provider dispatch
            └── LC-07 ApiKey (read safely, never logged)
                        │
                        ▼
                    Upstream API
            │
            ▼ (on error OR success)
        LC-05 Log Emitter (close span, emit duration + status)
            │
            ▼
LC-06 Shutdown Coordinator
    (drains when signal received)
```
