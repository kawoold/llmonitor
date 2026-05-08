# Domain Entities — Unit 4: Statistics & Prometheus Metrics

## TimeWindow

Parsed from the `?window=` query parameter. Defaults to `OneDay` when absent.

```
TimeWindow
  OneHour    ("1h")   — last 60 minutes
  OneDay     ("24h")  — last 24 hours  [DEFAULT]
  SevenDays  ("7d")   — last 7 days
  ThirtyDays ("30d")  — last 30 days
```

Each variant maps to an `INTERVAL` offset used in SQLite `datetime()` expressions:
- `1h`  → `-1 hours`
- `24h` → `-24 hours`
- `7d`  → `-7 days`
- `30d` → `-30 days`

Unknown string → parse error → HTTP 400.

---

## StatsSummary

Aggregate totals across all request_logs rows within the window, grouped by provider and model.

```
StatsSummary
  window          : String         — echo of resolved window label
  total_requests  : i64
  total_tokens    : i64
  prompt_tokens   : i64
  completion_tokens: i64
  by_provider     : Vec<ProviderSummary>

ProviderSummary
  provider        : String
  model           : String
  request_count   : i64
  prompt_tokens   : i64
  completion_tokens: i64
  total_tokens    : i64
```

---

## UsageDataPoint

One time-bucket entry in a usage time-series. Bucket granularity varies by window:
- `1h`  → 1-minute buckets  (up to 60 points)
- `24h` → 1-hour buckets    (up to 24 points)
- `7d`  → 6-hour buckets    (up to 28 points)
- `30d` → 1-day buckets     (up to 30 points)

```
UsageDataPoint
  bucket          : String    — ISO 8601 truncated to bucket granularity
  request_count   : i64
  prompt_tokens   : i64
  completion_tokens: i64
  total_tokens    : i64
```

---

## SessionStats

Per-session aggregate: groups request_logs by session_id within the window.

```
SessionStats
  session_id      : String
  request_count   : i64
  total_tokens    : i64
  first_seen      : String    — earliest created_at in window
  last_seen       : String    — latest created_at in window
```

---

## CacheStats

Aggregate of cache_read_tokens and cache_creation_tokens within the window.

```
CacheStats
  window                  : String
  cache_read_tokens       : i64    — tokens served from cache (cost saving)
  cache_creation_tokens   : i64    — tokens written to cache (cost)
  total_requests          : i64    — requests with any cache activity
  cache_hit_rate          : f64    — cache_read_tokens / (cache_read_tokens + cache_creation_tokens), or 0.0 if both are 0
```

---

## Extended UsageRecord (Unit 3 extension)

`UsageRecord` gains two new fields to carry cache data through the tracking pipeline:

```
UsageRecord (extended)
  id                  : String
  created_at          : String
  provider            : String
  model               : String
  session_id          : String
  prompt_tokens       : u32
  completion_tokens   : u32
  total_tokens        : u32
  cache_read_tokens   : u32    [NEW]
  cache_creation_tokens: u32   [NEW]
```

---

## MetricsRegistry

Owns a private `prometheus::Registry`. All metric families registered into that instance only.

```
MetricsRegistry
  registry            : prometheus::Registry     — per-instance, not global
  requests_total      : IntCounterVec            — labels: provider, model, status
  tokens_total        : IntCounterVec            — labels: provider, model, kind (prompt|completion)
  cache_tokens_total  : IntCounterVec            — labels: kind (read|creation)
```

`render()` uses `prometheus::TextEncoder` against the instance registry, returning valid Prometheus text exposition format.

---

## StatsError

```
StatsError
  DatabaseError(String)     — sqlx query failure
  InvalidWindow(String)     — unrecognised window string
```

Maps to HTTP responses:
- `DatabaseError` → 500
- `InvalidWindow`  → 400
