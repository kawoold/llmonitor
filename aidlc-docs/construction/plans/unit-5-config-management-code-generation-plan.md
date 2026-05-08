# Code Generation Plan — Unit 5: Config Management

## Unit Context

- **Workspace root**: `/home/kawoold/Development/rust/llmonitor`
- **Requirements covered**: FR-10, NFR-SEC-01, SECURITY-08, SECURITY-12, SECURITY-14
- **Build order**: 2nd (after Unit 1)

**Dependencies on other units**: Unit 1 (AppState, SqlitePool, axum router, auth stub, config stub, bootstrap stub)

**Provides to other units**:
- `ConfigService::get_anthropic_api_key()` — used by Unit 2 proxy handler to resolve the Claude API key from DB
- `AdminService` — real Basic Auth enforcement on all `/api/*` routes
- Migration `0001_config.sql` — creates `config` + `admin_users` tables

**Key changes to existing Unit 1 files**:
- `src/auth/mod.rs` — replace pass-through stub with real Basic Auth middleware
- `src/config/mod.rs` — add `ConfigService` and `AdminService` structs; keep `Config` for env-var startup
- `src/config/bootstrap.rs` — replace stub with real admin bootstrap logic
- `src/app_state.rs` — add `config_service: ConfigService`, `admin_service: AdminService`
- `src/lib.rs` — wire real auth middleware to `/api/*` routes; add config API routes
- `src/providers/mod.rs` — change `Provider::from_name` to accept `ApiKey` directly (not `&Config`)
- `src/proxy/handlers.rs` — call `ConfigService::get_anthropic_api_key()` async before resolving provider

---

## Generation Steps

### Step 1: Database Migration
- [x] Create `migrations/0001_config.sql` — creates `config` (key-value) and `admin_users` tables

### Step 2: Domain Types
- [x] Create `src/config/domain.rs` — `ConfigSnapshot`, `RedactedConfigResponse`, `UpdateSettingsRequest`, `UpdateCredentialsRequest`, `ConfigError`, `AuthError`, `AdminError`, `BootstrapError`

### Step 3: ConfigService Implementation
- [x] Create `src/config/service.rs` — `ConfigService` struct with `get_all()`, `get_redacted()`, `update_settings()`, `get_anthropic_api_key()`, `mask_api_key()` (private)

### Step 4: AdminService Implementation
- [x] Create `src/config/admin.rs` — `AdminService` struct with `verify()` (Argon2id + 500ms delay), `create()` (hash + insert), `update_password()` (hash + update), `any_exists()`

### Step 5: Bootstrap Implementation
- [x] Update `src/config/bootstrap.rs` — replace stub with real logic: check `admin_users`, read env vars, call `AdminService::create()`, fail-fast on missing env vars

### Step 6: Auth Middleware Implementation
- [x] Update `src/auth/mod.rs` — replace pass-through stub with real `basic_auth_middleware(State<AppState>, Request, Next)`: extract header → base64 decode → split → `AdminService::verify()` → 401 or pass-through; add `unauthorized_response()` helper

### Step 7: Config API Handlers
- [x] Create `src/config/handlers.rs` — `get_config()`, `update_config()`, `update_credentials()` axum handlers

### Step 8: Update AppState
- [x] Update `src/app_state.rs` — add `config_service: ConfigService`, `admin_service: AdminService`; update `AppState::new()` constructor

### Step 9: Update `src/config/mod.rs`
- [x] Update to declare all new submodules (`admin`, `domain`, `handlers`, `service`); update `ConfigService` from stub to re-export from `service.rs`

### Step 10: Update Provider & Proxy Handler
- [x] Update `src/providers/mod.rs` — change `Provider::from_name(name, config)` to `Provider::from_name(name, api_key: ApiKey)` — removes dependency on `&Config`
- [x] Update `src/proxy/routing.rs` — remove `resolve_provider()` (logic moves into handler)
- [x] Update `src/proxy/handlers.rs` — `chat_completion()` now fetches API key from `state.config_service.get_anthropic_api_key().await` then calls `Provider::from_name(name, api_key)`

### Step 11: Update Router (`src/lib.rs`)
- [x] Wire real `basic_auth_middleware` to `/api/*` via `route_layer(from_fn_with_state(...))`
- [x] Add routes: `GET /api/config`, `PUT /api/config`, `PUT /api/config/credentials`
- [x] Add `base64 = "0.21"` to `Cargo.toml` if not already a direct dependency

### Step 12: Unit Tests
- [x] Create `src/config/tests.rs` — tests for:
  - `mask_api_key`: empty, short (≤12), normal (>12) inputs
  - `AdminService::verify`: correct password → Ok, wrong password → Err (with ~500ms delay), unknown username → Err
  - `ConfigService::update_settings`: empty API key → ConfigError::EmptyApiKey
  - `ConfigService::get_all`: missing keys return defaults

### Step 13: Integration Tests
- [x] Create `tests/auth_test.rs` — missing Authorization header → 401, wrong credentials → 401, correct credentials → 200
- [x] Create `tests/config_api_test.rs` — GET /api/config (authenticated) → 200 + redacted body; PUT /api/config with valid key → 200; PUT /api/config with empty key → 422

### Step 14: Code Documentation Summary
- [x] Create `aidlc-docs/construction/unit-5-config-management/code/code-summary.md`

---

## Story / Requirement Traceability

| Step | Requirements |
|---|---|
| 1 | NFR-STORE-01 (SQLite migrations) |
| 2 | FR-10 (config management types) |
| 3 | FR-10 (GET/PUT /api/config), BR-U5-06, BR-U5-07, BR-U5-08, BR-U5-12, BR-U5-13 |
| 4 | NFR-SEC-01 (Argon2id), BR-U5-02, BR-U5-04, BR-U5-05, SECURITY-12 |
| 5 | BR-U5-01 (fail-fast startup), SECURITY-09 |
| 6 | NFR-SEC-01 (Basic Auth), BR-U5-03, SECURITY-08 |
| 7 | FR-10 (API endpoints), BR-U5-09, BR-U5-10 |
| 8–9 | NFR-PERF-02 (shared state) |
| 10 | FR-02 (provider routing now reads API key from DB) |
| 11 | FR-10 (routes wired), SECURITY-08 (auth on /api/*) |
| 12 | MAINT-02 (unit tests), PBT partial |
| 13 | MAINT-02 (integration tests) |
