# Code Summary — Unit 5: Config Management

## Files Created

| File | Purpose |
|---|---|
| `migrations/0001_config.sql` | Creates `config` (key-value) and `admin_users` tables |
| `src/config/domain.rs` | All domain types: ConfigSnapshot, RedactedConfigResponse, UpdateSettingsRequest, UpdateCredentialsRequest, ConfigError, AuthError, AdminError, BootstrapError, KNOWN_CONFIG_KEYS |
| `src/config/service.rs` | ConfigService: get_all, get_anthropic_api_key, get_redacted, update_settings (atomic tx), mask_api_key helper |
| `src/config/admin.rs` | AdminService: verify (Argon2id + 500ms delay), create, update_password, any_exists |
| `src/config/handlers.rs` | Axum handlers: get_config, update_config, update_credentials |
| `src/config/tests.rs` | Unit tests: mask_api_key variants, verify flows, update_settings validation, defaults |
| `tests/auth_test.rs` | Integration: missing auth → 401, wrong creds → 401, correct → 200 |
| `tests/config_api_test.rs` | Integration: GET redacted config, PUT valid key, PUT empty key → 422 |

## Files Modified

| File | Change |
|---|---|
| `src/config/bootstrap.rs` | Replaced stub: checks admin_users, reads env vars, calls AdminService::create, fail-fast |
| `src/config/mod.rs` | Added all new submodule declarations; re-exports ConfigService |
| `src/auth/mod.rs` | Replaced pass-through stub with real Basic Auth: base64 decode → AdminService::verify → 401 or pass-through |
| `src/app_state.rs` | Added config_service: ConfigService, admin_service: AdminService fields |
| `src/providers/mod.rs` | Changed Provider::from_name signature: accepts ApiKey directly instead of &Config; added is_known_name() |
| `src/proxy/routing.rs` | Refactored: extract_provider_name() + validate_provider_name() (API key resolution moved to handler) |
| `src/proxy/handlers.rs` | chat_completion() now calls config_service.get_anthropic_api_key().await before Provider::from_name |
| `src/proxy/tests.rs` | Updated to match new routing API (extract_provider_name, validate_provider_name) |
| `src/lib.rs` | Replaced stub management_routes with real routes + route_layer(from_fn_with_state(basic_auth_middleware)) |
| `src/main.rs` | Updated bootstrap call to pass &db |
| `Cargo.toml` | Added base64 = "0.21", rand_core = { version = "0.6", features = ["getrandom"] } |

## Key Design Decisions

1. **sqlx::query() over sqlx::query!()**: Macro form requires DATABASE_URL at compile time; using string-based API avoids this constraint.
2. **Delay inside verify_inner()**: 500ms sleep applied after any auth failure regardless of cause — prevents timing oracle on username enumeration.
3. **route_layer vs layer**: `route_layer` on the management Router applies auth only to those routes; outer `.layer()` calls would also apply to `/health` and `/metrics`.
4. **rand_core direct dep**: `argon2::password_hash::rand_core::OsRng` re-export not available without direct `rand_core` dependency — added explicitly.
5. **API key from ConfigService per request**: No in-memory cache (Q1=B); ~1ms SQLite WAL read accepted; always reflects current DB state.
