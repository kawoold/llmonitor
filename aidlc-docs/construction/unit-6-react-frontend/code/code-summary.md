# Code Summary — Unit 6: React Frontend

## Application Files Created

### Project Config
| File | Purpose |
|------|---------|
| `frontend/package.json` | npm dependencies and build scripts |
| `frontend/index.html` | HTML entry point with Tailwind CDN script |
| `frontend/tsconfig.json` | TypeScript config — strict mode, ES2020, react-jsx |
| `frontend/tsconfig.node.json` | TypeScript config for vite.config.ts |
| `frontend/vite.config.ts` | Vite build — outDir: frontend/dist, target: es2020 |

### Source Files
| File | Purpose |
|------|---------|
| `frontend/src/main.tsx` | ReactDOM entry point |
| `frontend/src/App.tsx` | Root: AuthProvider → ErrorBoundary → AuthGate or MainLayout |
| `frontend/src/types/api.ts` | All API response TypeScript interfaces |
| `frontend/src/context/AuthContext.tsx` | Auth state — AuthProvider, useAuth hook |
| `frontend/src/lib/apiFetch.ts` | HTTP wrapper — auth header, 401 throw, 1s retry |
| `frontend/src/components/ErrorBoundary.tsx` | Global error catch with reload fallback |
| `frontend/src/components/AuthGate.tsx` | Login form — tests GET /api/config |
| `frontend/src/components/NavTabs.tsx` | Dashboard / Cache / Config tab navigation |
| `frontend/src/components/WindowSelector.tsx` | 1h / 24h / 7d / 30d time window buttons |
| `frontend/src/components/SummaryCards.tsx` | Four stat cards for requests and token counts |
| `frontend/src/components/UsageLineChart.tsx` | Recharts LineChart for token usage over time |
| `frontend/src/components/ProviderBarChart.tsx` | Recharts BarChart — tokens by provider/model |
| `frontend/src/components/Dashboard.tsx` | Fetches summary + timeseries + providers |
| `frontend/src/components/CacheStatCards.tsx` | Four cache metric stat cards |
| `frontend/src/components/CacheBarChart.tsx` | Recharts BarChart — read vs creation tokens |
| `frontend/src/components/CachePanel.tsx` | Fetches cache stats, renders cards + chart |
| `frontend/src/components/ApiKeyForm.tsx` | API key update form — type="password", 3s success |
| `frontend/src/components/CredentialsForm.tsx` | Admin credentials form — logout on success |
| `frontend/src/components/ConfigPanel.tsx` | Fetches config, renders ApiKeyForm + CredentialsForm |
| `frontend/src/components/MainLayout.tsx` | Header + tab routing |

### Infrastructure Files
| File | Purpose |
|------|---------|
| `docker-compose.yml` | Single-service compose with named SQLite volume |
| `.env.example` | Documents ANTHROPIC_API_KEY, ADMIN_USERNAME, ADMIN_PASSWORD, DATABASE_URL |

## NFR Compliance

| NFR | Status |
|-----|--------|
| SEC-01 (sessionStorage only) | AuthContext writes/reads sessionStorage["llm_auth"] exclusively |
| SEC-02 (apikey type=password) | ApiKeyForm input has type="password" |
| SEC-03 (credentials type=password) | CredentialsForm password input has type="password" |
| SEC-04 (no console.log of secrets) | ErrorBoundary logs no error details; no console.log anywhere |
| REL-01 (ErrorBoundary) | ErrorBoundary wraps full tree; shows error + Reload button |
| REL-02 (3-state loading) | Dashboard, CachePanel, ConfigPanel all have loading/error/success states |
| REL-03 (401 → logout) | apiFetch throws UnauthorizedError; all panels call logout() on catch |
| PERF-01 (es2020) | vite.config.ts build.target = "es2020" |
| PERF-02 (ResponsiveContainer) | All Recharts charts wrapped in ResponsiveContainer width="100%" height={300} |
| PERF-03 (no polling) | All fetches triggered by user interaction only |
| MAINT-01 (strict: true) | tsconfig.json strict: true + noUnusedLocals + noUnusedParameters |
| MAINT-02 (data-testid) | All interactive elements have data-testid attributes |
| MAINT-03 (types/api.ts) | All API interfaces defined in src/types/api.ts; no inline types |
| USE-01 (disabled submit) | All submit buttons disabled when form invalid or saving |
| USE-02 (inline messages) | All success/error messages inline; no alert() calls |
| USE-03 (window selector style) | Active: bg-blue-600 text-white; inactive: bg-gray-100 text-gray-700 |
