# Unit 3: Token Tracking & Session Engine — Code Generation Plan

## Unit Context

**Goal**: Replace the stub `src/tracking/mod.rs` with a real tracking pipeline: SHA-256 session IDs, mpsc channel dispatch, interval-driven background writer, graceful drain on shutdown, and wiring into the ProxyHandler.

**Existing stub to replace**: `src/tracking/mod.rs` — contains `UsageRecord` stub, `TrackingService::new_stub()`; must be fully replaced.

**Dependencies**: Unit 1 types (`OpenAiMessage`, `MessageContent`), Unit 5 (`SqlitePool`), Unit 2 (`RawUsage` from successful completion)

**Files to create**:
- `migrations/0002_request_logs.sql`
- `src/tracking/types.rs`
- `src/tracking/session.rs`
- `src/tracking/service.rs`
- `src/tracking/writer.rs`
- `aidlc-docs/construction/unit-3-token-tracking/code/code-summary.md`

**Files to modify**:
- `Cargo.toml` — add `sha2 = "0.10"`
- `src/tracking/mod.rs` — replace stub with proper module root
- `src/app_state.rs` — accept `TrackingService` as constructor parameter (remove internal stub creation)
- `src/lib.rs` — add `test_helpers::test_tracking_service()`; update `test_helpers` to pass TrackingService to AppState
- `src/main.rs` — create channel + writer before AppState; spawn writer; await JoinHandle after serve
- `src/proxy/handlers.rs` — call `state.tracking_service.record()` after successful completion

**Note on field rename**: AppState currently uses `tracking` for the TrackingService field. We'll keep `tracking_service` as the field name for consistency with other service fields.

---

## Steps

### Step 1: Add `sha2` dependency and create migration
- [x] Add `sha2 = "0.10"` to `[dependencies]` in `Cargo.toml`
- [x] Create `migrations/0002_request_logs.sql` with `request_logs` table and 4 indexes

### Step 2: Create `src/tracking/types.rs` — UsageRecord
- [ ] Define `UsageRecord` with fields: `id`, `created_at`, `provider`, `model`, `session_id`, `prompt_tokens`, `completion_tokens`, `total_tokens`
- [ ] Define constants: `CHANNEL_CAPACITY = 1_000`, `FLUSH_INTERVAL_MS = 500`, `BATCH_SIZE = 50`

### Step 3: Create `src/tracking/session.rs` — SessionEngine
- [ ] Implement `SessionEngine::derive_session_id(messages: &[OpenAiMessage]) -> String`
  - Canonical form: join `"{role}:{content_text}"` with `"\n"`
  - `content_text()`: Text → as-is; Parts → join text parts with `" "`, skip non-text
  - SHA-256 via `sha2::Sha256::digest()`, hex via `format!("{hash:x}")`
- [ ] Unit tests: known input → known output, empty messages
- [ ] Proptest: determinism (same input → same output), output is always 64 hex chars

### Step 4: Create `src/tracking/service.rs` — TrackingService
- [ ] `TrackingService` struct with `Arc<mpsc::Sender<UsageRecord>>`
- [ ] `new(sender: Arc<mpsc::Sender<UsageRecord>>) -> Self`
- [ ] `record(&self, record: UsageRecord)` — `try_send`, WARN on Full, ERROR on Closed
- [ ] Unit test: record() on a full channel logs warn and does not panic

### Step 5: Create `src/tracking/writer.rs` — TrackingWriterService
- [ ] `TrackingWriterService::new(receiver, db)` and `spawn(self) -> JoinHandle<()>`
- [ ] `select!` loop: accumulate from receiver; drain + flush on 500ms tick; final flush on channel close
- [ ] `flush()`: per-record `INSERT OR IGNORE`; retry once after 50ms on failure; ERROR + skip on second failure
- [ ] Unit test: records written to DB after flush interval elapses

### Step 6: Replace `src/tracking/mod.rs` — module root
- [ ] Declare submodules: `session`, `service`, `types`, `writer`
- [ ] Re-export: `TrackingService`, `UsageRecord`, `TrackingWriterService`, `CHANNEL_CAPACITY`

### Step 7: Update `src/app_state.rs`
- [ ] Change `AppState::new(db, config)` signature to `AppState::new(db, config, tracking_service: TrackingService)`
- [ ] Replace `TrackingService::new_stub()` with the passed-in `tracking_service`
- [ ] Rename field `tracking` → `tracking_service` (already named this in the functional design; check current name)

### Step 8: Update `src/main.rs` — channel, writer spawn, drain
- [ ] Create mpsc channel before AppState construction
- [ ] Construct real `TrackingService` and `TrackingWriterService`
- [ ] Spawn writer: `let writer_handle = writer.spawn()`
- [ ] Pass `tracking_service` to `AppState::new()`
- [ ] After `axum::serve(...).await?`, await `writer_handle` for graceful drain

### Step 9: Update `src/lib.rs` — test helpers
- [ ] Add `test_helpers::test_tracking_service()` — creates a drain-only channel for tests
- [ ] Update `test_helpers::build_test_app()` pattern to pass TrackingService to AppState

### Step 10: Update `src/proxy/handlers.rs` — tracking wiring
- [ ] After successful `provider.chat_completion(request.clone())`:
  - Derive `session_id` from `request.messages` (clone messages before moving into chat_completion)
  - Build `UsageRecord`
  - Call `state.tracking_service.record(usage_record)`
- [ ] Note: `request` must be cloned before the `chat_completion()` call so messages are available

### Step 11: Unit tests — writer integration
- [ ] In `tests/tracking_test.rs`: submit a record via TrackingService → wait >500ms → query DB → assert row exists

### Step 12: Verify compilation and run all tests
- [ ] `cargo build` — zero errors
- [ ] `cargo test` — all existing tests + new tests pass

### Step 13: Write code summary
- [ ] Create `aidlc-docs/construction/unit-3-token-tracking/code/code-summary.md`
