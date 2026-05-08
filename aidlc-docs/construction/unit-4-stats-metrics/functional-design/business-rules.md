# Business Rules — Unit 4: Statistics & Prometheus Metrics

## BR-U4-01: TimeWindow default
When `?window=` is absent from a stats endpoint request, the resolved window MUST be `24h`.

## BR-U4-02: TimeWindow validation
When `?window=` is present with a value other than `1h`, `24h`, `7d`, or `30d`, the endpoint MUST return HTTP 400 with error code `invalid_window`.

## BR-U4-03: Stats endpoint authentication
All `/api/stats/*` endpoints require Basic Auth (inherits from the existing `/api` route group auth middleware). No special handling needed — nesting under `management_routes` is sufficient.

## BR-U4-04: Metrics endpoint — no authentication
`GET /metrics` MUST NOT require authentication. It is already wired separately from the `/api` group in `lib.rs`.

## BR-U4-05: MetricsRegistry isolation
`MetricsRegistry::new()` MUST create a fresh `prometheus::Registry` per call. It MUST NOT use `prometheus::default_registry()`. This prevents panic-on-duplicate-registration when multiple instances are created in the same process (e.g., during tests).

## BR-U4-06: Cache token columns
`request_logs` MUST store `cache_read_tokens` and `cache_creation_tokens` as `INTEGER NOT NULL DEFAULT 0`. Rows from before migration 0003 receive value 0 via the column default.

## BR-U4-07: Cache hit rate calculation
`cache_hit_rate` in `CacheStats` MUST be computed as:
```
cache_read_tokens / (cache_read_tokens + cache_creation_tokens)
```
When both are 0, `cache_hit_rate` MUST be `0.0` (not NaN, not a divide-by-zero error).

## BR-U4-08: Time-series bucket granularity
`usage_timeseries` MUST use the following SQLite `strftime` format strings per window to produce consistent bucket labels:
- `1h`  → `strftime('%Y-%m-%dT%H:%M:00Z', created_at)` — per-minute
- `24h` → `strftime('%Y-%m-%dT%H:00:00Z', created_at)` — per-hour
- `7d`  → strftime with 6-hour grouping via `(strftime('%H', created_at) / 6) * 6` — per-6-hours
- `30d` → `strftime('%Y-%m-%dT00:00:00Z', created_at)` — per-day

## BR-U4-09: Prometheus metric naming
All custom Prometheus metrics MUST use the `llmonitor_` prefix:
- `llmonitor_requests_total` — labels: `provider`, `model`, `status`
- `llmonitor_tokens_total` — labels: `provider`, `model`, `kind` (values: `prompt`, `completion`)
- `llmonitor_cache_tokens_total` — labels: `kind` (values: `read`, `creation`)

## BR-U4-10: Prometheus counter update in handler
After a successful `chat_completion` response, `handlers.rs` MUST call:
- `state.metrics.record_request(provider, model, status_code)` — already present (stub)
- `state.metrics.record_tokens(provider, model, prompt, completion)` — already present (stub)
- `state.metrics.record_cache(cache_read, cache_creation)` — NEW call to be added

## BR-U4-11: StatsService DB injection
`StatsService` MUST accept a `SqlitePool` in its constructor and hold it as a field. The existing stub `StatsService::new()` (no args) MUST be replaced with `StatsService::new(db: SqlitePool)`. `AppState::new()` MUST pass `db.clone()` when constructing `StatsService`.
