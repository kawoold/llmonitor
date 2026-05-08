# Infrastructure Design — Unit 1: Proxy Core

## Deployment Model

- **Runtime**: Single Docker container
- **Binary**: One Rust binary serving all endpoints (proxy, management API, metrics, frontend)
- **Storage**: SQLite database file on a mounted Docker volume
- **TLS**: Terminated externally by a reverse proxy (nginx / Caddy / Traefik); binary listens on plain HTTP
- **Ports**: Single port 8080 for all traffic (proxy, API, metrics, frontend)

---

## Logical Component → Infrastructure Mapping

| Logical Component | Infrastructure | Notes |
|---|---|---|
| Axum HTTP server | Docker container process | Single process; Tokio multi-thread runtime |
| SQLite database | Docker named volume (`llmonitor-data`) | WAL mode; mounted at `LLMONITOR_DB_PATH` |
| Frontend static assets | Embedded in binary (`rust-embed`) | No separate file system mount required |
| Prometheus metrics (`/metrics`) | Same port 8080 | Scraped by Prometheus container (configured in docker-compose in Unit 6) |
| TLS termination | External reverse proxy | Not in scope for this binary; documented in deployment notes |
| Log output | Container stdout | Captured by Docker log driver; JSON format |

---

## Environment Variables

All runtime configuration uses environment variables for bootstrap only. Ongoing config lives in the DB.

| Variable | Required | Default | Description |
|---|---|---|---|
| `LLMONITOR_DB_PATH` | Yes | — | Absolute path to SQLite file (e.g. `/data/llmonitor.db`) |
| `LLMONITOR_PORT` | No | `8080` | TCP port the HTTP server binds to |
| `LLMONITOR_ADMIN_USER` | First run only | — | Admin username for initial credential bootstrap |
| `LLMONITOR_ADMIN_PASSWORD` | First run only | — | Admin password for initial credential bootstrap (min 8 chars) |
| `RUST_LOG` | No | `info` | Log level filter (e.g. `info`, `llmonitor=debug`) |

**Security note**: `LLMONITOR_ADMIN_USER` and `LLMONITOR_ADMIN_PASSWORD` are read once at first startup, hashed with Argon2id, stored in the DB, and should be removed from the container environment after initial setup (or passed via Docker secrets, not `docker run -e`).

---

## SQLite Configuration

Applied on every connection open via `PRAGMA` statements:

```sql
PRAGMA journal_mode = WAL;       -- enables concurrent reads with single writer
PRAGMA synchronous = NORMAL;     -- fsync on checkpoint only; safe with WAL
PRAGMA foreign_keys = ON;        -- enforce FK constraints
PRAGMA busy_timeout = 5000;      -- 5s wait if DB is locked before error
```

Connection pool settings (sqlx `SqlitePoolOptions`):
- `max_connections`: 10
- `min_connections`: 1
- `acquire_timeout`: 5 seconds

---

## Container Resource Profile

| Resource | Recommendation | Notes |
|---|---|---|
| CPU | 0.5–1 vCPU | Async I/O; proxy overhead is minimal |
| Memory | 128–256 MB | Rust binary ~20MB; SQLite page cache adds overhead |
| Disk (volume) | 1–10 GB | Depends on request volume and retention |
| Network | Standard | Outbound HTTPS to `api.anthropic.com` required |

---

## Security Boundary

```
Internet / Internal network
        │  HTTPS (443)
        ▼
  Reverse Proxy (nginx / Caddy / Traefik)
        │  HTTP (8080) — internal Docker network only
        ▼
  llmonitor container
        │  reads/writes
        ▼
  SQLite volume (llmonitor-data)
```

- The llmonitor container port 8080 MUST NOT be exposed directly to the internet
- Only the reverse proxy should have inbound internet access
- Outbound from llmonitor: `api.anthropic.com:443` (HTTPS, provider API calls)
