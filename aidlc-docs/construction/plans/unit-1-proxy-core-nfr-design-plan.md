# NFR Design Plan — Unit 1: Proxy Core

## Plan Checkboxes

- [x] Answer questions below
- [x] Generate `nfr-design-patterns.md`
- [x] Generate `logical-components.md`

---

## Questions

### Question 1: Upstream Retry Strategy

If the Anthropic API returns a 5xx error, should the proxy automatically retry the request?

A) No retries — return the upstream error immediately; let the client decide to retry
B) Single retry — retry once after a short fixed delay (500ms) on 5xx or network error; return error on second failure
C) Exponential backoff — up to 3 retries with exponential backoff (500ms, 1s, 2s) on 5xx or network error
D) Other (please describe after [Answer]: tag below)

[Answer]: C

---

### Question 2: Circuit Breaker for Upstream Provider

Should the proxy implement a circuit breaker that temporarily stops forwarding requests to a provider after repeated failures?

A) Yes — open circuit after 5 consecutive failures; half-open probe after 30 seconds; return 503 while open
B) No — no circuit breaker; rely on timeout and retry settings; keep it simple
C) Other (please describe after [Answer]: tag below)

[Answer]: B

---

Please fill in all `[Answer]:` tags and let me know when done.
