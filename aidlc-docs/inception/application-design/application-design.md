# Application Design — llmonitor LLM Proxy

## Overview

llmonitor is a single Rust binary that acts as an OpenAI-compatible LLM proxy with usage tracking, analytics, and a React management frontend. It routes client requests to upstream providers (initially Anthropic Claude) and asynchronously persists token usage to SQLite.

## Architecture Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Project structure | Single Rust crate with domain modules | Simpler build, no workspace overhead for a single binary |
| Provider dispatch | Enum (`Provider::Claude`) | Zero-cost dispatch; extensible by adding enum variants |
| Token tracking write path | Bounded mpsc channel + background writer | Non-blocking response path; backpressure; enables batching |
| Frontend serving | Embedded via rust-embed | Single self-contained Docker binary; no separate static file volume |
| Admin bootstrap | Env vars on first startup | 12-factor compatible; clear error if not set; credentials hashed immediately |

## Component Map

```
src/
├── main.rs                  C-01 HttpServer
├── config/
│   ├── service.rs           C-10 ConfigService
│   └── bootstrap.rs        C-13 AdminBootstrapService
├── proxy/
│   ├── handlers.rs          C-02 ProxyHandler
│   ├── routing.rs           (provider resolution)
│   ├── frontend.rs          C-12 FrontendAssets
│   └── error.rs             (error translation)
├── providers/
│   ├── mod.rs               C-03 Provider enum
│   └── claude/
│       ├── mod.rs           C-04 ClaudeAdapter
│       └── types.rs         (Anthropic API types)
├── tracking/
│   ├── service.rs           C-06 TrackingService
│   ├── writer.rs            C-07 TrackingWriterService
│   ├── session.rs           C-05 SessionEngine
│   └── types.rs             (UsageRecord, RawUsage)
├── stats/
│   └── service.rs           C-08 StatsService
├── metrics/
│   └── registry.rs          C-09 MetricsRegistry
├── auth/
│   └── middleware.rs        C-11 AuthMiddleware
└── db/
    └── mod.rs               (SqlitePool setup, WAL mode)

migrations/
├── 0001_initial.sql         (request_logs + config tables)

frontend/
├── src/                     React + TypeScript SPA
└── dist/                    (build output, embedded by rust-embed)
```

## HTTP Route Map

| Route | Auth | Handler | Component |
|---|---|---|---|
| `POST /v1/chat/completions` | None | chat_completion | C-02 ProxyHandler |
| `GET /api/stats/summary` | Basic | stats summary | C-08 StatsService |
| `GET /api/stats/usage` | Basic | usage timeseries | C-08 StatsService |
| `GET /api/stats/sessions` | Basic | session stats | C-08 StatsService |
| `GET /api/stats/cache` | Basic | cache stats | C-08 StatsService |
| `GET /api/config` | Basic | get config | C-10 ConfigService |
| `PUT /api/config` | Basic | update config | C-10 ConfigService |
| `PUT /api/config/credentials` | Basic | update credentials | C-10 ConfigService |
| `GET /metrics` | None | prometheus | C-09 MetricsRegistry |
| `GET /*` (fallback) | None | SPA fallback | C-12 FrontendAssets |

## Key Data Types

### UsageRecord (persisted to SQLite)
- `id`: UUID primary key
- `timestamp`: UTC datetime
- `provider`: e.g. "claude"
- `model`: e.g. "claude-opus-4-5"
- `session_id`: SHA-256 hex of prior message history
- `input_tokens`, `output_tokens`, `total_tokens`
- `cache_hit_tokens`, `cache_miss_tokens`
- `duration_ms`, `http_status`

### Config (DB-backed)
- `claude_api_key`: Anthropic API key (stored encrypted or as plaintext in SQLite — see NFR Design)
- `proxy_timeout_secs`, `max_body_size_bytes`
- `tracking_batch_size`, `tracking_flush_interval_ms`
- `admin_username`, `admin_password_hash` (Argon2id)

## Security Posture Summary

- **Proxy API** (`/v1/*`): No auth — trusted internal network model
- **Management API** (`/api/*`): HTTP Basic Auth → Argon2id password verification
- **Prometheus** (`/metrics`): No auth (metrics contain no sensitive data)
- **Secrets in logs**: API keys never logged; password hashes never logged
- **Input validation**: All API endpoints use typed serde deserialization + explicit size limits
- **Error responses**: Generic messages only; no stack traces exposed

## Component Reference Index

| ID | Name | Module |
|---|---|---|
| C-01 | HttpServer | `src/main.rs`, `src/middleware/` |
| C-02 | ProxyHandler | `src/proxy/handlers.rs` |
| C-03 | Provider enum | `src/providers/mod.rs` |
| C-04 | ClaudeAdapter | `src/providers/claude/` |
| C-05 | SessionEngine | `src/tracking/session.rs` |
| C-06 | TrackingService | `src/tracking/service.rs` |
| C-07 | TrackingWriterService | `src/tracking/writer.rs` |
| C-08 | StatsService | `src/stats/service.rs` |
| C-09 | MetricsRegistry | `src/metrics/registry.rs` |
| C-10 | ConfigService | `src/config/service.rs` |
| C-11 | AuthMiddleware | `src/auth/middleware.rs` |
| C-12 | FrontendAssets | `src/proxy/frontend.rs` |
| C-13 | AdminBootstrapService | `src/config/bootstrap.rs` |

For detailed method signatures see `component-methods.md`.
For service orchestration patterns see `services.md`.
For dependency graph and data flows see `component-dependency.md`.
