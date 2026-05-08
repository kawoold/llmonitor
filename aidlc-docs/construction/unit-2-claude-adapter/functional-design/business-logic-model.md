# Business Logic Model — Unit 2: Claude Provider Adapter

## Overview

Unit 2 implements `ClaudeAdapter` and wires it into the `Provider` enum, replacing the Unit 1 stub. All HTTP calls to Anthropic go through `ClaudeAdapter`. Retry logic lives in `Provider::chat_completion()`, not in the adapter.

---

## 1. Request Translation Pipeline (OpenAI → Anthropic)

```
OpenAiChatRequest
    │
    ▼
Step 1: Extract & concatenate system messages
    - Filter messages where role == "system"
    - Join their content strings with "\n\n"
    - Set as AnthropicRequest.system (None if no system messages)
    - Remove system messages from the messages list

    │
    ▼
Step 2: Resolve model name
    - Look up model in MODEL_ALIAS_MAP
    - If found → use canonical Anthropic model ID
    - If not found → pass through as-is (already validated by accepted_models list)

    │
    ▼
Step 3: Translate messages (user and assistant only)
    For each remaining OpenAI message:
        - role: "user" | "assistant" → same in Anthropic
        - content: String → [AnthropicContentBlock { type: "text", text: content, cache_control: None }]
        - content: Parts(Vec<ContentPart>) → map each text part; image parts → image_url content block

    │
    ▼
Step 4: Apply cache control hints
    - Read "anthropic_cache_control" from OpenAiChatRequest.extra
    - If absent → skip
    - If present:
        - If "system": true → add cache_control: {type: "ephemeral"} to last block of system string
          (implement as wrapping system in an AnthropicContentBlock list if Anthropic API supports it)
          NOTE: Anthropic's system field accepts String OR [{type:"text", text:"...", cache_control:{}}]
        - For each index in "messages": [i, j, ...]
          → add cache_control: {type: "ephemeral"} to the LAST content block of Anthropic message at index i

    │
    ▼
Step 5: Map remaining parameters
    - max_tokens:   req.max_tokens.unwrap_or(4096)   ← required by Anthropic
    - temperature:  req.temperature (pass through)
    - top_p:        req.top_p (pass through)
    - stop:         req.stop → stop_sequences
    - stream:       NOT set (batch only in Unit 2)

    │
    ▼
AnthropicRequest
```

---

## 2. Response Translation Pipeline (Anthropic → OpenAI)

```
AnthropicResponse
    │
    ▼
Step 1: Extract text content
    - Concatenate text blocks from response.content where type == "text"
    - Result: completion_text: String

    │
    ▼
Step 2: Map stop reason
    - "end_turn"       → "stop"
    - "max_tokens"     → "length"
    - "stop_sequence"  → "stop"
    - "tool_use"       → "tool_calls"
    - other            → "stop"

    │
    ▼
Step 3: Build OpenAiChatResponse
    - id:      "chatcmpl-{uuid_v4}"
    - object:  "chat.completion"
    - created: current Unix timestamp (seconds)
    - model:   response.model
    - choices: [{
          index: 0,
          message: { role: "assistant", content: completion_text },
          finish_reason: mapped stop reason
      }]
    - usage: OpenAiUsage {
          prompt_tokens:     response.usage.input_tokens,
          completion_tokens: response.usage.output_tokens,
          total_tokens:      input + output
      }

    │
    ▼
Step 4: Build RawUsage (for tracking pipeline)
    - prompt_tokens:         response.usage.input_tokens
    - completion_tokens:     response.usage.output_tokens
    - cache_read_tokens:     response.usage.cache_read_input_tokens.unwrap_or(0)
    - cache_creation_tokens: response.usage.cache_creation_input_tokens.unwrap_or(0)

    │
    ▼
(OpenAiChatResponse, RawUsage)
```

---

## 3. HTTP Call (ClaudeAdapter::send)

```
AnthropicRequest
    │
    ▼
Build reqwest::Request:
    - POST https://api.anthropic.com/v1/messages
    - Headers:
        x-api-key: {api_key}
        anthropic-version: "2023-06-01"
        content-type: application/json
        accept: application/json
    - Body: serde_json::to_vec(&anthropic_request)
    - Timeout: from Config (default 30s)

    │
    ▼
Send request
    │
    ├─ Network error / timeout → ProviderError::Timeout
    │
    ▼
Inspect HTTP status:
    - 200 → deserialize AnthropicResponse → Ok(response)
    - 401 → ProviderError::AuthFailed
    - 400 → deserialize AnthropicError → ProviderError::ClientError(error.error.message)
    - 429 → ProviderError::RateLimited
    - 529 → ProviderError::RateLimited  (Anthropic overload)
    - 500/502/503 → ProviderError::ServerError(status.to_string())
    - other → ProviderError::ServerError(status.to_string())
```

---

## 4. Retry Logic (Provider::chat_completion — Q5=B)

```
attempt = 1
loop:
    result = ClaudeAdapter::send(request.clone())
    if Ok → return Ok
    if Err(e):
        retryable = matches!(e, ServerError(_) | RateLimited)
        if !retryable OR attempt >= 3 → return Err(e)
        delay = base_delay(attempt) * jitter_factor()
            base_delay: 500ms, 1000ms, 2000ms (for attempts 1, 2, 3)
            jitter: uniform random ±10% → multiply by rand in [0.9, 1.1]
        sleep(delay)
        attempt += 1
```

Non-retryable errors: `AuthFailed`, `ClientError(_)`, `Timeout` (timeout is not a 5xx — retrying a timed-out request risks double-execution).

---

## 5. Model Alias Map

| Client-provided name | Canonical Anthropic model ID |
|---|---|
| `claude-3-opus` | `claude-3-opus-20240229` |
| `claude-3-sonnet` | `claude-3-sonnet-20240229` |
| `claude-3-haiku` | `claude-3-haiku-20240307` |
| `claude-3-5-sonnet` | `claude-3-5-sonnet-20241022` |
| `claude-3-5-haiku` | `claude-3-5-haiku-20241022` |
| `claude-sonnet-4` | `claude-sonnet-4-6` |
| `claude-opus-4` | `claude-opus-4-7` |
| `claude-haiku-4` | `claude-haiku-4-5-20251001` |

Names not in this map are passed through unchanged (already validated against the `accepted_models` list).

---

## 6. Error Translation Table

| Anthropic HTTP status | ProviderError variant | ProxyError (via From) |
|---|---|---|
| 400 | ClientError(msg) | UpstreamClientError(msg) |
| 401 | AuthFailed | UpstreamAuthError |
| 429 | RateLimited | UpstreamRateLimit |
| 529 | RateLimited | UpstreamRateLimit |
| 500/502/503 | ServerError(status) | UpstreamServerError(status) |
| Timeout | Timeout | UpstreamTimeout |
