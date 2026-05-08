# Tech Stack Decisions — Unit 4: Statistics & Prometheus Metrics

## Already Fixed (inherited from prior units)

| Concern | Technology | Rationale |
|---------|-----------|-----------|
| HTTP framework | Axum 0.7 | Established in Unit 1 |
| Database | SQLite via sqlx 0.7 | Established in Unit 1 |
| Async runtime | Tokio | Established in Unit 1 |
| Query style | `sqlx::query()` runtime queries | No `DATABASE_URL` at compile time |
| Serialisation | serde / serde_json | Established in Unit 1 |

## New Decisions — Unit 4

| Concern | Decision | Rationale |
|---------|----------|-----------|
| Prometheus client | `prometheus = "0.13"` (already in Cargo.toml) | Already declared; no new dep needed |
| Registry strategy | Per-instance `prometheus::Registry` | Prevents duplicate-registration panic in tests |
| Stats query result types | Plain Rust structs with `#[derive(Serialize)]` | Consistent with existing response types; no ORM needed |
| `TimeWindow` parsing | `impl TryFrom<&str> for TimeWindow` | Idiomatic Rust; integrates cleanly with Axum query extractor |
| Stats error type | `thiserror`-derived `StatsError` enum | Consistent with existing error handling pattern |
| Stats endpoint location | `src/stats/handlers.rs` (new file) + route wiring in `lib.rs` | Keeps handler logic co-located with the service it calls |
