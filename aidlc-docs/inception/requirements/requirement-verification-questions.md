# Requirements Verification Questions — LLM Proxy (llmonitor)

Please answer each question by filling in the letter choice after the `[Answer]:` tag.
If none of the provided options match, choose the last "Other" option and describe your preference.
Let me know when you've finished answering all questions.

---

## Section 1: Technology Stack

## Question 1
What programming language should be used to build this proxy?

A) Rust (the workspace directory name `rust/llmonitor` suggests this is your intent)
B) Go
C) Python
D) TypeScript / Node.js
E) Other (please describe after [Answer]: tag below)

[Answer]: A. I would also like to provide a web-based frontend for the proxy that can be used for configuration and analysis. The frontend should be built in typescript and react.

---

## Question 2
What is the preferred approach for the HTTP server / proxy framework?

A) Axum (Rust async web framework — recommended for Rust)
B) Actix-web (Rust high-performance web framework)
C) Hyper (low-level Rust HTTP library)
D) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Section 2: Core Proxy Behavior

## Question 3
How should the proxy handle provider routing? When a client sends an OpenAI-format request, how does the proxy know which backend to call?

A) Static configuration — a single backend provider is configured at startup (e.g. always route to Claude)
B) Request header — client specifies the target provider via a custom HTTP header (e.g. `X-Provider: claude`)
C) API key prefix — the proxy infers the provider from the API key format or prefix
D) Model name mapping — the proxy routes based on the model name in the request (e.g. `claude-3-opus` → Anthropic)
E) Other (please describe after [Answer]: tag below)

[Answer]: B

---

## Question 4
Should the proxy support streaming responses (Server-Sent Events / SSE)?

A) Yes — streaming must be supported, it is essential for real-time UX
B) Yes — streaming should be supported but is lower priority than core functionality
C) No — streaming is not required initially, batch responses only
D) Other (please describe after [Answer]: tag below)

[Answer]: B

---

## Question 5
How should the proxy authenticate incoming client requests?

A) Pass-through — the proxy forwards the client's API key directly to the upstream provider (no proxy-level auth)
B) Proxy API keys — the proxy issues its own API keys; clients authenticate to the proxy, and the proxy uses stored upstream keys
C) No authentication — the proxy is internal/trusted and requires no client auth
D) Other (please describe after [Answer]: tag below)

[Answer]: C

---

## Section 3: Token Tracking & Storage

## Question 6
Where should token usage data be stored?

A) SQLite — embedded database, simple deployment, no external dependencies
B) PostgreSQL — production-grade relational database
C) In-memory only — no persistence, statistics reset on restart (suitable for development/PoC)
D) ClickHouse or time-series DB — optimized for analytics workloads
E) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 7
At what granularity should token usage be tracked?

A) Per-request — track input tokens, output tokens, model, and timestamp for every request
B) Per-request + per-session — also group requests by session/conversation ID
C) Per-request + per-user — associate usage with the authenticated user or API key
D) All of the above — per-request, per-session, and per-user tracking
E) Other (please describe after [Answer]: tag below)

[Answer]: B

---

## Question 8
Should the proxy track cost (estimated USD) in addition to token counts?

A) Yes — calculate estimated cost based on published model pricing
B) Yes — but cost calculation should be configurable/pluggable (pricing changes over time)
C) No — track token counts only; cost calculation is out of scope
D) Other (please describe after [Answer]: tag below)

[Answer]: C

---

## Section 4: Statistics & Reporting

## Question 9
How should usage statistics be exposed to operators/users?

A) REST API only — expose statistics via JSON endpoints (e.g. `/v1/stats`, `/v1/usage`)
B) REST API + simple web dashboard — serve a basic HTML/JS dashboard alongside the API
C) REST API + Prometheus metrics — expose metrics in Prometheus format for integration with Grafana etc.
D) All three — REST API, dashboard, and Prometheus metrics
E) Other (please describe after [Answer]: tag below)

[Answer]: D

---

## Question 10
What time-range aggregations should the statistics API support?

A) Real-time only — current session/day totals
B) Rolling windows — last 1h, 24h, 7d, 30d
C) Custom date range — arbitrary start/end date queries
D) Rolling windows + custom date range
E) Other (please describe after [Answer]: tag below)

[Answer]: B

---

## Section 5: Provider Support

## Question 11
For the Claude (Anthropic) provider integration, which Claude-specific features should the proxy expose beyond the OpenAI-compatible layer?

A) None initially — only OpenAI-compatible endpoints for Claude
B) Extended thinking / reasoning tokens — expose Anthropic-specific thinking parameter and track thinking token usage separately
C) Prompt caching — expose Anthropic cache control headers and track cache hit/miss rates
D) Both extended thinking and prompt caching
E) Other (please describe after [Answer]: tag below)

[Answer]: C

---

## Question 12
Should the proxy support adding more providers in the future beyond Claude (e.g. Google Gemini, AWS Bedrock, Ollama)?

A) Yes — design the provider abstraction to be easily extensible (plugin-style)
B) Yes — but only formally support Claude now; future providers can be added ad-hoc
C) No — Claude is the only provider needed, no extensibility required
D) Other (please describe after [Answer]: tag below)

[Answer]: B

---

## Section 6: Deployment & Operations

## Question 13
What is the primary deployment model for this proxy?

A) Single binary / local development tool — runs on developer machines alongside other services
B) Docker container — packaged as a container for easy deployment anywhere
C) Self-hosted server — deployed on a VPS or on-premises machine
D) All of the above should be supported (single binary that's also containerisable)
E) Other (please describe after [Answer]: tag below)

[Answer]: B

---

## Question 14
Should the proxy support configuration via a file, environment variables, or both?

A) Environment variables only (12-factor app style)
B) Configuration file only (TOML or YAML)
C) Both — configuration file with environment variable overrides
D) Other (please describe after [Answer]: tag below)

[Answer]: D. Configuration should be provided through a Rest API that can be called through the UI component. It should be persisted in the DB

---

## Section 7: Extensions

## Question 15 — Security Extension
Should security extension rules be enforced for this project?

A) Yes — enforce all SECURITY rules as blocking constraints (recommended for production-grade applications)
B) No — skip all SECURITY rules (suitable for PoCs, prototypes, and experimental projects)
X) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 16 — Property-Based Testing Extension
Should property-based testing (PBT) rules be enforced for this project?

A) Yes — enforce all PBT rules as blocking constraints (recommended for projects with business logic, data transformations, serialization, or stateful components)
B) Partial — enforce PBT rules only for pure functions and serialization round-trips
C) No — skip all PBT rules
X) Other (please describe after [Answer]: tag below)

[Answer]: B
