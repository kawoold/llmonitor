# Integration Test Instructions

## Purpose

Verify that all units work together as a running system: the Rust binary serves the embedded frontend, proxies requests to Anthropic, writes tracking data, and exposes correct stats through the management API.

## Integration Test Scenarios

### Scenario 1: Proxy → Tracking → Stats Pipeline

**Description**: A proxied request increments token counts visible in stats endpoints.

**Setup**:
```bash
# Start llmonitor pointing at a mock upstream (e.g., httpbin or a local netcat listener)
DATABASE_URL=sqlite://integration_test.db \
ADMIN_USERNAME=admin \
ADMIN_PASSWORD=testpass \
ANTHROPIC_API_KEY=sk-ant-fake \
./target/release/llmonitor &
SERVER_PID=$!
```

**Test Steps**:
1. Send a POST to `/v1/messages` with a valid Claude request body
2. Wait 1 second (flush interval)
3. GET `/api/stats/summary?window=1h` with Basic auth
4. Assert `total_requests >= 1` and `total_tokens > 0`

**Cleanup**:
```bash
kill $SERVER_PID
rm -f integration_test.db
```

---

### Scenario 2: Config Update → Proxy Uses New Key

**Description**: Updating the API key via management API causes subsequent proxied requests to use the new key.

**Test Steps**:
1. PUT `/api/config` with `{"api_key": "sk-ant-new-key"}`
2. GET `/api/config` — assert `api_key_masked` reflects new key
3. Send a proxied request — verify upstream `Authorization` header contains new key (capture via proxy/mock)

---

### Scenario 3: Credentials Update → Old Creds Rejected

**Description**: After updating admin credentials, old credentials return 401.

**Test Steps**:
1. POST `/api/credentials` with `{"username": "newadmin", "password": "newpass"}`
2. GET `/api/config` with old `admin:testpass` — assert 401
3. GET `/api/config` with `newadmin:newpass` — assert 200

---

### Scenario 4: Frontend Served from Binary

**Description**: The embedded frontend is served at `/`.

**Test Steps**:
```bash
curl -s http://localhost:3000/ | grep -q "llmonitor"
# Expect: exit code 0 (string found in HTML)

curl -s -o /dev/null -w "%{http_code}" http://localhost:3000/assets/index-*.js
# Expect: 200
```

---

### Scenario 5: Cache Stats Accumulation

**Description**: Requests with cache token fields are reflected in `/api/stats/cache`.

**Setup**: Inject a row directly into SQLite with non-zero cache tokens:
```bash
sqlite3 integration_test.db \
  "INSERT INTO request_logs (provider, model, prompt_tokens, completion_tokens, total_tokens, cache_read_tokens, cache_creation_tokens, created_at) \
   VALUES ('anthropic', 'claude-sonnet-4-6', 100, 50, 150, 200, 0, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'));"
```

**Assert**: GET `/api/stats/cache?window=1h` returns `cache_read_tokens: 200` and `cache_hit_rate > 0`.

---

## Running Integration Tests

The integration test suite is currently manual (no automated runner). Execute each scenario in order after confirming `cargo test` passes:

```bash
# 1. Build everything
cd frontend && npm run build && cd ..
cargo build --release

# 2. Run unit tests first
cargo test

# 3. Start server for integration testing
export DATABASE_URL=sqlite://integration_test.db
export ADMIN_USERNAME=admin
export ADMIN_PASSWORD=testpass
export ANTHROPIC_API_KEY=sk-ant-fake
./target/release/llmonitor &

# 4. Run scenarios (using curl)
# ... (see scenarios above)

# 5. Cleanup
kill %1
rm -f integration_test.db
```

## Expected Outcomes

| Scenario | Pass Condition |
|----------|----------------|
| Proxy → Tracking → Stats | `total_requests >= 1` after flush |
| Config update | Masked key updated; proxy uses new key |
| Credentials update | Old creds rejected (401); new creds accepted (200) |
| Frontend served | `/` returns HTML containing "llmonitor" |
| Cache stats | `cache_read_tokens` matches injected value |
