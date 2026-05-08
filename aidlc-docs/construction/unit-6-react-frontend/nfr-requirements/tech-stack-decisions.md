# Tech Stack Decisions — Unit 6: React Frontend

## Frontend Tech Stack

| Concern | Decision | Rationale |
|---------|----------|-----------|
| Framework | React 18 + TypeScript | Standard SPA framework; TypeScript strict mode for correctness |
| Build tool | Vite 5 | Fast HMR, native ESM, simple config; `npm create vite` scaffold |
| Charts | Recharts | Pure-React SVG charts; TypeScript types included; no extra config |
| Styling | Tailwind CSS Play CDN | No PostCSS build step; loaded via `<script>` in `index.html`; acceptable for an embedded admin tool |
| HTTP | Native `fetch` | No axios or other HTTP library; browser fetch is sufficient for the small API surface |
| State management | React `useState` + `useEffect` | No external store needed; component-local state is sufficient for this app size |
| Error handling | Custom `ErrorBoundary` class component | React class component (required for `componentDidCatch`); single instance at App level |
| Build target | `es2020` | Modern browser only; no polyfills; cleaner output |

## Build Integration

| Concern | Decision |
|---------|----------|
| `outDir` | `../frontend/dist` relative to `frontend/` source (= `frontend/dist/` at workspace root) |
| `base` | `/` — all asset paths are absolute from server root |
| Embedding | `rust-embed` `#[folder = "frontend/dist/"]` in `src/proxy/frontend.rs` — no changes needed to Rust side |
| Build command | `cd frontend && npm run build` before `cargo build` in production |

## Infrastructure

| Concern | Decision |
|---------|----------|
| docker-compose | Version `"3.8"` syntax; single service `llmonitor`; named volume for SQLite |
| `.env.example` | Documents 4 env vars with examples and descriptions |
| Dockerfile | Already exists from Unit 1 — no changes needed |
