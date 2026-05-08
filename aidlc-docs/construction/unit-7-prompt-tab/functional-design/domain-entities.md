# Domain Entities — Unit 7: Prompt Tab

## Backend Entities

### ModelOption
Represents a single selectable model in the prompt playground.

| Field | Type | Description |
|-------|------|-------------|
| `provider` | `String` | Provider name, e.g. `"anthropic"` |
| `model` | `String` | Model ID, e.g. `"claude-sonnet-4-6"` |
| `display_name` | `String` | Human-readable label, e.g. `"Claude Sonnet 4.6"` |

Hardcoded set returned by `GET /api/models`:
```json
[
  {"provider":"anthropic","model":"claude-opus-4-7","display_name":"Claude Opus 4.7"},
  {"provider":"anthropic","model":"claude-sonnet-4-6","display_name":"Claude Sonnet 4.6"},
  {"provider":"anthropic","model":"claude-haiku-4-5-20251001","display_name":"Claude Haiku 4.5"}
]
```

---

## Frontend Entities

### Message
A single turn in the conversation.

| Field | Type | Description |
|-------|------|-------------|
| `role` | `"user" \| "assistant"` | Who sent this message |
| `content` | `string` | Message text |
| `cancelled` | `boolean?` | True if stream was cancelled before completion |

### ConversationState (PromptTab component state)

| Field | Type | Description |
|-------|------|-------------|
| `messages` | `Message[]` | Full conversation history |
| `input` | `string` | Current textarea draft |
| `selectedModel` | `string` | Model ID currently selected |
| `models` | `ModelOption[]` | Fetched from `/api/models` |
| `maxTokens` | `number` | 1–4096, default 1024 |
| `temperature` | `number` | 0.0–1.0, step 0.1, default 1.0 |
| `streaming` | `boolean` | True while a response is in flight |
| `error` | `string \| null` | Inline error message |

### SseEvent (parsed from streaming response)
Anthropic SSE events passed through from the proxy. The frontend cares about:

| Event type | Relevant field | Action |
|------------|---------------|--------|
| `content_block_delta` where `delta.type === "text_delta"` | `delta.text` | Append to current assistant message |
| `message_stop` | — | Mark streaming complete |
| `error` | `error.message` | Surface as inline error |

---

## TypeScript Types (`src/types/api.ts` additions)

```ts
export interface ModelOption {
  provider: string;
  model: string;
  display_name: string;
}

export interface ChatMessage {
  role: "user" | "assistant";
  content: string;
  cancelled?: boolean;
}
```
