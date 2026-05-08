# NFR Requirements — Unit 6: React Frontend

## Performance

**NFR-U6-PERF-01**: Vite build target is `es2020`. No polyfills. Modern syntax (optional chaining, nullish coalescing, async/await) used freely. Bundle must be tree-shaken by Vite's default Rollup config.

**NFR-U6-PERF-02**: Recharts components wrapped in `<ResponsiveContainer>` with explicit `width="100%"` and `height={300}`. No fixed pixel widths that cause layout overflow on smaller viewports.

**NFR-U6-PERF-03**: All API fetches are triggered by user interaction (tab change, window change, form submit) — no polling. No unnecessary re-fetches on unrelated state changes.

---

## Reliability

**NFR-U6-REL-01**: A single global `ErrorBoundary` class component wraps the entire app below `<AuthGate>`. Any uncaught render error shows a full-page fallback: error message + "Reload" button that calls `window.location.reload()`.

**NFR-U6-REL-02**: Every async data fetch has three explicit UI states: loading (spinner), error (message with the error text), and success (rendered data). Empty data is a valid success state and must be handled with an appropriate empty-state message rather than a crash.

**NFR-U6-REL-03**: Any 401 response from `apiFetch` MUST clear `sessionStorage["llm_auth"]` and reload the page, returning the user to `<AuthGate>`. This applies to all API calls, including background fetches.

---

## Security

**NFR-U6-SEC-01**: Credentials stored only in `sessionStorage` — cleared automatically when the browser tab is closed. Never stored in `localStorage`, cookies, or any other persistent mechanism.

**NFR-U6-SEC-02**: The API key input field (`apikey-input`) MUST have `type="password"` to prevent shoulder-surfing when entering a new key. The displayed masked value from the server is shown as placeholder text, not a pre-filled value.

**NFR-U6-SEC-03**: The credentials password input (`credentials-password-input`) MUST have `type="password"`.

**NFR-U6-SEC-04**: No secrets (API keys, passwords) are logged to the browser console at any point.

---

## Maintainability

**NFR-U6-MAINT-01**: `tsconfig.json` enables `strict: true`. All TypeScript errors must be resolved before `npm run build` succeeds. No `// @ts-ignore` or `as any` casts without an explanatory comment.

**NFR-U6-MAINT-02**: All interactive elements have stable `data-testid` attributes per the naming scheme in BR-U6-14. These must not change unless the element's purpose changes.

**NFR-U6-MAINT-03**: API response types in `src/types/api.ts` are the single source of truth for TypeScript shapes. Components import from there, not from inline type definitions.

---

## Usability

**NFR-U6-USE-01**: Submit buttons are disabled (`disabled` attribute) when their form is invalid (empty required fields) or while a save is in progress. This prevents double-submission.

**NFR-U6-USE-02**: Success and error messages are displayed inline near the relevant form or chart section, not as browser `alert()` dialogs.

**NFR-U6-USE-03**: The window selector (1h/24h/7d/30d) clearly indicates the active selection via a distinct visual state (Tailwind `bg-blue-600 text-white` on active, `bg-gray-100` on inactive).
