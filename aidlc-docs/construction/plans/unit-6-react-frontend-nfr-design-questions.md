# Unit 6: React Frontend — NFR Design Questions

Please answer each question by filling in the letter choice after the `[Answer]:` tag.
If none of the options match your needs, choose the last option (Other) and describe your preference.
Let me know when you're done.

---

## Question 1
Auth state (the base64 credentials string) needs to be accessible in multiple components: `App`, `apiFetch`, and the logout trigger in any component that receives a 401. How should auth state be shared?

A) **React Context** — a small `AuthContext` provides `{ creds, login, logout }` to the whole tree. Components call `useAuth()` instead of reading `sessionStorage` directly. The logout function clears sessionStorage and triggers a re-render cleanly.

B) **Prop drilling from App** — `App` holds `creds` in `useState`, passes `onLogout` down through `MainLayout` to individual components. Simpler — no context boilerplate — acceptable for a shallow 3-level tree.

C) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 2
When an API fetch fails (network error, 500 response), the error state is shown with a message. Should the UI offer automatic retry?

A) **Manual retry only** — show an error message and a "Retry" button that re-triggers the fetch. No automatic retries. Simple and predictable; the user decides when to try again.

B) **One automatic retry** — on any non-401 fetch failure, wait 1 second and retry once automatically before showing the error state. Handles transient errors transparently.

C) Other (please describe after [Answer]: tag below)

[Answer]: B
