# NFR Requirements Questions — Unit 2: Claude Provider Adapter

Most NFR decisions for Unit 2 are inherited from Unit 1 (timeout=30s, RUST_LOG, INFO level, token bucket rate limiter). The questions below cover the few areas specific to the Anthropic HTTP adapter.

---

**Q1 — Outbound request logging verbosity**

When ClaudeAdapter calls the Anthropic API, what should be logged?

A. **Minimal** — log only `model`, HTTP status code, and token counts (prompt/completion/cache). Never log request/response bodies (safest for prompt privacy)
B. **Standard** — log model + status + tokens at INFO; log full serialized request body at DEBUG level (useful for debugging, but may expose prompt content)
C. **Status only** — log only success/failure and HTTP status code, nothing else

[Answer]: A

---

**Q2 — reqwest connection pool configuration**

`reqwest::Client` pools TCP connections automatically by default (no explicit limit). Should the adapter:

A. **Use defaults** — rely on reqwest's built-in connection pool (adequate for most single-instance proxy workloads)
B. **Configure limits** — set an explicit `max_connections_per_host` limit (e.g., 10) to bound resource usage under high concurrency

[Answer]: A

---

**Q3 — Malformed `anthropic_cache_control` extension field**

If the client sends `anthropic_cache_control` that fails JSON deserialization (e.g., wrong types), how should the adapter respond?

A. **Ignore silently** — treat a malformed field as absent; proceed without cache control (tolerant/lenient)
B. **Return 400** — fail fast with a client error explaining the malformed extension field (strict/explicit)

[Answer]: B
