# Code Summary — Unit 1: Proxy Core

## Files Created

### Project Structure
| File | Purpose |
|---|---|
| `Cargo.toml` | Single-crate project with all dependencies (axum, sqlx, tower-http, rust-embed, argon2, prometheus) |
| `.gitignore` | Excludes target/, *.db, .env, frontend/node_modules/, frontend/dist/ (except .gitkeep) |
| `.dockerignore` | Excludes target/, *.db, .git/, *.md (keeps frontend/dist/) |
| `Dockerfile` | Multi-stage build: Node frontend → Rust builder (embedded dist/) → debian:bookworm-slim runtime |
| `migrations/.gitkeep` | Placeholder; real migrations added in Units 3 and 5 |
| `frontend/dist/index.html` | Placeholder until Unit 6 React build |
| `src/lib.rs` | Library crate root — exports all modules, `build_router()`, and `test_helpers` |
| `src/main.rs` | Binary entry point — startup sequence, graceful shutdown |

### Proxy Core
| File | Purpose |
|---|---|
| `src/proxy/mod.rs` | Module declarations |
| `src/proxy/types.rs` | OpenAI schema types: `OpenAiChatRequest`, `OpenAiMessage`, `MessageContent`, `OpenAiChatResponse`, `Choice`, `OpenAiUsage`, `OpenAiStreamChunk`, `StreamChoice`, `MessageDelta`, `RawUsage` |
| `src/proxy/error.rs` | `ProxyError` enum (12 variants) with `IntoResponse` mapping to OpenAI error envelope format |
| `src/proxy/routing.rs` | `resolve_provider()` — extracts `X-Provider` header, delegates to `Provider::from_name()` |
| `src/proxy/handlers.rs` | `chat_completion()` — full pipeline handler; `chat_completion_stream()` — 501 stub |
| `src/proxy/health.rs` | `health_handler()` — SQLite ping + uptime calculation |
| `src/proxy/frontend.rs` | `FrontendAssets` rust-embed struct; `serve_asset()` and SPA fallback `serve_index()` |

### Providers
| File | Purpose |
|---|---|
| `src/providers/mod.rs` | `Provider` enum (Claude variant), `ProviderError`, `Provider::from_name()`, accepted model lists, stub `chat_completion()` returning `ProviderError::NotImplemented` |
| `src/providers/claude/mod.rs` | `ClaudeAdapter` struct with `api_key` field — full implementation in Unit 2 |

### Configuration & Bootstrap
| File | Purpose |
|---|---|
| `src/config/mod.rs` | `Config` struct with env var loading; `ConfigService` stub |
| `src/config/types.rs` | `ApiKey` newtype — `Debug`/`Display` always emit `[REDACTED]` (LC-07) |
| `src/config/bootstrap.rs` | `bootstrap()` stub — real admin credential seeding in Unit 5 |

### Infrastructure
| File | Purpose |
|---|---|
| `src/db/mod.rs` | `create_pool()` — SQLite WAL mode + `synchronous=NORMAL` + `foreign_keys=ON` + `busy_timeout=5000`, pool of 10 |
| `src/app_state.rs` | `AppState` — holds `db`, `tracking`, `metrics`, `config`, `stats`, `started_at` |

### Middleware
| File | Purpose |
|---|---|
| `src/middleware/mod.rs` | Module declarations |
| `src/middleware/rate_limit.rs` | `TokenBucket` (capacity 100, refill 20/s); `RateLimitLayer` Tower middleware; `Retry-After` header on 429 |
| `src/middleware/security_headers.rs` | `security_headers_middleware` — adds X-Content-Type-Options, X-Frame-Options, Referrer-Policy, Cache-Control, CSP |

### Stub Modules (Implemented in Later Units)
| File | Implemented In |
|---|---|
| `src/tracking/mod.rs` | Unit 3 |
| `src/stats/mod.rs` | Unit 4 |
| `src/metrics/mod.rs` | Unit 4 |
| `src/auth/mod.rs` | Unit 5 |

### Tests
| File | Coverage |
|---|---|
| `src/proxy/tests.rs` | Provider resolution, case-insensitivity, error HTTP status codes, model validation |
| `src/proxy/types_tests.rs` | PBT round-trips for `OpenAiChatRequest` and `MessageContent` |
| `src/middleware/rate_limit.rs` (inline) | Token bucket: allow/deny/retry-after |
| `tests/health_test.rs` | `GET /health` → 200 + `{"status":"ok"}` |
| `tests/proxy_validation_test.rs` | Missing X-Provider → 400; unknown provider → 400 |

## Key Design Decisions

1. **Library + binary split**: `src/lib.rs` exposes all modules so integration tests can link against the library without re-compiling.
2. **Route path correction**: Proxy routes nested under `/v1` use `/chat/completions` (not `/v1/chat/completions`) to avoid double-prefix.
3. **Security headers as axum middleware**: `SetResponseHeaderLayer` from tower-http 0.5 does not support the `set-response-header` feature; replaced with a custom `axum::middleware::from_fn` layer.
4. **ApiKey redaction**: Newtype ensures API keys are never accidentally logged in any `{:?}` or `{}` format.
5. **test_helpers always compiled**: Not gated by `#[cfg(test)]` so integration tests in `tests/` can call `test_helpers::test_db()`.
