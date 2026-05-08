# Tech Stack Decisions — Unit 3: Token Tracking & Session Engine

## SHA-256: sha2 crate (new direct dependency)

**Decision**: Add `sha2 = "0.10"` as a direct dependency.

**Rationale**: `sha2` is already a transitive dependency (via `argon2`), but must be declared directly so the import path is stable and the dependency is explicit. Used by `SessionEngine::derive_session_id()`.

**Cargo.toml addition**:
```toml
sha2 = "0.10"
```

**Usage**:
```rust
use sha2::{Digest, Sha256};
let mut hasher = Sha256::new();
hasher.update(canonical.as_bytes());
let result = hasher.finalize();
hex::encode(result)  // or format with {:x}
```

**Alternative for hex encoding**: Use `format!("{:x}", result)` to avoid a `hex` crate dependency (Sha256 output implements `LowerHex`).

---

## Async Channel: tokio::sync::mpsc (existing)

**Decision**: Use `tokio::sync::mpsc::channel(1_000)` from the existing `tokio` dependency.

**Rationale**: Already available. Provides bounded async channel with `try_send()` for non-blocking sends. No additional crate needed.

---

## Timer: tokio::time::interval (existing)

**Decision**: Use `tokio::time::interval(Duration::from_millis(500))` for the flush ticker.

**Rationale**: Already available via `tokio` full feature set. Provides a reliable interval tick even under load.

---

## Database Writes: sqlx::query() (existing)

**Decision**: Use the runtime `sqlx::query()` API (not `query!()` macros) for inserts, consistent with Units 1 and 5.

**Rationale**: No `DATABASE_URL` at compile time; the string-based API avoids that requirement.

---

## Property-Based Testing: proptest (existing)

**Decision**: Use `proptest` (already in `[dev-dependencies]`) for `SessionEngine` property tests.

**Rationale**: Already a dev dependency. Testing determinism and output stability with arbitrary message arrays is a natural fit for proptest strategies.

---

## No New Dependencies Beyond sha2

| Capability | Crate | Status |
|---|---|---|
| SHA-256 hashing | `sha2 0.10` | **NEW** (direct dep) |
| Async bounded channel | `tokio::sync::mpsc` | Existing |
| Interval timer | `tokio::time::interval` | Existing |
| SQLite writes | `sqlx 0.7` | Existing |
| UUID generation | `uuid 1` | Existing |
| Timestamps | `chrono 0.4` | Existing |
| Property tests | `proptest 1` | Existing (dev) |
