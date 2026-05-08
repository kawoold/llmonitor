# Unit 4: Statistics & Prometheus Metrics — NFR Requirements Questions

Please answer each question by filling in the letter choice after the `[Answer]:` tag.
If none of the options match your needs, choose the last option (Other) and describe your preference.
Let me know when you're done.

---

## Question 1
Stats endpoints query `request_logs` with GROUP BY and aggregation. As the table grows (e.g. 100k+ rows), queries may slow down. Indexes on `created_at`, `provider`, and `model` are already in migration 0002.

What is the acceptable response time target for `/api/stats/*` endpoints?

A) Sub-second (< 1 second) for typical sizes (up to ~100k rows) is sufficient — no in-memory result cache needed. If queries slow down, indexes can be tuned later.

B) Add a short-lived in-memory cache (e.g. 30-second TTL per window) so repeated dashboard refreshes hit memory rather than SQLite. Accept slightly stale data.

C) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 2
When a `StatsService` database query fails (e.g. SQLite locked, I/O error), how should the stats endpoint respond?

A) Return HTTP 500 with a structured JSON error body (`{ "error": { "code": "...", "message": "..." } }`). Consistent with the existing proxy error format.

B) Return HTTP 200 with an empty/zero result and an `"error"` field in the response body so the frontend can display a degraded state rather than an error screen.

C) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 3
The `llmonitor_requests_total` Prometheus counter uses labels `provider`, `model`, and `status` (HTTP status code). The `model` label creates one time-series per distinct model value seen. For a personal proxy hitting a handful of Claude models, this is ~5–10 series total.

Is the current label design acceptable, or should it be simplified?

A) Accept the current design — `provider`, `model`, `status` on `requests_total` and `provider`, `model`, `kind` on `tokens_total`. Cardinality is bounded and acceptable for a personal/team tool.

B) Remove `model` from `requests_total` — keep it only on `tokens_total`. Reduces series count; model breakdown for request counts is not essential.

C) Other (please describe after [Answer]: tag below)

[Answer]: A
