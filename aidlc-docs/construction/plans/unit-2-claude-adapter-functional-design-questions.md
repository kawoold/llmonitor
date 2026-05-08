# Functional Design Questions — Unit 2: Claude Provider Adapter

Fill in each `[Answer]:` tag and reply "answered" when done.

---

**Q1 — System prompt handling**

OpenAI messages can include messages with `role: "system"`. Anthropic has a dedicated top-level `system` field and does not accept `role: "system"` in the messages array. How should the translation work?

A. Extract the **first** system message into Anthropic's `system` field; ignore any subsequent system messages
B. **Concatenate** all system messages (joined with `\n\n`) into Anthropic's `system` field; remove them from the messages array
C. Convert system messages to `role: "user"` messages with a `[SYSTEM]:` prefix (least preferred — degrades prompt structure)

[Answer]: B

---

**Q2 — Model name mapping**

Anthropic model IDs and OpenAI-style names are the same strings (e.g. `claude-sonnet-4-6`). Should the adapter:

A. **Direct pass-through** — send the model name the client provides directly to Anthropic (reject if not in the accepted models list, which already exists)
B. **Alias map** — maintain a table mapping friendly aliases (e.g. `claude-3-sonnet`) to canonical Anthropic IDs (e.g. `claude-3-sonnet-20240229`); enables version-agnostic client usage

[Answer]: B

---

**Q3 — Prompt cache control: how do clients signal caching?**

Anthropic's prompt caching requires `"cache_control": {"type": "ephemeral"}` embedded in specific message content blocks. How should clients of this proxy signal that they want caching applied?

A. **Client-driven, OpenAI extension fields** — clients embed cache control in a provider-specific extension key in the request JSON (e.g., `"anthropic_cache_control": {"messages": [0, 1]}` listing which message indices to cache); proxy inserts `cache_control` blocks at those positions
B. **Auto-apply to system prompt** — proxy automatically adds `cache_control: ephemeral` to the system prompt (if present) on every request; no client configuration needed
C. **Not supported in this unit** — skip cache control forwarding; only track cache tokens from response metadata (Anthropic reports cache hits even without explicit control directives from prior cached requests)

[Answer]: A

---

**Q4 — Streaming implementation in Unit 2**

A. **Implement full SSE streaming** — proxy connects to Anthropic's SSE stream, translates each event chunk to OpenAI format, and retransmits as OpenAI SSE (`data: {...}\n\n` format); `POST /v1/chat/completions` with `"stream": true` returns a streaming response
B. **Defer streaming to a later iteration** — Unit 2 implements batch mode only; streaming requests return `501 Not Implemented` (same as Unit 1 stub); streaming addressed post-MVP

[Answer]: B

---

**Q5 — Retry logic location**

The NFR design from Unit 1 specified exponential backoff retries (3 attempts, 500ms→1s→2s ±10% jitter) on Anthropic 5xx responses. Where should this live?

A. **Inside ClaudeAdapter** — the adapter retries failed HTTP calls internally before returning `ProviderError`; the proxy handler always sees a final result
B. **In the Provider dispatch layer** — `Provider::chat_completion()` wraps the adapter call in retry logic; ClaudeAdapter only makes one attempt

[Answer]: B
