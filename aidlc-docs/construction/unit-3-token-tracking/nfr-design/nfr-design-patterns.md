# NFR Design Patterns — Unit 3: Token Tracking & Session Engine

## Pattern 1: Fire-and-Forget Channel Dispatch

**Applies to**: `TrackingService::record()`

**Design**: Uses `tokio::sync::mpsc::Sender::try_send()` — completes synchronously without suspending the caller. If the channel is full, the record is discarded with a WARN log. The caller (ProxyHandler) never awaits any tracking operation.

```rust
pub fn record(&self, record: UsageRecord) {
    match self.sender.try_send(record) {
        Ok(()) => {}
        Err(TrySendError::Full(_)) => {
            warn!("tracking channel full, dropping record");
        }
        Err(TrySendError::Closed(_)) => {
            error!("tracking channel closed unexpectedly");
        }
    }
}
```

**Rationale**: Guarantees zero async overhead on the hot path regardless of writer throughput.

---

## Pattern 2: Interval-Driven Background Writer

**Applies to**: `TrackingWriterService`

**Design**: A long-running Tokio task uses `tokio::select!` to receive records continuously and flush them on a fixed 500ms ticker. The writer is fully decoupled from the request path — it runs independently on the async runtime.

```
loop {
    select! {
        record = receiver.recv() => accumulate or detect shutdown
        _ = ticker.tick()        => flush buffer to SQLite
    }
}
```

**Rationale**: Decouples write latency from request handling. The 500ms interval provides predictable persistence latency without per-request DB overhead.

---

## Pattern 3: Retry-Once with Jitter-Free Delay

**Applies to**: Individual record inserts inside `TrackingWriterService::flush()`

**Design**: On insert failure, sleep exactly 50ms (no jitter needed — single writer, no thundering herd) then retry once. Second failure → log ERROR and skip. Unlike the Provider retry (which jitters to prevent upstream stampedes), this retry is against local SQLite and a fixed 50ms delay is sufficient.

```
INSERT OR IGNORE → Err → sleep 50ms → INSERT OR IGNORE → Err → log ERROR, skip
```

**Rationale**: Handles transient WAL lock contention (e.g., another connection mid-checkpoint) without complex logic.

---

## Pattern 4: Idempotent Inserts (`INSERT OR IGNORE`)

**Applies to**: All `request_logs` inserts

**Design**: Every insert uses `INSERT OR IGNORE INTO request_logs (...)`. Since `id` is the UUID v4 primary key, a duplicate insert on retry is silently ignored rather than raising a constraint error.

**Rationale**: Makes the retry safe without requiring existence checks or upsert logic.

---

## Pattern 5: Cooperative Shutdown via Channel Close

**Applies to**: Graceful drain pattern between `AppState` and `TrackingWriterService`

**Design**: The sender half of the mpsc channel is held by `TrackingService` inside `AppState`. When the shutdown signal arrives:

1. `AppState` (or the `TrackingService` inside it) is dropped, which drops the `Arc<Sender>`.
2. When the last `Arc<Sender>` drops, the channel is closed.
3. The writer's `receiver.recv()` arm returns `None`, triggering the drain-and-exit path.
4. `main.rs` awaits the writer's `JoinHandle` before process exit.

No explicit shutdown channel or `CancellationToken` is needed — the mpsc channel close is the shutdown signal.

---

## Pattern 6: Pure Deterministic Hash Function

**Applies to**: `SessionEngine::derive_session_id()`

**Design**: A pure, stateless function — no I/O, no random values, no global state. Takes `&[OpenAiMessage]`, returns a 64-char hex `String`. Guaranteed to return the same output for the same input on any call, on any thread, at any time.

```rust
pub fn derive_session_id(messages: &[OpenAiMessage]) -> String {
    use sha2::{Digest, Sha256};
    let canonical = messages.iter()
        .map(|m| format!("{}:{}", m.role, content_text(&m.content)))
        .collect::<Vec<_>>()
        .join("\n");
    let hash = Sha256::digest(canonical.as_bytes());
    format!("{hash:x}")
}
```

**Property tested** (proptest):
- Determinism: `derive_session_id(msgs) == derive_session_id(msgs)` for any `msgs`
- Non-emptiness: result is always a 64-char lowercase hex string
