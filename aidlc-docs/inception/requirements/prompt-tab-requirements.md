# Requirements — Prompt Tab Feature

## Intent Analysis

- **Request type**: New Feature
- **Scope**: Multiple Components (Rust backend endpoint + React frontend components)
- **Complexity**: Moderate-Complex (streaming SSE, markdown rendering, multi-turn state)

---

## Functional Requirements

**FR-PT-01**: A fourth tab "Prompt" is added to NavTabs alongside Dashboard, Cache, Config.

**FR-PT-02**: A new `GET /api/models` management endpoint (auth-protected) returns the list of supported provider/model combinations as JSON.

**FR-PT-03**: The PromptTab fetches `GET /api/models` on mount and populates a model selector dropdown. The first model is selected by default.

**FR-PT-04**: The PromptTab maintains a multi-turn conversation: each submitted prompt is appended to the message list as a user message; the assistant response is appended as an assistant message. All prior messages are sent as `messages` context on each new request.

**FR-PT-05**: The user can compose a message in a textarea. Pressing Send (or Ctrl+Enter) submits it.

**FR-PT-06**: The user can configure `max_tokens` (integer, 1–4096, default 1024) and `temperature` (float, 0.0–2.0, step 0.1, default 1.0) via inline controls in the PromptTab.

**FR-PT-07**: Responses are streamed from the proxy using the Anthropic streaming SSE format (`stream: true` in the request body). Tokens are appended to the assistant message bubble as they arrive.

**FR-PT-08**: Assistant response text is rendered as markdown (bold, italic, code blocks, lists, etc.).

**FR-PT-09**: A "Clear" button resets the conversation history.

**FR-PT-10**: While a response is streaming, the Send button is disabled and a stop/cancel control is shown. If the user cancels, the partial response is retained and marked as cancelled.

**FR-PT-11**: Errors (network, non-2xx, 401) are shown inline below the conversation. 401 triggers logout per the existing pattern.

---

## Non-Functional Requirements

**NFR-PT-01**: Streaming is implemented via `fetch` + `ReadableStream` (not EventSource — the proxy returns raw SSE chunks over a single POST). The stream reader decodes `data: {...}` lines and extracts `delta.text` from each `content_block_delta` event.

**NFR-PT-02**: `react-markdown` is added to frontend dependencies for markdown rendering of assistant messages.

**NFR-PT-03**: All interactive elements have `data-testid` attributes consistent with the existing naming scheme.

**NFR-PT-04**: The `GET /api/models` endpoint is protected by existing Basic auth middleware — no new auth mechanism.

**NFR-PT-05**: The PromptTab sends requests to `/v1/messages` (the existing proxy route), not to any new proxy endpoint. No changes to the proxy forwarding logic.

---

## Backend Change Required

One new route: `GET /api/models` → returns `[{ provider: string, model: string, display_name: string }]`.

The model list is hardcoded in the handler for now (matches Unit 2 Claude provider adapter's supported models). No database changes required.

---

## Out of Scope

- Saving conversation history to the database
- System prompt configuration
- Image/file attachments
- Multiple concurrent conversations / tabs
