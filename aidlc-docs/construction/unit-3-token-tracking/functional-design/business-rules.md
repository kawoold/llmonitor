# Business Rules — Unit 3: Token Tracking & Session Engine

## BR-U3-01: Record Only Successful Requests

**Rule**: A `UsageRecord` MUST be created and submitted ONLY when `provider.chat_completion()` returns `Ok`. Failed requests (any `Err`) MUST NOT produce a tracking record.

**Rationale**: Tracking failed requests would inflate token counts; token usage is only meaningful when Anthropic confirms a response.

---

## BR-U3-02: Session ID from Full Message Array

**Rule**: `SessionEngine::derive_session_id()` MUST hash ALL messages in the request's `messages` array (including the final user message). The canonical input is: each message serialized as `"{role}:{content_text}"`, joined with `"\n"`.

**Rationale**: Q2=B — hash the complete conversation context so that identical requests always produce the same session_id, enabling grouping of repeated queries.

---

## BR-U3-03: Session ID Is Always Present

**Rule**: Every `UsageRecord.session_id` MUST be a non-empty 64-character lowercase hex SHA-256 string. There is no nil/null session ID.

**Rationale**: With Q2=B, even single-message requests produce a valid hash. Session grouping applies to all requests uniformly.

---

## BR-U3-04: `record()` Is Non-Blocking

**Rule**: `TrackingService::record()` MUST use `try_send()` (non-blocking). It MUST NOT use `send().await` or any other blocking/async mechanism that could delay the HTTP response.

**Rationale**: Tracking latency must never be visible to API clients.

---

## BR-U3-05: Channel-Full Does Not Fail the Request

**Rule**: When the mpsc channel is full and `try_send()` returns `Err(Full)`, the record MUST be silently dropped (after logging a WARN). The proxy handler MUST still return `200 OK` to the client.

**Rationale**: Tracking is a best-effort side effect; request reliability takes priority.

---

## BR-U3-06: Channel Capacity Is 1 000

**Rule**: The mpsc channel MUST be created with `tokio::sync::mpsc::channel(1_000)`.

**Rationale**: Q3=B — balances memory usage and burst tolerance for a single-instance proxy.

---

## BR-U3-07: Flush Interval Is 500ms (Interval-Only Trigger)

**Rule**: The `TrackingWriterService` MUST flush its buffer every 500ms using a `tokio::time::interval`. It MUST NOT trigger early on batch-size accumulation.

**Rationale**: Q4=B — simpler, predictable write pattern. Max latency between record creation and DB persistence is ~500ms.

---

## BR-U3-08: Batch Size Cap Is 50

**Rule**: Each flush cycle MUST process at most 50 records from the buffer before yielding. Records beyond 50 in a single tick wait for the next tick.

**Note**: With Q4=B (interval-only), the buffer is fully drained each tick — but if more than 50 arrive between ticks, they wait. This prevents monopolizing the DB connection.

---

## BR-U3-09: DB Insert Failure — Retry Once

**Rule**: If an `INSERT` fails in the writer, the writer MUST wait 50ms and retry the insert exactly once. If the retry also fails, the record MUST be logged at ERROR level and discarded. The writer MUST continue processing subsequent records.

**Rationale**: Q5=B — transient failures (lock contention) are handled by one retry. Persistent failures (disk full) are logged and dropped rather than halting the writer.

---

## BR-U3-10: Idempotent Inserts

**Rule**: The `INSERT` statement MUST use `INSERT OR IGNORE` to handle the (rare) case where a record with the same `id` is inserted twice (e.g., due to retry). The UUID `id` is the primary key.

**Rationale**: Prevents duplicate rows on retry without raising an error.

---

## BR-U3-11: `total_tokens` Is Derived

**Rule**: `UsageRecord.total_tokens` MUST equal `prompt_tokens + completion_tokens`. It is computed at record creation time, not stored separately from the component fields.

**Rationale**: Avoids arithmetic inconsistency between stored fields.

---

## BR-U3-12: Graceful Drain on Shutdown

**Rule**: When the mpsc sender is dropped (shutdown signal received), the writer task MUST drain and flush all remaining buffered records before exiting. It MUST NOT exit immediately when the channel closes.

**Rationale**: Prevents data loss during normal process shutdown (SIGTERM, restart).

---

## BR-U3-13: SHA-256 Content Normalisation

**Rule**: For messages with `Parts` content (array of content parts), ONLY text parts are included in the session hash. Non-text parts (e.g., image_url) are skipped. Text parts are joined with a single space `" "`.

**Rationale**: Keeps the hash stable across equivalent text-only representations and avoids including binary/URL content that may vary.
