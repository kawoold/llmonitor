# Unit 4: Statistics & Prometheus Metrics — Code Generation Plan

## Unit Context

**Purpose**: Replace `StatsService` and `MetricsRegistry` stubs with real implementations. Add cache token persistence to `request_logs`. Expose four `/api/stats/*` endpoints and a working `/metrics` Prometheus endpoint.

**Dependencies**: Unit 3 complete (request_logs table exists, TrackingService wired). Unit 2 complete (RawUsage carries cache_read_tokens, cache_creation_tokens).

**Workspace root**: `/home/kawoold/Development/rust/llmonitor`

**Key design decisions**:
- Per-instance `prometheus::Registry` (Q2-A: no global registry)
- Composite index `(created_at, provider, model)` in migration 0003 (Q1-A)
- `IntoResponse` for `StatsError` (consistent with ProxyError pattern) (Q2-A)
- `?window=1h|24h|7d|30d` query param, default `24h` (Q3-A)
- All 4 StatsService methods implemented in full (Q4-A)
- Cache token columns added to request_logs (FD-Q1-A)

---

## Steps

- [ ] **Step 1**: Create `migrations/0003_request_logs_cache_tokens.sql` — ALTER TABLE to add `cache_read_tokens` and `cache_creation_tokens` INTEGER columns (DEFAULT 0), plus composite index `(created_at, provider, model)`

- [ ] **Step 2**: Extend `src/tracking/types.rs` — add `cache_read_tokens: u32` and `cache_creation_tokens: u32` fields to `UsageRecord`; update `make_record()` in writer tests to include the new fields

- [ ] **Step 3**: Update `src/tracking/writer.rs` — extend `insert_record()` SQL to bind `cache_read_tokens` and `cache_creation_tokens`; update `make_record()` helper in test module

- [ ] **Step 4**: Update `src/proxy/handlers.rs` — populate `cache_read_tokens` and `cache_creation_tokens` in `UsageRecord` from `raw_usage`; add `state.metrics.record_cache(raw_usage.cache_read_tokens, raw_usage.cache_creation_tokens)` call after existing `record_tokens` call

- [ ] **Step 5**: Create `src/stats/types.rs` — `TimeWindow` enum with `TryFrom<&str>`, `StatsError` enum with `IntoResponse`, and all five response types: `StatsSummary`, `ProviderSummary`, `UsageDataPoint`, `SessionStats`, `CacheStats`; unit tests for TimeWindow parsing and StatsError HTTP mapping

- [ ] **Step 6**: Create `src/stats/service.rs` — real `StatsService { db: SqlitePool }` with all four query methods (`summary`, `usage_timeseries`, `session_stats`, `cache_stats`); internal helpers `window_cutoff()` and `bucket_expr()`; unit tests using in-memory SQLite for each method

- [ ] **Step 7**: Create `src/stats/handlers.rs` — four Axum handler functions and `resolve_window()` helper; each handler extracts `Query<HashMap<String, String>>`, resolves TimeWindow, calls StatsService, returns `Json<T>`

- [ ] **Step 8**: Replace `src/stats/mod.rs` stub — wire `pub mod types`, `pub mod service`, `pub mod handlers`; re-export `StatsService`, `StatsError`; remove old stub types (`StatsResponse`, `get_stats`)

- [ ] **Step 9**: Replace `src/metrics/mod.rs` stub — real `MetricsRegistry` with per-instance `prometheus::Registry`, three `IntCounterVec` metrics, `record_request()`, `record_tokens()`, `record_cache()` (new), and `render()`; unit tests for counter increments and render output format

- [ ] **Step 10**: Update `src/app_state.rs` — change `stats: StatsService::new()` to `stats: StatsService::new(db.clone())` in `AppState::new()`

- [ ] **Step 11**: Update `src/lib.rs` — add four `/api/stats/*` routes to `management_routes`; add necessary imports for stats handlers

- [ ] **Step 12**: Create `tests/stats_test.rs` — integration tests: stats endpoints return 200 OK with correct JSON shape on empty DB; unauthenticated request returns 401; unknown window returns 400

- [ ] **Step 13**: Run `cargo build && cargo test` — confirm zero errors, all tests pass

- [ ] **Step 14**: Create `aidlc-docs/construction/unit-4-stats-metrics/code/code-summary.md`
