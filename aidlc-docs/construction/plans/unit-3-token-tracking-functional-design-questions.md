# Functional Design Questions — Unit 3: Token Tracking & Session Engine

---

**Q1 — `request_logs` schema fields**

Which fields should be persisted in the `request_logs` table? Select the set that best matches your needs:

A. **Minimal** — `id`, `created_at`, `provider`, `model`, `session_id`, `prompt_tokens`, `completion_tokens`, `total_tokens`
B. **Standard** — everything in A, plus `cache_read_tokens`, `cache_creation_tokens`, `duration_ms` (wall time of the Anthropic call)
C. **Extended** — everything in B, plus `request_id` (from tower-http request-id header), `finish_reason`

[Answer]: A

---

**Q2 — Session ID: which messages are hashed?**

The session ID is a SHA-256 hash of "prior messages". For a multi-turn request with messages `[user, assistant, user]`, which messages should be included in the hash input?

A. **All messages except the last user message** — i.e., the messages that represent the conversation history *before* this turn. Turn 1 (single user message) → empty history → nil session ID.
B. **All messages in the array** — hash the full messages list including the new user message. Turn 1 with one message → hash of that one message.

[Answer]: B

---

**Q3 — mpsc channel capacity**

The `TrackingService` holds an mpsc sender. When the channel is full, `record()` is non-blocking (logs a warning, request still completes). What should the channel capacity be?

A. **100** — small buffer; backpressure warning appears quickly under load (good for testing that path)
B. **1 000** — medium buffer; handles bursts without pressure warnings in normal operation
C. **10 000** — large buffer; nearly never fills under realistic single-instance load

[Answer]: B

---

**Q4 — Writer flush trigger**

The `TrackingWriterService` runs a background loop with `batch_size=50` and `flush_interval=500ms`. When should it flush pending records to SQLite?

A. **Either condition** — flush as soon as 50 records accumulate OR 500ms passes since last flush, whichever comes first (lower latency for low-traffic periods)
B. **Interval only** — flush every 500ms regardless of batch size (simpler, predictable)

[Answer]: B

---

**Q5 — DB write failure in the background writer**

If the SQLite `INSERT` fails inside `TrackingWriterService` (e.g., disk full, constraint error), what should happen?

A. **Log and drop** — log the error at ERROR level and discard the failed batch; writer continues processing subsequent records
B. **Log and retry once** — attempt the insert a second time after a brief pause; if it fails again, drop and continue
C. **Log and stop writer** — treat persistent DB errors as fatal and stop the writer task (proxy requests still complete, but tracking silently stops until restart)

[Answer]: B
