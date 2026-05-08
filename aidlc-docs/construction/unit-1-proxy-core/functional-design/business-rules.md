# Business Rules — Unit 1: Proxy Core

## BR-01: X-Provider Header is Mandatory

- Every request to `POST /v1/chat/completions` MUST include the `X-Provider` header
- If absent: return 400, OpenAI error type `invalid_request_error`, list all known providers in message
- Header value is case-insensitive; normalised to lowercase before matching

## BR-02: Provider Must Be Known

- The normalised provider name MUST match a variant in the `Provider` enum
- Unknown names return 400 with the list of supported provider names
- Supported providers list is derived from `Provider::all_names()` — never hardcoded in error strings

## BR-03: Provider Must Be Configured

- A known provider is only usable if its API key is present in the `Config` from `ConfigService`
- If API key is absent or empty string: return 503 (not 400 — the client is not at fault)
- Error message: "Provider '<name>' is not configured. Set its API key via the configuration API."

## BR-04: Model Name Must Be Accepted by Provider

- `request.model` MUST be in the provider's `accepted_models()` list
- Comparison is case-sensitive (model names are case-sensitive in practice)
- If model is not accepted: return 400 with the list of accepted models for that provider
- The accepted model list is owned by each `Provider` variant, not centralised

## BR-05: Request Body Must Be Valid

- `messages` field MUST be present and non-empty (minimum 1 message)
- `model` field MUST be present and non-empty string
- Messages MUST have valid `role` values: `"user"`, `"assistant"`, `"system"`
- Unknown top-level fields in the request body are captured in `extra_fields` and passed through (forward compatibility)
- Request body size MUST NOT exceed `max_body_size_bytes` from Config (default 10 MB)
- Violation: return 400 `invalid_request_error` with field-level detail

## BR-06: Global Rate Limit

- A single global token bucket is shared across ALL clients for the proxy API (`/v1/*`)
- Bucket capacity = `rate_limit_capacity` from Config (default: 60 requests)
- Refill rate = `rate_limit_refill_per_sec` from Config (default: 10 tokens/second)
- When bucket is empty: return 429, include `Retry-After` header in seconds until next refill
- The rate limit applies to the proxy endpoint only; management API (`/api/*`) is not rate-limited

## BR-07: Error Responses Are Always OpenAI-Format

- Every error response from the proxy MUST use the OpenAI error envelope format
- Internal error details (stack traces, file paths, database errors) MUST NOT appear in responses
- The `INTERNAL_ERROR` message is always the fixed string "An internal error occurred." regardless of actual cause
- Upstream provider errors that are safe to forward (client errors) MAY include the sanitised provider message

## BR-08: Health Check Degraded ≠ Unhealthy

- `GET /health` always returns HTTP 200 regardless of DB connectivity
- Status field in response body communicates actual health: `"ok"` or `"degraded"`
- This allows load balancers to keep the instance in rotation while operators are alerted by monitoring tools reading the JSON body

## BR-09: Streaming Requests Use Same Validation

- `stream: true` requests pass through the same validation steps (BR-01 through BR-05)
- The response is SSE format instead of JSON, but errors before streaming begins are still JSON OpenAI error format
- Once streaming has begun (first SSE chunk sent), errors mid-stream are logged server-side but cannot be sent as a well-formed error response

## BR-10: Request ID Propagation

- Every request receives a UUID v4 request ID from the middleware
- The request ID is set as `X-Request-Id` response header on all responses (including errors)
- The request ID is included in all server-side log entries for that request
- The request ID is stored in the `UsageRecord` for traceability (added by Unit 3)

## BR-11: Startup Is All-or-Nothing

- If DB connection fails at startup: process exits with non-zero exit code
- If migrations fail: process exits with non-zero exit code
- If admin bootstrap fails (no admin, no env vars): process exits with non-zero exit code and a clear error message
- The HTTP server MUST NOT start accepting connections until all startup steps succeed
