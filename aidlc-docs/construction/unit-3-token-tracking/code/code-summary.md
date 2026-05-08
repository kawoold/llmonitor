# Unit 3 — Token Tracking: Code Summary

## Status: COMPLETE ✅

## Files Created / Modified

| File | Action | Description |
|------|--------|-------------|
| `migrations/0002_request_logs.sql` | Created | `request_logs` table + 4 indexes |
| `src/tracking/types.rs` | Created | `UsageRecord`, `CHANNEL_CAPACITY`, `FLUSH_INTERVAL`, `BATCH_SIZE` |
| `src/tracking/session.rs` | Created | `SessionEngine::derive_session_id()` — SHA-256 of messages |
| `src/tracking/service.rs` | Created | `TrackingService` — fire-and-forget `try_send()` wrapper |
| `src/tracking/writer.rs` | Created | `TrackingWriterService` — batched `INSERT OR IGNORE`, retry-once |
| `src/tracking/mod.rs` | Replaced stub | Wires all sub-modules, re-exports public API |
| `src/app_state.rs` | Updated | Added `tracking_service: TrackingService` field, updated `new()` signature |
| `src/main.rs` | Updated | Wires channel, `TrackingWriterService`, graceful shutdown drain |
| `src/lib.rs` | Updated | Added `test_tracking_service()` to `test_helpers` |
| `src/proxy/handlers.rs` | Updated | Clones messages, builds `UsageRecord`, calls `tracking_service.record()` |
| `Cargo.toml` | Updated | Added `sha2 = "0.10"` |
| `tests/tracking_test.rs` | Created | 2 integration tests: single record, multiple records |
| `tests/auth_test.rs` | Updated | `AppState::new()` now takes 3 args |
| `tests/config_api_test.rs` | Updated | `AppState::new()` now takes 3 args |
| `tests/health_test.rs` | Updated | `AppState::new()` now takes 3 args |
| `tests/proxy_validation_test.rs` | Updated | `AppState::new()` now takes 3 args |

## Key Design Decisions

- **Fire-and-forget**: `try_send()` drops records on full channel (WARN logged) rather than back-pressuring the HTTP response path.
- **Deterministic session ID**: SHA-256 of `"{role}:{content}\n"` per message — stable across restarts, no state required.
- **INSERT OR IGNORE**: Idempotent writes tolerate retry duplicates without constraint errors.
- **Graceful shutdown**: Dropping `Arc<Sender>` closes the channel; writer drains the buffer before exiting; `JoinHandle` awaited in main.

## Test Coverage

- 55 unit tests (all passing)
- 12 integration tests (all passing)
- 2 new integration tests in `tests/tracking_test.rs`
- proptest: determinism + 64-char hex output for `SessionEngine`
