# NFR Requirements — Unit 5: Config Management

## Performance

| ID | Requirement | Value | Rationale |
|---|---|---|---|
| NFR-U5-PERF-01 | Config API response time | < 50ms p99 (excluding auth delay) | Admin-only; not on hot path |
| NFR-U5-PERF-02 | API key DB read per proxy request | ~1ms (SQLite WAL, no cache) | Q1=B; accepted overhead; WAL read doesn't block writes |
| NFR-U5-PERF-03 | Argon2id hash time | < 1s on target hardware | Verify only on admin requests; not on proxy hot path |

## Security

| ID | Requirement | Value | Rationale |
|---|---|---|---|
| NFR-U5-SEC-01 | Password hashing algorithm | Argon2id | SECURITY-12, BR-U5-02 |
| NFR-U5-SEC-02 | Argon2id parameters | memory=65536 KiB, iterations=3, parallelism=4 | Sufficient cost for single-user server; adjustable in future |
| NFR-U5-SEC-03 | Auth failure response delay | Fixed 500ms (BR-U5-04) | Brute-force mitigation; Q3=A |
| NFR-U5-SEC-04 | Auth failure log fields | username (not password), timestamp, request ID | SECURITY-14; no credential leakage |
| NFR-U5-SEC-05 | Generic auth error messages | Username found/not-found indistinguishable | SECURITY-09, BR-U5-05 |
| NFR-U5-SEC-06 | API key never in logs or responses | Masked in GET response; ApiKey type used throughout | SECURITY-06, SECURITY-03, BR-U5-08 |
| NFR-U5-SEC-07 | All SQL parameterized | No string interpolation in queries | SECURITY-05, NFR-SEC-05 |
| NFR-U5-SEC-08 | Config API rate limiting | None (Q2=B) | Admin-only endpoint; auth + delay sufficient |
| NFR-U5-SEC-09 | Password minimum length | 8 characters | BR-U5-10 |
| NFR-U5-SEC-10 | Config updates atomic | SQLite transaction | BR-U5-13; prevents partial state |

## Reliability

| ID | Requirement | Value | Rationale |
|---|---|---|---|
| NFR-U5-REL-01 | Startup fail-fast | Exit non-zero if no admin exists and env vars absent | BR-U5-01; clear operator signal |
| NFR-U5-REL-02 | Config read fallback | Return documented defaults for absent keys | BR-U5-12; prevents hard failures on fresh DB |
| NFR-U5-REL-03 | Auth errors fail-closed | Any unexpected error during auth → 401 (never 500) | SECURITY-15; avoids accidentally open endpoint |

## Observability

| ID | Requirement | Value | Rationale |
|---|---|---|---|
| NFR-U5-OBS-01 | Auth failure logging | `tracing::warn!` with username and request ID | SECURITY-14 |
| NFR-U5-OBS-02 | Bootstrap logging | `tracing::info!` on admin creation, `tracing::error!` on abort | Operator visibility |
| NFR-U5-OBS-03 | Config update logging | `tracing::info!` with changed keys (never values) | Audit trail without credential leakage |

## Maintainability

| ID | Requirement | Value | Rationale |
|---|---|---|---|
| NFR-U5-MAINT-01 | Unit tests | AdminService::verify, masking function, bootstrap logic | MAINT-02 |
| NFR-U5-MAINT-02 | Integration tests | Full Basic Auth round-trip, config GET/PUT, credential update | MAINT-02 |
