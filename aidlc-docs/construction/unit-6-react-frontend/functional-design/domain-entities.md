# Domain Entities — Unit 6: React Frontend

## API Response Types (TypeScript mirrors of Rust structs)

```typescript
// src/types/api.ts

interface ProviderSummary {
  provider: string;
  model: string;
  request_count: number;
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
}

interface StatsSummary {
  window: string;
  total_requests: number;
  total_tokens: number;
  prompt_tokens: number;
  completion_tokens: number;
  by_provider: ProviderSummary[];
}

interface UsageDataPoint {
  bucket: string;
  request_count: number;
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
}

interface SessionStats {
  session_id: string;
  request_count: number;
  total_tokens: number;
  first_seen: string;
  last_seen: string;
}

interface CacheStats {
  window: string;
  cache_read_tokens: number;
  cache_creation_tokens: number;
  total_requests: number;
  cache_hit_rate: number;
}

interface ConfigResponse {
  anthropic_api_key: string;      // masked by backend
  request_timeout_secs: number;
  rate_limit_capacity: number;
  rate_limit_refill_per_sec: number;
}

interface ConfigUpdate {
  anthropic_api_key?: string;
  request_timeout_secs?: number;
}

interface CredentialsUpdate {
  username: string;
  password: string;
}
```

---

## Auth State

Stored in `sessionStorage` under key `"llm_auth"`.

Value: base64-encoded `"username:password"` (same format as HTTP Basic Auth header value after `"Basic "`).

```
sessionStorage["llm_auth"] = btoa("admin:password123")
```

Cleared on logout or on any 401 response from the API.

---

## Window State

```typescript
type TimeWindow = "1h" | "24h" | "7d" | "30d";
const DEFAULT_WINDOW: TimeWindow = "24h";
```

Held in component state, not URL params. Shared between Dashboard and CachePanel via prop drilling from App.

---

## Component Tree

```
App
├── AuthGate          (shown when no sessionStorage credentials)
└── MainLayout        (shown when authenticated)
    ├── NavTabs       (Dashboard | Cache | Config)
    ├── Dashboard     (active tab)
    │   ├── WindowSelector
    │   ├── SummaryCards
    │   ├── UsageLineChart
    │   └── ProviderBarChart
    ├── CachePanel    (active tab)
    │   ├── WindowSelector
    │   ├── CacheStatCards
    │   └── CacheBarChart
    └── ConfigPanel   (active tab)
        ├── ApiKeyForm
        └── CredentialsForm
```
