# Unit 6: React Frontend — Code Generation Plan

## Unit Context

**Unit**: Unit 6 — React Frontend  
**Dependencies**: Units 1, 5 (proxy routes and config API must exist); Units 3, 4 (tracking and stats API must exist)  
**Application Code Root**: `/home/kawoold/Development/rust/llmonitor/frontend/`  
**Build Output**: `frontend/dist/` (embedded by rust-embed in `src/proxy/frontend.rs`)  
**Documentation**: `aidlc-docs/construction/unit-6-react-frontend/code/`

## Key Design Decisions

- React 18 + TypeScript strict mode, Vite 5, Recharts, Tailwind CDN (no PostCSS)
- Auth state via `AuthContext` — `useAuth()` hook, sessionStorage only
- `apiFetch` — plain async function, one automatic retry (1s) on non-401 failure
- Global `ErrorBoundary` class component at App level
- `es2020` build target, no polyfills
- All API types in `src/types/api.ts` — components import from there only
- All interactive elements have `data-testid` attributes

## Steps

### Step 1: Scaffold Vite + React + TypeScript Project
- [x] Create `frontend/` directory at workspace root
- [x] Write `frontend/package.json` with dependencies: `react`, `react-dom`, `recharts`; devDependencies: `vite`, `@vitejs/plugin-react`, `typescript`, `@types/react`, `@types/react-dom`
- [x] Write `frontend/index.html` with Tailwind CDN `<script>` tag, meta charset/viewport, root div, and Vite entry point script

### Step 2: Configure TypeScript
- [x] Write `frontend/tsconfig.json` with `strict: true`, `target: "ES2020"`, `lib: ["ES2020","DOM","DOM.Iterable"]`, `jsx: "react-jsx"`, `module: "ESNext"`, `moduleResolution: "bundler"`, `noEmit: true`
- [x] Write `frontend/tsconfig.node.json` for Vite config file

### Step 3: Configure Vite
- [x] Write `frontend/vite.config.ts` with `@vitejs/plugin-react`, `build.outDir: "../frontend/dist"`, `build.target: "es2020"`, `base: "/"`

### Step 4: API Types (`src/types/api.ts`)
- [x] Write `frontend/src/types/api.ts` with all API response interfaces:
  - `TimeWindow` type union (`"1h" | "24h" | "7d" | "30d"`)
  - `StatsSummary`, `ProviderSummary`, `UsageDataPoint`
  - `CacheStats`, `ConfigResponse`

### Step 5: Auth Context (`src/context/AuthContext.tsx`)
- [x] Write `frontend/src/context/AuthContext.tsx`:
  - `AuthContextValue` interface: `{ creds: string | null, login: (creds: string) => void, logout: () => void }`
  - `AuthProvider` reads `sessionStorage["llm_auth"]` as initial state
  - `login()` writes sessionStorage + sets state
  - `logout()` removes sessionStorage key + sets state to null
  - `useAuth()` hook export

### Step 6: apiFetch (`src/lib/apiFetch.ts`)
- [x] Write `frontend/src/lib/apiFetch.ts`:
  - `UnauthorizedError` class
  - `apiFetch(path: string, creds: string, options?: RequestInit): Promise<Response>`
  - Attaches `Authorization: "Basic " + creds` header
  - On 401: throws `UnauthorizedError` (no retry)
  - On network error or non-ok non-401: waits 1000ms, retries once, throws on second failure

### Step 7: ErrorBoundary (`src/components/ErrorBoundary.tsx`)
- [x] Write `frontend/src/components/ErrorBoundary.tsx`:
  - Class component implementing `getDerivedStateFromError` and `componentDidCatch`
  - Fallback: full-page centered error message + "Reload" button calling `window.location.reload()`

### Step 8: AuthGate (`src/components/AuthGate.tsx`)
- [x] Write `frontend/src/components/AuthGate.tsx`:
  - Controlled inputs for username and password (`type="password"` for password — NFR-U6-SEC-03)
  - Submit calls `GET /api/config` with `btoa("user:pass")` credentials
  - 200 → calls `login(creds)`; 401 → shows inline error
  - Submit button disabled while loading
  - `data-testid`: `auth-username-input`, `auth-password-input`, `auth-submit-button`

