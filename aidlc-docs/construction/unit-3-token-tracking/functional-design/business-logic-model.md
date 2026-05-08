# Business Logic Model — Unit 3: Token Tracking & Session Engine

## 1. Request Tracking Pipeline (ProxyHandler integration)

```
POST /v1/chat/completions
    │
    ▼
[existing Unit 2 flow: translate → send → translate_response]
    │
    ▼ (on success only)
Build UsageRecord:
    - id:                 Uuid::new_v4().to_string()
    - created_at:         Utc::now().to_rfc3339()
    - provider:           provider.name()
    - model:              response.model (canonical name from Anthropic response)
    - session_id:         SessionEngine::derive_session_id(&request.messages)
    - prompt_tokens:      raw_usage.prompt_tokens
    - completion_tokens:  raw_usage.completion_tokens
    - total_tokens:       prompt_tokens + completion_tokens
    │
    ▼
TrackingService::record(usage_record)
    │
    ├─ channel not full → send to mpsc channel (non-blocking)
    └─ channel full     → log WARN, discard record, return (request still completes)
    │
    ▼
Return OpenAiChatResponse to client (tracking does not block response)
```

**Key principle**: Tracking failure MUST NOT fail the proxy request. The `record()` call is always fire-and-forget.

---

## 2. Session ID Derivation Algorithm

```
Input: messages: &[OpenAiMessage]

For each message in messages:
    role_str = message.role          // "system", "user", "assistant"
    content_str = match message.content:
        Text(s)    → s
        Parts(vec) → join all text parts with " " (skip non-text parts)
    line = "{role_str}:{content_str}"

canonical = join all lines with "\n"
hash_bytes = SHA-256(canonical.as_bytes())
session_id = lowercase hex string of hash_bytes (64 chars)
```

**Properties**:
- Deterministic: same messages → same session_id, always
- Stable across request attempts (retry does not change session_id)
- Different message order → different session_id (order matters)
- Multi-turn grouping: requests sharing all messages except last user message hash differently from the full conversation — tracking groups requests by their FULL message context (Q2=B)

**Example**:
```
messages = [
    {role: "user", content: "hello"}
]
canonical = "user:hello"
session_id = sha256("user:hello") = "2cf24db..."

messages = [
    {role: "user",      content: "hello"},
    {role: "assistant", content: "hi there"},
    {role: "user",      content: "follow up"}
]
canonical = "user:hello\nassistant:hi there\nuser:follow up"
session_id = sha256(...) = different hash
```

---

## 3. TrackingService — Non-Blocking Record

```
TrackingService::record(record: UsageRecord):
    match sender.try_send(record):
        Ok(())   → done (record queued)
        Err(Full) → warn!(channel_full = true, "tracking channel full, dropping record")
        Err(Closed) → error!("tracking channel closed unexpectedly")
```

`try_send` is non-blocking and completes in O(1). The caller (ProxyHandler) is never suspended.

---

## 4. TrackingWriterService — Background Flush Loop

```
spawn background task:
    buffer: Vec<UsageRecord> = Vec::with_capacity(BATCH_SIZE)
    ticker = tokio::time::interval(FLUSH_INTERVAL)  // 500ms

    loop:
        select!:
            record = receiver.recv() =>
                if record is Some:
                    buffer.push(record)
                    // No immediate flush; accumulate until tick
                else:
                    // Channel closed (shutdown signal received)
                    flush(buffer) if not empty
                    break

            _ = ticker.tick() =>
                if buffer is not empty:
                    flush(buffer)
                    buffer.clear()

fn flush(records: &[UsageRecord], db: &SqlitePool):
    for each record in records:
        result = sqlx::query(INSERT OR IGNORE INTO request_logs ...).execute(db).await
        if result is Err:
            warn!("DB insert failed, retrying once: {e}")
            sleep(50ms)
            result2 = retry INSERT
            if result2 is Err:
                error!("DB insert failed after retry, dropping record: {e}")
            // continue to next record regardless
```

**Design notes**:
- `INSERT OR IGNORE` protects against duplicate IDs on retry
- Flush is per-record (not per-batch-transaction) to isolate individual failures
- Writer continues after any individual insert failure

---

## 5. Graceful Shutdown Integration

```
main.rs / shutdown sequence:
    1. Receive shutdown signal (SIGTERM / Ctrl-C)
    2. Drop TrackingService sender (closes the channel)
    3. Await writer task JoinHandle
       → writer detects channel closed, flushes remaining buffer, exits
    4. Axum server shutdown
```

The writer task completes its final flush before the process exits, ensuring in-flight records are not lost during normal shutdowns.

---

## 6. ProxyHandler Changes (wiring)

In `src/proxy/handlers.rs`, after `provider.chat_completion()` succeeds:

```rust
// Existing:
let (response, raw_usage) = provider.chat_completion(request.clone()).await?;

// New (tracking wiring):
let usage_record = UsageRecord {
    id: Uuid::new_v4().to_string(),
    created_at: Utc::now().to_rfc3339(),
    provider: provider.name().to_string(),
    model: response.model.clone(),
    session_id: SessionEngine::derive_session_id(&request.messages),
    prompt_tokens: raw_usage.prompt_tokens,
    completion_tokens: raw_usage.completion_tokens,
    total_tokens: raw_usage.prompt_tokens + raw_usage.completion_tokens,
};
state.tracking_service.record(usage_record);
```

Note: `request.messages` must be cloned/referenced before `request` is moved into `chat_completion()`. The handler should clone `request.messages` for session ID derivation.
