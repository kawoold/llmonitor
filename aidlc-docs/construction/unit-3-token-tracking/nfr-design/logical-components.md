# Logical Components — Unit 3: Token Tracking & Session Engine

## Component 1: SessionEngine

**File**: `src/tracking/session.rs`

**Responsibility**: Pure, stateless derivation of session IDs from message arrays. No fields — all methods are free functions or associated functions.

**Public API**:
```rust
pub struct SessionEngine;

impl SessionEngine {
    pub fn derive_session_id(messages: &[OpenAiMessage]) -> String;
}

fn content_text(content: &MessageContent) -> String;  // private helper
```

**Tested by**: proptest property tests (determinism, non-emptiness)

---

## Component 2: TrackingService

**File**: `src/tracking/service.rs`

**Responsibility**: Thin wrapper around the mpsc sender. Holds an `Arc<Sender<UsageRecord>>` so it can be cheaply cloned into `AppState` and shared across request handlers.

**Public API**:
```rust
#[derive(Clone)]
pub struct TrackingService {
    sender: Arc<mpsc::Sender<UsageRecord>>,
}

impl TrackingService {
    pub fn new(sender: Arc<mpsc::Sender<UsageRecord>>) -> Self;
    pub fn record(&self, record: UsageRecord);  // fire-and-forget
}
```

**Shutdown contract**: When the `Arc<Sender>` reference count drops to zero, the channel closes automatically — signalling the writer to drain and exit.

---

## Component 3: TrackingWriterService

**File**: `src/tracking/writer.rs`

**Responsibility**: Background Tokio task that drains the mpsc receiver, batches records, and flushes them to SQLite on a 500ms interval. Handles per-record insert failures with a single retry.

**Public API**:
```rust
pub struct TrackingWriterService {
    receiver: mpsc::Receiver<UsageRecord>,
    db: SqlitePool,
}

impl TrackingWriterService {
    pub fn new(receiver: mpsc::Receiver<UsageRecord>, db: SqlitePool) -> Self;
    pub fn spawn(self) -> tokio::task::JoinHandle<()>;
}
```

**Internal structure**:
- `flush(buffer: &[UsageRecord], db: &SqlitePool)` — per-record insert with one retry
- Constants: `FLUSH_INTERVAL = 500ms`, `BATCH_SIZE = 50`, `CHANNEL_CAPACITY = 1_000`

---

## Component 4: UsageRecord (shared type)

**File**: `src/tracking/types.rs`

**Responsibility**: Data transfer object passed through the mpsc channel from ProxyHandler to TrackingWriterService.

```rust
#[derive(Debug, Clone)]
pub struct UsageRecord {
    pub id: String,
    pub created_at: String,
    pub provider: String,
    pub model: String,
    pub session_id: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
```

---

## Component 5: Module Root

**File**: `src/tracking/mod.rs`

**Responsibility**: Declares and re-exports tracking submodules. Exposes `CHANNEL_CAPACITY` for use in `AppState` construction.

```rust
pub mod session;
pub mod service;
pub mod types;
pub mod writer;

pub use service::TrackingService;
pub use types::UsageRecord;
pub use writer::TrackingWriterService;
```

---

## Wiring Summary

```
main.rs / build_router():
    let (tx, rx) = mpsc::channel::<UsageRecord>(CHANNEL_CAPACITY);
    let tracking_service = TrackingService::new(Arc::new(tx));
    let writer = TrackingWriterService::new(rx, db.clone());
    let writer_handle = writer.spawn();

    // Store handle for graceful shutdown await
    // Store tracking_service in AppState

ProxyHandler (handlers.rs):
    state.tracking_service.record(UsageRecord { ... });

Shutdown:
    drop(state)  // drops Arc<Sender>, closes channel
    writer_handle.await  // waits for drain to complete
```
