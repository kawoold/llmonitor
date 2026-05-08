# Deployment Architecture — Unit 1: Proxy Core

## Dockerfile (Multi-Stage Build)

```
Stage 1: frontend-builder (Node)
  FROM node:22-alpine AS frontend-builder
  WORKDIR /app/frontend
  COPY frontend/package.json frontend/package-lock.json ./
  RUN npm ci --ignore-scripts
  COPY frontend/ ./
  RUN npm run build
  -- output: /app/frontend/dist/

Stage 2: rust-builder
  FROM rust:1.78-slim-bookworm AS rust-builder
  WORKDIR /app
  -- Install build dependencies (pkg-config, libssl-dev for reqwest)
  RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
  -- Cache dependency layer (copy manifests first)
  COPY Cargo.toml Cargo.lock ./
  RUN mkdir src && echo 'fn main(){}' > src/main.rs && cargo build --release && rm -rf src
  -- Copy real source + frontend dist (required by rust-embed at compile time)
  COPY src/ ./src/
  COPY migrations/ ./migrations/
  COPY --from=frontend-builder /app/frontend/dist/ ./frontend/dist/
  RUN cargo build --release
  -- output: /app/target/release/llmonitor

Stage 3: runtime image
  FROM debian:bookworm-slim AS runtime
  -- Install only runtime dependencies (CA certs for HTTPS to Anthropic)
  RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
  -- Create non-root user
  RUN useradd -r -u 1001 -g root llmonitor
  WORKDIR /app
  COPY --from=rust-builder /app/target/release/llmonitor ./llmonitor
  -- Volume mount point for SQLite
  RUN mkdir /data && chown llmonitor /data
  USER llmonitor
  EXPOSE 8080
  ENV LLMONITOR_PORT=8080
  ENV LLMONITOR_DB_PATH=/data/llmonitor.db
  ENTRYPOINT ["/app/llmonitor"]
```

Key decisions:
- Pinned base image tags (no `latest`) — SECURITY-10 compliant
- Non-root user (`llmonitor`, UID 1001) — SECURITY-06 compliant
- Frontend built in its own stage; dist/ copied into Rust builder for `rust-embed`
- CA certificates installed for outbound TLS to `api.anthropic.com`
- SQLite data directory `/data` is a volume mount point

---

## Docker Volume

```
Volume name: llmonitor-data
Mount path:  /data   (inside container)
Usage:       SQLite database file at /data/llmonitor.db
Backup:      Copy /data/llmonitor.db while server is stopped, or use SQLite online backup API
```

---

## Deployment Topology (Single-Host)

```
Host machine
├── reverse-proxy container (nginx / Caddy)
│     Listens: 443 (HTTPS, public)
│     Forwards: → llmonitor:8080 (internal Docker network)
│
├── llmonitor container
│     Image:   llmonitor:latest (from local build or registry)
│     Port:    8080 (internal only, NOT host-bound)
│     Volume:  llmonitor-data → /data
│     Env:     LLMONITOR_DB_PATH, RUST_LOG
│
└── llmonitor-data volume
      Persists: /data/llmonitor.db
```

docker-compose.yml will be added in Unit 6 with the full multi-service definition.

---

## Build Commands

```bash
# Build Docker image
docker build -t llmonitor:latest .

# Run for first time (with admin bootstrap)
docker run -d \
  --name llmonitor \
  -p 8080:8080 \
  -v llmonitor-data:/data \
  -e LLMONITOR_DB_PATH=/data/llmonitor.db \
  -e LLMONITOR_ADMIN_USER=admin \
  -e LLMONITOR_ADMIN_PASSWORD=changeme123 \
  -e RUST_LOG=info \
  llmonitor:latest

# Subsequent runs (admin already in DB; env vars not needed)
docker run -d \
  --name llmonitor \
  -p 8080:8080 \
  -v llmonitor-data:/data \
  -e LLMONITOR_DB_PATH=/data/llmonitor.db \
  -e RUST_LOG=info \
  llmonitor:latest
```

---

## Environment Variable Reference (Complete)

| Variable | Required | Default | Sensitive | Description |
|---|---|---|---|---|
| `LLMONITOR_DB_PATH` | Yes | — | No | Absolute path to SQLite file |
| `LLMONITOR_PORT` | No | `8080` | No | HTTP listen port |
| `LLMONITOR_ADMIN_USER` | First run | — | Yes | Initial admin username |
| `LLMONITOR_ADMIN_PASSWORD` | First run | — | Yes | Initial admin password (min 8 chars) |
| `RUST_LOG` | No | `info` | No | Log level (e.g. `info`, `llmonitor=debug,sqlx=warn`) |

---

## Health Check (Docker)

```dockerfile
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1
```

Returns:
- `200` with `{"status":"ok"}` — healthy
- `200` with `{"status":"degraded"}` — unhealthy DB but binary running (orchestrator keeps container alive; alert via monitoring)
- Non-200 or timeout — container marked unhealthy
