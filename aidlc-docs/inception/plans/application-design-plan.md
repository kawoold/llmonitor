# Application Design Plan — llmonitor LLM Proxy

## Plan Overview

This plan covers the high-level component identification, interface design, and service layer architecture for llmonitor. Detailed business logic is deferred to Functional Design (per unit, Construction phase).

## Design Plan Checkboxes

- [x] Answer design questions (Steps 1-2 below)
- [x] Generate `components.md` — component definitions and responsibilities
- [x] Generate `component-methods.md` — method signatures and interface contracts
- [x] Generate `services.md` — service orchestration patterns
- [x] Generate `component-dependency.md` — dependency map and data flow
- [x] Generate `application-design.md` — consolidated design document
- [x] Validate design completeness and security compliance

---

## Design Questions — Please Fill In [Answer]: Tags

### Question 1: Rust Project Structure

Should the project use a Cargo workspace (multiple crates) or a single Rust crate with internal modules?

A) Single crate with modules — simpler, one `Cargo.toml`, all code in `src/` with module folders per domain (e.g. `src/proxy/`, `src/providers/`, `src/tracking/`)
B) Cargo workspace — separate crates per major component (e.g. `crates/proxy-core/`, `crates/providers/`, `crates/tracking/`), enables independent compilation and clearer boundaries
C) Other (please describe after [Answer]: tag below)

[Answer]: A

---

### Question 2: Provider Dispatch Strategy

How should the proxy dispatch requests to the correct provider implementation at runtime?

A) Trait object (`Box<dyn Provider>`) — dynamic dispatch; providers are registered in a map and looked up by name at request time; easy to add new providers without recompiling
B) Enum dispatch — a `Provider` enum with a variant per supported provider; match arms call the concrete implementation; zero-cost but requires recompile to add providers
C) Other (please describe after [Answer]: tag below)

[Answer]: B

---

### Question 3: Async Token Tracking Write Path

Token usage must be written to SQLite without blocking the response path. How should the async write pipeline be structured?

A) Fire-and-forget Tokio task — `tokio::spawn` a DB write task per request; simple but no backpressure or batching
B) Bounded mpsc channel — requests send usage records to a dedicated background writer task via a bounded channel; provides backpressure and batching opportunities
C) Other (please describe after [Answer]: tag below)

[Answer]: B

---

### Question 4: React Frontend Build & Serving Integration

How should the compiled React/TypeScript frontend (static assets) be integrated into the Rust binary and served?

A) Embedded at compile time — use `include_dir!` or `rust-embed` to embed the frontend dist folder into the Rust binary; single self-contained binary, no separate file serving needed
B) Served from filesystem — the Rust server serves static files from a configurable directory path; frontend is built separately and mounted as a Docker volume or copied into the image
C) Other (please describe after [Answer]: tag below)

[Answer]: A

---

### Question 5: Configuration Bootstrap

The first time the proxy starts with an empty database, how should the initial admin credentials be provided?

A) Environment variables — read `LLMONITOR_ADMIN_USER` and `LLMONITOR_ADMIN_PASSWORD` on first startup; hash and store them; refuse to start if neither DB record exists nor env vars provided
B) Interactive CLI prompt — on first startup, the binary detects no admin exists and interactively prompts for credentials on the terminal
C) Other (please describe after [Answer]: tag below)

[Answer]: A

---

Please fill in all `[Answer]:` tags above, then let me know when you are done.
