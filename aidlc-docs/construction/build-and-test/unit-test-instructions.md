# Unit Test Execution

## Rust Tests

All Rust tests (unit + integration) are run with a single command. Tests use an in-memory SQLite database — no external services required.

### Run All Tests

```bash
cargo test
```

### Run a Specific Test File

```bash
cargo test --test auth_test
cargo test --test config_api_test
cargo test --test health_test
cargo test --test proxy_validation_test
cargo test --test stats_test
cargo test --test tracking_test
```

### Run Tests with Output

```bash
cargo test -- --nocapture
```

### Expected Results

| Test File | Coverage Area | Approx Count |
|-----------|--------------|-------------|
| `tests/auth_test.rs` | 401 enforcement, Basic auth decoding | ~4 |
| `tests/config_api_test.rs` | GET/PUT /api/config, POST /api/credentials | ~8 |
| `tests/health_test.rs` | GET /health response | ~2 |
| `tests/proxy_validation_test.rs` | Request validation, provider routing | ~6 |
| `tests/stats_test.rs` | Stats endpoints, window param, empty DB | ~6 |
| `tests/tracking_test.rs` | UsageRecord write, channel flush | ~4 |

All tests must pass (0 failures) before the build is considered clean.

### Test Flags

```bash
# Run only tests matching a pattern
cargo test stats

# Run tests single-threaded (avoids SQLite file contention if using file DBs)
cargo test -- --test-threads=1
```

## Frontend TypeScript Check

The frontend has no separate test runner. TypeScript strict-mode compilation acts as the primary correctness check.

```bash
cd frontend && npm run build
```

A clean build with no TypeScript errors is the passing criterion. Vite will print type errors to stdout and exit non-zero if any exist.

## What Each Test Covers

### auth_test.rs
- Requests without `Authorization` header return 401
- Requests with wrong credentials return 401
- Requests with correct credentials pass through

### config_api_test.rs
- `GET /api/config` returns masked API key
- `PUT /api/config` updates API key, persists across requests
- `POST /api/credentials` updates username/password, old credentials rejected after

### health_test.rs
- `GET /health` returns 200 with `{"status":"ok"}`

### proxy_validation_test.rs
- Missing `model` field returns 400
- Unknown provider model returns 400
- Valid Claude request is forwarded (mock upstream)

### stats_test.rs
- All stats endpoints require auth (401 without creds)
- Empty DB returns zero totals (not 500)
- Invalid `?window=` value returns 400
- `?window=` defaults to 24h when omitted
- `/api/stats/sessions` returns array
- `/api/stats/cache` returns `cache_hit_rate: 0.0` on empty DB

### tracking_test.rs
- `UsageRecord` is written to SQLite within flush interval
- Channel `try_send` does not block the proxy path
- `cache_read_tokens` and `cache_creation_tokens` are persisted correctly
