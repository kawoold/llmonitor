# Tech Stack Decisions — Unit 2: Claude Provider Adapter

## HTTP Client: reqwest 0.11 (existing)

**Decision**: Use the existing `reqwest` dependency (version 0.11) with default configuration.

**Rationale**:
- Already a project dependency from Unit 1's stub provider.
- `reqwest::Client` is `Clone + Send + Sync`, holds an internal connection pool, and supports async JSON request/response.
- Default pool settings are sufficient for a single-instance proxy workload.

**Configuration**:
- `reqwest::Client::new()` — no custom builder settings required.
- Client stored as a field on `ClaudeAdapter` and reused across requests.

---

## UUID Generation: uuid 1.x (existing)

**Decision**: Use the existing `uuid` crate with the `v4` feature for generating `chatcmpl-{uuid_v4}` response IDs (BR-U2-10).

**Rationale**: Already a project dependency; `Uuid::new_v4()` is sufficient.

---

## Random Jitter: rand_core 0.6 (existing)

**Decision**: Use `rand_core::OsRng` to generate jitter for retry delays without adding the `rand` crate.

**Rationale**:
- `rand_core` is already a direct dependency (added in Unit 5 for Argon2id salt generation).
- Adding `rand` would be an unnecessary dependency; the jitter range [0.9, 1.1] can be computed with manual arithmetic from a raw `u32`.

**Implementation sketch**:
```rust
use rand_core::{OsRng, RngCore};

fn jitter_factor() -> f64 {
    let raw = OsRng.next_u32() as f64 / u32::MAX as f64; // [0.0, 1.0)
    0.9 + raw * 0.2 // [0.9, 1.1)
}
```

---

## Serialization: serde_json 1.x (existing)

**Decision**: Use existing `serde` / `serde_json` for serializing `AnthropicRequest` and deserializing `AnthropicResponse` / `AnthropicErrorResponse`.

**Rationale**: Already used project-wide. `#[serde(skip_serializing_if = "Option::is_none")]` and `#[serde(untagged)]` cover all required shapes.

---

## No New Dependencies

Unit 2 introduces **zero new crate dependencies**. All required capabilities are covered by existing dependencies:

| Capability | Crate | Already present |
|---|---|---|
| Async HTTP client | `reqwest 0.11` | Yes |
| JSON serialization | `serde_json 1` | Yes |
| UUID v4 generation | `uuid 1` | Yes |
| Random jitter | `rand_core 0.6` | Yes |
| Async runtime | `tokio 1` | Yes |
| Structured logging | `tracing 0.1` | Yes |
| Error types | `thiserror 1` | Yes |
