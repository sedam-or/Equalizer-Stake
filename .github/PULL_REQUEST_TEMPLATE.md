## What & why
<!-- One or two sentences. Link the issue. -->

Task / spec: <!-- e.g. T-212, SCH-03 -->

## Checklist
- [ ] Tests added or updated (unit / integration / API / e2e as relevant)
- [ ] No hard-coded business values (POL-02). New rules are policy fields with defaults and `en`/`id` labels
- [ ] OpenAPI and TS client regenerated if the API changed (`make api`)
- [ ] Migrations are forward-only, and `sqlx prepare` was run
- [ ] Docs updated for user-visible changes
- [ ] Commits signed off (`-s`), Conventional Commit messages
