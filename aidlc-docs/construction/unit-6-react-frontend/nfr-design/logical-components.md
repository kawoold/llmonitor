# Logical Components — Unit 6: React Frontend

## AuthContext

**File**: `src/context/AuthContext.tsx`

**Responsibility**: Single source of truth for auth credentials. Reads initial state from sessionStorage on mount. Provides `login` and `logout` to the component tree.

**Interface**:
```tsx
interface AuthContextValue {
  creds: string | null;       // base64 "user:pass" or null if logged out
  login: (creds: string) => void;
  logout: () => void;
}
const AuthContext = React.createContext<AuthContextValue>(null!);
export const useAuth = () => React.useContext(AuthContext);
```

**Provider behavior**:
- `login(creds)` → `sessionStorage.setItem("llm_auth", creds)` + `setCreds(creds)`
- `logout()` → `sessionStorage.removeItem("llm_auth")` + `setCreds(null)`
- Initial state: `sessionStorage.getItem("llm_auth") ?? null`

**Consumers**: `AuthGate` (calls `login`), `MainLayout` (calls `logout` via logout button), `apiFetch` callers (read `creds` via `useAuth()`)

---

## apiFetch

**File**: `src/lib/apiFetch.ts`

**Responsibility**: All HTTP calls go through this function. Attaches auth header, handles 401 logout, and retries once on transient failure.

**Signature**:
```ts
async function apiFetch(
  path: string,
  creds: string,
  options?: RequestInit
): Promise<Response>
```

**Not a hook** — plain async function so it can be called from `useEffect` or event handlers without hook rules constraints.

**Retry logic**:
```
try:
  res = await fetch(path, { Authorization: "Basic " + creds, ...options })
  if res.status === 401 → throw new UnauthorizedError()   // caller triggers logout
  if res.ok → return res
  // non-401, non-ok: fall through to retry
  wait 1000ms
  res2 = await fetch(...)
  if res2.ok → return res2
  throw new Error(`HTTP ${res2.status}`)
catch (network error):
  wait 1000ms
  retry once, then rethrow
```

**401 handling**: Throws a typed `UnauthorizedError`. Callers catch it and call `logout()` from context. This keeps `apiFetch` free of direct context dependency.

---

## ErrorBoundary

**File**: `src/components/ErrorBoundary.tsx`

**Responsibility**: Catch any React render-phase error below it and display a safe fallback instead of a blank screen.

**Class component** (required for `componentDidCatch`):
```tsx
interface State { hasError: boolean; message: string }
class ErrorBoundary extends React.Component<React.PropsWithChildren, State>
```

**Fallback render**:
```tsx
<div className="min-h-screen flex items-center justify-center">
  <div>
    <p>Something went wrong: {this.state.message}</p>
    <button onClick={() => window.location.reload()}>Reload</button>
  </div>
</div>
```

**Placement in tree**: `<AuthProvider> → <ErrorBoundary> → <AuthGate | MainLayout>`

---

## Component File Layout

```
frontend/
├── index.html                    # Tailwind CDN script tag, Vite entry point
├── vite.config.ts                # outDir: "../frontend/dist", target: "es2020"
├── tsconfig.json                 # strict: true, es2020
├── package.json
└── src/
    ├── main.tsx                  # ReactDOM.createRoot, renders <App />
    ├── App.tsx                   # AuthProvider, ErrorBoundary, AuthGate/MainLayout switch
    ├── types/
    │   └── api.ts                # All API response TypeScript interfaces (single source of truth)
    ├── context/
    │   └── AuthContext.tsx       # AuthContext, AuthProvider, useAuth
    ├── lib/
    │   └── apiFetch.ts           # apiFetch, UnauthorizedError
    └── components/
        ├── ErrorBoundary.tsx
        ├── AuthGate.tsx
        ├── MainLayout.tsx
        ├── NavTabs.tsx
        ├── WindowSelector.tsx
        ├── Dashboard.tsx
        ├── SummaryCards.tsx
        ├── UsageLineChart.tsx
        ├── ProviderBarChart.tsx
        ├── CachePanel.tsx
        ├── CacheStatCards.tsx
        ├── CacheBarChart.tsx
        ├── ConfigPanel.tsx
        ├── ApiKeyForm.tsx
        └── CredentialsForm.tsx
```

---

## TypeScript Types (`src/types/api.ts`)

All API response shapes defined here. No inline type definitions in components.

```ts
export type TimeWindow = "1h" | "24h" | "7d" | "30d";

export interface StatsSummary {
  total_requests: number;
  total_tokens: number;
  prompt_tokens: number;
  completion_tokens: number;
}

export interface ProviderSummary {
  provider: string;
  model: string;
  request_count: number;
  prompt_tokens: number;
  completion_tokens: number;
}

export interface UsageDataPoint {
  bucket: string;
  total_tokens: number;
  prompt_tokens: number;
  completion_tokens: number;
}

export interface CacheStats {
  total_requests: number;
  cache_read_tokens: number;
  cache_creation_tokens: number;
  cache_hit_rate: number;
}

export interface ConfigResponse {
  api_key_masked: string;
}
```

---

## NFR Rule Traceability

| NFR Rule | Satisfied By |
|----------|-------------|
| NFR-U6-SEC-01 | `AuthContext.logout()` removes only `sessionStorage["llm_auth"]`; `login()` writes only to sessionStorage |
| NFR-U6-SEC-02 | `apikey-input` field type set to `"password"` in `ApiKeyForm` |
| NFR-U6-SEC-03 | `credentials-password-input` field type set to `"password"` in `CredentialsForm` |
| NFR-U6-SEC-04 | No `console.log` calls with creds; `ErrorBoundary.componentDidCatch` logs structure only |
| NFR-U6-REL-01 | `ErrorBoundary` at App level with reload fallback |
| NFR-U6-REL-02 | Three-state pattern (`loading`/`error`/data) in Dashboard, CachePanel, ConfigPanel |
| NFR-U6-REL-03 | `apiFetch` throws `UnauthorizedError` on 401; callers catch and call `logout()` |
| NFR-U6-PERF-01 | `vite.config.ts` sets `target: "es2020"` |
| NFR-U6-PERF-02 | All Recharts wrapped in `<ResponsiveContainer width="100%" height={300}>` |
| NFR-U6-PERF-03 | Fetches triggered only by user interaction (tab change, window selector, form submit) |
| NFR-U6-MAINT-01 | `tsconfig.json` has `"strict": true` |
| NFR-U6-MAINT-02 | All interactive elements have `data-testid` per BR-U6-14 naming scheme |
| NFR-U6-MAINT-03 | `src/types/api.ts` is sole location of API type definitions |
| NFR-U6-USE-01 | Submit buttons disabled when form invalid or save in progress |
| NFR-U6-USE-02 | Success/error messages inline, no `alert()` |
| NFR-U6-USE-03 | Active window button: `bg-blue-600 text-white`; inactive: `bg-gray-100` |
