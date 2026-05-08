# Frontend Components — Unit 7: Prompt Tab

## Component Hierarchy (additions)

```
MainLayout
└── PromptTab (new tab)
    ├── [model selector dropdown]
    ├── [max_tokens + temperature controls]
    ├── MessageList
    │   └── MessageBubble (×N)
    ├── [error banner]
    └── [textarea + Send/Stop + Clear buttons]
```

---

## PromptTab

**File**: `frontend/src/components/PromptTab.tsx`

**State**:
| Field | Type | Default |
|-------|------|---------|
| `messages` | `ChatMessage[]` | `[]` |
| `input` | `string` | `""` |
| `models` | `ModelOption[]` | `[]` |
| `selectedModel` | `string` | first model from `/api/models` |
| `maxTokens` | `number` | `1024` |
| `temperature` | `number` | `1.0` |
| `streaming` | `boolean` | `false` |
| `error` | `string \| null` | `null` |
| `modelsLoading` | `boolean` | `true` |

**Refs**: `abortControllerRef: React.MutableRefObject<AbortController \| null>`

**On mount**: fetches `GET /api/models` via `apiFetch`, populates `models`, sets `selectedModel` to `models[0].model`.

**Interactions**:
- Model selector `<select>` → `setSelectedModel`
- Max tokens `<input type="number" min={1} max={4096}>` → `setMaxTokens`
- Temperature `<input type="range" min={0} max={1} step={0.1}>` + numeric display → `setTemperature`
- Textarea → `setInput`; `onKeyDown` Ctrl+Enter → `handleSubmit()`
- Send button → `handleSubmit()` (disabled if `streaming || input.trim() === ""`)
- Stop button (visible only when `streaming`) → `handleStop()`
- Clear button → reset messages/input/error (disabled when `streaming`)

**data-testid**:
- `prompt-model-select`
- `prompt-max-tokens-input`
- `prompt-temperature-input`
- `prompt-input`
- `prompt-send-button`
- `prompt-stop-button`
- `prompt-clear-button`

---

## MessageBubble

**File**: `frontend/src/components/MessageBubble.tsx`

**Props**:
```ts
interface Props {
  message: ChatMessage;
}
```

**Rendering**:
- `role === "user"`: right-aligned bubble, plain text, gray background
- `role === "assistant"`: left-aligned bubble, white background, border; content rendered via `<ReactMarkdown>`
- If `message.cancelled === true`: small `(cancelled)` badge appended below the content

**data-testid**: `message-bubble-user` / `message-bubble-assistant`

---

## NavTabs (updated)

`Tab` type gains `"prompt"`:
```ts
export type Tab = "dashboard" | "cache" | "config" | "prompt";
```

New entry added to `TABS` array:
```ts
{ id: "prompt", label: "Prompt", testId: "nav-prompt" }
```

---

## MainLayout (updated)

Renders `<PromptTab />` when `activeTab === "prompt"`:
```tsx
{activeTab === "prompt" && <PromptTab />}
```

---

## package.json (updated)

`react-markdown` added to `dependencies`:
```json
"react-markdown": "^9.0.1"
```

---

## API Surface Used by PromptTab

| Endpoint | Purpose |
|----------|---------|
| `GET /api/models` | Populate model selector on mount |
| `POST /v1/chat/completions` | Send prompt, receive streaming SSE response |
