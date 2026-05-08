# Business Logic Model — Unit 4: Statistics & Prometheus Metrics

## Overview

Unit 4 has two independent concerns:
1. **DB-backed analytics** — `StatsService` queries `request_logs` with rolling window filters
2. **In-process Prometheus counters** — `MetricsRegistry` updated synchronously by the proxy handler after each response

They share no state and are wired separately into `AppState`.

---

## 1. Schema Extension (feeds both concerns)

### Migration 0003
Adds two columns to `request_logs`:
```sql
ALTER TABLE request_logs ADD COLUMN cache_read_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE request_logs ADD COLUMN cache_creation_tokens INTEGER NOT NULL DEFAULT 0;
```

### UsageRecord extension
`cache_read_tokens: u32` and `cache_creation_tokens: u32` added to `UsageRecord`.
Populated in `handlers.rs` from `raw_usage.cache_read_tokens` / `raw_usage.cache_creation_tokens`.
TrackingWriterService INSERT extended to bind both new fields.

---

## 2. StatsService Query Logic

### TimeWindow → SQL cutoff

```
fn window_cutoff(window: TimeWindow) -> &'static str {
    match window {
        OneHour    => "datetime('now', '-1 hours')",
        OneDay     => "datetime('now', '-24 hours')",
        SevenDays  => "datetime('now', '-7 days')",
        ThirtyDays => "datetime('now', '-30 days')",
    }
}
```

All queries apply `WHERE created_at >= {cutoff}`.

---

### summary()

```sql
SELECT
    provider,
    model,
    COUNT(*) as request_count,
    SUM(prompt_tokens) as prompt_tokens,
    SUM(completion_tokens) as completion_tokens,
    SUM(total_tokens) as total_tokens
FROM request_logs
WHERE created_at >= {cutoff}
GROUP BY provider, model
ORDER BY total_tokens DESC
```

Top-level totals (`total_requests`, `total_tokens`, etc.) are computed in Rust by summing the per-provider rows.

---

### usage_timeseries()

Bucket granularity per window (BR-U4-08):

```
1h  → strftime('%Y-%m-%dT%H:%M:00Z', created_at)
24h → strftime('%Y-%m-%dT%H:00:00Z', created_at)
7d  → strftime('%Y-%m-%dT', created_at) || printf('%02d', (CAST(strftime('%H', created_at) AS INTEGER) / 6) * 6) || ':00:00Z'
30d → strftime('%Y-%m-%dT00:00:00Z', created_at)
```

```sql
SELECT
    {bucket_expr} as bucket,
    COUNT(*) as request_count,
    SUM(prompt_tokens) as prompt_tokens,
    SUM(completion_tokens) as completion_tokens,
    SUM(total_tokens) as total_tokens
FROM request_logs
WHERE created_at >= {cutoff}
GROUP BY bucket
ORDER BY bucket ASC
```

---

### session_stats()

```sql
SELECT
    session_id,
    COUNT(*) as request_count,
    SUM(total_tokens) as total_tokens,
    MIN(created_at) as first_seen,
    MAX(created_at) as last_seen
FROM request_logs
WHERE created_at >= {cutoff}
GROUP BY session_id
ORDER BY total_tokens DESC
LIMIT 100
```

---

### cache_stats()

```sql
SELECT
    SUM(cache_read_tokens) as cache_read_tokens,
    SUM(cache_creation_tokens) as cache_creation_tokens,
    COUNT(*) FILTER (WHERE cache_read_tokens > 0 OR cache_creation_tokens > 0) as total_requests
FROM request_logs
WHERE created_at >= {cutoff}
```

`cache_hit_rate` computed in Rust:
```
if cache_read + cache_creation == 0 { 0.0 }
else { cache_read as f64 / (cache_read + cache_creation) as f64 }
```

---

## 3. MetricsRegistry Logic

### Construction

```
MetricsRegistry::new() {
    let registry = prometheus::Registry::new();
    let requests_total = IntCounterVec::new(opts, &["provider", "model", "status"]);
    let tokens_total = IntCounterVec::new(opts, &["provider", "model", "kind"]);
    let cache_tokens_total = IntCounterVec::new(opts, &["kind"]);
    registry.register(Box::new(requests_total.clone()));
    registry.register(Box::new(tokens_total.clone()));
    registry.register(Box::new(cache_tokens_total.clone()));
    Self { registry, requests_total, tokens_total, cache_tokens_total }
}
```

### record_request(provider, model, status)

```
requests_total
    .with_label_values(&[provider, model, &status.to_string()])
    .inc();
```

### record_tokens(provider, model, prompt, completion)

```
tokens_total.with_label_values(&[provider, model, "prompt"]).inc_by(prompt as u64);
tokens_total.with_label_values(&[provider, model, "completion"]).inc_by(completion as u64);
```

### record_cache(cache_read, cache_creation)

```
cache_tokens_total.with_label_values(&["read"]).inc_by(cache_read as u64);
cache_tokens_total.with_label_values(&["creation"]).inc_by(cache_creation as u64);
```

### render()

```
let mut buffer = String::new();
let encoder = prometheus::TextEncoder::new();
let metric_families = self.registry.gather();
encoder.encode_utf8(&metric_families, &mut buffer)?;
buffer
```

---

## 4. REST Endpoint Wiring

All four endpoints added to the existing `management_routes` builder (which already has Basic Auth applied):

```
GET /api/stats/summary?window=24h   → stats::handlers::get_summary
GET /api/stats/usage?window=24h     → stats::handlers::get_usage
GET /api/stats/sessions?window=24h  → stats::handlers::get_sessions
GET /api/stats/cache?window=24h     → stats::handlers::get_cache
```

Query param parsing: `axum::extract::Query<HashMap<String, String>>`, key `"window"`.
Missing key → default `"24h"`. Unrecognised value → return `StatsError::InvalidWindow`.

---

## 5. Interaction with Existing Pipeline

`handlers.rs` changes after this unit:
1. `state.metrics.record_cache(raw_usage.cache_read_tokens, raw_usage.cache_creation_tokens)` — new call after existing `record_tokens` call
2. `UsageRecord` construction gains `cache_read_tokens` and `cache_creation_tokens` fields from `raw_usage`

No other existing code changes are needed.
