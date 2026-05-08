# NFR Requirements — Unit 2: Claude Provider Adapter

## Performance

### NFR-U2-P01: Anthropic API Timeout
- All outbound HTTP requests to Anthropic MUST respect the `timeout` value from `Config` (default 30s, established in Unit 1).
- Requests exceeding the timeout MUST be aborted and result in `ProviderError::Timeout`.

### NFR-U2-P02: Connection Pooling
- `reqwest::Client` MUST be constructed once and held in `ClaudeAdapter` (not created per request).
- Default reqwest connection pool settings are used (no explicit `max_connections_per_host` limit).
- This ensures TCP connections are reused across requests within the same adapter instance.

---

## Reliability

### NFR-U2-R01: Retry Behavior
- Retry logic lives in `Provider::chat_completion()`, not inside `ClaudeAdapter`.
- 3 attempts maximum; exponential backoff: 500ms → 1000ms → 2000ms with ±10% uniform jitter.
- Retry only on `ProviderError::ServerError(_)` and `ProviderError::RateLimited`.
- Never retry on `AuthFailed`, `ClientError(_)`, or `Timeout`.

### NFR-U2-R02: Idempotent Request Construction
- `translate_request()` is a pure function (no I/O, no side effects).
- The retry loop MAY clone the `AnthropicRequest` before each attempt without observable side effects.

### NFR-U2-R03: Out-of-Range Cache Index Handling
- Out-of-range indices in `anthropic_cache_control.messages` MUST be silently ignored (per BR-U2-04).

---

## Security

### NFR-U2-S01: API Key Redaction in Logs
- The Anthropic API key MUST NOT appear in any log line (inherited from Unit 1 LC-07 / BR-U2-08).
- `ApiKey` newtype implements `Debug`/`Display` as `[REDACTED]`.

### NFR-U2-S02: Prompt Privacy in Logs
- Request and response bodies MUST NOT be logged at any log level.
- Only model name, HTTP status code, and token counts (prompt/completion/cache) are logged.

### NFR-U2-S03: Malformed Cache Control Extension
- If the `anthropic_cache_control` JSON field fails deserialization, the adapter MUST return `ProviderError::ClientError` with a descriptive message (surfaces as HTTP 400 to the client).
- This prevents silent behaviour on typos in provider-specific extension fields.

---

## Observability

### NFR-U2-O01: Outbound Request Logging (Minimal)
At `INFO` level, log on each Anthropic API call:
- Model name resolved (after alias expansion)
- HTTP status code received
- Token counts: `input_tokens`, `output_tokens`, `cache_read_input_tokens` (if non-zero), `cache_creation_input_tokens` (if non-zero)

At `WARN` level, log:
- Each retry attempt number, delay applied, and the error that triggered it

At `ERROR` level, log:
- Final failure after all retry attempts exhausted

### NFR-U2-O02: Span Integration
- `ClaudeAdapter::send()` SHOULD be instrumented with a `tracing::instrument` span containing model name and attempt number.

---

## Maintainability

### NFR-U2-M01: Translation Functions Are Pure
- `translate_request()` and `translate_response()` must remain pure (no I/O, no global state).
- This makes them unit-testable without network mocking.

### NFR-U2-M02: No New Crate Dependencies for Jitter
- Jitter for retry delays is computed using `rand_core::OsRng` (already a direct dependency) without adding the `rand` crate.
- Implementation: generate a `u32` from `OsRng`, map to `[0.9, 1.1]` via modulo arithmetic.
