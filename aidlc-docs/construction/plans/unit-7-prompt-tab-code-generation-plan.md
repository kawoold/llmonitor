# Unit 7: Prompt Tab — Code Generation Plan

## Unit Context

**Unit**: Unit 7 — Prompt Tab
**Application Code Root**: `/home/kawoold/Development/rust/llmonitor/`
**Frontend Root**: `frontend/`
**Documentation**: `aidlc-docs/construction/unit-7-prompt-tab/code/`

## Key Design Decisions

- Streaming via `ClaudeAdapter::send_stream()` → raw `reqwest::Response` piped to Axum body
- Usage extracted inline from SSE stream (parses `message_start` + `message_delta` events)
- Basic auth middleware added to `/v1/chat/completions` proxy route
- `GET /api/models` wired into auth-protected management router
- Frontend: `fetch` + `ReadableStream` + `AbortController` for streaming
- `react-markdown` for assistant message rendering

## Steps

### Step 1: Add `ModelOption` to `src/proxy/types.rs`
- [x] Add `ModelOption` struct (serializable) with `provider`, `model`, `display_name` fields

### Step 2: Create `src/proxy/models.rs`
- [x] Write `get_models()` async handler returning `Json<Vec<ModelOption>>` with hardcoded 3-model list
- [x] No state required — pure handler

### Step 3: Implement `ClaudeAdapter::send_stream()` in `src/providers/claude/mod.rs`
- [x] Add `send_stream(req: AnthropicRequest, timeout_secs: u64) -> Result<reqwest::Response, ProviderError>`
- [x] Sends to Anthropic with `stream: true` in the JSON body
- [x] Returns the raw `reqwest::Response` without consuming the body (caller streams it)
- [x] Update `AnthropicRequest` in `src/providers/claude/types.rs` to include `stream: bool` field

### Step 4: Implement streaming handler in `src/proxy/handlers.rs`
- [x] Add `chat_completion_stream_inner()` async fn:
  - Validates provider + model (same as non-streaming path)
  - Gets API key from config
  - Translates request via `ClaudeAdapter::translate_request()` (with `stream: true`)
  - Calls `claude_adapter.send_stream()`
  - Spawns a background task that:
    - Reads `bytes_stream()` chunks from the upstream response
    - Forwards each chunk via `tokio::sync::mpsc::Sender<Result<Bytes, std::io::Error>>`
    - Accumulates SSE text to parse `message_start` (input_tokens, model) and `message_delta` (output_tokens) events
    - After stream ends, records `UsageRecord` via `tracking_service.record()`
  - Returns `Response<Body>` with `Content-Type: text/event-stream`, `Cache-Control: no-cache`, `X-Accel-Buffering: no`
- [x] Modify `chat_completion()` to branch: if `request.stream == Some(true)` → delegate to `chat_completion_stream_inner()`

### Step 5: Update `src/lib.rs`
- [x] Add `use proxy::models::get_models;` import
- [x] Add `GET /api/models` to `management_routes` (auth-protected via existing middleware)
- [x] Add `basic_auth_middleware` to `proxy_routes` (wraps the `/v1/chat/completions` route)
- [x] Update proxy test helpers in `tests/` to include auth credentials when calling proxy route

### Step 6: Add `ModelOption` + `ChatMessage` to `frontend/src/types/api.ts`
- [x] Add `ModelOption` interface: `provider`, `model`, `display_name`
- [x] Add `ChatMessage` interface: `role: "user" | "assistant"`, `content: string`, `cancelled?: boolean`

### Step 7: Add `react-markdown` to `frontend/package.json`
- [x] Add `"react-markdown": "^9.0.1"` to `dependencies`

### Step 8: Create `frontend/src/components/MessageBubble.tsx`
- [x] Props: `{ message: ChatMessage }`
- [x] User messages: right-aligned, gray bubble, plain text
- [x] Assistant messages: left-aligned, white bubble with border, `<ReactMarkdown>` for content
- [x] Cancelled badge if `message.cancelled === true`

### Step 9: Create `frontend/src/components/PromptTab.tsx`
- [x] On mount: fetch `GET /api/models`, set `models` + `selectedModel` (first model)
- [x] Render: model `<select>`, max_tokens number input, temperature range input + display
- [x] Render: scrollable message list of `<MessageBubble>` components
- [x] Render: textarea (Enter to submit, Shift+Enter for newline), Send/Stop button, Clear button
- [x] Implement streaming via `ReadableStream`, parses OpenAI SSE `choices[0].delta.content` events
- [x] Implement cancel: calls `abortControllerRef.current.abort()`, sets `cancelled: true` on last message
- [x] Error handling: inline error banner; remove empty assistant bubble on non-abort failure

### Step 10: Update `frontend/src/components/NavTabs.tsx`
- [x] Extend `Tab` type to include `"prompt"`
- [x] Add `{ id: "prompt", label: "Prompt", testId: "nav-prompt" }` to TABS array

### Step 11: Update `frontend/src/components/MainLayout.tsx`
- [x] Import `PromptTab`
- [x] Add `{activeTab === "prompt" && <PromptTab />}`

### Step 12: Update `tests/proxy_validation_test.rs`
- [x] Add auth credentials to any test helpers that call `/v1/chat/completions` (now auth-protected)
- [x] Added `missing_auth_on_proxy_returns_401` test
- [x] Removed obsolete `streaming_request_returns_501` test

### Step 13: Code Summary Documentation
- [x] Write `aidlc-docs/construction/unit-7-prompt-tab/code/code-summary.md`

## Story Coverage

| Business Rule | Implemented In |
|--------------|----------------|
| BR-PT-01 (Prompt tab in nav) | NavTabs, MainLayout |
| BR-PT-02 (GET /api/models auth) | lib.rs management_routes |
| BR-PT-03 (3 models, Sonnet default) | models.rs, PromptTab |
| BR-PT-04 (Basic auth on proxy) | lib.rs proxy_routes |
| BR-PT-05 (strip auth header) | ClaudeAdapter.send_stream (sets x-api-key, not Authorization) |
| BR-PT-06 (full history sent) | PromptTab.handleSubmit |
| BR-PT-07 (cancelled flag not sent) | PromptTab message mapping |
| BR-PT-08 (Send disabled while streaming) | PromptTab |
| BR-PT-09 (Ctrl+Enter submit) | PromptTab textarea onKeyDown |
| BR-PT-10 (Clear button) | PromptTab |
| BR-PT-11 (stream: true in body) | PromptTab.handleSubmit |
| BR-PT-12 (append delta.text) | PromptTab SSE parser |
| BR-PT-13 (AbortController) | PromptTab.handleSubmit |
| BR-PT-14 (keep partial on cancel) | PromptTab.handleStop |
| BR-PT-15 (streaming=false on stop) | PromptTab |
| BR-PT-16 (max_tokens control) | PromptTab |
| BR-PT-17 (temperature control) | PromptTab |
| BR-PT-18 (error removes bubble) | PromptTab catch handler |
| BR-PT-19 (401 → logout) | PromptTab |
| BR-PT-20 (models load error) | PromptTab mount error |
