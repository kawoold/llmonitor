# Logical Components — Unit 4: Statistics & Prometheus Metrics

## Component Map

```
src/
├── stats/
│   ├── mod.rs          — replaces stub; re-exports public API
│   ├── service.rs      — StatsService (DB-backed queries)
│   ├── handlers.rs     — Axum handlers for /api/stats/* endpoints  [NEW]
│   ├── types.rs        — TimeWindow, StatsSummary, UsageDataPoint,  [NEW]
│   │                     SessionStats, CacheStats, StatsError
│   └── (mod.rs wires all sub-modules)
├── metrics/
│   └── mod.rs          — replaces stub; real MetricsRegistry
├── proxy/
│   └── handlers.rs     — updated: record_cache() call + UsageRecord cache fields
├── tracking/
│   ├── types.rs        — updated: cache_read_tokens, cache_creation_tokens fields
│   └── writer.rs       — updated: INSERT binds two new cache columns
├── lib.rs              — updated: /api/stats/* routes added to management_routes
│                         AppState::new() updated (StatsService takes db)
└── migrations/
    └── 0003_request_logs_cache_tokens.sql  [NEW]
```

---

## StatsService

**Location**: `src/stats/service.rs`

Holds a `SqlitePool`. Exposes four async query methods, each accepting a `TimeWindow`. Returns typed result structs or `StatsError`.

```
StatsService
  db: SqlitePool

  summary(window: TimeWindow)         -> Result<StatsSummary, StatsError>
  usage_timeseries(window: TimeWindow)-> Result<Vec<UsageDataPoint>, StatsError>
  session_stats(window: TimeWindow)   -> Result<Vec<SessionStats>, StatsError>
  cache_stats(window: TimeWindow)     -> Result<CacheStats, StatsError>
```

Internal helper:
```
fn window_cutoff(window: TimeWindow) -> &'static str
  — returns SQLite datetime expression string for WHERE clause
fn bucket_expr(window: TimeWindow) -> &'static str
  — returns strftime expression for usage_timeseries GROUP BY bucket
```

---

## TimeWindow

**Location**: `src/stats/types.rs`

```
enum TimeWindow { OneHour, OneDay, SevenDays, ThirtyDays }

impl TryFrom<&str> for TimeWindow
  "1h"  → OneHour
  "24h" → OneDay
  "7d"  → SevenDays
  "30d" → ThirtyDays
  _     → Err(StatsError::InvalidWindow(s.to_string()))
```

Default (when `?window` is absent): `TimeWindow::OneDay`.

---

## StatsError

**Location**: `src/stats/types.rs`

```
enum StatsError {
  DatabaseError(String),   — from sqlx::Error
  InvalidWindow(String),   — from TimeWindow::try_from
}

impl From<sqlx::Error> for StatsError
impl IntoResponse for StatsError
  DatabaseError → 500, code "database_error"
  InvalidWindow → 400, code "invalid_window"
```

---

## Stats Response Types

**Location**: `src/stats/types.rs`

All types derive `Serialize`. Fields use `i64` for DB-sourced integers (sqlx returns i64 from INTEGER columns) and `f64` for computed ratios.

```
StatsSummary { window, total_requests, total_tokens, prompt_tokens, completion_tokens, by_provider }
ProviderSummary { provider, model, request_count, prompt_tokens, completion_tokens, total_tokens }
UsageDataPoint { bucket, request_count, prompt_tokens, completion_tokens, total_tokens }
SessionStats { session_id, request_count, total_tokens, first_seen, last_seen }
CacheStats { window, cache_read_tokens, cache_creation_tokens, total_requests, cache_hit_rate }
```

---

## Stats Handlers

**Location**: `src/stats/handlers.rs`

Four Axum handler functions. All extract `Query<HashMap<String, String>>` for the `window` param, resolve `TimeWindow`, delegate to `StatsService`, and return `Json<T>`.

```
get_summary(State(state), Query(params))   -> Result<Json<StatsSummary>, StatsError>
get_usage(State(state), Query(params))     -> Result<Json<Vec<UsageDataPoint>>, StatsError>
get_sessions(State(state), Query(params))  -> Result<Json<Vec<SessionStats>>, StatsError>
get_cache(State(state), Query(params))     -> Result<Json<CacheStats>, StatsError>
```

Window resolution helper (shared across all four):
```
fn resolve_window(params: &HashMap<String, String>) -> Result<TimeWindow, StatsError>
  params.get("window").map(|s| TimeWindow::try_from(s.as_str())).unwrap_or(Ok(TimeWindow::OneDay))
```

---

## MetricsRegistry

**Location**: `src/metrics/mod.rs` (replaces stub)

```
MetricsRegistry
  registry: prometheus::Registry           — per-instance, not global
  requests_total: IntCounterVec            — labels: provider, model, status
  tokens_total: IntCounterVec             — labels: provider, model, kind
  cache_tokens_total: IntCounterVec       — labels: kind

  new() -> Self                           — registers all metrics into instance registry
  record_request(&self, provider, model, status: u16)
  record_tokens(&self, provider, model, prompt: u32, completion: u32)
  record_cache(&self, cache_read: u32, cache_creation: u32)   [NEW method]
  render(&self) -> String                 — TextEncoder against instance registry
```

---

## Migration 0003

**Location**: `migrations/0003_request_logs_cache_tokens.sql`

```sql
ALTER TABLE request_logs ADD COLUMN cache_read_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE request_logs ADD COLUMN cache_creation_tokens INTEGER NOT NULL DEFAULT 0;
CREATE INDEX idx_request_logs_window_group ON request_logs(created_at, provider, model);
```

---

## AppState Changes

`StatsService::new()` (no-arg stub) → `StatsService::new(db: SqlitePool)`.
`AppState::new()` passes `db.clone()` to `StatsService::new()`.
`stats` field type changes from `StatsService` (stub) to `StatsService` (real, DB-backed).

No change to `AppState`'s public interface — `stats` field was already present.
