# Functional Design Questions — Unit 5: Config Management

Fill in each `[Answer]:` tag and reply "answered" when done.

---

**Q1 — Config table storage model**

How should the `config` table store provider API keys and proxy settings?

A. Single row with typed columns (one row, columns: `anthropic_api_key`, `request_timeout_secs`, `rate_limit_capacity`, etc.)
B. Key-value store (rows: `key TEXT, value TEXT` — flexible, easy to extend)
C. Two tables: `settings` (key-value for proxy config) + `admin_credentials` (typed columns for username/password hash)

[Answer]: B

---

**Q2 — Admin credential update: require old password?**

When `PUT /api/config/credentials` is called to change the admin password, should the request body require the current password before accepting the new one?

A. Yes — require current password (re-authentication on update)
B. No — Basic Auth on the endpoint is sufficient; no re-auth required in body

[Answer]: B

---

**Q3 — Brute-force protection: delay algorithm**

The auth middleware should delay responses on repeated Basic Auth failures. What mechanism?

A. Fixed delay: 500ms per failed attempt (simple, predictable)
B. Linear delay: 100ms × failure_count (up to cap of 5s)
C. Exponential delay: 100ms, 200ms, 400ms, 800ms… (up to cap of 5s)
D. No delay — just log failures (defer protection to reverse proxy / firewall)

[Answer]: A

---

**Q4 — Single admin vs multiple admin users**

A. Single admin user only (one username/password pair in DB — simpler)
B. Multiple admin users supported (rows keyed by username)

[Answer]: B

---

**Q5 — API key masking in `GET /api/config` response**

When returning config via `GET /api/config`, how should the Anthropic API key be masked?

A. Full redaction: always return `"[REDACTED]"` regardless of whether a key is set
B. Presence indicator: return `"sk-ant-***...***"` (first 8 + last 4 chars visible if set, `""` if unset)
C. Boolean indicator: return `"anthropic_api_key_set": true/false` — no key value at all

[Answer]: B

---

**Q6 — Clearing the Anthropic API key via `PUT /api/config`**

If a PUT request sends an empty string for `anthropic_api_key`:

A. Accept it — clear the key (proxy returns 503 until a new key is set)
B. Reject it — 422 Unprocessable Entity (key cannot be emptied once set)

[Answer]: B
