# Performance Test Instructions

## Scope

llmonitor is a local admin tool and single-tenant proxy. Formal load testing is N/A for the management UI. The proxy path has one meaningful performance characteristic: it must not materially add latency to upstream requests.

## Proxy Overhead Check

**Goal**: Confirm the proxy adds < 5ms overhead vs. direct upstream calls.

**Tool**: `wrk` or `curl` with timing.

```bash
# Direct upstream baseline (replace with actual upstream URL)
curl -o /dev/null -s -w "Total: %{time_total}s\n" \
  https://api.anthropic.com/v1/messages \
  -H "x-api-key: $ANTHROPIC_API_KEY" \
  -H "anthropic-version: 2023-06-01" \
  -H "content-type: application/json" \
  -d '{"model":"claude-haiku-4-5","max_tokens":10,"messages":[{"role":"user","content":"hi"}]}'

# Via llmonitor proxy
curl -o /dev/null -s -w "Total: %{time_total}s\n" \
  http://localhost:3000/v1/messages \
  -H "x-api-key: $ANTHROPIC_API_KEY" \
  -H "anthropic-version: 2023-06-01" \
  -H "content-type: application/json" \
  -d '{"model":"claude-haiku-4-5","max_tokens":10,"messages":[{"role":"user","content":"hi"}]}'
```

**Pass condition**: Proxy total time ≤ direct + 5ms (network variance excluded).

## Tracking Channel Back-pressure Check

**Goal**: Confirm `try_send()` never blocks the proxy response path even under burst traffic.

```bash
# Send 100 concurrent proxy requests using wrk (2 threads, 10 connections, 5s)
wrk -t2 -c10 -d5s \
  -H "x-api-key: $ANTHROPIC_API_KEY" \
  -H "anthropic-version: 2023-06-01" \
  -H "content-type: application/json" \
  -s post.lua \
  http://localhost:3000/v1/messages
```

Where `post.lua`:
```lua
wrk.method = "POST"
wrk.body = '{"model":"claude-haiku-4-5","max_tokens":1,"messages":[{"role":"user","content":"hi"}]}'
wrk.headers["content-type"] = "application/json"
```

**Pass condition**: No 5xx errors; p99 latency ≤ p99 of direct upstream + 10ms.

## Management API Response Time

The management API (stats queries against SQLite) should respond quickly even with a large dataset.

```bash
# Seed 100k rows (optional stress test)
sqlite3 llmonitor.db < scripts/seed_test_data.sql

# Time a stats query
time curl -s -u admin:testpass \
  "http://localhost:3000/api/stats/summary?window=30d"
```

**Pass condition**: Response under 500ms with 100k rows.

## Notes

- llmonitor is designed for single-operator use — there are no multi-tenancy or high-concurrency requirements
- The Anthropic API itself is the dominant latency source; proxy overhead is negligible by design
- SQLite is sufficient for the expected write rate (one row per proxied request)
