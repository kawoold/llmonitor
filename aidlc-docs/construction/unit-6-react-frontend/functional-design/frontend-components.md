# Frontend Components — Unit 6: React Frontend

## Component Hierarchy

```
App (auth state owner)
├── AuthGate
└── MainLayout
    ├── NavTabs
    ├── Dashboard (tab)
    │   ├── WindowSelector
    │   ├── SummaryCards
    │   ├── UsageLineChart
    │   └── ProviderBarChart
    ├── CachePanel (tab)
    │   ├── WindowSelector
    │   ├── CacheStatCards
    │   └── CacheBarChart
    └── ConfigPanel (tab)
        ├── ApiKeyForm
        └── CredentialsForm
```

---

## AuthGate

**Props**: `onLogin: (base64Creds: string) => void`

**State**: `username: string`, `password: string`, `error: string | null`, `loading: boolean`

**Interactions**:
- Username and password inputs (controlled)
- Submit button calls `GET /api/config`; on 200 calls `onLogin(btoa("user:pass"))`
- Shows inline error on 401

**data-testid**: `auth-username-input`, `auth-password-input`, `auth-submit-button`

---

## NavTabs

**Props**: `activeTab: Tab`, `onTabChange: (tab: Tab) => void`

**Type**: `type Tab = "dashboard" | "cache" | "config"`

**Interactions**: Three tab buttons; active tab highlighted with Tailwind ring/border

**data-testid**: `nav-dashboard`, `nav-cache`, `nav-config`

---

## WindowSelector

**Props**: `value: TimeWindow`, `onChange: (w: TimeWindow) => void`

**Interactions**: Four buttons (`1h`, `24h`, `7d`, `30d`); active one has distinct Tailwind bg

**data-testid**: `window-selector-1h`, `window-selector-24h`, `window-selector-7d`, `window-selector-30d`

---

## Dashboard

**Props**: none (reads from App state via context or top-level fetch)

**State**: `window: TimeWindow`, `summary: StatsSummary | null`, `timeseries: UsageDataPoint[]`, `loading: boolean`, `error: string | null`

**Interactions**:
- `WindowSelector` triggers data refetch on change
- Displays SummaryCards, UsageLineChart, ProviderBarChart

**Loading state**: spinner overlay
**Error state**: red error banner with message
**Empty state**: charts show "No data for this window"

---

## SummaryCards

**Props**: `summary: StatsSummary`

Renders 4 stat cards: Total Requests, Total Tokens, Prompt Tokens, Completion Tokens.

---

## UsageLineChart

**Props**: `data: UsageDataPoint[]`

Recharts `<ResponsiveContainer>` wrapping `<LineChart>`. X-axis: `bucket` (truncated label). Y-axis: token count. Lines: `total_tokens` (primary), `prompt_tokens`, `completion_tokens`.

Empty: renders "No data" text centered in the chart area.

---

## ProviderBarChart

**Props**: `data: ProviderSummary[]`

Recharts `<BarChart>`. X-axis: `provider/model` label. Bars: `prompt_tokens` and `completion_tokens` stacked or grouped.

Empty: renders "No data" text.

---

## CachePanel

**Props**: none

**State**: `window: TimeWindow`, `cacheData: CacheStats | null`, `loading: boolean`, `error: string | null`

**Interactions**: `WindowSelector` triggers refetch

**Empty state** (total_requests === 0): "No cache activity in this window" message replaces chart

---

## CacheStatCards

**Props**: `data: CacheStats`

Four stat cards:
- Hit Rate: `(data.cache_hit_rate * 100).toFixed(1) + "%"`
- Read Tokens: `data.cache_read_tokens`
- Creation Tokens: `data.cache_creation_tokens`
- Cached Requests: `data.total_requests`

---

## CacheBarChart

**Props**: `data: CacheStats`

Recharts `<BarChart>` with two bars: "Read" (cache_read_tokens) and "Creation" (cache_creation_tokens). Single data point.

---

## ConfigPanel

**Props**: none

**State**: `config: ConfigResponse | null`, `apiKeyDraft: string`, `apiKeySaving: boolean`, `apiKeyError: string | null`, `apiKeySuccess: boolean`, `credUsername: string`, `credPassword: string`, `credSaving: boolean`, `credError: string | null`

---

## ApiKeyForm

**Props**: `currentMaskedKey: string`, `onSave: (key: string) => Promise<void>`

**Interactions**:
- Input shows placeholder with masked key hint
- Submit disabled if input empty
- Shows success banner for 3s after save

**data-testid**: `apikey-input`, `apikey-submit-button`

---

## CredentialsForm

**Props**: `onSave: (username: string, password: string) => Promise<void>`

**Interactions**:
- Both fields required; submit disabled if either empty
- On success: shows "Credentials updated. Logging out..." then clears sessionStorage + reloads

**data-testid**: `credentials-username-input`, `credentials-password-input`, `credentials-submit-button`
