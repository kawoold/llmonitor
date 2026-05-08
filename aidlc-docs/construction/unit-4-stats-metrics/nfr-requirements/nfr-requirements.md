# NFR Requirements — Unit 4: Statistics & Prometheus Metrics

## Performance

**NFR-U4-PERF-01**: Stats endpoint response time target is sub-second (< 1 second) for `request_logs` tables up to ~100k rows. No in-memory result cache is required. Existing indexes on `created_at`, `provider`, and `model` (migration 0002) are sufficient for the rolling window queries. If queries degrade at larger sizes, index tuning is a future concern.

**NFR-U4-PERF-02**: `GET /metrics` Prometheus text rendering is in-process (no DB query). It must complete in < 10ms under all conditions.

---

## Reliability

**NFR-U4-REL-01**: When a `StatsService` database query fails, the endpoint returns HTTP 500 with a structured JSON error body consistent with the existing proxy error format:
```json
{ "error": { "code": "database_error", "message": "<detail>" } }
```
No partial/stale data is returned on failure.

**NFR-U4-REL-02**: `MetricsRegistry` counter updates are infallible from the caller's perspective. The `with_label_values(...).inc()` / `inc_by()` calls do not return errors. Any internal panic is a programming error (invalid label set), not a runtime failure.

**NFR-U4-REL-03**: `cache_hit_rate` computation is zero-safe: when both `cache_read_tokens` and `cache_creation_tokens` are 0, the result is `0.0` — never NaN or a divide-by-zero panic.

---

## Security

**NFR-U4-SEC-01**: All `/api/stats/*` endpoints inherit Basic Auth from the existing `management_routes` route group. No additional auth logic is needed.

**NFR-U4-SEC-02**: `GET /metrics` requires no authentication. It is already wired outside the auth-protected route group in `lib.rs`.

**NFR-U4-SEC-03**: Stats query results contain aggregated token counts and session IDs (SHA-256 hashes) — no raw message content, no API keys. No additional data masking is needed.

---

## Scalability / Cardinality

**NFR-U4-SCALE-01**: Prometheus label design — `llmonitor_requests_total` uses labels `provider`, `model`, `status`. `llmonitor_tokens_total` uses labels `provider`, `model`, `kind`. For a personal/team proxy with a handful of Claude models, total time-series count is bounded at ~10–20 series. This is acceptable; no label reduction is needed.

**NFR-U4-SCALE-02**: `session_stats()` query applies `LIMIT 100` to bound response size. No pagination is required at this stage.

---

## Maintainability

**NFR-U4-MAINT-01**: `StatsService` uses `sqlx::query()` (runtime queries, not `query!` macros) — consistent with the rest of the codebase, which does not have `DATABASE_URL` set at compile time.

**NFR-U4-MAINT-02**: `MetricsRegistry` uses a per-instance `prometheus::Registry` (not the global default registry). This allows multiple instances in tests without panic-on-duplicate-registration.

**NFR-U4-MAINT-03**: `TimeWindow` parsing returns a typed error (`StatsError::InvalidWindow`) rather than panicking or silently defaulting. Callers receive HTTP 400 with error code `invalid_window`.
