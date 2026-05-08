# Business Rules — Unit 2: Claude Provider Adapter

## BR-U2-01: System Messages Concatenated into Anthropic `system` Field

**Rule**: All OpenAI messages with `role: "system"` MUST be extracted from the messages array, concatenated (joined with `"\n\n"`), and placed in Anthropic's top-level `system` field. They MUST NOT appear in the `messages` array sent to Anthropic.

**Rationale**: Anthropic's API does not accept `role: "system"` in the messages array. Q1=B.

---

## BR-U2-02: Default `max_tokens` When Client Omits It

**Rule**: If the OpenAI request does not include `max_tokens`, the adapter MUST supply `max_tokens: 4096` in the Anthropic request. Anthropic's API requires `max_tokens` to be present.

**Rationale**: Anthropic rejects requests without `max_tokens`; a sensible default prevents unnecessary errors.

---

## BR-U2-03: Model Alias Resolution

**Rule**: Before sending a request to Anthropic, the adapter MUST look up the model name in `MODEL_ALIAS_MAP`. If a match is found, the canonical Anthropic model ID MUST be used. If no match, the client-provided name is used unchanged.

**Rationale**: Allows clients to use short aliases (e.g., `claude-3-sonnet`) without specifying date-suffixed IDs. Q2=B.

---

## BR-U2-04: Cache Control Applied Only to Specified Positions

**Rule**: Cache control (`cache_control: {type: "ephemeral"}`) is inserted ONLY when the client explicitly requests it via the `anthropic_cache_control` extension field. Auto-application is not performed.

- `"system": true` → add cache_control to the last block of the system prompt (using Anthropic's block-form system field)
- `"messages": [i, ...]` → add cache_control to the LAST content block of Anthropic message at the given index

Out-of-range message indices MUST be ignored (not an error).

**Rationale**: Q3=A — client-driven; proxy must not make caching decisions on the client's behalf.

---

## BR-U2-05: Streaming Requests Return 501

**Rule**: If the OpenAI request includes `"stream": true`, the handler MUST return `501 Not Implemented`. Unit 2 implements batch mode only.

**Rationale**: Q4=B — deferred.

---

## BR-U2-06: Retry on 5xx and 529 Only

**Rule**: `Provider::chat_completion()` MUST retry on `ProviderError::ServerError(_)` and `ProviderError::RateLimited` (which covers both 429 and 529). It MUST NOT retry on `AuthFailed`, `ClientError(_)`, or `Timeout`.

- Maximum attempts: 3
- Delays: 500ms → 1000ms → 2000ms with ±10% uniform jitter
- After 3 failed attempts, return the last error

**Rationale**: Q5=B; matches the NFR design from Unit 1.

---

## BR-U2-07: Anthropic API Version Header Required

**Rule**: Every request to Anthropic MUST include the header `anthropic-version: 2023-06-01`. Requests without this header will be rejected by Anthropic.

---

## BR-U2-08: API Key Never Logged

**Rule**: The Anthropic API key MUST NOT appear in any log output. It is stored in `ApiKey(String)` which redacts itself in `Debug`/`Display` (inherited from Unit 1 LC-07).

**Rationale**: SECURITY-06, SECURITY-03.

---

## BR-U2-09: Cache Token Extraction Is Best-Effort

**Rule**: `cache_read_input_tokens` and `cache_creation_input_tokens` are `Option<u32>` in the Anthropic response. If absent, they MUST be treated as 0 in `RawUsage`. This is not an error condition.

**Rationale**: Anthropic only includes these fields when caching is active; older API versions may omit them entirely.

---

## BR-U2-10: Response `id` Must Follow OpenAI Format

**Rule**: The `id` field in `OpenAiChatResponse` MUST have the format `chatcmpl-{uuid_v4}` to match OpenAI's convention. The Anthropic response ID (e.g. `msg_...`) MUST NOT be used as-is.

**Rationale**: Clients may depend on the `chatcmpl-` prefix for ID pattern matching.

---

## BR-U2-11: Role Mapping Is Strict

**Rule**: Only `role: "user"` and `role: "assistant"` are valid in Anthropic's messages array. Any other role (after system messages are extracted per BR-U2-01) MUST cause a `ProviderError::ClientError` with a descriptive message.

**Rationale**: Prevents sending malformed requests to Anthropic (e.g., `role: "tool"` without tool support implemented).

---

## BR-U2-12: Stop Reason Mapping

| Anthropic `stop_reason` | OpenAI `finish_reason` |
|---|---|
| `"end_turn"` | `"stop"` |
| `"max_tokens"` | `"length"` |
| `"stop_sequence"` | `"stop"` |
| `"tool_use"` | `"tool_calls"` |
| `null` / other | `"stop"` |
