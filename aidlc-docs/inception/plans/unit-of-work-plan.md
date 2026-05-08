# Unit of Work Plan — llmonitor LLM Proxy

## Plan Checkboxes

- [x] Answer planning questions below
- [x] Generate `unit-of-work.md` — unit definitions, scope, components per unit
- [x] Generate `unit-of-work-dependency.md` — dependency matrix and build sequencing
- [x] Generate `unit-of-work-story-map.md` — functional requirements mapped to units
- [x] Validate unit boundaries and coverage

---

## Proposed Unit Breakdown (from Execution Plan & Application Design)

The application design identified 13 components across 6 logical domains. The proposed units are:

| # | Unit Name | Components | Description |
|---|---|---|---|
| 1 | Proxy Core | C-01, C-02, C-03, C-12 | Axum server, OpenAI schema types, request pipeline, routing, middleware stack, frontend asset serving, Docker/deployment |
| 2 | Claude Provider Adapter | C-04 | Anthropic API client, OpenAI↔Anthropic translation, prompt cache metadata, streaming |
| 3 | Token Tracking & Session Engine | C-05, C-06, C-07 | SQLite schema/migrations, session ID derivation, mpsc write channel, background writer |
| 4 | Statistics & Prometheus Metrics | C-08, C-09 | Rolling window aggregation queries, statistics REST API, Prometheus metrics endpoint |
| 5 | Configuration Management | C-10, C-11, C-13 | DB-backed config CRUD, Basic Auth middleware, Argon2id credentials, admin bootstrap |
| 6 | React Frontend | (separate build) | TypeScript + React SPA, Vite build, analytics dashboard, cache analytics, config UI |

---

## Planning Questions

### Question 1: Unit Build Sequence

The proposed units have dependencies (e.g. Unit 3 must exist before Unit 4 can query data). What is your preferred sequence for building the units?

A) Foundation-first: 1 (Proxy Core) → 5 (Config) → 2 (Claude Adapter) → 3 (Tracking) → 4 (Stats) → 6 (Frontend)
B) Pipeline-first: 1 (Proxy Core) → 2 (Claude Adapter) → 3 (Tracking) → 5 (Config) → 4 (Stats) → 6 (Frontend)
C) Let me define a custom order (describe after [Answer]: tag)

[Answer]: A

---

### Question 2: Frontend Unit Timing

The React frontend (Unit 6) depends on the management API endpoints (Unit 5) and statistics API (Unit 4) being stable. Should it be built last, or should frontend scaffolding and API contract definition happen earlier?

A) Build last — complete all Rust units first, then build the full React frontend against the finished API
B) Scaffold early — create the frontend project structure and API client types as part of Unit 1, then complete the UI in the final pass
C) Other (please describe after [Answer]: tag below)

[Answer]: A

---

### Question 3: Infrastructure Design Scope

Infrastructure Design is planned for Unit 1 only (Dockerfile, SQLite volume, ports, env vars). Should subsequent units contribute to infrastructure (e.g. docker-compose for local dev, environment variable documentation)?

A) Unit 1 only — define the full Docker setup in Unit 1; subsequent units just reference it
B) Incremental — Unit 1 defines the Dockerfile; a docker-compose.yml and full env var docs are added as part of the final unit (Unit 6) once all config is known
C) Other (please describe after [Answer]: tag below)

[Answer]: B

---

Please fill in all `[Answer]:` tags and let me know when done.
