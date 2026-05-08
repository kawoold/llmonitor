# Unit 4: Statistics & Prometheus Metrics — NFR Design Questions

Please answer each question by filling in the letter choice after the `[Answer]:` tag.
If none of the options match your needs, choose the last option (Other) and describe your preference.
Let me know when you're done.

---

## Question 1
The stats queries filter on `created_at` and GROUP BY `provider`, `model`. Migration 0002 added separate single-column indexes on each. A composite index on `(created_at, provider, model)` would let SQLite satisfy the WHERE + GROUP BY in a single index scan, improving GROUP BY performance at larger table sizes.

Should migration 0003 add a composite index alongside the cache token columns?

A) Yes — add `CREATE INDEX idx_request_logs_window_group ON request_logs(created_at, provider, model)` in migration 0003. Better query plan for stats aggregations now rather than retrofitting later.

B) No — the existing single-column indexes are sufficient for the ~100k row target. Defer composite index to a future migration if benchmarks show a problem.

C) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 2
`StatsError` needs to map to HTTP responses (400 for `InvalidWindow`, 500 for `DatabaseError`). The codebase already has `ProxyError` implementing `axum::response::IntoResponse` directly, which lets handlers return `Result<Json<T>, StatsError>` and Axum converts the error automatically.

Which error-to-HTTP mapping pattern should `StatsError` use?

A) Implement `IntoResponse` for `StatsError` — consistent with the existing `ProxyError` pattern. Handlers return `Result<Json<T>, StatsError>`. Structured JSON error body with `code` and `message` fields, matching the proxy error format.

B) Map errors manually in each handler with `.map_err()` — explicit conversion at the call site, no trait impl needed. More verbose but error → status mapping is visible in the handler body.

C) Other (please describe after [Answer]: tag below)

[Answer]: A
