# Unit of Work Dependencies — llmonitor LLM Proxy

## Build Sequence (Resolved)

```
Unit 1: Proxy Core
  └── Unit 5: Config Management
        └── Unit 2: Claude Provider Adapter
              └── Unit 3: Token Tracking & Session Engine
                    └── Unit 4: Statistics & Prometheus Metrics
                          └── Unit 6: React Frontend
```

All dependencies are linear — each unit depends on the one before it.

---

## Dependency Matrix

| Unit | Depends On | Reason |
|---|---|---|
| Unit 1: Proxy Core | None | Foundation — no upstream dependencies |
| Unit 5: Config Management | Unit 1 | Needs Axum router, DB pool, migration runner from Unit 1 |
| Unit 2: Claude Adapter | Unit 5 | Reads Claude API key from ConfigService (Unit 5) |
| Unit 3: Token Tracking | Unit 2 | Hooks into ProxyHandler (Unit 1) after real provider (Unit 2) is wired; request_logs migration must run |
| Unit 4: Statistics | Unit 3 | Queries request_logs table (Unit 3); MetricsRegistry updates wired into pipeline (Unit 1/2) |
| Unit 6: React Frontend | Unit 4 | All API endpoints (`/api/stats/*`, `/api/config`, `/metrics`) must be stable |

---

## Inter-Unit Interface Contracts

### Unit 1 → Unit 5
- **Provides**: `SqlitePool`, Axum `Router` extension point, migration runner (`sqlx::migrate!`)
- **Requires from Unit 5**: Config table migration `0001_config.sql`; `ConfigService` + `AuthMiddleware` wired into `AppState` and router

### Unit 5 → Unit 2
- **Provides**: `ConfigService::get_config()` returning `Config { claude_api_key, .. }`
- **Requires from Unit 2**: `Provider::Claude(ClaudeAdapter)` constructed with API key from `Config`; `Provider::from_name("claude", &config)` functional

### Unit 2 → Unit 3
- **Provides**: `RawUsage` extracted from provider responses inside `ProxyHandler`
- **Requires from Unit 3**: `TrackingService::record(UsageRecord)` available in `AppState`; `SessionEngine::derive_session_id()` callable from `ProxyHandler`; `request_logs` migration `0002_request_logs.sql`

### Unit 3 → Unit 4
- **Provides**: Populated `request_logs` table rows with all fields including `session_id`, `cache_hit_tokens`, `cache_miss_tokens`
- **Requires from Unit 4**: `StatsService` methods available in `AppState`; `MetricsRegistry::record_request/tokens/cache()` callable from `ProxyHandler`; `/api/stats/*` and `/metrics` routes registered in router

### Unit 4 → Unit 6
- **Provides**: All `/api/stats/*` endpoints, `/api/config` endpoints, `/metrics` endpoint — all stable and returning correct data
- **Requires from Unit 6**: Compiled `frontend/dist/` present at `cargo build` time for `rust-embed` to embed; `docker-compose.yml` and `.env.example` added to project root

---

## Shared Resources

| Resource | Owner (Created By) | Consumers |
|---|---|---|
| `SqlitePool` | Unit 1 | All units |
| `AppState` struct | Unit 1 (skeleton) | Extended by Units 5, 3, 4 |
| `Arc<ConfigService>` | Unit 5 | Units 2, 5, 11 (auth middleware) |
| `mpsc channel` | Unit 3 | Unit 3 (sender in TrackingService), Unit 3 (receiver in TrackingWriterService) |
| `request_logs` table | Unit 3 | Units 4 (queries) |
| `config` table | Unit 5 | Units 5 (CRUD), 2 (API key read) |
| `Arc<MetricsRegistry>` | Unit 4 | Unit 1/2 (ProxyHandler calls record methods) |
| `frontend/dist/` | Unit 6 (build) | Unit 1 (rust-embed at compile time) |

---

## Database Migration Sequence

| Migration File | Unit | Table Created |
|---|---|---|
| `migrations/0001_config.sql` | Unit 5 | `config` |
| `migrations/0002_request_logs.sql` | Unit 3 | `request_logs` |

Migrations are applied in order at server startup via `sqlx::migrate!()` (wired in Unit 1's DB setup).

---

## Build Validation Gates

After each unit, the following must pass before starting the next:

| After Unit | Validation Gate |
|---|---|
| Unit 1 | `cargo build` succeeds; `GET /health` returns 200; Docker image builds |
| Unit 5 | Admin bootstrap works; `GET /api/config` returns 200 with Basic Auth; 401 without |
| Unit 2 | `POST /v1/chat/completions` with `X-Provider: claude` returns real Anthropic response |
| Unit 3 | Request to proxy results in `request_logs` row in SQLite; session IDs consistent |
| Unit 4 | `GET /api/stats/summary` returns non-empty JSON; `/metrics` returns valid Prometheus text |
| Unit 6 | `npm run build` + `cargo build` succeeds; dashboard shows real data; docker-compose works |
