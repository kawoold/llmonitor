# Functional Design Plan — Unit 1: Proxy Core

## Plan Checkboxes

- [x] Answer design questions below
- [x] Generate `business-logic-model.md` — request pipeline, routing logic, error translation
- [x] Generate `domain-entities.md` — OpenAI schema types, error types, shared types
- [x] Generate `business-rules.md` — validation rules, routing rules, rate limit policy

---

## Context

Unit 1 establishes the Axum HTTP server, OpenAI-compatible schema types, provider routing via `X-Provider` header, and the full middleware stack. The functional design focuses on the business logic within the request pipeline — not infrastructure or NFR patterns (those come next).

---

## Functional Design Questions

### Question 1: OpenAI Model Name Handling

When a client sends a request with a model name that doesn't match any Claude model (e.g. `model: "gpt-4o"`), how should the proxy behave?

A) Pass-through — forward the model name as-is to the upstream provider; let the provider reject it if invalid
B) Reject unknown models — return a 400 error listing accepted model names for the requested provider
C) Ignore — always replace the model name with the provider's configured default model
D) Other (please describe after [Answer]: tag below)

[Answer]: B

---

### Question 2: Rate Limit Scope

The proxy endpoint (`/v1/*`) must have rate limiting (NFR-SEC-02, SECURITY-11). What should the rate limit be scoped to?

A) Per IP address — one limit bucket per client IP
B) Global — a single shared limit across all clients (protects the proxy from any burst)
C) Both — a per-IP limit AND a global burst limit
D) Other (please describe after [Answer]: tag below)

[Answer]: B

---

### Question 3: Health Check Endpoint

What should the `GET /health` response return?

A) Simple `200 OK` with plain text `"ok"` — minimal, no information leaked
B) JSON status object — `{"status": "ok", "db": "connected", "uptime_secs": N}` — useful for monitoring
C) Other (please describe after [Answer]: tag below)

[Answer]: B

---

### Question 4: Missing X-Provider Header Behaviour

When a client sends `POST /v1/chat/completions` without the `X-Provider` header, what error should be returned?

A) `400 Bad Request` with an OpenAI-format error body: `{"error": {"type": "invalid_request_error", "message": "Missing required header: X-Provider"}}`
B) `400 Bad Request` with the error body also listing the supported providers
C) Other (please describe after [Answer]: tag below)

[Answer]: B

---

Please fill in all `[Answer]:` tags and let me know when done.