### Step 9: NavTabs (`src/components/NavTabs.tsx`)
- [x] Write `frontend/src/components/NavTabs.tsx`:
  - `Tab` type: `"dashboard" | "cache" | "config"`
  - Props: `activeTab: Tab`, `onTabChange: (tab: Tab) => void`
  - Three tab buttons with Tailwind active/inactive styles
  - `data-testid`: `nav-dashboard`, `nav-cache`, `nav-config`

### Step 10: WindowSelector (`src/components/WindowSelector.tsx`)
- [x] Write `frontend/src/components/WindowSelector.tsx`:
  - Props: `value: TimeWindow`, `onChange: (w: TimeWindow) => void`
  - Four buttons: `1h`, `24h`, `7d`, `30d`
  - Active: `bg-blue-600 text-white`; inactive: `bg-gray-100 text-gray-700`
  - `data-testid`: `window-selector-1h`, `window-selector-24h`, `window-selector-7d`, `window-selector-30d`

### Step 11: SummaryCards (`src/components/SummaryCards.tsx`)
- [x] Write `frontend/src/components/SummaryCards.tsx`:
  - Props: `summary: StatsSummary`
  - Four stat cards: Total Requests, Total Tokens, Prompt Tokens, Completion Tokens

### Step 12: UsageLineChart (`src/components/UsageLineChart.tsx`)
- [x] Write `frontend/src/components/UsageLineChart.tsx`:
  - Props: `data: UsageDataPoint[]`
  - Recharts `<ResponsiveContainer width="100%" height={300}>` wrapping `<LineChart>`
  - Lines: `total_tokens`, `prompt_tokens`, `completion_tokens`
  - Empty state: "No data for this window" centered in chart area

### Step 13: ProviderBarChart (`src/components/ProviderBarChart.tsx`)
- [x] Write `frontend/src/components/ProviderBarChart.tsx`:
  - Props: `data: ProviderSummary[]`
  - Recharts `<ResponsiveContainer>` wrapping `<BarChart>`
  - X-axis: `provider/model` label; bars: `prompt_tokens` and `completion_tokens` grouped
  - Empty state: "No data" text

### Step 14: Dashboard (`src/components/Dashboard.tsx`)
- [x] Write `frontend/src/components/Dashboard.tsx`:
  - State: `window: TimeWindow`, `summary: StatsSummary | null`, `timeseries: UsageDataPoint[]`, `providers: ProviderSummary[]`, `loading: boolean`, `error: string | null`
  - `useEffect` fetches `/api/stats/summary`, `/api/stats/usage`, `/api/stats/providers` on window change
  - Uses `apiFetch` — catches `UnauthorizedError` → calls `logout()`
  - Loading: spinner; Error: red banner with message; Success: renders SummaryCards + charts
  - Empty data handled as valid success state

### Step 15: CacheStatCards (`src/components/CacheStatCards.tsx`)
- [x] Write `frontend/src/components/CacheStatCards.tsx`:
  - Props: `data: CacheStats`
  - Four cards: Hit Rate (`(data.cache_hit_rate * 100).toFixed(1) + "%"`), Read Tokens, Creation Tokens, Cached Requests

### Step 16: CacheBarChart (`src/components/CacheBarChart.tsx`)
- [x] Write `frontend/src/components/CacheBarChart.tsx`:
  - Props: `data: CacheStats`
  - Recharts `<ResponsiveContainer>` wrapping `<BarChart>` with two bars: "Read" and "Creation"

### Step 17: CachePanel (`src/components/CachePanel.tsx`)
- [x] Write `frontend/src/components/CachePanel.tsx`:
  - State: `window: TimeWindow`, `cacheData: CacheStats | null`, `loading: boolean`, `error: string | null`
  - Fetches `/api/stats/cache?window=...` on window change
  - Empty state (`total_requests === 0`): "No cache activity in this window"
  - Loading/error states same as Dashboard

