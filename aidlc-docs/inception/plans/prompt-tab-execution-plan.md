# Execution Plan — Prompt Tab Feature

## Change Impact Assessment

| Area | Impact | Detail |
|------|--------|--------|
| User-facing | Yes | New Prompt tab visible in NavTabs |
| Structural | Minimal | New component files + one new Rust handler |
| Data model | No | No schema changes |
| API | Yes | New `GET /api/models` management endpoint |
| NFR | Minimal | `react-markdown` added to frontend deps; streaming SSE read pattern |

## Risk Assessment

- **Risk Level**: Low-Medium
- **Rollback**: Easy — new tab is additive; existing tabs unaffected
- **Testing Complexity**: Moderate — streaming behavior requires careful handling

## Recommended Execution Plan

### 🔵 INCEPTION PHASE

| Stage | Decision | Rationale |
|-------|----------|-----------|
| Workspace Detection | COMPLETED | Existing llmonitor project |
| Reverse Engineering | SKIPPED | Not needed for additive feature on known codebase |
| Requirements Analysis | COMPLETED | Questions answered, requirements document created |
| User Stories | SKIP | Single-developer technical feature, no multi-persona UX |
| Workflow Planning | IN PROGRESS | This document |
| Application Design | SKIP | Component structure is clear from requirements |
| Units Generation | SKIP | Single unit of work |

### 🟢 CONSTRUCTION PHASE — Unit 7: Prompt Tab

| Stage | Decision | Rationale |
|-------|----------|-----------|
| Functional Design | **EXECUTE** | Streaming SSE parsing, multi-turn state, and models endpoint contract need detailed design before coding |
| NFR Requirements | SKIP | No new tech stack decisions; `react-markdown` is a straightforward dep addition |
| NFR Design | SKIP | No new NFR patterns; existing auth, error, and loading patterns apply unchanged |
| Infrastructure Design | SKIP | No infrastructure changes |
| Code Generation | **EXECUTE** | Implementation |
| Build and Test | **EXECUTE** | Verification |

## Unit 7: Prompt Tab — Scope

**Backend (Rust)**:
- New handler: `GET /api/models` → returns hardcoded model list
- Wire into management router (auth-protected)

**Frontend (React/TypeScript)**:
- `frontend/src/types/api.ts` — add `ModelOption` type
- `frontend/src/components/PromptTab.tsx` — main component
- `frontend/src/components/MessageBubble.tsx` — renders a single message (user or assistant, with markdown)
- Update `frontend/src/components/NavTabs.tsx` — add "prompt" tab
- Update `frontend/src/components/MainLayout.tsx` — render `<PromptTab />`
- Update `frontend/package.json` — add `react-markdown`

## Success Criteria

- Prompt tab appears in the nav alongside Dashboard, Cache, Config
- Model selector populates from `/api/models`
- Multi-turn conversation works: each message adds to history
- Streaming response renders tokens progressively
- Assistant messages render markdown
- Temperature and max_tokens controls work
- Clear button resets conversation
- 401 triggers logout; other errors shown inline
