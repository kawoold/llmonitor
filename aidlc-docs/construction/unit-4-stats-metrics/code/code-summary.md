# Unit 4 — Statistics & Prometheus Metrics: Code Summary

## Status: COMPLETE ✅

## Files Created / Modified

| File | Action | Description |
|------|--------|-------------|
| `migrations/0003_request_logs_cache_tokens.sql` | Created | ALTER TABLE adds `cache_read_tokens`, `cache_creation_tokens`; composite index `(created_at, provider, model)` |
| `src/tracking/types.rs` | Updated | `UsageRecord` gains `cache_read_tokens: u32` and `cache_creation_tokens: u32` |
| `src/tracking/service.rs` | Updated | `dummy_record()` test helper updated with new fields |
| `src/tracking/writer.rs` | Updated | INSERT SQL extended to 10 columns; `make_record()` test helper updated |
| `src/proxy/handlers.rs` | Updated | `UsageRecord` populates cache fields from `raw_usage`; `record_cache()` call added; `created_at` uses `rfc3339_opts(Secs, true)` for `Z`-suffix format compatible with SQLite comparisons |
| `src/stats/types.rs` | Created | `TimeWindow` enum + `TryFrom<&str>`; `StatsError` + `IntoResponse`; all 5 response types |
| `src/stats/service.rs` | Created | `StatsService { db }` with all 4 query methods; `window_cutoff()` + `bucket_expr()` helpers; 6 unit tests |
| `src/stats/handlers.rs` | Created | 4 Axum handlers + `resolve_window()` helper |
| `src/stats/mod.rs` | Replaced stub | Wires `types`, `service`, `handlers`; re-exports `StatsService`, `StatsError` |
| `src/metrics/mod.rs` | Replaced stub | Real `MetricsRegistry` with per-instance `prometheus::Registry`; 3 `IntCounterVec` counters; `record_cache()` method; 5 unit tests |
| `src/app_state.rs` | Updated | `StatsService::new(db.clone())` replaces no-arg stub call |
| `src/lib.rs` | Updated | 4 `/api/stats/*` routes added to `management_routes` |
| `tests/stats_test.rs` | Created | 6 integration tests: auth, empty results, invalid window, all 4 endpoints |
| `tests/tracking_test.rs` | Updated | `make_record()` updated with new `UsageRecord` fields |

## Key Design Decisions

- **`Z`-suffix timestamps**: `handlers.rs` stores `created_at` as `"2026-05-08T14:30:00Z"` (via `rfc3339_opts(Secs, true)`). SQLite cutoffs use `strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '...')` for lexicographically correct rolling-window comparisons.
- **Per-instance `prometheus::Registry`**: No global registry; each `MetricsRegistry::new()` creates a fresh registry. Tests can create multiple instances without panic.
- **Dynamic SQL for stats queries**: `format!()` with only hardcoded static strings from `window_cutoff()` and `bucket_expr()` — no user input interpolated, no injection risk.
- **`COALESCE` on aggregations**: Ensures `SUM()` returns `0` not `NULL` on empty tables.

## Test Coverage

- 69 unit tests (all passing)
- 18 integration tests (all passing)
- 6 new integration tests in `tests/stats_test.rs`
- 6 new unit tests in `src/stats/service.rs`
- 3 new unit tests in `src/stats/types.rs`
- 5 unit tests in `src/metrics/mod.rs`
