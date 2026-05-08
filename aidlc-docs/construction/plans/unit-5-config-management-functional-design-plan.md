# Functional Design Plan — Unit 5: Config Management

## Unit Context
- **Purpose**: DB-backed config store, Basic Auth on `/api/*`, Argon2id credentials, admin bootstrap
- **Components**: C-10 ConfigService, C-11 AuthMiddleware, C-13 AdminBootstrapService
- **Database**: Migration `0001_config.sql` — `config` table
- **API endpoints**: GET /api/config, PUT /api/config, PUT /api/config/credentials
- **Depends on**: Unit 1 (AppState, DB pool, middleware scaffold, auth stub)

## Plan Checkboxes

- [x] Q1: Config table storage model → B (key-value store)
- [x] Q2: Admin credential update flow → B (no body re-auth; Basic Auth sufficient)
- [x] Q3: Brute-force protection → A (fixed 500ms delay per failure)
- [x] Q4: Admin users → B (multiple admin users, keyed by username)
- [x] Q5: API key masking → B (first 8 + last 4 chars visible, "" if unset)
- [x] Q6: Empty API key → B (reject with 422)
- [x] Analyze answers for ambiguities — none found
- [x] Generate business-logic-model.md
- [x] Generate business-rules.md
- [x] Generate domain-entities.md
