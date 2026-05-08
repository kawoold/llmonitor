# Unit 7: Prompt Tab — Functional Design Questions

Please answer each question by filling in the letter choice after the `[Answer]:` tag.
If none of the options match your needs, choose the last option (Other) and describe your preference.
Let me know when you're done.

---

## Question 1
Which Claude models should appear in the `/api/models` endpoint and the frontend selector?

A) **Current trio** — `claude-opus-4-7`, `claude-sonnet-4-6`, `claude-haiku-4-5-20251001`. Matches what the proxy currently supports.

B) **Sonnet and Haiku only** — `claude-sonnet-4-6`, `claude-haiku-4-5-20251001`. Opus is expensive; exclude it from the prompt playground.

C) Other (please describe after [Answer]: tag below — list the exact model IDs you want)

[Answer]: A

---

## Question 2
When the user cancels a streaming response mid-flight (via a Stop button), what should happen to the partial text already received?

A) **Keep the partial response** — display what arrived so far, marked with a "(cancelled)" indicator. The conversation can continue from there.

B) **Discard the partial response** — remove the in-progress assistant bubble entirely, as if the message was never sent.

C) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 3
The `/v1/messages` proxy route currently has no authentication. Should the PromptTab send the admin Basic auth credentials when calling it, or call it without credentials?

A) **Send admin credentials** — include the `Authorization: Basic ...` header (same credentials stored in sessionStorage). This way every proxy request is tied to an authenticated session, and a 401 would still trigger logout. Requires adding Basic auth middleware to the proxy route on the backend.

B) **No credentials on proxy route** — call `/v1/messages` without an Authorization header. The proxy is already running inside the trusted admin UI context; the API key it uses belongs to the server config, not the browser session.

C) Other (please describe after [Answer]: tag below)

[Answer]: A
