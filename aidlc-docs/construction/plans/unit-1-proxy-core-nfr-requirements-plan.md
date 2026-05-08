# NFR Requirements Plan — Unit 1: Proxy Core

## Plan Checkboxes

- [x] Answer questions below
- [x] Generate `nfr-requirements.md`
- [x] Generate `tech-stack-decisions.md`

---

## Context

Most NFRs for Unit 1 are already established by the requirements document (performance <10ms overhead, security baseline, structured logging, Docker deployment). This plan captures the remaining open tech stack decisions and a few configuration value questions.

---

## Questions

### Question 1: Upstream Request Timeout

How long should the proxy wait for the upstream provider (Anthropic) before returning a 504 Gateway Timeout?

A) 30 seconds — reasonable for long Claude responses; unlikely to hit for most requests
B) 60 seconds — generous; accommodates very long generations and thinking-heavy prompts
C) Configurable via the config API, with a default of 30 seconds
D) Other (please describe after [Answer]: tag below)

[Answer]: C

---

### Question 2: Structured Logging Level

What should the default log level be, and should it be configurable at runtime?

A) INFO level by default; configurable only at startup via `RUST_LOG` environment variable
B) INFO level by default; configurable at runtime via the config API (stored in DB, applied on reload)
C) Other (please describe after [Answer]: tag below)

[Answer]: A

---

### Question 3: Rate Limit Defaults

The global rate limit has configurable capacity and refill rate (BR-06). What should the out-of-the-box defaults be?

A) 100 requests capacity, 20 tokens/second refill (~1,200 RPM sustained)
B) 60 requests capacity, 10 tokens/second refill (~600 RPM sustained)
C) Other values (describe after [Answer]: tag below)

[Answer]: A

---

Please fill in all `[Answer]:` tags and let me know when done.
