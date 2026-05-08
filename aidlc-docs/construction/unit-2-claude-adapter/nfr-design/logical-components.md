# Logical Components — Unit 2: Claude Provider Adapter

## Component 1: ClaudeAdapter

**File**: `src/providers/claude/mod.rs`

**Responsibility**: Owns the `reqwest::Client` and `ApiKey`; sends serialized `AnthropicRequest` to Anthropic's API; deserializes `AnthropicResponse` or maps HTTP errors to `ProviderError`.

**Methods**:
- `new(api_key: ApiKey) -> Self` — constructs client once
- `async send(&self, req: AnthropicRequest) -> Result<AnthropicResponse, ProviderError>` — single HTTP attempt

**Does NOT**:
- Retry (retry lives in `Provider::chat_completion()`)
- Translate request/response formats (handled by translation functions)

---

## Component 2: Request Translator

**File**: `src/providers/claude/mod.rs` (associated function on `ClaudeAdapter`)

**Responsibility**: Pure function converting `OpenAiChatRequest` → `Result<AnthropicRequest, ProviderError>`.

**Steps applied** (in order):
1. Parse and validate `anthropic_cache_control` extension (fail-fast if malformed)
2. Extract and concatenate system messages (BR-U2-01)
3. Resolve model alias (BR-U2-03)
4. Translate user/assistant messages — reject unknown roles (BR-U2-11)
5. Apply cache control hints (BR-U2-04)
6. Map remaining parameters (max_tokens default BR-U2-02, temperature, top_p, stop)

**Signature**: `pub fn translate_request(req: &OpenAiChatRequest) -> Result<AnthropicRequest, ProviderError>`

---

## Component 3: Response Translator

**File**: `src/providers/claude/mod.rs` (associated function on `ClaudeAdapter`)

**Responsibility**: Pure function converting `AnthropicResponse` → `(OpenAiChatResponse, RawUsage)`.

**Steps applied** (in order):
1. Concatenate text blocks from `content` array
2. Map `stop_reason` → `finish_reason` (BR-U2-12)
3. Build `OpenAiChatResponse` with `chatcmpl-{uuid_v4}` ID (BR-U2-10)
4. Build `RawUsage` with best-effort cache token extraction (BR-U2-09)

**Signature**: `pub fn translate_response(resp: AnthropicResponse) -> (OpenAiChatResponse, RawUsage)`

---

## Component 4: Model Alias Resolver

**File**: `src/providers/claude/types.rs`

**Responsibility**: Maps client-provided model names to canonical Anthropic model IDs.

**Elements**:
- `MODEL_ALIAS_MAP: &[(&str, &str)]` — 8-entry constant table
- `pub fn resolve_model(name: &str) -> &str` — lookup with fallthrough

---

## Component 5: Retry Executor

**File**: `src/providers/mod.rs` (inside `Provider::chat_completion()`)

**Responsibility**: Wraps `ClaudeAdapter::send()` with exponential backoff retry logic. Not a standalone struct — implemented as a loop inside the `Provider::Claude` match arm.

**Behaviour**:
- Up to 3 attempts
- Delays: 500ms, 1000ms, 2000ms (each multiplied by jitter factor from `OsRng`)
- Retries on `ServerError` and `RateLimited` only
- Logs WARN on each retry, ERROR on final failure
- Returns `Ok` on first success, `Err(last_error)` after exhausting attempts

---

## Component 6: Jitter Generator

**File**: Inline in retry logic (`src/providers/mod.rs`)

**Responsibility**: Produces a uniform random multiplier in [0.9, 1.1) using `rand_core::OsRng` — no `rand` crate required.

```rust
fn jitter_factor() -> f64 {
    use rand_core::{OsRng, RngCore};
    let raw = OsRng.next_u32() as f64 / u32::MAX as f64;
    0.9 + raw * 0.2
}
```
