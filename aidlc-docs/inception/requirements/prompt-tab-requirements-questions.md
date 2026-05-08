# Prompt Tab — Requirements Questions

Please answer each question by filling in the letter choice after the `[Answer]:` tag.
If none of the options match your needs, choose the last option (Other) and describe your preference.
Let me know when you're done.

---

## Question 1
Should the proxy response be streamed to the UI as tokens arrive, or displayed all at once after the full response is received?

A) **Streaming** — tokens appear progressively as they arrive (requires SSE/chunked response handling in the frontend)

B) **All at once** — wait for the complete response, then display it in full

C) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 2
Should the Prompt tab support a multi-turn conversation (chat history visible, each message adds to context), or single-turn only (one prompt → one response, no history)?

A) **Multi-turn** — conversation history is shown and sent with each new message (like a chat UI)

B) **Single-turn** — one prompt in, one response out; a "Clear" button resets for the next attempt

C) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 3
Where should the list of available models/providers come from?

A) **Hardcoded in the frontend** — a fixed list matching what the proxy currently supports (e.g. `claude-sonnet-4-6`, `claude-haiku-4-5`, `claude-opus-4-7`). Simple, no backend change needed.

B) **A new backend endpoint** — add a `GET /api/models` route that returns the list of supported provider/model combinations. Frontend fetches it at runtime.

C) Other (please describe after [Answer]: tag below)

[Answer]: B

---

## Question 4
Beyond the prompt text and model selection, should the user be able to configure additional generation parameters?

A) **Prompt + model only** — no extra controls. The proxy uses sensible defaults for `max_tokens`, `temperature`, etc.

B) **Prompt + model + max_tokens** — expose a single numeric input for max tokens (e.g. 1–4096), keep everything else at defaults.

C) **Prompt + model + max_tokens + temperature** — expose both controls.

D) Other (please describe after [Answer]: tag below)

[Answer]: C

---

## Question 5
How should the response be displayed?

A) **Formatted text** — render the assistant's text content as plain text in a styled response box. Ignore non-text content blocks.

B) **Markdown rendered** — parse and render markdown in the assistant response (bold, code blocks, lists, etc.)

C) **Raw JSON** — show the full API response body in a `<pre>` block (useful for debugging)

D) Other (please describe after [Answer]: tag below)

[Answer]: B
