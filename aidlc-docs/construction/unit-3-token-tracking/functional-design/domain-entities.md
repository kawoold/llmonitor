# Domain Entities — Unit 3: Token Tracking & Session Engine

## Database Schema

```sql
-- migrations/0002_request_logs.sql
CREATE TABLE IF NOT EXISTS request_logs (
    id            TEXT    PRIMARY KEY NOT NULL,   -- UUID v4
    created_at    TEXT    NOT NULL,               -- ISO 8601 UTC timestamp
    provider      TEXT    NOT NULL,               -- e.g. "claude"
    model         TEXT    NOT NULL,               -- resolved canonical model name
    session_id    TEXT    NOT NULL,               -- SHA-256 hex of messages array
    prompt_tokens    INTEGER NOT NULL DEFAULT 0,
    completion_tokens INTEGER NOT NULL DEFAULT 0,
    total_tokens     INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_request_logs_session_id  ON request_logs(session_id);
CREATE INDEX IF NOT EXISTS idx_request_logs_created_at  ON request_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_request_logs_provider    ON request_logs(provider);
CREATE INDEX IF NOT EXISTS idx_request_logs_model       ON request_logs(model);
```

---

## Rust Types

```rust
// src/tracking/types.rs

/// In-flight record passed through the mpsc channel.
#[derive(Debug, Clone)]
pub struct UsageRecord {
    pub id: String,           // UUID v4 (generated at request time)
    pub created_at: String,   // ISO 8601 UTC (generated at request time)
    pub provider: String,
    pub model: String,
    pub session_id: String,   // SHA-256 hex of serialized messages
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
```

---

## SessionEngine

```rust
// src/tracking/session.rs

pub struct SessionEngine;

impl SessionEngine {
    /// Derives a stable session ID from the full messages array.
    /// Identical messages → identical session_id.
    ///
    /// Algorithm:
    ///   1. Serialize each message as "{role}:{content_text}"
    ///      (for Parts content, join text segments with " ")
    ///   2. Join all message strings with "\n"
    ///   3. SHA-256 hash the UTF-8 bytes
    ///   4. Return lowercase hex string (64 chars)
    pub fn derive_session_id(messages: &[OpenAiMessage]) -> String;
}
```

---

## TrackingService

```rust
// src/tracking/service.rs

#[derive(Clone)]
pub struct TrackingService {
    sender: Arc<mpsc::Sender<UsageRecord>>,
}

impl TrackingService {
    pub fn new(sender: Arc<mpsc::Sender<UsageRecord>>) -> Self;

    /// Non-blocking send. Logs WARN if channel is full; never blocks the caller.
    pub fn record(&self, record: UsageRecord);
}
```

---

## TrackingWriterService

```rust
// src/tracking/writer.rs

pub struct TrackingWriterService {
    receiver: mpsc::Receiver<UsageRecord>,
    db: SqlitePool,
    flush_interval: Duration,   // 500ms
    batch_size: usize,          // 50
}

impl TrackingWriterService {
    pub fn new(receiver: mpsc::Receiver<UsageRecord>, db: SqlitePool) -> Self;

    /// Spawns the background writer task. Returns a JoinHandle for graceful shutdown.
    pub fn spawn(self) -> tokio::task::JoinHandle<()>;
}
```

---

## AppState Integration

```rust
// AppState gains:
pub tracking_service: TrackingService,
```

Channel construction at startup:
```rust
let (tx, rx) = tokio::sync::mpsc::channel::<UsageRecord>(1_000);
let tracking_service = TrackingService::new(Arc::new(tx));
let writer = TrackingWriterService::new(rx, db.clone());
writer.spawn();  // detached background task
```

---

## Constants

```rust
// src/tracking/writer.rs
pub const FLUSH_INTERVAL: Duration = Duration::from_millis(500);
pub const BATCH_SIZE: usize = 50;
pub const CHANNEL_CAPACITY: usize = 1_000;
```
