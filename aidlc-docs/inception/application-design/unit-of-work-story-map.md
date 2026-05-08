# Unit of Work — Requirements Map — llmonitor LLM Proxy

> Note: User Stories stage was skipped (clear technical requirements, single developer). This document maps Functional Requirements (FR-xx) and Non-Functional Requirements (NFR-xx) to units of work.

---

## Functional Requirements Coverage

| Requirement | Description | Primary Unit | Supporting Unit |
|---|---|---|---|
| FR-01 | OpenAI-compatible proxy API (`POST /v1/chat/completions`) | Unit 1 | Unit 2 |
| FR-02 | Provider routing via `X-Provider` header | Unit 1 | — |
| FR-03 | Claude provider integration + prompt cache tracking | Unit 2 | Unit 3 |
| FR-04 | Streaming SSE support | Unit 2 | Unit 1 |
| FR-05 | Token usage tracking (all fields) | Unit 3 | Unit 2 |
| FR-06 | Session/conversation thread ID derivation | Unit 3 | — |
| FR-07 | Statistics REST API (summary, usage, sessions, cache) | Unit 4 | Unit 3 |
| FR-08 | Prometheus metrics endpoint | Unit 4 | Unit 1 |
| FR-09 | React frontend (dashboard + config UI) | Unit 6 | Unit 4, Unit 5 |
| FR-10 | Configuration management API (CRUD, Basic Auth) | Unit 5 | Unit 1 |
| FR-11 | Provider extensibility (Provider enum pattern) | Unit 1 | Unit 2 |

---

## Non-Functional Requirements Coverage

| Requirement | Description | Primary Unit |
|---|---|---|
| NFR-PERF-01 | <10ms proxy overhead; async DB writes | Unit 3 (async write path) |
| NFR-PERF-02 | 100 concurrent connections; Tokio async | Unit 1 |
| NFR-STORE-01 | SQLite storage; WAL mode; auto migrations | Unit 1, Unit 5, Unit 3 |
| NFR-DEPLOY-01 | Docker multi-stage build; volume mount | Unit 1 (Dockerfile), Unit 6 (docker-compose) |
| NFR-OBS-01 | Structured JSON logging; request ID; no secrets | Unit 1 |
| NFR-SEC-01 | Basic Auth on management API; Argon2id | Unit 5 |
| NFR-SEC-02 | Rate limiting on proxy endpoint | Unit 1 |
| NFR-SEC-03 | TLS support / HSTS | Unit 1 |
| NFR-SEC-04 | HTTP security headers | Unit 1 |
| NFR-SEC-05 | Input validation; parameterized queries | Unit 1, Unit 2, Unit 3, Unit 5 |
| NFR-SEC-06 | No secrets in logs; generic error responses | Unit 1, Unit 5 |
| NFR-SEC-07 | Cargo.lock; dependency vulnerability scan | Unit 1 (Cargo.lock), Unit 6 (package-lock.json) |

---

## Security Rules Coverage by Unit

| SECURITY Rule | Unit 1 | Unit 5 | Unit 2 | Unit 3 | Unit 4 | Unit 6 |
|---|---|---|---|---|---|---|
| SECURITY-01 (Encryption at rest/transit) | DB WAL + TLS config | — | HTTPS to Anthropic | — | — | HTTPS in nginx/proxy |
| SECURITY-02 (Access logging) | Request ID + access log middleware | — | — | — | — | — |
| SECURITY-03 (Structured logging) | tracing + JSON format | Credential ops logged | Provider errors | Write errors | Query errors | N/A (frontend) |
| SECURITY-04 (HTTP security headers) | Middleware layer | — | — | — | — | CSP for React app |
| SECURITY-05 (Input validation) | Body size limits; serde validation | Config update validation | Request body validation | UsageRecord validation | Query param validation | API client validation |
| SECURITY-06 (Least privilege) | N/A (no IAM) | — | — | — | — | — |
| SECURITY-07 (Network config) | Docker port exposure | — | — | — | — | docker-compose network |
| SECURITY-08 (App-level access control) | Route separation | Basic Auth middleware | — | — | Auth on stats routes | — |
| SECURITY-09 (Hardening) | No default creds; generic errors | No default admin | — | — | — | No inline scripts |
| SECURITY-10 (Supply chain) | Cargo.lock | — | reqwest pinned | — | — | package-lock.json |
| SECURITY-11 (Secure design) | Rate limiting; separation of concerns | Auth logic isolated | — | — | — | — |
| SECURITY-12 (Auth/credentials) | — | Argon2id; brute-force protection | — | — | — | Secure cookie attrs |
| SECURITY-13 (Integrity) | Deserialization validation | — | Anthropic response validation | — | — | SRI if external CDN |
| SECURITY-14 (Alerting/monitoring) | — | Auth failure logging | — | Write failure logging | — | — |
| SECURITY-15 (Exception handling) | Global error handler | Fail-closed on auth error | Provider error handling | Write error handling | Query error handling | — |

---

## Feature Completeness by Unit

| Feature Area | Unit 1 | Unit 5 | Unit 2 | Unit 3 | Unit 4 | Unit 6 |
|---|---|---|---|---|---|---|
| HTTP server running | DONE | — | — | — | — | — |
| OpenAI API schema | DONE | — | — | — | — | — |
| Provider routing | DONE | — | — | — | — | — |
| Admin auth | — | DONE | — | — | — | — |
| Config management API | — | DONE | — | — | — | — |
| Claude API integration | — | — | DONE | — | — | — |
| Streaming SSE | — | — | DONE | — | — | — |
| Token persistence | — | — | — | DONE | — | — |
| Session ID derivation | — | — | — | DONE | — | — |
| Statistics API | — | — | — | — | DONE | — |
| Prometheus metrics | — | — | — | — | DONE | — |
| React dashboard | — | — | — | — | — | DONE |
| docker-compose | — | — | — | — | — | DONE |
| **End-to-end proxy** | Scaffold | Config | Working | + Tracking | + Analytics | + UI |
