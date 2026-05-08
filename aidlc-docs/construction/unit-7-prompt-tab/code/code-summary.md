# Unit 7 – Prompt Tab: Code Summary

## Overview

Added a Prompt tab to the llmonitor frontend that sends streaming chat completions through the proxy, with model selection, multi-turn conversation history, and markdown rendering.

## Files Changed

### Backend

| File | Change |
|------|--------|
| `src/proxy/types.rs` | Added `ModelOption` struct (`provider`, `model`, `display_name`) |
| `src/proxy/models.rs` | New handler `get_models()` returning hardcoded 3-model list |
| `src/proxy/mod.rs` | Declared `pub mod models` |
| `src/providers/claude/types.rs` | Added `stream: Option<bool>` to `AnthropicRequest` |
| `src/providers/claude/mod.rs` | Added `send_stream()` returning raw `reqwest::Response`; propagated `stream` in `translate_request()` |
| `src/proxy/handlers.rs` | `chat_completion` branches on `stream: true` → `chat_completion_stream_inner`; SSE passthrough with background task + mpsc + ReceiverStream; inline usage parsing via `parse_sse_usage()` |
| `src/lib.rs` | Added `/api/models` to management routes; added `basic_auth_middleware` to proxy routes |
| `Cargo.toml` | Added `bytes = "1"`, `tokio-stream` with `sync` feature |

### Frontend

| File | Change |
|------|--------|
| `frontend/src/types/api.ts` | Added `ModelOption` and `ChatMessage` interfaces |
| `frontend/package.json` | Added `react-markdown: ^9.0.1` |
| `frontend/src/components/MessageBubble.tsx` | New — renders a single chat message (user: plain text, assistant: react-markdown) with `[cancelled]` badge |
| `frontend/src/components/PromptTab.tsx` | New — full streaming Prompt tab with model selector, temp/max_tokens controls, multi-turn history, AbortController cancel, error handling |
| `frontend/src/components/NavTabs.tsx` | Added `"prompt"` tab type and entry |
| `frontend/src/components/MainLayout.tsx` | Imported and rendered `<PromptTab />` for `activeTab === "prompt"` |

### Tests

| File | Change |
|------|--------|
| `tests/proxy_validation_test.rs` | Updated all proxy tests to include Basic auth credentials; added `missing_auth_on_proxy_returns_401`; removed obsolete `streaming_request_returns_501` |

## Key Design Decisions

- **Streaming passthrough**: Background tokio task forwards raw SSE bytes via `mpsc::channel` while parsing usage events inline — no full buffering.
- **Auth on proxy route**: `basic_auth_middleware` applied via `route_layer` so it runs before rate limiting middleware on `/v1/chat/completions`.
- **AbortController cancel**: Keeps partial response displayed with `[cancelled]` badge rather than clearing it.
- **`/api/models` auth-protected**: Returns the canonical model list; fetched once on tab mount using stored credentials.
