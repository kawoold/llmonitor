# Business Logic Model — Unit 5: Config Management

## Storage Architecture

Two SQLite tables in migration `0001_config.sql`:

| Table | Purpose |
|---|---|
| `config` | Key-value proxy settings (anthropic_api_key, timeouts, rate limits) |
| `admin_users` | Admin credentials (username → Argon2id password hash) |

---

## Config Table Operations

### Read All Settings (`ConfigService::get_all`)
1. SELECT all rows from `config` table
2. Map each key to a field in `ConfigSnapshot`
3. Missing keys fall back to hard-coded defaults
4. Return `ConfigSnapshot`

### Read Single Value (`ConfigService::get`)
1. SELECT value WHERE key = ?
2. Return `Option<String>`

### Upsert Setting (`ConfigService::set`)
1. Validate key is in the known set (reject unknown keys with `InvalidKey` error)
2. INSERT OR REPLACE INTO config (key, value, updated_at) VALUES (?, ?, NOW())
3. Return `Ok(())`

### Update Settings Atomically (`ConfigService::update_settings`)
1. Validate `UpdateSettingsRequest`:
   - `anthropic_api_key` if present: must be non-empty (reject with `ValidationError::EmptyApiKey`)
2. Begin SQLite transaction
3. For each provided field, call `set(key, value)` within transaction
4. Commit transaction
5. Return `Ok(ConfigSnapshot)` of updated state

### Get Redacted Config (`ConfigService::get_redacted`)
1. Call `get_all()`
2. Apply masking to `anthropic_api_key`:
   - If empty/unset → `""`
   - If set and len > 12 → `first_8_chars + "***...***" + last_4_chars`
   - If set and len ≤ 12 → `"***...***"` (fully hidden if too short to partially reveal)
3. Return `RedactedConfigSnapshot`

---

## Admin User Operations

### Bootstrap Check (`AdminBootstrapService::run`)
1. Query: `SELECT COUNT(*) FROM admin_users`
2. If count > 0 → bootstrap complete, return `Ok(())`
3. If count == 0:
   a. Read `LLMONITOR_ADMIN_USER` env var
   b. Read `LLMONITOR_ADMIN_PASSWORD` env var
   c. If either is missing → return `Err(BootstrapError::MissingEnvVars)` → server aborts
   d. Hash password with Argon2id (memory=64MiB, iterations=3, parallelism=4)
   e. INSERT INTO admin_users (username, password_hash, created_at, updated_at)
   f. Log: "Admin user created from environment variables"
   g. Return `Ok(())`

### Verify Credentials (`AdminService::verify`)
1. SELECT password_hash FROM admin_users WHERE username = ?
2. If row not found → **sleep 500ms** → return `Err(AuthError::InvalidCredentials)`
3. Verify password against hash using Argon2id
4. If mismatch → **sleep 500ms** → return `Err(AuthError::InvalidCredentials)`
5. If match → return `Ok(())`

### Create Admin User (`AdminService::create`)
1. Validate username: non-empty, max 64 chars, alphanumeric + `_-.` only
2. Validate password: non-empty, min 8 chars
3. Hash password with Argon2id
4. INSERT INTO admin_users (username, password_hash, created_at, updated_at)
5. On UNIQUE constraint violation → return `Err(AdminError::UserAlreadyExists)`

### Update Admin Password (`AdminService::update_password`)
1. Validate new password: non-empty, min 8 chars
2. Hash new password with Argon2id
3. UPDATE admin_users SET password_hash = ?, updated_at = NOW() WHERE username = ?
4. If 0 rows affected → return `Err(AdminError::UserNotFound)`

---

## Auth Middleware Pipeline

```
Request arrives at /api/* route
    │
    ▼
Extract Authorization header
    │ Missing?
    ├─► 401 + WWW-Authenticate: Basic realm="llmonitor"
    │
    ▼
Decode base64 → "username:password"
    │ Malformed?
    ├─► 401 + WWW-Authenticate
    │
    ▼
AdminService::verify(username, password)
    │ Err(InvalidCredentials)?
    ├─► [already slept 500ms inside verify()]
    │   401 + WWW-Authenticate + log warning (username, source)
    │
    ▼
Pass request to handler
```

---

## API Endpoint Logic

### GET /api/config
1. Auth middleware validates Basic Auth (above)
2. Call `ConfigService::get_redacted()`
3. Return 200 + `RedactedConfigSnapshot` JSON

### PUT /api/config
Request body: `UpdateSettingsRequest`
1. Auth middleware validates Basic Auth
2. Deserialize body; return 400 if malformed
3. Call `ConfigService::update_settings(request)`
   - Returns 422 if `anthropic_api_key` is present but empty
4. Return 200 + updated `RedactedConfigSnapshot` JSON

### PUT /api/config/credentials
Request body: `UpdateCredentialsRequest { username: String, new_password: String }`
1. Auth middleware validates Basic Auth (authenticates as the calling admin)
2. Deserialize body; return 400 if malformed
3. Call `AdminService::update_password(calling_username, new_password)`
   - Calling username extracted from the verified Basic Auth header
   - Returns 422 if new_password fails validation
4. Return 200 + `{ "message": "Password updated successfully" }`

---

## Startup Sequence (Unit 5 additions)

```
[Unit 1] DB pool created
[Unit 1] Migrations run (now includes 0001_config.sql)
    ▼
AdminBootstrapService::run() ← NEW in Unit 5
    │ MissingEnvVars?
    ├─► Log error: "No admin users found and LLMONITOR_ADMIN_USER/PASSWORD not set"
    │   process::exit(1)
    │
    ▼
AppState created (ConfigService + AdminService populated)
    ▼
Router built (AuthMiddleware applied to /api/* — now real, not stub)
    ▼
Server starts
```
