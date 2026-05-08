# NFR Design Patterns — Unit 4: Statistics & Prometheus Metrics

## Resilience Pattern: Immediate Error Propagation

Stats queries do not retry on failure. SQLite errors are wrapped in `StatsError::DatabaseError` and propagated immediately to the HTTP layer as HTTP 500. Rationale: SQLite failures at this layer are either transient (WAL lock contention) or indicate a persistent fault — retrying inside the request handler adds latency without a meaningful success probability improvement. The client can retry the request.

```
StatsService query → Err(sqlx::Error) → StatsError::DatabaseError → HTTP 500
```

---

## Performance Pattern: Composite Index for Rolling Window Queries

Migration 0003 adds a composite index `(created_at, provider, model)` on `request_logs`. This allows SQLite to satisfy the WHERE clause (`created_at >= cutoff`) and the GROUP BY (`provider, model`) via a single index scan rather than a table scan + sort. Effective for rolling window aggregation queries across all four `StatsService` methods.

```sql
CREATE INDEX idx_request_logs_window_group
    ON request_logs(created_at, provider, model);
```

The existing single-column indexes (`idx_request_logs_created_at`, `idx_request_logs_provider`, `idx_request_logs_model`) are retained. SQLite's query planner will select the composite index for stats queries.

---

## Security Pattern: Inherited Auth via Route Nesting

All `/api/stats/*` routes are added to the existing `management_routes` `Router`, which has the Basic Auth middleware applied at the `route_layer` level. No new security logic is required — auth is enforced structurally by route group membership.

```
management_routes (Basic Auth applied)
  ├── /config         GET, PUT
  ├── /config/credentials  PUT
  ├── /stats/summary  GET   ← added in Unit 4
  ├── /stats/usage    GET   ← added in Unit 4
  ├── /stats/sessions GET   ← added in Unit 4
  └── /stats/cache    GET   ← added in Unit 4
```

---

## Error Mapping Pattern: IntoResponse for StatsError

`StatsError` implements `axum::response::IntoResponse`, matching the existing `ProxyError` pattern. Stats handlers return `Result<Json<T>, StatsError>` — Axum calls `into_response()` on the error variant automatically.

Error → HTTP mapping:

| Variant | Status | Code |
|---------|--------|------|
| `DatabaseError(String)` | 500 | `"database_error"` |
| `InvalidWindow(String)` | 400 | `"invalid_window"` |

JSON error body structure (consistent with proxy errors):
```json
{
  "error": {
    "code": "invalid_window",
    "message": "unknown window '1w'; valid values are 1h, 24h, 7d, 30d"
  }
}
```

---

## Scalability Pattern: Bounded Result Sets

`session_stats()` applies `LIMIT 100` in SQL. Other queries (`summary`, `usage_timeseries`, `cache_stats`) produce naturally bounded result sets:
- `summary`: one row per distinct `(provider, model)` pair — bounded by model count
- `usage_timeseries`: one row per time bucket — bounded by window size (max 60 points for `1h`)
- `cache_stats`: single aggregate row

No pagination required at current scale targets.
