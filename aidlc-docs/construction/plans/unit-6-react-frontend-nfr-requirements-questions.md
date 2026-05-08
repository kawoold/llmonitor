# Unit 6: React Frontend — NFR Requirements Questions

Please answer each question by filling in the letter choice after the `[Answer]:` tag.
If none of the options match your needs, choose the last option (Other) and describe your preference.
Let me know when you're done.

---

## Question 1
TypeScript compilation strictness affects how many type errors the compiler catches at build time.

Which TypeScript strictness level should be used?

A) `strict: true` in tsconfig — enables all strict checks (`strictNullChecks`, `noImplicitAny`, etc.). More upfront ceremony but catches real bugs early. Recommended for greenfield projects.

B) Relaxed (`strict: false`) — fewer compiler complaints, faster initial development. Allows implicit `any` and optional null checks.

C) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 2
The Vite build targets specific browser versions, affecting whether modern JavaScript syntax is transpiled and whether polyfills are included.

Which browser target should Vite use?

A) Modern only — `target: 'es2020'` in vite.config.ts. Targets last 2 versions of Chrome, Firefox, and Safari. No polyfills, smaller bundle, cleaner syntax. Appropriate for a local admin tool where the operator controls the browser.

B) Broader support — `target: 'es2015'` (Vite default). Transpiles to wider compatibility. Slightly larger bundle. Appropriate if the tool may be accessed from older browsers.

C) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 3
React error boundaries catch rendering errors and show a fallback UI instead of a blank screen.

Which error boundary scope should be used?

A) Global boundary at App level — one `<ErrorBoundary>` wrapping the entire app. Simple implementation. Any uncaught render error shows a full-page fallback with an error message and a reload button.

B) Per-section boundaries — Dashboard, CachePanel, and ConfigPanel each have their own `<ErrorBoundary>`. If one section fails, the others remain usable and the nav tabs still function.

C) Other (please describe after [Answer]: tag below)

[Answer]: A
