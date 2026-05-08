# Business Logic Model — Unit 7: Prompt Tab

## Backend: Streaming Implementation

The `chat_completion` handler is modified to branch on `stream: true`:

```
chat_completion(request):
  if request.stream == Some(true):
    → delegate to streaming path (see below)
  else:
    → existing non-streaming path (unchanged)

streaming path:
  1. extract_provider_name(headers) — same as non-streaming
  2. validate_provider_name(name)
  3. api_key = config_service.get_anthropic_api_key()
  4. provider = Provider::from_name(name, api_key)
  5. provider.accepts_model(request.model) — validate, else 400
  6. anthropic_req = ClaudeAdapter::translate_request(request) — sets stream: true
  7. raw_resp = claude_adapter.send_stream(anthropic_req) — returns reqwest::Response
  8. body = axum::body::Body::from_stream(raw_resp.bytes_stream())
  9. return Response with:
       Content-Type: text/event-stream
       Cache-Control: no-cache
       body = body
```

### Tracking in Streaming Mode

Usage data arrives in the Anthropic SSE stream in the `message_delta` event:
```
data: {"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":42}}
```

The streaming handler uses a passthrough that:
1. Reads the SSE bytes stream chunk by chunk
2. Forwards each chunk immediately to the client
3. Accumulates a line buffer to detect `message_delta` events
4. On seeing a `message_delta` event: extracts `usage.output_tokens` and records the usage record asynchronously via `tracking_service.record()`

For input tokens, the `message_start` event contains them:
```
data: {"type":"message_start","message":{"usage":{"input_tokens":10,...}}}
```

The passthrough accumulates both values and writes the tracking record after `message_stop`.

### ClaudeAdapter::send_stream

New method alongside `send()`:
```rust
pub async fn send_stream(
    &self,
    req: AnthropicRequest,  // stream field must be true
    timeout_secs: u64,
) -> Result<reqwest::Response, ProviderError>
```

- Sends request to Anthropic with `stream: true` in body
- Does NOT call `.json()` on the response body — returns the raw `reqwest::Response`
- Caller pipes `response.bytes_stream()` to the Axum body

### Auth Header Stripping (BR-PT-05)

The `chat_completion` handler already reads the API key from `AppState.config_service`. The `ClaudeAdapter::send()` sets `x-api-key` from `self.api_key`. The incoming `Authorization` header from the browser is never forwarded — the handler only extracts `x-provider` from incoming headers, not `Authorization`.

---

## Backend: Models Endpoint

```
GET /api/models → [ModelOption; 3]  (hardcoded, no DB)
```

Handler in `src/proxy/models.rs`:
```rust
pub async fn get_models() -> Json<Vec<ModelOption>> {
    Json(vec![
        ModelOption { provider: "anthropic", model: "claude-opus-4-7",         display_name: "Claude Opus 4.7"   },
        ModelOption { provider: "anthropic", model: "claude-sonnet-4-6",        display_name: "Claude Sonnet 4.6" },
        ModelOption { provider: "anthropic", model: "claude-haiku-4-5-20251001",display_name: "Claude Haiku 4.5" },
    ])
}
```

Wired into management router (auth-protected).

---

## Frontend: PromptTab Submit Flow

```
handleSubmit():
  1. Append { role: "user", content: input } to messages
  2. Append { role: "assistant", content: "" } to messages  (in-progress bubble)
  3. Clear input
  4. Set streaming = true, error = null
  5. controller = new AbortController()
  6. res = await fetch("/v1/chat/completions", {
       method: "POST",
       headers: { Authorization: "Basic " + creds, Content-Type: "application/json" },
       body: JSON.stringify({
         model: selectedModel,
         messages: history,        // all prior turns
         stream: true,
         max_tokens: maxTokens,
         temperature: temperature,
       }),
       signal: controller.signal,
     })
  7. if res.status === 401 → logout()
  8. if !res.ok → throw Error(`HTTP ${res.status}`)
  9. reader = res.body.getReader()
 10. decoder = new TextDecoder()
 11. lineBuffer = ""
 12. loop:
       chunk = await reader.read()
       if chunk.done → break
       lineBuffer += decoder.decode(chunk.value, { stream: true })
       for each complete line in lineBuffer:
         if line starts with "data: ":
           json = line.slice(6)
           if json === "[DONE]" → break loop
           event = JSON.parse(json)
           if event.type === "content_block_delta" && event.delta.type === "text_delta":
             append event.delta.text to messages[last].content
           if event.type === "message_stop":
             break loop
 13. set streaming = false
```

### Cancellation Flow
```
handleStop():
  controller.abort()
  → fetch throws AbortError
  → catch (AbortError):
      set messages[last].cancelled = true
      set streaming = false
```

### Error Flow (non-cancel failure)
```
catch (non-AbortError):
  remove last message (the empty assistant bubble)
  set error = err.message
  set streaming = false
```
