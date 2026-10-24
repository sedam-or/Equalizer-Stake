## What & why
<!-- One or two sentences. Link the issue. -->

Task / spec: <!-- e.g. T-212, SCH-03 -->

## Checklist
- [ ] Task file updated per `WORKFLOW.md`: Report filled (AC → tests, checks run, manual checks, security, rollback plan, dependencies affected), `status: review`, `tasks/INDEX.md` regenerated
- [ ] Tests added or updated (unit / integration / API / e2e as relevant)
- [ ] No hard-coded business values (POL-02). New rules are policy fields with defaults and `en`/`id` labels
- [ ] OpenAPI and TS client regenerated if the API changed (`make api`)
- [ ] Migrations are forward-only, and `sqlx prepare` was run
- [ ] Docs updated for user-visible changes
- [ ] Follows `spec/16-code-quality.md`; any `#[expect]` / `biome-ignore` has a reason; PR ≤ ~400 changed lines (excl. generated)
- [ ] Security: new fields classified (SBD-10). If this touches auth, tenancy, PII, files, exports, money or external systems, the `security-review` label is set and the abuse-case tests are added (`spec/18-security-by-design.md`)
- [ ] Surgical diff (`CLAUDE.md`): every changed line traces to the linked task or issue; no drive-by refactors or reformatting; assumptions stated above
- [ ] Commits signed off (`-s`), Conventional Commit messages
