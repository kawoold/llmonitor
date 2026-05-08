# Requirements Clarification Questions — LLM Proxy (llmonitor)

I detected two contradictions in your responses that need clarification before I can proceed.

---

## Contradiction 1: No Authentication vs. Security-Enforced Configuration API

**Issue**: You indicated **no client authentication** (Q5: C) but also want:
- Configuration managed via a REST API accessible through the UI (Q14: D)
- Security extension rules enforced as blocking constraints (Q15: A)

An unauthenticated REST API that controls proxy configuration (provider keys, routing rules, etc.) is a significant security risk, even in internal deployments — especially with security rules enforced.

### Clarification Question 1
How should the **configuration/management REST API** (the one used by the React UI) be protected?

A) Basic authentication — a single admin username + password configured at startup (simple, suitable for internal tools)
B) Token-based authentication — an admin token/secret configured at startup, passed in Authorization header by the UI
C) No authentication on the management API either — the proxy and its configuration API are fully internal/trusted
D) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Ambiguity 1: Session Identification Without User Authentication

**Issue**: You want **per-request + per-session token tracking** (Q7: B), but the proxy has **no authentication** (Q5: C). Without an identity layer, sessions must be identified by some other mechanism.

### Clarification Question 2
How should sessions be identified for usage tracking when there is no user authentication?

A) Client-provided session ID — clients pass an optional `X-Session-ID` header; the proxy groups requests by this ID
B) Conversation thread ID — inferred from the message history structure in the request body (if messages share context)
C) Skip per-session tracking — since there's no auth, only track per-request; aggregate statistics are still available
D) Other (please describe after [Answer]: tag below)

[Answer]: B

---

Please answer both clarification questions by filling in the letter choice after each `[Answer]:` tag, then let me know when you're done.
