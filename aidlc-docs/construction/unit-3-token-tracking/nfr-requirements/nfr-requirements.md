# NFR Requirements — Unit 3: Token Tracking & Session Engine

## Performance

### NFR-U3-P01: Zero Blocking on Request Path
- `TrackingService::record()` MUST complete in O(1) time.
- It uses `mpsc::Sender::try_send()` — non-blocking. The request handler MUST NOT await any tracking I/O.
- SHA-256 session ID derivation is CPU-only (no I/O) and completes in microseconds for typical message arrays.

### NFR-U3-P02: Background Write Latency
- Records MUST be persisted to SQLite within one flush interval (≤500ms) of being queued under normal conditions.
- Batch size cap of 50 prevents a single flush from monopolizing the SQLite WAL write lock.

### NFR-U3-P03: Minimal Memory Footprint
- The mpsc channel holds at most 1 000 `UsageRecord` structs in memory at peak. Each record is approximately 200–400 bytes, giving a maximum channel memory of ~400KB.

---

## Reliability

### NFR-U3-R01: Retry-Once on Insert Failure
- A single failed `INSERT` triggers a 50ms sleep and one retry attempt. This handles transient WAL lock contention.
- Persistent failures (disk full) are logged and dropped; the writer continues.

### NFR-U3-R02: Graceful Drain on Shutdown
- When the sender is dropped (shutdown signal), the writer task drains and flushes all buffered records before exiting.
- The writer's `JoinHandle` MUST be awaited during shutdown to guarantee the drain completes.

### NFR-U3-R03: Tracking Failure Does Not Affect Clients
- Any error in the tracking pipeline (channel full, DB failure) MUST NOT propagate to the proxy response. The response path has no `?` operator on tracking calls.

---

## Security

### NFR-U3-S01: No Prompt Content in Logs or DB
- The `request_logs` table MUST NOT store message content, user identifiers, or any prompt/response text.
- Only metadata (token counts, model, provider, session ID, timestamp) is persisted.

### NFR-U3-S02: Session ID Is Not Reversible
- The SHA-256 session ID is a one-way hash. The original message content CANNOT be reconstructed from it.
- This is by design: session grouping without storing prompt data.

---

## Observability

### NFR-U3-O01: Structured Log Events
- `WARN` when `try_send` returns `Full` (includes current channel capacity context)
- `ERROR` when an individual DB insert fails after retry (includes record id and error)
- `DEBUG` for each successful record queued (optional, off by default under INFO level)

### NFR-U3-O02: Writer Task Lifecycle
- Log at `INFO` when the writer task starts and when it completes its final drain
- Log record counts per flush cycle at `DEBUG` level

---

## Maintainability

### NFR-U3-M01: Property-Based Testing for SessionEngine
- `SessionEngine::derive_session_id()` MUST be covered by proptest property tests verifying:
  - Determinism: same input → same output, always
  - Sensitivity: different messages → (with overwhelming probability) different output
  - Non-emptiness: any non-empty input produces a 64-char hex string

### NFR-U3-M02: Pure Session ID Function
- `derive_session_id()` is a pure function — no I/O, no state, no randomness. Unit-testable with simple inputs.

### NFR-U3-M03: `INSERT OR IGNORE` for Idempotence
- The SQL insert uses `INSERT OR IGNORE` so retrying an already-inserted UUID is a no-op, not an error.
