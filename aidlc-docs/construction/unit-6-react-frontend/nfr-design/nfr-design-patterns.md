# NFR Design Patterns — Unit 6: React Frontend

## Pattern 1: React Context for Auth State (NFR-U6-SEC-01, NFR-U6-REL-03)

**Problem**: Auth credentials must be readable by `apiFetch`, `AuthGate`, and any component that can trigger logout on 401. Prop drilling through MainLayout would couple unrelated components.

**Solution**: `AuthContext` with `{ creds: string | null, login: (creds: string) => void, logout: () => void }`

```
AuthProvider (wraps App tree)
  └── creds state in useState<string | null>
      login() → writes sessionStorage["llm_auth"] + sets state
      logout() → clears sessionStorage["llm_auth"] + sets state to null
```

**Enforcement points**:
- `apiFetch` imports `useAuth()` or receives `creds` as parameter — decided: `apiFetch(path, options, creds)` is a plain function that takes `creds` as an argument (not a hook) so it can be called from event handlers
- `useAuth()` hook exposes `{ creds, login, logout }` to components
- On 401: component calls `logout()` from context — clears storage and triggers re-render back to `AuthGate`

**Initialization**: On mount, `AuthProvider` reads `sessionStorage["llm_auth"]` as initial creds value. If present, user is already authenticated.

---

## Pattern 2: Resilient Fetch with One Automatic Retry (NFR-U6-REL-02)

**Problem**: Transient network blips should not immediately surface an error to the user. But retrying indefinitely or on 401 would cause problems.

**Solution**: `apiFetch` wraps native `fetch` with a single automatic retry on non-401 failure.

```
apiFetch(path, options, creds):
  1. attempt = await fetch(path, { headers: { Authorization: "Basic " + creds }, ...options })
  2. if response.ok → return response
  3. if response.status === 401 → logout() + return (caller handles gracefully)
  4. if non-401 error (network error OR non-ok status):
     - wait 1000ms
     - retry = await fetch(...) [same request]
     - if retry.ok → return retry
     - else throw Error with status/message from retry
```

**Retry scope**: Applies to all API calls except explicit logout/auth flows. The 1-second delay is a fixed constant — no exponential backoff for a single retry.

**Error propagation**: After retry exhaustion, `apiFetch` throws. Caller catches in `try/catch` inside `useEffect` and sets `error` state for display.

---

## Pattern 3: Global ErrorBoundary (NFR-U6-REL-01)

**Problem**: React render errors propagate up and crash the whole tree, leaving a blank screen.

**Solution**: One `ErrorBoundary` class component at App level, below `<AuthProvider>` but above `<AuthGate>` and `<MainLayout>`.

```tsx
class ErrorBoundary extends React.Component<{children}, {hasError: boolean, error: Error | null}> {
  static getDerivedStateFromError(error) { return { hasError: true, error } }
  componentDidCatch(error, info) { /* no console.log of secrets — log structure only */ }
  render() {
    if (this.state.hasError) return <ErrorFallback error={this.state.error} />
    return this.props.children
  }
}
```

**Fallback UI**: Full-page centered message with the error text and a "Reload" button calling `window.location.reload()`.

**Scope**: Catches render-phase errors only. Async errors (fetch failures) are handled per-component via `try/catch` in `useEffect`.

---

## Pattern 4: Tailwind CDN + Vite Coexistence (NFR-U6-PERF-01)

**Problem**: Tailwind CSS Play CDN scans the DOM at runtime for class names. Vite's build process does not need to know about Tailwind. No PostCSS config needed.

**Solution**: Load CDN via `<script>` tag in `index.html`. Vite builds the JS/TS bundle. The CDN script runs in the browser after load.

```html
<!-- index.html -->
<script src="https://cdn.tailwindcss.com"></script>
```

**Constraint**: Class names must appear as complete strings (not dynamically constructed) so the CDN scanner can detect them. For dynamic active/inactive states, use a conditional ternary with both full class strings.

```tsx
// Correct — CDN sees both class strings
className={isActive ? "bg-blue-600 text-white" : "bg-gray-100 text-gray-700"}
```

---

## Pattern 5: Three-State Data Loading (NFR-U6-REL-02)

**Problem**: Every async fetch must handle loading, error, and success states without crashes.

**Solution**: Standard state shape per data-fetching component:

```tsx
const [data, setData] = useState<T | null>(null);
const [loading, setLoading] = useState(false);
const [error, setError] = useState<string | null>(null);
```

**State transitions**:
```
idle → loading (on fetch start)
loading → success (setData, setLoading(false))
loading → error (setError(msg), setLoading(false))
```

**Empty data**: `data !== null` but empty array/zero values — renders empty-state message, not a spinner or error banner. This is the success path.
