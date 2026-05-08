# Unit 6: React Frontend — Functional Design Questions

Please answer each question by filling in the letter choice after the `[Answer]:` tag.
If none of the options match your needs, choose the last option (Other) and describe your preference.
Let me know when you're done.

---

## Question 1
The dashboard needs to render token usage charts (by provider/model, over time). A chart library is required.

Which charting approach should be used?

A) **Recharts** — pure-React SVG charting library. Simple API, TypeScript types included, no extra build config. Line chart for time-series, bar chart for provider breakdown.

B) **Plain HTML tables** — no chart library at all. Data displayed as sorted tables with totals. Zero dependency weight; fully readable without interaction.

C) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 2
The UI needs styling. For a personal/team admin tool, a lightweight approach is appropriate.

Which styling approach should be used?

A) **Tailwind CSS** — utility-first CSS via CDN (no build step needed for Tailwind itself). Fast to compose, no custom CSS files needed.

B) **Plain CSS module per component** — one `.module.css` file per component. No framework dependency; explicit control over every style.

C) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 3
The UI needs Basic Auth credentials (username + password) to call `/api/*` endpoints. How long should credentials persist in the browser?

A) **sessionStorage** — credentials cleared when the browser tab is closed. Appropriate for an admin tool; user re-enters on each session.

B) **localStorage** — credentials persist across browser restarts. More convenient; acceptable since this is a local/internal tool, not public-facing.

C) Other (please describe after [Answer]: tag below)

[Answer]: A

---

## Question 4
Unit 6 includes docker-compose, .env.example, and deployment documentation alongside the React SPA. Should all deliverables be implemented in this session?

A) Yes — implement everything: React SPA, `npm run build` integration, `docker-compose.yml`, `.env.example`. Complete the full Unit 6 scope.

B) SPA only for now — implement the React SPA and build integration. Defer docker-compose and .env.example to a follow-up (they don't block frontend functionality).

C) Other (please describe after [Answer]: tag below)

[Answer]: A
