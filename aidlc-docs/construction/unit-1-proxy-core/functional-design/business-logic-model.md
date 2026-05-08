# Business Logic Model — Unit 1: Proxy Core

## 1. Request Pipeline

### 1.1 Non-Streaming Request Flow

```
Incoming POST /v1/chat/completions
        │
        ▼
[MW-1] Request ID Injection
  - Generate UUID v4 request ID
  - Attach to request as extension
  - Set X-Request-Id response header
        │
        ▼
[MW-2] Global Rate Limit Check
  - Check global token bucket (shared across all clients)
  - If bucket empty: return 429 Too Many Requests
      body: OpenAI error { type: "rate_limit_error", message: "Rate limit exceeded" }
      header: Retry-After: <seconds until bucket refills>
  - If bucket has tokens: consume one token, continue
        │
        ▼
[MW-3] Request Body Size Guard
  - If Content-Length or streaming body exceeds max_body_size: return 413 Payload Too Large
        │
        ▼
[HANDLER] ProxyHandler::chat_completion()
  │
  ├─► Step 1: Deserialize & Validate Request Body
  │     - Parse JSON into OpenAiChatRequest
  │     - Validate required fields: messages (non-empty), model (non-empty string)
  │     - On parse failure: return 400 with OpenAI error body
  │
  ├─► Step 2: Resolve Provider
  │     - Extract X-Provider header value
  │     - If header absent: return 400 with "Missing X-Provider header" + supported providers list
  │     - Normalise to lowercase
  │     - Look up provider by name in Provider enum
  │     - If unrecognised: return 400 with "Unknown provider" + supported providers list
  │
  ├─► Step 3: Validate Model Name
  │     - Ask Provider for its list of accepted model names
  │     - If request.model not in accepted list: return 400 with accepted model names
  │     - (Accepted models are defined per-provider in the Provider enum)
  │
  ├─► Step 4: Dispatch to Provider
  │     - Call Provider::chat_completion(request)
  │     - On provider error: translate ProviderError → OpenAI error response
  │         4xx upstream → 400/401/429 to client (pass status through)
  │         5xx upstream → 502 Bad Gateway
  │         timeout      → 504 Gateway Timeout
  │
  ├─► Step 5: Record Timing
  │     - Calculate request duration from start time to response received
  │
  └─► Step 6: Return Response
        - Serialize OpenAiChatResponse as JSON
        - Set Content-Type: application/json
        - Set X-Request-Id header
        - Return 200 OK
```

### 1.2 Streaming Request Flow

Identical to non-streaming through Steps 1–3. Step 4 differs:

```
  ├─► Step 4 (Stream): Dispatch to Provider (streaming mode)
  │     - Call Provider::chat_completion_stream(request) → SSE stream
  │     - Wrap stream as Axum SSE response
  │     - Forward each OpenAiStreamChunk as `data: <json>\n\n`
  │     - Terminate stream with `data: [DONE]\n\n`
  │     - Duration measured until final chunk received
  │     - Usage record built from final `usage` chunk
```

---

## 2. Provider Resolution Algorithm

```
Input: X-Provider header value (string or absent)

1. If absent:
   → Error: PROVIDER_HEADER_MISSING
   → Response: 400, body lists all known provider names

2. Normalise: lowercase(trim(value))

3. Match against known providers:
   "claude" → Provider::Claude
   (future: "openai", "gemini", "bedrock", ...)
   no match → Error: PROVIDER_UNKNOWN
   → Response: 400, body lists all known provider names

4. Construct Provider variant:
   - Read provider API key from ConfigService
   - If API key absent/empty → Error: PROVIDER_NOT_CONFIGURED
   → Response: 503 Service Unavailable, "Provider not configured"
```

---

## 3. Model Validation Algorithm

```
Input: request.model (string), resolved Provider

1. Ask Provider::accepted_models() → Vec<String>

2. If request.model in accepted_models:
   → Continue

3. If not found:
   → Error: MODEL_NOT_ACCEPTED
   → Response: 400
   → body: {
       "error": {
           "type": "invalid_request_error",
           "param": "model",
           "message": "Model '<model>' is not accepted for provider '<provider>'.
                       Accepted models: <comma-separated list>"
       }
   }
```

---

## 4. Error Translation Model

All errors returned to clients use the OpenAI error envelope:

```json
{
  "error": {
    "type":    "<error_type>",
    "message": "<human-readable message>",
    "param":   "<field name, if applicable>",
    "code":    "<optional machine-readable code>"
  }
}
```

| Proxy Error | HTTP Status | type | message |
|---|---|---|---|
| PROVIDER_HEADER_MISSING | 400 | invalid_request_error | Missing required header: X-Provider. Supported providers: [list] |
| PROVIDER_UNKNOWN | 400 | invalid_request_error | Unknown provider '<x>'. Supported providers: [list] |
| PROVIDER_NOT_CONFIGURED | 503 | api_error | Provider '<x>' is not configured. Set its API key via the configuration API. |
| MODEL_NOT_ACCEPTED | 400 | invalid_request_error | Model '<m>' is not accepted for provider '<p>'. Accepted models: [list] |
| REQUEST_PARSE_ERROR | 400 | invalid_request_error | Invalid request body: <detail> |
| RATE_LIMIT_EXCEEDED | 429 | rate_limit_error | Rate limit exceeded. Retry after <n> seconds. |
| UPSTREAM_CLIENT_ERROR | 400–422 | invalid_request_error | <forwarded provider message, sanitised> |
| UPSTREAM_AUTH_ERROR | 401 | authentication_error | Provider authentication failed. Check your API key configuration. |
| UPSTREAM_RATE_LIMIT | 429 | rate_limit_error | Upstream provider rate limit exceeded. |
| UPSTREAM_SERVER_ERROR | 502 | api_error | Upstream provider returned an error. |
| UPSTREAM_TIMEOUT | 504 | api_error | Upstream provider timed out. |
| INTERNAL_ERROR | 500 | api_error | An internal error occurred. |

**Rule**: Error messages sent to clients MUST NOT include stack traces, internal paths, or database details. The INTERNAL_ERROR message is always generic.

---

## 5. Health Check Logic

```
GET /health

1. Ping SQLite: execute "SELECT 1"
   - Success: db_status = "connected"
   - Failure: db_status = "error"

2. Calculate uptime_secs from server start timestamp

3. Determine overall status:
   - If db_status == "connected": status = "ok", HTTP 200
   - If db_status == "error":    status = "degraded", HTTP 200
     (still return 200 so load balancers don't kill a temporarily degraded instance;
      monitoring tools read the JSON body)

4. Response body:
{
  "status":      "ok" | "degraded",
  "db":          "connected" | "error",
  "uptime_secs": <integer>
}
```

---

## 6. Startup Sequence

```
1. Load environment variables (DB path, admin credentials for bootstrap)
2. Open SQLite connection pool (WAL mode, pragmas)
3. Run database migrations (sqlx::migrate!())
4. AdminBootstrapService::bootstrap() — ensure admin exists
5. Load ConfigService (reads config from DB)
6. Initialise TrackingService channel (mpsc, bounded)
7. Spawn TrackingWriterService background task
8. Initialise MetricsRegistry
9. Build AppState (Arc<AppState>)
10. Build Axum router (mount all routes + middleware)
11. Bind TCP listener on configured port
12. Await shutdown signal (SIGTERM / SIGINT / ctrl-c)
13. Graceful shutdown: stop accepting new connections, drain in-flight requests, drop TrackingService sender (triggers writer drain)
```
