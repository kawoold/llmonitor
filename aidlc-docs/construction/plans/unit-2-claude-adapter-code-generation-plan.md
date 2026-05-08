# Unit 2: Claude Provider Adapter — Code Generation Plan

## Unit Context

**Goal**: Replace the Unit 1 `ClaudeAdapter` stub with a complete implementation that translates OpenAI requests → Anthropic API and back, with retry logic, cache control support, and proper error mapping.

**Dependencies**: Unit 1 types (`OpenAiChatRequest`, `OpenAiChatResponse`, `RawUsage`, `ApiKey`, `ProxyError`, `ProviderError`), Unit 5 (`ConfigService` for API key retrieval)

**Files to create**:
- `src/providers/claude/types.rs` (new)

**Files to modify**:
- `src/providers/claude/mod.rs` (stub → full implementation)
- `src/providers/mod.rs` (stub `chat_completion()` → real retry logic; `accepted_models()` fix)
- `src/proxy/handlers.rs` (add stream=true → 501 check)

**Files to add tests to**:
- `src/providers/claude/mod.rs` (inline `#[cfg(test)]` module)
- `tests/proxy_validation_test.rs` (streaming 501 integration test)

**Documentation**:
- `aidlc-docs/construction/unit-2-claude-adapter/code/code-summary.md` (new)

---

## Steps

### Step 1: Create `src/providers/claude/types.rs` — Anthropic API types
- [x] Define `AnthropicRequest`, `AnthropicSystem` (untagged enum), `AnthropicMessage`, `AnthropicContentBlock`, `AnthropicCacheControl`
- [x] Define `AnthropicResponse`, `AnthropicResponseBlock`, `AnthropicUsage`
- [x] Define `AnthropicErrorResponse`, `AnthropicErrorDetail`
- [x] Define `AnthropicCacheControlHint` (parsed from `extra["anthropic_cache_control"]`)
- [x] Add `MODEL_ALIAS_MAP` constant (8 entries)
- [x] Add `resolve_model(name: &str) -> &str` function
- [x] Add constants: `ANTHROPIC_API_URL`, `ANTHROPIC_VERSION`, `DEFAULT_MAX_TOKENS`

### Step 2: Implement `ClaudeAdapter` in `src/providers/claude/mod.rs`
- [x] Add `reqwest::Client` field to struct; update `new()` to construct client
- [x] Implement `translate_request(req: &OpenAiChatRequest) -> Result<AnthropicRequest, ProviderError>`:
  - Parse `anthropic_cache_control` from `req.extra` — fail with `ClientError` if malformed JSON
  - Extract and concatenate system messages (BR-U2-01)
  - Resolve model alias (BR-U2-03)
  - Translate remaining messages — reject unknown roles with `ClientError` (BR-U2-11)
  - Apply cache control hints: system=true → last system block; message indices → last block of that message (BR-U2-04)
  - Map parameters: `max_tokens` default 4096 (BR-U2-02), `temperature`, `top_p`, `stop`
- [x] Implement `translate_response(resp: AnthropicResponse) -> (OpenAiChatResponse, RawUsage)`:
  - Concatenate text blocks from `content`
  - Map `stop_reason` → `finish_reason` (BR-U2-12)
  - Build response with `chatcmpl-{uuid_v4}` ID (BR-U2-10)
  - Build `RawUsage` with best-effort cache tokens (BR-U2-09)
- [x] Implement `async send(&self, req: AnthropicRequest) -> Result<AnthropicResponse, ProviderError>`:
  - POST to `ANTHROPIC_API_URL` with headers: `x-api-key`, `anthropic-version`, `content-type`, `accept` (BR-U2-07)
  - Handle response status: 200→Ok, 401→AuthFailed, 400→ClientError(msg), 429/529→RateLimited, 5xx→ServerError, timeout→Timeout
  - Log at INFO: model + HTTP status + token counts (NFR-U2-O01); never log body (NFR-U2-S02)

### Step 3: Update `src/providers/mod.rs` — retry logic and model acceptance
- [x] Expand `accepted_models()` to include all alias names so `provider.accepts_model("claude-sonnet-4")` returns `true`
- [x] Implement real `chat_completion()` with retry loop:
  - Clone request per attempt
  - Call `ClaudeAdapter::translate_request()` + `send()`
  - On success: call `translate_response()` and return
  - On retryable error (`ServerError` or `RateLimited`) and attempts < 3: sleep with jitter then retry
  - On non-retryable error or exhausted retries: return error
  - Log WARN per retry attempt; ERROR on final failure

### Step 4: Update `src/proxy/handlers.rs` — stream=true guard
- [x] In `chat_completion()` handler, check `request.stream == Some(true)` before provider dispatch; if true, return `ProxyError::StreamingNotSupported` (or inline 501 JSON response) per BR-U2-05

### Step 5: Unit tests in `src/providers/claude/mod.rs`
- [x] `test_resolve_model_alias()` — verify aliases map to canonical IDs
- [x] `test_resolve_model_passthrough()` — verify unknown names pass through unchanged
- [x] `test_translate_request_system_concat()` — multiple system messages concatenated (BR-U2-01)
- [x] `test_translate_request_default_max_tokens()` — max_tokens defaulted to 4096 (BR-U2-02)
- [x] `test_translate_request_model_alias()` — alias resolved in output (BR-U2-03)
- [x] `test_translate_request_unknown_role()` — returns `ClientError` (BR-U2-11)
- [x] `test_translate_request_cache_control_system()` — system=true adds cache_control to last system block (BR-U2-04)
- [x] `test_translate_request_cache_control_message()` — message index adds cache_control to last content block (BR-U2-04)
- [x] `test_translate_request_cache_control_oob()` — out-of-range index silently ignored (BR-U2-04)
- [x] `test_translate_request_malformed_cache_control()` — malformed JSON returns `ClientError` (NFR-U2-S03)
- [x] `test_translate_response_stop_reasons()` — all stop_reason mappings (BR-U2-12)
- [x] `test_translate_response_id_format()` — id starts with "chatcmpl-" (BR-U2-10)
- [x] `test_translate_response_cache_tokens_absent()` — absent cache fields treated as 0 (BR-U2-09)

### Step 6: Integration test — streaming 501
- [x] In `tests/proxy_validation_test.rs`, add test: POST `/v1/chat/completions` with `stream: true` → expect 501 response

### Step 7: Verify compilation and run tests
- [x] Run `cargo build` — confirm zero errors
- [x] Run `cargo test` — confirm all tests pass (unit + integration)
- [x] Note any deviations from plan

### Step 8: Write code summary
- [x] Create `aidlc-docs/construction/unit-2-claude-adapter/code/code-summary.md` — list all created/modified files, test count, and any deviations
