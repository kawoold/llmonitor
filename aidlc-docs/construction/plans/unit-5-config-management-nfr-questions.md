# NFR Questions — Unit 5: Config Management

Fill in each `[Answer]:` tag and reply "answered" when done.

---

**Q1 — Config hot-reload: cache the API key in AppState?**

Every proxy request resolves the Provider via the Anthropic API key. If `ConfigService::get_anthropic_api_key()` queries SQLite on every request, it adds a DB round-trip to every proxied call.

A. Cache in AppState — store the current `Config` snapshot in an `Arc<RwLock<ConfigSnapshot>>` inside AppState; `PUT /api/config` updates both the DB and the in-memory cache (sub-millisecond reads on hot path)
B. Query DB each time — no cache; accept ~1ms SQLite overhead per proxy request (simpler, always consistent)

[Answer]: B

---

**Q2 — Rate limiting on `/api/*` routes?**

The proxy endpoint (`/v1/*`) has the global token-bucket rate limiter from Unit 1. Should the management API (`/api/*`) also be rate limited?

A. Yes — apply the same global rate limiter to `/api/*` as well
B. No — `/api/*` is low-traffic (admin use only); rate limiting is overkill; rely on auth + brute-force delay
C. Separate limit — apply a stricter per-endpoint limit (e.g., 10 req/min) on `/api/*` specifically

[Answer]: B
