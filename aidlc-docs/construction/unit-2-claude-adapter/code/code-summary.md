# Code Summary — Unit 2: Claude Provider Adapter

## Files Created

- `src/providers/claude/types.rs` — All Anthropic API types (request, response, error), `MODEL_ALIAS_MAP`, `resolve_model()`, `AnthropicCacheControlHint`, and constants (`ANTHROPIC_API_URL`, `ANTHROPIC_VERSION`, `DEFAULT_MAX_TOKENS`)

## Files Modified

- `src/providers/claude/mod.rs` — Full `ClaudeAdapter` implementation: `translate_request()` (pure, all BR-U2 rules), `translate_response()` (pure), `send()` (async HTTP with error mapping and INFO logging), plus 15 inline unit tests
- `src/providers/mod.rs` — Real `chat_completion()` with 3-attempt retry loop (exponential backoff + OsRng jitter), `accepted_models()` expanded to include all 8 aliases alongside 8 canonical IDs
- `src/proxy/handlers.rs` — Added `stream == Some(true)` → `ProxyError::StreamingNotSupported` guard (BR-U2-05)
- `src/proxy/error.rs` — Added `StreamingNotSupported` variant (→ HTTP 501)
- `tests/proxy_validation_test.rs` — Added `streaming_request_returns_501` integration test

## Test Results

**54 tests total — all passing, zero warnings**

| Suite | Tests |
|---|---|
| `providers::claude` unit tests | 15 |
| Other unit tests (config, rate limit, proxy, types) | 29 |
| `auth_test` integration | 3 |
| `config_api_test` integration | 3 |
| `health_test` integration | 1 |
| `proxy_validation_test` integration | 3 |

## Business Rules Covered

| Rule | Test |
|---|---|
| BR-U2-01: System messages concatenated | `test_translate_request_system_concat` |
| BR-U2-02: Default max_tokens=4096 | `test_translate_request_default_max_tokens` |
| BR-U2-03: Model alias resolution | `test_translate_request_model_alias`, `test_resolve_model_alias` |
| BR-U2-04: Cache control at specified positions | `test_translate_request_cache_control_*` (3 tests) |
| BR-U2-05: Streaming → 501 | `streaming_request_returns_501` |
| BR-U2-06: Retry on ServerError/RateLimited only | Logic in `Provider::chat_completion()` |
| BR-U2-07: Anthropic version header | Set in `ClaudeAdapter::send()` |
| BR-U2-08: API key never logged | `ApiKey` Debug/Display redact (Unit 1); never passed to tracing fields |
| BR-U2-09: Cache tokens best-effort | `test_translate_response_cache_tokens_absent` |
| BR-U2-10: Response id = chatcmpl-{uuid} | `test_translate_response_id_format` |
| BR-U2-11: Strict role mapping | `test_translate_request_unknown_role` |
| BR-U2-12: Stop reason mapping | `test_translate_response_stop_reasons` |

## Deviations from Plan

- `ContentPart` import unused in test module (removed to eliminate warning)
- `AnthropicRequest` and `AnthropicSystem` required explicit `#[derive(Clone)]` (not noted in plan — needed for retry loop cloning)
- `ApiKey::as_str()` does not exist; used `api_key.0.as_str()` directly
- Disk full during initial build — `cargo clean` freed 3.1GB before successful compile
