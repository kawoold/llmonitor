# llmonitor

A self-hosted Anthropic API proxy and observability server. Point your client at llmonitor instead of Anthropic directly — it forwards requests using a centrally-managed API key, logs every request to a local SQLite database, and serves a dashboard for monitoring usage, token consumption, and prompt cache efficiency.

## Features

- **Dual proxy modes**: OpenAI-compatible (`/v1/chat/completions`) and native Anthropic (`/v1/messages`) endpoints
- **Streaming support**: SSE streaming on both proxy modes with inline usage parsing
- **Usage tracking**: every request logged to SQLite with token counts, model, provider, and session
- **Session grouping**: deterministic SHA-256 session IDs derived from message history
- **Prompt cache visibility**: tracks `cache_read_tokens` and `cache_creation_tokens` per request
- **Prometheus metrics**: `/metrics` endpoint for scraping
- **Embedded dashboard**: React SPA baked into the binary — no separate server needed
- **Rate limiting**: token-bucket rate limiter on the proxy endpoint
- **Live config**: API key and rate limit settings updatable at runtime via the UI or API
- **Single binary**: zero runtime dependencies beyond the binary itself

## Quick Start

### Docker Compose (recommended)

```yaml
services:
  llmonitor:
    image: ghcr.io/your-org/llmonitor:latest
    ports:
      - "8080:8080"
    environment:
      ANTHROPIC_API_KEY: sk-ant-...
      ADMIN_USERNAME: admin
      ADMIN_PASSWORD: changeme
    volumes:
      - llmonitor_data:/data
    restart: unless-stopped

volumes:
  llmonitor_data:
```

### Binary

```bash
export ANTHROPIC_API_KEY=sk-ant-...
export ADMIN_USERNAME=admin
export ADMIN_PASSWORD=changeme
./llmonitor
```

The server starts on `0.0.0.0:8080` by default. Open `http://localhost:8080` for the dashboard.

## Configuration

All settings can be provided as environment variables at startup. The API key and rate limits can also be updated at runtime via the dashboard or the `/api/config` endpoint.

| Variable | Default | Description |
|---|---|---|
| `ANTHROPIC_API_KEY` | _(none)_ | Anthropic API key forwarded to upstream requests |
| `ADMIN_USERNAME` | _(required)_ | Username for the management UI and API |
| `ADMIN_PASSWORD` | _(required)_ | Password for the management UI and API |
| `DATABASE_URL` | `llmonitor.db` | SQLite file path (plain path or `sqlite://` URI) |
| `LISTEN_ADDR` | `0.0.0.0:8080` | Address and port to bind |
| `REQUEST_TIMEOUT_SECS` | `30` | Upstream request timeout in seconds |
| `RUST_LOG` | `llmonitor=info` | Log level filter |

## Proxy Usage

### OpenAI-compatible endpoint

Point any OpenAI SDK client at llmonitor by changing the base URL and providing Basic auth credentials. Set the `x-provider` header to specify the provider.

```python
import openai

client = openai.OpenAI(
    base_url="http://localhost:8080/v1",
    api_key="admin:changeme",  # Basic auth: username:password
    default_headers={"x-provider": "anthropic"},
)

response = client.chat.completions.create(
    model="claude-sonnet-4-6",
    messages=[{"role": "user", "content": "Hello"}],
)
```

Model aliases are supported — `claude-sonnet-4` resolves to `claude-sonnet-4-6`, `claude-opus-4` to `claude-opus-4-7`, etc.

**Prompt caching** can be controlled via the `anthropic_cache_control` extension field:

```python
response = client.chat.completions.create(
    model="claude-sonnet-4-6",
    messages=[
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content": "Hello"},
    ],
    extra_body={
        "anthropic_cache_control": {
            "system": True,       # cache the system prompt
            "messages": [0],      # cache message at index 0
        }
    },
)
```

### Native Anthropic passthrough

Send requests in the standard Anthropic Messages API format. The server substitutes its configured API key and forwards everything else verbatim, including `anthropic-beta` headers.

```python
import anthropic

client = anthropic.Anthropic(
    base_url="http://localhost:8080",
    api_key="irrelevant",  # server uses its own key; auth is via Basic auth below
    default_headers={"Authorization": "Basic YWRtaW46Y2hhbmdlbWU="},
)

message = client.messages.create(
    model="claude-sonnet-4-6",
    max_tokens=1024,
    messages=[{"role": "user", "content": "Hello"}],
)
```

## API Reference

All management endpoints require HTTP Basic authentication.

| Method | Path | Description |
|---|---|---|
| `GET` | `/health` | Health check (no auth) |
| `GET` | `/metrics` | Prometheus metrics (no auth) |
| `GET` | `/api/config` | Get current configuration (API key redacted) |
| `PUT` | `/api/config` | Update configuration settings |
| `PUT` | `/api/config/credentials` | Change admin password |
| `GET` | `/api/models` | List available models |
| `GET` | `/api/stats/summary?window=24h` | Token and request totals by provider/model |
| `GET` | `/api/stats/usage?window=24h` | Time-bucketed usage timeseries |
| `GET` | `/api/stats/sessions?window=24h` | Top sessions by token consumption |
| `GET` | `/api/stats/cache?window=24h` | Prompt cache hit rate and token breakdown |

**Time windows**: `1h`, `24h`, `7d`, `30d` (default: `24h`)

## Building from Source

**Prerequisites**: Rust 1.87+, Node.js 22+

```bash
# Build frontend
cd frontend && npm ci && npm run build && cd ..

# Build binary (frontend assets are embedded at compile time)
cargo build --release

# Run
./target/release/llmonitor
```

**Run tests**:

```bash
cargo test
```

## Docker

```bash
# Build image (builds frontend and Rust binary in separate stages)
docker build -t llmonitor .

# Run
docker run -p 8080:8080 \
  -e ANTHROPIC_API_KEY=sk-ant-... \
  -e ADMIN_USERNAME=admin \
  -e ADMIN_PASSWORD=changeme \
  -v llmonitor_data:/data \
  llmonitor
```

## Architecture

```
Client
  │
  ▼
Axum HTTP server
  ├── POST /v1/chat/completions   ← OpenAI-compat proxy (rate limited, auth required)
  ├── POST /v1/messages           ← Native Anthropic passthrough (auth optional)
  ├── GET  /api/*                 ← Management API (auth required)
  ├── GET  /metrics               ← Prometheus
  └── GET  /                      ← Embedded React dashboard
  │
  ├── Anthropic API (upstream)
  └── SQLite database
        ├── config          (key/value settings)
        ├── admin_users     (Argon2-hashed credentials)
        └── request_logs    (one row per proxied request)
```

Usage records flow from request handlers through an async mpsc channel to a background writer task that batch-inserts to SQLite on a 500ms tick. The server drains this channel gracefully on shutdown.

## Supported Models

| Display Name | Model ID |
|---|---|
| Claude Opus 4.7 | `claude-opus-4-7` |
| Claude Sonnet 4.6 | `claude-sonnet-4-6` |
| Claude Haiku 4.5 | `claude-haiku-4-5-20251001` |

Aliases: `claude-opus-4`, `claude-sonnet-4`, `claude-haiku-4`, `claude-3-opus`, `claude-3-sonnet`, `claude-3-haiku`, `claude-3-5-sonnet`, `claude-3-5-haiku`
