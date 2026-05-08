# Unit 4: Statistics & Prometheus Metrics — Functional Design Questions

Please answer each question by filling in the letter choice after the `[Answer]:` tag.
If none of the options match your needs, choose the last option (Other) and describe your preference.
Let me know when you're done.

---

## Question 1
`RawUsage` already carries `cache_read_tokens` and `cache_creation_tokens` from the Anthropic response, but the `request_logs` table has no columns for them and `UsageRecord` does not include them. The design calls for a `GET /api/stats/cache` endpoint showing cache hit/miss rates.

How should cache token data be handled?

A) Add migration `0003_request_logs_cache_tokens.sql` with two new INTEGER columns (`cache_read_tokens`, `cache_creation_tokens` DEFAULT 0). Extend `UsageRecord`, update the writer INSERT, and populate from `raw_usage` in `handlers.rs`. Enables real cache analytics from the DB.

B) Skip DB storage for now. `GET /api/stats/cache` returns empty/zero data. Cache columns can be added later once the frontend is built.

C) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 2
The Prometheus `MetricsRegistry` needs to register metric families (counters, histograms). In Rust, registering duplicate metric names into the same registry panics at runtime. This matters in tests where multiple `MetricsRegistry::new()` calls happen in the same process.

Which registry isolation strategy should be used?

A) Per-instance custom `prometheus::Registry` — each `MetricsRegistry::new()` creates its own isolated registry. `render()` encodes only that instance's metrics. Tests never share state and can call `new()` freely.

B) Global default registry (`prometheus::default_registry()`) — simpler code path, but tests must never construct two `MetricsRegistry` instances in the same process or a panic occurs on duplicate registration.

C) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 3
All four `GET /api/stats/*` endpoints need a rolling time window filter (`1h`, `24h`, `7d`, `30d`). How should the window be specified by API callers?

A) Query parameter — e.g. `GET /api/stats/summary?window=24h`. Default to `24h` when the parameter is absent. Return HTTP 400 for unrecognised values.

B) Path segment — e.g. `GET /api/stats/summary/24h`. No query string parsing; the window is part of the route. Requires four route variants per endpoint.

C) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 4
The application design specifies four `StatsService` methods: `summary`, `usage_timeseries`, `session_stats`, and `cache_stats`. Should all four be implemented in this unit?

A) Implement all four methods and all four REST endpoints in full — complete the Unit 4 scope as designed.

B) Implement only `summary` and `usage_timeseries` (the two most useful for an initial dashboard). Stub `session_stats` and `cache_stats` to return empty data; complete them in a follow-on pass.

C) Other (please describe after [Answer]: tag below)

[Answer]: A
