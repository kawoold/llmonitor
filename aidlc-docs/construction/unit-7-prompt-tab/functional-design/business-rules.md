# Business Rules — Unit 7: Prompt Tab

## Navigation

**BR-PT-01**: NavTabs gains a fourth tab `"prompt"` with label "Prompt" and `data-testid="nav-prompt"`. Tab order: Dashboard → Cache → Config → Prompt.

---

## Model Endpoint

**BR-PT-02**: `GET /api/models` is protected by the existing Basic auth middleware (same as all `/api/*` routes). Returns a JSON array of `ModelOption` objects — hardcoded, not database-backed.

**BR-PT-03**: The three models returned are in this fixed order: Opus 4.7, Sonnet 4.6, Haiku 4.5. Sonnet 4.6 is selected by default in the frontend.

---

## Proxy Route Auth

**BR-PT-04**: The `/v1/chat/completions` proxy route gains Basic auth protection (same middleware as management routes). The PromptTab includes `Authorization: Basic <creds>` on all requests to this route. A 401 response triggers logout per the existing pattern.

**BR-PT-05**: The proxy handler strips the incoming `Authorization` header before forwarding to Anthropic, replacing it with the server-side `x-api-key` header. This prevents the admin credentials from reaching the upstream API.

---

## Conversation

**BR-PT-06**: On submit, the full `messages` array (all prior turns plus the new user message) is sent as the `messages` field in the OpenAI-format request body. This gives the model full context of the conversation.

**BR-PT-07**: The `role` field in each message sent to the proxy is exactly `"user"` or `"assistant"`. The `cancelled` flag is a frontend-only field and is never sent to the proxy.

**BR-PT-08**: The Send button is disabled while `streaming === true` or while `input.trim() === ""`.

**BR-PT-09**: Ctrl+Enter in the textarea triggers submit (same as clicking Send), provided the form is not disabled.

**BR-PT-10**: The Clear button sets `messages = []`, clears `input`, and clears `error`. It is disabled while `streaming === true`.

---

## Streaming

**BR-PT-11**: The PromptTab sends `stream: true` in the JSON request body. The proxy translates this into a native Anthropic streaming request and pipes the SSE response back verbatim.

**BR-PT-12**: As each `content_block_delta` SSE event arrives with `delta.type === "text_delta"`, its `delta.text` is appended to the last message in the `messages` array (the in-progress assistant message). React re-renders on each append.

**BR-PT-13**: An `AbortController` is created at the start of each fetch. Its `signal` is passed to `fetch()`. Cancellation calls `controller.abort()`.

**BR-PT-14**: On cancellation, the partial assistant response already displayed is retained. The message is marked `cancelled: true` and displayed with a `"(cancelled)"` badge. The `streaming` flag is set to `false` and the conversation resumes as normal.

**BR-PT-15**: On `message_stop` event (or stream close), `streaming` is set to `false`.

---

## Parameters

**BR-PT-16**: `max_tokens` is sent as `max_tokens` in the request body. Range: 1–4096. Default: 1024. The input is a number field; non-integer values round down.

**BR-PT-17**: `temperature` is sent as `temperature` in the request body. Range: 0.0–1.0, step 0.1. Default: 1.0.

---

## Error Handling

**BR-PT-18**: If the fetch fails (network error or non-2xx after the stream begins), the error message is shown inline below the conversation. The in-progress assistant bubble is removed (not kept as a cancelled message — only user-initiated cancellation keeps the partial).

**BR-PT-19**: A 401 response from `/v1/chat/completions` clears sessionStorage and reloads the page (same logout behavior as all other 401s).

**BR-PT-20**: If `GET /api/models` fails, an inline error is shown in the Prompt tab where the selector would appear. The tab is still usable once models load on retry.
