# Business Rules — Unit 6: React Frontend

## Authentication

**BR-U6-01**: On app load, check `sessionStorage.getItem("llm_auth")`. If absent or empty, render `AuthGate` and block all API calls.

**BR-U6-02**: Auth header for all API requests: `"Basic " + sessionStorage.getItem("llm_auth")`.

**BR-U6-03**: On login form submit, issue `GET /api/config` with the candidate credentials. 200 → store credentials and show main UI. 401 → display "Invalid credentials" inline, do not store.

**BR-U6-04**: Any 401 response from any API call after login MUST clear sessionStorage and re-render `AuthGate`.

**BR-U6-05**: Credentials stored as `btoa("username:password")`. The colon (`:`) is the separator; usernames must not contain colons (enforced at the UI form level: username field forbids colon input).

## Dashboard

**BR-U6-06**: Default time window is `24h`. Window selector shows four options: `1h`, `24h`, `7d`, `30d`.

**BR-U6-07**: Dashboard data (summary + timeseries) fetches on mount and on every window change. No polling timer.

**BR-U6-08**: Loading and error states MUST be shown for each data fetch. Empty states MUST be handled gracefully (zero values displayed, empty charts show "No data" message).

## Cache Panel

**BR-U6-09**: `cache_hit_rate` from the API (0.0–1.0) MUST be displayed as a percentage rounded to 1 decimal place.

**BR-U6-10**: When `total_requests === 0`, display "No cache activity in this window" instead of a chart.

## Config Panel

**BR-U6-11**: The API key field displays the masked value from the server. To update, the user types a new key and submits. An empty submission MUST be blocked client-side (do not send an empty string).

**BR-U6-12**: After a successful PUT, re-fetch and re-display the updated (masked) config to confirm the change was applied.

**BR-U6-13**: The credentials form requires both username and password fields to be non-empty before enabling submit.

## Component Automation

**BR-U6-14**: All interactive elements MUST have `data-testid` attributes using the pattern `{component}-{element-role}`:
- `auth-username-input`, `auth-password-input`, `auth-submit-button`
- `window-selector-1h`, `window-selector-24h`, `window-selector-7d`, `window-selector-30d`
- `apikey-input`, `apikey-submit-button`
- `credentials-username-input`, `credentials-password-input`, `credentials-submit-button`
- `nav-dashboard`, `nav-cache`, `nav-config`

## Build Integration

**BR-U6-15**: `npm run build` MUST produce output in `frontend/dist/`. The Vite `outDir` config MUST be set to `../frontend/dist` relative to the `frontend/` source directory (i.e., `frontend/dist/` at workspace root).

**BR-U6-16**: `cargo build` embeds `frontend/dist/` via `rust-embed`. The `frontend/dist/index.html` placeholder MUST be replaced by the real build output before `cargo build` is run in production.

## Infrastructure

**BR-U6-17**: `docker-compose.yml` MUST mount a named volume for SQLite DB persistence at the path referenced by `LLMONITOR_DB_PATH`.

**BR-U6-18**: `.env.example` MUST document all `LLMONITOR_*` environment variables: `LLMONITOR_DB_PATH`, `LLMONITOR_LISTEN_ADDR`, `LLMONITOR_ADMIN_USER`, `LLMONITOR_ADMIN_PASSWORD`.
