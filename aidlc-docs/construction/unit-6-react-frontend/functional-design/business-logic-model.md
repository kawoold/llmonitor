# Business Logic Model — Unit 6: React Frontend

## 1. Auth Flow

```
App renders
  └─ sessionStorage["llm_auth"] present?
       NO  → render <AuthGate>
             user enters username + password
             submit → GET /api/config with Authorization: Basic btoa("user:pass")
               200 → sessionStorage["llm_auth"] = btoa("user:pass")
                     render <MainLayout>
               401 → show inline error, clear field
       YES → render <MainLayout>

Any API call returns 401 after login:
  → sessionStorage.removeItem("llm_auth")
  → re-render <AuthGate>
```

---

## 2. API Client Layer

`src/api/client.ts` — shared fetch wrapper:

```typescript
function authHeader(): string {
  return "Basic " + (sessionStorage.getItem("llm_auth") ?? "");
}

async function apiFetch<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(path, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      Authorization: authHeader(),
      ...init?.headers,
    },
  });
  if (res.status === 401) {
    sessionStorage.removeItem("llm_auth");
    window.location.reload();
    throw new Error("Unauthorized");
  }
  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    throw new Error(body?.error?.message ?? `HTTP ${res.status}`);
  }
  return res.json() as Promise<T>;
}
```

Typed wrappers in `src/api/stats.ts` and `src/api/config.ts` call `apiFetch` with the correct path and return type.

---

## 3. Dashboard Data Flow

```
Dashboard component mounts / window changes
  → setLoading(true)
  → Promise.all([fetchSummary(window), fetchUsage(window)])
      success → setSummary(data), setTimeseries(data), setLoading(false)
      error   → setError(message), setLoading(false)

Render:
  loading  → spinner
  error    → error message with retry button
  data     → SummaryCards + UsageLineChart + ProviderBarChart
```

SummaryCards shows: total requests, total tokens, prompt tokens, completion tokens — all for the selected window.

UsageLineChart: Recharts `<LineChart>` with `bucket` on X-axis, `total_tokens` as the line (optionally `prompt_tokens` + `completion_tokens` as additional lines).

ProviderBarChart: Recharts `<BarChart>` grouped by `provider + model`, bars for `prompt_tokens` and `completion_tokens`.

---

## 4. Cache Panel Data Flow

```
CachePanel mounts / window changes
  → fetchCache(window)
      success → setCacheData(data)
      total_requests === 0 → show "No cache activity" message
      total_requests > 0   → show CacheStatCards + CacheBarChart

CacheStatCards:
  - Hit rate: (cache_hit_rate * 100).toFixed(1) + "%"
  - Read tokens: cache_read_tokens (tokens served from cache)
  - Creation tokens: cache_creation_tokens (tokens written to cache)
  - Requests with cache: total_requests

CacheBarChart: Recharts <BarChart> with two bars:
  - "Cache Read" (cache_read_tokens) — cost saving
  - "Cache Creation" (cache_creation_tokens) — cost
```

---

## 5. Config Panel Data Flow

```
ConfigPanel mounts
  → fetchConfig()
      success → display masked key in read-only field, populate form fields

ApiKeyForm submit:
  → validate: key !== "" (client-side)
  → PUT /api/config { anthropic_api_key: newKey }
      success → re-fetch config, show "Saved" banner for 3s
      error   → show error message

CredentialsForm submit:
  → validate: username !== "" AND password !== ""
  → PUT /api/config/credentials { username, password }
      success → show "Credentials updated. You will be logged out." → clear sessionStorage → reload
      error   → show error message
```

After credentials change, the stored auth hash is stale, so the user must log in again. Log-out + reload is the correct UX.

---

## 6. Project Structure

```
frontend/
├── index.html              ← Tailwind CDN script tag; Vite SPA entry
├── package.json            ← deps: react, react-dom, recharts; devDeps: vite, typescript, @types/*
├── tsconfig.json
├── vite.config.ts          ← outDir: "../frontend/dist" (writes to workspace root frontend/dist/)
└── src/
    ├── main.tsx
    ├── App.tsx             ← auth state, tab routing
    ├── api/
    │   ├── client.ts
    │   ├── stats.ts
    │   └── config.ts
    ├── components/
    │   ├── AuthGate.tsx
    │   ├── Dashboard.tsx
    │   ├── CachePanel.tsx
    │   ├── ConfigPanel.tsx
    │   ├── WindowSelector.tsx
    │   ├── NavTabs.tsx
    │   └── charts/
    │       ├── UsageLineChart.tsx
    │       ├── ProviderBarChart.tsx
    │       └── CacheBarChart.tsx
    └── types/
        └── api.ts
```

---

## 7. Infrastructure Deliverables

`docker-compose.yml` at workspace root:
- Service `llmonitor` built from local `Dockerfile`
- Port mapping: `8080:8080`
- Volume: `llmonitor-data:/data` mounted at `/data`
- Env vars: `LLMONITOR_DB_PATH`, `LLMONITOR_LISTEN_ADDR`, `LLMONITOR_ADMIN_USER`, `LLMONITOR_ADMIN_PASSWORD`

`.env.example` at workspace root — documents all environment variables with descriptions and example values.
