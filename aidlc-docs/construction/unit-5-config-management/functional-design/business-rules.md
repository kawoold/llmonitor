# Business Rules — Unit 5: Config Management

## BR-U5-01: Server Must Not Start Without At Least One Admin

**Rule**: On startup, if the `admin_users` table has zero rows AND `LLMONITOR_ADMIN_USER`/`LLMONITOR_ADMIN_PASSWORD` env vars are absent, the process MUST exit with a non-zero code and a clear error message.

**Rationale**: The management API would be inaccessible and no credentials could be set without an existing admin.

---

## BR-U5-02: Admin Password Hashed with Argon2id

**Rule**: Admin passwords MUST be hashed using Argon2id with parameters: memory=65536 KiB (64 MiB), iterations=3, parallelism=4. Plaintext passwords MUST NEVER be stored or logged.

**Rationale**: Argon2id is the current best practice for password hashing; NFR-SEC-01, SECURITY-12.

---

## BR-U5-03: All /api/* Routes Require Basic Auth

**Rule**: Every request to any path under `/api/*` MUST pass HTTP Basic Authentication. Requests missing the `Authorization` header or supplying invalid credentials MUST receive `401 Unauthorized` with a `WWW-Authenticate: Basic realm="llmonitor"` header.

**Rationale**: FR-10, NFR-SEC-01, SECURITY-08.

---

## BR-U5-04: Failed Auth Responses Are Delayed by 500ms

**Rule**: Any failed authentication attempt (wrong password OR unknown username) MUST incur a 500ms fixed delay before returning the 401 response.

**Rationale**: Brute-force mitigation; SECURITY-12. The delay is inside `AdminService::verify()` so it applies uniformly regardless of which code path reached the error.

---

## BR-U5-05: Auth Failure Responses Are Generic

**Rule**: The `401` response body and the `WWW-Authenticate` header MUST NOT indicate whether the username was found or only the password was wrong. Both cases return identical error messages.

**Rationale**: Prevents username enumeration; SECURITY-09, SECURITY-15.

---

## BR-U5-06: Anthropic API Key Cannot Be Emptied Once Set

**Rule**: A `PUT /api/config` request that includes `"anthropic_api_key": ""` (empty string) MUST be rejected with `422 Unprocessable Entity`. Omitting the field entirely (absent from JSON) is allowed and means "leave unchanged".

**Rationale**: Prevents accidental key deletion that would silently break the proxy (Q6=B).

---

## BR-U5-07: Config Keys Are a Closed Set

**Rule**: Only the following keys are valid in the `config` table: `anthropic_api_key`, `request_timeout_secs`, `rate_limit_capacity`, `rate_limit_refill_per_sec`. Attempts to write unknown keys MUST be rejected.

**Rationale**: Prevents config table pollution and typo-based key creation.

---

## BR-U5-08: API Key Masked in GET Responses

**Rule**: The `anthropic_api_key` value returned by `GET /api/config` MUST be masked:
- Empty/unset → return `""`
- Set, length > 12 → return `first_8_chars + "***...***" + last_4_chars`
- Set, length ≤ 12 → return `"***...***"`

Plaintext API keys MUST NEVER appear in any response body or log entry.

**Rationale**: SECURITY-06, SECURITY-03.

---

## BR-U5-09: Password Update Applies Only to the Calling Admin's Account

**Rule**: `PUT /api/config/credentials` updates the password of the admin user identified by the Basic Auth credentials used to authenticate the request. An admin cannot change another admin's password via this endpoint.

**Rationale**: Least-privilege credential management.

---

## BR-U5-10: New Password Minimum Length

**Rule**: Any new password (bootstrap or update) MUST be at least 8 characters. Shorter passwords MUST be rejected with a `422 Unprocessable Entity`.

**Rationale**: Minimum viable password policy; SECURITY-12.

---

## BR-U5-11: Admin Username Format

**Rule**: Admin usernames MUST be non-empty, at most 64 characters, and contain only alphanumeric characters plus `_`, `-`, `.`. Invalid usernames MUST be rejected during creation.

**Rationale**: Prevents injection and interoperability issues with the Basic Auth header format.

---

## BR-U5-12: Config Reads Use Defaults for Missing Keys

**Rule**: If a key is absent from the `config` table, `ConfigService::get_all()` MUST return the documented default value for that field rather than an error.

| Key | Default |
|---|---|
| `anthropic_api_key` | `""` (empty) |
| `request_timeout_secs` | `30` |
| `rate_limit_capacity` | `100` |
| `rate_limit_refill_per_sec` | `20` |

**Rationale**: Allows the server to function before the user has configured any settings, and prevents hard failures on a fresh DB.

---

## BR-U5-13: Config Updates Are Atomic

**Rule**: When `PUT /api/config` updates multiple fields, all updates MUST succeed or all MUST be rolled back. Partial updates are not allowed.

**Rationale**: Prevents inconsistent state where some settings are updated and others are not.