### Step 18: ApiKeyForm (`src/components/ApiKeyForm.tsx`)
- [x] Write `frontend/src/components/ApiKeyForm.tsx`:
  - Props: `currentMaskedKey: string`, `onSave: (key: string) => Promise<void>`
  - Input `type="password"` (NFR-U6-SEC-02), placeholder shows masked key hint
  - Submit disabled if input empty or saving in progress (NFR-U6-USE-01)
  - Shows success banner for 3s after save (via `setTimeout` clearing `apiKeySuccess`)
  - `data-testid`: `apikey-input`, `apikey-submit-button`

### Step 19: CredentialsForm (`src/components/CredentialsForm.tsx`)
- [x] Write `frontend/src/components/CredentialsForm.tsx`:
  - Both fields required; password input `type="password"` (NFR-U6-SEC-03)
  - Submit disabled if either field empty or saving
  - On success: shows "Credentials updated. Logging out..." then `logout()` + `window.location.reload()`
  - `data-testid`: `credentials-username-input`, `credentials-password-input`, `credentials-submit-button`

### Step 20: ConfigPanel (`src/components/ConfigPanel.tsx`)
- [x] Write `frontend/src/components/ConfigPanel.tsx`:
  - State: `config: ConfigResponse | null`, form state for api key and credentials
  - Fetches `/api/config` on mount to populate `currentMaskedKey`
  - Renders `ApiKeyForm` and `CredentialsForm`
  - Handles save: calls `PUT /api/config` (api key) or `POST /api/credentials` via `apiFetch`

### Step 21: MainLayout (`src/components/MainLayout.tsx`)
- [x] Write `frontend/src/components/MainLayout.tsx`:
  - Props: none (reads from `useAuth()`)
  - State: `activeTab: Tab`
  - Renders `NavTabs` + active panel (`Dashboard`, `CachePanel`, or `ConfigPanel`)
  - Header with app title and logout button (calls `logout()` from `useAuth()`)

### Step 22: App (`src/App.tsx`)
- [x] Write `frontend/src/App.tsx`:
  - Wraps tree in `AuthProvider` → `ErrorBoundary`
  - If `creds !== null`: renders `<MainLayout />`; else renders `<AuthGate />`

### Step 23: Entry Point (`src/main.tsx`)
- [x] Write `frontend/src/main.tsx`:
  - `ReactDOM.createRoot(document.getElementById("root")!).render(<React.StrictMode><App /></React.StrictMode>)`

### Step 24: docker-compose.yml
- [x] Write `docker-compose.yml` at workspace root:
  - Version `"3.8"`, single service `llmonitor`
  - Named volume for SQLite persistence
  - `env_file: .env` and `ports: ["3000:3000"]`

### Step 25: .env.example
- [x] Write `.env.example` at workspace root:
  - Documents 4 env vars: `ANTHROPIC_API_KEY`, `ADMIN_USERNAME`, `ADMIN_PASSWORD`, `DATABASE_URL` (with examples and descriptions)

### Step 26: Code Summary Documentation
- [x] Write `aidlc-docs/construction/unit-6-react-frontend/code/code-summary.md` with file list and brief descriptions

## Story Coverage

| Business Rule | Implemented In |
|--------------|----------------|
| BR-U6-01 (Login flow) | AuthGate, AuthContext, App |
| BR-U6-02 (401 logout) | apiFetch, all panels |
| BR-U6-03 (sessionStorage only) | AuthContext |
| BR-U6-04 (Dashboard data) | Dashboard |
| BR-U6-05 (Window selector) | WindowSelector, Dashboard, CachePanel |
| BR-U6-06 (Line chart) | UsageLineChart |
| BR-U6-07 (Bar chart) | ProviderBarChart |
| BR-U6-08 (Cache panel) | CachePanel, CacheStatCards, CacheBarChart |
| BR-U6-09 (Config fetch) | ConfigPanel |
| BR-U6-10 (API key save) | ApiKeyForm, ConfigPanel |
| BR-U6-11 (Credentials save + logout) | CredentialsForm |
| BR-U6-12 (One retry) | apiFetch |
| BR-U6-13 (ErrorBoundary) | ErrorBoundary, App |
| BR-U6-14 (data-testid) | All interactive components |
| BR-U6-15 (Build integration) | vite.config.ts |
| BR-U6-16 (docker-compose) | docker-compose.yml |
| BR-U6-17 (.env.example) | .env.example |
| BR-U6-18 (Tailwind CDN) | index.html |
