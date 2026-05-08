# Stage 1: Build frontend
FROM node:22.14-bookworm-slim AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package*.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

# Stage 2: Build Rust binary
FROM rust:1.87-bookworm AS rust-builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
COPY migrations/ ./migrations/
COPY --from=frontend-builder /app/frontend/dist/ ./frontend/dist/
RUN cargo build --release

# Stage 3: Minimal runtime image
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd --gid 1001 llmonitor \
    && useradd --uid 1001 --gid llmonitor --shell /bin/sh --no-create-home llmonitor

WORKDIR /app
COPY --from=rust-builder /app/target/release/llmonitor /app/llmonitor

RUN mkdir -p /data && chown llmonitor:llmonitor /data

USER llmonitor

VOLUME ["/data"]

EXPOSE 8080

ENV DATABASE_URL=/data/llmonitor.db
ENV LISTEN_ADDR=0.0.0.0:8080
ENV RUST_LOG=llmonitor=info,tower_http=info

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget -qO- http://localhost:8080/health || exit 1

ENTRYPOINT ["/app/llmonitor"]
