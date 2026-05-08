# Build Instructions

## Prerequisites

| Requirement | Version | Notes |
|-------------|---------|-------|
| Rust toolchain | 1.75+ | `rustup show` to verify |
| cargo | (bundled with Rust) | |
| Node.js | 20+ | `node --version` |
| npm | 10+ | `npm --version` |
| SQLite (dev libs) | 3.35+ | `libsqlite3-dev` on Debian/Ubuntu |

## Environment Variables

Copy `.env.example` to `.env` and fill in values before running:

```bash
cp .env.example .env
```

Required variables:

| Variable | Example | Purpose |
|----------|---------|---------|
| `ANTHROPIC_API_KEY` | `sk-ant-...` | Forwarded to Anthropic API |
| `ADMIN_USERNAME` | `admin` | Management UI login |
| `ADMIN_PASSWORD` | `changeme` | Management UI login |
| `DATABASE_URL` | `sqlite://llmonitor.db` | SQLite file path |

## Build Steps

### 1. Install Frontend Dependencies

```bash
cd frontend && npm install && cd ..
```

### 2. Build Frontend

Produces `frontend/dist/` — embedded by rust-embed at compile time.

```bash
cd frontend && npm run build && cd ..
```

Expected output: Vite build summary showing `frontend/dist/index.html` and hashed JS/CSS assets.

### 3. Build Rust Binary

```bash
cargo build --release
```

This embeds the `frontend/dist/` contents via `rust-embed`. The frontend build (Step 2) **must** complete before this step.

### 4. Verify Build Success

```
target/release/llmonitor
```

Run the binary to confirm it starts:

```bash
DATABASE_URL=sqlite://test.db \
ADMIN_USERNAME=admin \
ADMIN_PASSWORD=secret \
ANTHROPIC_API_KEY=sk-ant-test \
./target/release/llmonitor
```

Expected: `Listening on 0.0.0.0:3000`

### 5. Docker Build (Optional)

```bash
# Build frontend first, then Docker image
cd frontend && npm run build && cd ..
docker build -t llmonitor:latest .
```

Or via docker-compose (builds and runs):

```bash
docker-compose up --build
```

## Build Artifacts

| Artifact | Location | Description |
|----------|----------|-------------|
| Rust binary | `target/release/llmonitor` | Single self-contained binary |
| Frontend bundle | `frontend/dist/` | Embedded into the binary at compile time |
| Docker image | `llmonitor:latest` | (if Docker build run) |

## Troubleshooting

### `frontend/dist` not found during cargo build
Run `cd frontend && npm run build` before `cargo build`.

### TypeScript errors during frontend build
Run `cd frontend && npm run build` — tsc errors will print to stdout. All strict-mode errors must be resolved.

### `libsqlite3-dev` missing
On Debian/Ubuntu: `sudo apt install libsqlite3-dev`  
On macOS: SQLite is bundled with Xcode CLT.

### sqlx offline mode error
If `DATABASE_URL` is not set: `export DATABASE_URL=sqlite://llmonitor.db` before building.
