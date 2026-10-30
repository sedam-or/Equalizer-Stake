---
id: T-113
title: Add CorePolicy, the settings API (policies and modules), meta endpoints and the audit read API
milestone: M1
status: review
depends_on: [T-104, T-105, T-106, T-109h, T-112b]
spec_refs: [POL-06, POL-07, POL-10, POL-12, MOD-06, MOD-09, API-14, AUD-07, PRD-06, STM-12, STM-13, VAL-23, AUTH-01, AUTH-04]
estimated_size: M
---

## Goal
The owner can read, preview and save any enabled module's rules, toggle modules, and see the live summary. The SPA can discover modules, navigation and tenant branding from meta endpoints. Owners and admins can read the audit log within their category scope.

## Context
- `CorePolicy` (04 §6.1), in `eq_core::policy::core_policy`, MODULE Core:
```rust
pub struct CorePolicy {
  pub locale: Locale,                 // E, default en
  pub currency: Currency,             // E, default USD
  pub timezone: TzName,               // E, IANA, default "UTC" (newtype over chrono_tz::Tz, serde as string, schema enum of all tz names)
  pub week_starts_on: Weekday,        // A, Mon|Sun only (validate), default Mon
  pub terminology: Terminology,       // A, { overrides: BTreeMap<TermKey, I18nText> }, default {}
  pub privacy: PrivacyPolicy,         // A
  pub branding: Branding,             // A, x-ui hidden in setup (group advanced, widget "hidden" — the SPA edits it on the Branding page)
}
pub struct Branding { pub brand_hue: u16 /*0..=359, 255*/, pub corner_style: CornerStyle /*sharp|default|round, default*/, pub logo_file_id: Option<Uuid> /*None*/ }
pub enum TermKey { Instructor, Guardian, Student, Offering, Session, Resource, Location, Enrollment }
pub struct PrivacyPolicy { pub deletion_sla_business_days: u16 /*1..=30, 14*/, pub block_deletion_with_open_invoices: bool /*true*/, pub photo_retention_days: u16 /*7..=3650, 365*/ }
```
  - POL-12: changing `currency` when any invoice exists → `PolicyViolation{rule:"core.currency.locked"}`. Implement this through a port `CurrencyLockCheck` in `eq_core::ports`, with a default impl returning "unlocked". Billing registers the real one in M4.
  - Saving CorePolicy also updates the `tenants.locale/currency/timezone` columns (denormalised for tenant resolution, T-104) in the same tx.
  - Summary lines: `policy.core.summary.locale_currency {locale, currency}`, `policy.core.summary.timezone {timezone}`.
- Settings API (04 §7):

| Method | Path | Perm | Body / Result |
|---|---|---|---|
| GET | `/settings/policies` | core.policy.read | `[{module, version, effective_from, schema, document, summary, pending_future?: {version, effective_from}}]` for enabled modules (core, identity, audit always) |
| GET | `/settings/policies/{module}` | core.policy.read | the above + `history: [{version, effective_from, created_by: {id, display_name}, note}]` |
| GET | `/settings/policies/{module}/versions/{v}` | core.policy.read | the full stored document |
| POST | `/settings/policies/{module}/preview` | core.policy.write | `{document}` → `{valid, errors: [FieldError], summary: [SummaryLine]}` (no write) |
| PUT | `/settings/policies/{module}` | core.policy.write | `{document, effective_from?, note?}` + `If-Match: "v<current>"` → Versioned (runs `validate_policy` then `validate_policy_in_context`); 409 `version_conflict`, 428, 422 `validation`/`policy_violation` |
| GET | `/settings/modules` | core.modules.read | same shape as `/meta/modules` `modules[]` |
| PUT | `/settings/modules` | core.modules.write | `{enabled: [ModuleId]}` (the full set of optional modules wanted on) → `/meta/modules` body; 422 `module_dependency {missing: {module: [..]}}`, 422 `module_required_by {required_by: [..]}` |

  - `effective_from` must be ≥ now − 1 min, else 422 `validation` at `/effective_from`. Absent means now.
  - A disabled module's policy endpoints → 404 `module_disabled`.
- Meta (API-14, MOD-09):
  - `GET /meta/modules` (any authenticated) → `{modules: [{id, compiled, enabled, always_on, requires_all, requires_any, has_data, policy_version}], nav: [NavItem filtered by perms + enabled]}`.
  - `has_data` comes from `Module::has_data(tx, tenant)` (trait method from T-105).
  - `compiled:false` entries are listed for all `ModuleId`s not in the registry.
  - `GET /meta/app` (public, tenant-resolved, or unprovisioned) → `{version, commit, mode, brand: {name, logo_url?}, tenant?: {name, locale, currency, timezone, terminology}, features: {captcha: false, guests: false}, setup_required: bool}`. `brand.name` comes from `eq_core::brand::NAME`; hue, corner style and logo come from `CorePolicy.branding` (logo URL `/api/v1/files/{id}`, public within the tenant per SEC-15; until T-303 exists, return `logo_url: null`).
- Permissions declared by core:

| key | roles |
|---|---|
| core.policy.read | owner, admin |
| core.policy.write | owner |
| core.modules.read | owner, admin |
| core.modules.write | owner |

- Audit read API (AUD-07):
  - `GET /audit/entries?from&to&actor_id&action(prefix)&entity_type&entity_id&category&limit&cursor` (perm `audit.read`: owner, admin).
  - **Admins are filtered to `AuditPolicy.admin_categories` in the query layer.** The owner sees all.
  - `GET /audit/entries/{id}`. `GET /audit/export?...` → `text/csv` streamed; the export itself is audited (`audit.export`, compliance). `GET /audit/verify?from_seq&to_seq` (owner only, via perm `audit.verify`: owner).
  - Entry DTO: `{id, seq, at, actor: {id?, display_name?, role}, action, entity_type, entity_id, category, before, after, request_id, sealed: bool}`. `ip` is shown to owners only.

- **Alignment with spec 20–24** (added with the canonical-behaviour specs):
  - `GET /meta/app` adds `rules: { password_min_length }` (public, S0) for client-side hints (VAL-23, D-21).
  - Module and policy transitions follow STM-12/13: scheduled policy versions are not editable (D-34); `effective_from` may be at most 5 minutes in the past (D-24).
- RBAC matrix test (spec 25 §6): `crates/eq-api/tests/fixtures/rbac_matrix.csv` is a transcription of the spec 25 §6 table, one row per permission: `key,owner,admin,instructor,guardian,student`, each cell `y` when the matrix shows ✅, `own`, `linked`, `self` or `read`, else `n`. The test builds the registry with every compiled module and asserts that, for each module present, the set of `(key, roles)` from its `PermissionDef`s equals the fixture rows whose key starts with `<module>.`. Rows of modules that aren't compiled in are skipped, so later module tasks only need the fixture to stay true.

## Scope
### In
- Track the split task contract and dependency order only.
### Out
- All implementation and tests, which are assigned to the child parts below.

## Split
| Part | Owns | Depends on | Status |
|---|---|---|---|
| T-113a | Core policy and versioned settings API; AC 1, 2, 3, 4, 9 | T-102, T-104, T-105, T-106, T-109h, T-112b | todo |
| T-113b | Module metadata, RBAC registry and audit read API; AC 5, 6, 7, 8, 10 | T-113a | todo |

## Files
| Path | Action | Purpose |
|---|---|---|
| `tasks/M1-core/T-113a-*.md` | create or modify | Path named in this task's contract. |
| `tasks/M1-core/T-113b-*.md` | create or modify | Path named in this task's contract. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
No triggers. The task changes only its listed domain, presentation or static configuration behavior.

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. Every original acceptance criterion and test is assigned exactly once to a child part in the Split table.

## Tests to write
| File | Test names |
|---|---|
| n/a | Child parts own all implementation tests. |

## Definition of done
- [x] Every original criterion and test name is assigned once to a dependent child.
- [x] The task index reflects the child dependency chain.

## Report
**Plan** —
1. Compare the parent split table with the two child frontmatters → verify current status and exact dependency IDs.
2. Enumerate every original AC and named test across the child files → verify all are assigned once with no duplicate test name.
3. Update only this parent task's split table for mechanical drift → verify it matches the child files and generated index.
4. Regenerate/check `tasks/INDEX.md`, run the task linter and its tests, and inspect the documentation-only diff → verify all required task gates.
5. Set this tracker task to `review`, commit one signed task commit on `dev`, and push `dev`.

**Assumptions** — Child frontmatter is authoritative for child status and dependencies; `tasks/INDEX.md` is generated from that frontmatter. This parent owns split tracking only; the two child task files already assign the implementation scope and tests, so no implementation or child-file edit is needed.

**Test plan** — Given child parts T-113a and T-113b, verify their AC sets are disjoint and union to AC1–AC10, their named test sets have no duplicate, T-113a depends on the shipped required tasks, and T-113b depends on T-113a. Then run `scripts/tasks-index.sh --check`, `python3 -I scripts/tasks-lint.py`, and the task-linter unit suite.

**Summary** — Corrected the split tracker to match the existing child task statuses and dependency IDs. The ten acceptance criteria and twelve named test cases remain assigned exactly once; no child task, implementation or API contract was changed.

**Spec read** — Read the task and child inlined contracts for POL-06/07/10/12, MOD-06/09, API-14, AUD-07, STM-12/13, VAL-23 and AUTH-01/04; additionally read `AGENTS.md` §4, `spec/16-code-quality.md` §12, `spec/17-clean-architecture.md` §§1 and 4, `spec/18-security-by-design.md` §5, and `spec/19-error-handling.md` §10. No further source-spec lookup was needed for this tracker-only change.

**Dependency review (Step 2)** — Read the Reports for T-104, T-105, T-106, T-109h and T-112b, including their Deviations, Follow-ups and RATIFY items; inspected shipped tenant-context, Module/Registry/`has_data`, PolicyStore, AuditPolicy and CoreIdentityDirectory boundaries. The relevant dependencies are all `review`. Noted their ratifications/follow-ups without changing their code or task files: T-104's TD-035–037 resolver contract; T-105's resolved `utoipa` 6 / `utoipa-axum` 0.3 and TD-038–040; T-106's TD-041–053 and CQ-101 review; T-109h's TD-090–092 and deferred database checks; T-112b's TD-173–177, API-15 follow-up, T-114 generator and T-306 translations.

**Design reference (Step 3)** — n/a; this parent only tracks the backend/API split and makes no UI change.

**Files** — Modified this task's split table, Report and Questions; added TD-178/TD-179 in `tasks/DECISIONS.md`; regenerated `tasks/INDEX.md`. No child task file or code file was modified.

**Acceptance criteria → tests**
| AC | Test / child allocation | Result |
|---|---|---|
| AC1 (each original AC/test assigned once) | Contract audit: T-113a owns AC1–4/9 and five tests; T-113b owns AC5–8/10 and seven tests. | Pass: AC1–10 each once; 12 unique test names. |

**Checks run** —
- ✅ Contract allocation audit: AC1–10 assigned exactly once; 12 child test names unique; child statuses and dependency IDs match frontmatter.
- ✅ `scripts/tasks-index.sh --check` after status regeneration.
- ✅ `python3 -I scripts/tasks-lint.py`.
- ✅ `python3 -I -m unittest discover -s scripts/tests -p 'test_tasks_lint.py'` — 8/8.
- ❌ `python3 -I -m unittest scripts/tests/test_tasks_lint.py` — `-I` caused an import error for module `scripts` (TD-179); equivalent isolated test discovery passed 8/8.
- ✅ `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh`.
- ✅ `python3 -I scripts/erd.py --check`; migration check n/a because no migration changed.
- ✅ `git diff --cached --check`; ✅ `gitleaks protect --staged --redact` — no leaks found.
- Not run: Rust formatting, Clippy and workspace tests; this task changes only tracker documentation and generated index.
- Not run: web checks, `make dev`, `make api`, SQLx prepare and E2E; no web, API, query or runtime behavior changed, and this is not an integration task.
- Not run: hosted CI; it requires the hosted environment.

**Manual checks (Step 6)** — n/a; no runtime behavior. Verified `git branch -a` and `git ls-remote --heads origin`: only `main`, `staging`, `dev` exist, and current `dev` equals `origin/dev` before task work.

**Resource accounting** — Before: fixed target `8.9G`; `docker system df`: Images 8 total/7 active/14.55GB, Containers 10/10/10.49MB, Volumes 7/7/5.146GB, Build Cache 0. After: target `8.9G`; Images 8/7/14.55GB, Containers 10/10/10.49MB, Volumes 7/7/5.181GB, Build Cache 0. No Docker container, image, volume or database was created or removed by this task; the existing `ritma-db-1` and unrelated resources were untouched. The volume size estimate increased by 0.035GB without a task Docker action; its cause is outside this task and no unowned resource was removed. No new Cargo target or test database was created.

**Debug log (Step 7)** — The attachment's unittest module invocation failed because `python -I` removes the repository from `sys.path`, causing `ModuleNotFoundError: No module named 'scripts'`. Used the equivalent isolated discovery command `python3 -I -m unittest discover -s scripts/tests -p 'test_tasks_lint.py'`; all 8 tests passed (TD-179).

**Self code review (Step 8)** — The change is limited to this parent tracker, its TD record and generated index. Child files remain authoritative and unchanged; no spec, business behavior, permission, event, schema or security control was altered.

**Performance (Step 9)** — n/a; no runtime or query change.

**Security (Step 10)** — No triggers per the task's Security section. No endpoint, stored field, secret, access control or data flow changed; no SBD-81 implementation audit applies.

**Deviations** — RATIFY: inherited TD-063 (split the CQ-101-sized source contract into dependent parts), TD-066 (task's no-trigger Security classification), TD-067 (complete required task headings), TD-178 (align this parent split table with child frontmatter), and TD-179 (use isolated unittest discovery after the specified module invocation could not import `scripts`). The original Questions remain as history.

**Dependencies affected** — T-113a and T-113b remain `todo`; their existing contracts are unchanged. T-113a is next after this tracker becomes `review`; T-113b remains dependent on T-113a.

**Rollback plan** — Revert the single T-113 commit; regenerate `tasks/INDEX.md` from task frontmatter. No database or other external state was changed.

**Follow-ups** — Architect ratification of TD-178/179 and inherited TD-063/066/067; proceed to T-113a on the next task turn.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification; TD-063):** the original task exceeds CQ-101 limits → split its acceptance criteria and named tests across the listed dependent parts, preserving every original test name exactly once.
- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.
- **Self-resolved (implementer, run 25, needs architect ratification; TD-178):** the parent split table says both children are draft and T-113a depends on T-109, while the child frontmatter says todo and T-109h → update only the parent tracker to match its child files and generated index, because child frontmatter is authoritative and no implementation contract changes.
- **Self-resolved (implementer, run 25, needs architect ratification; TD-179):** the task-linter test command with `python -I -m unittest scripts/tests/test_tasks_lint.py` cannot import `scripts` under isolated mode → use unittest discovery with the explicit test directory/pattern, because it runs the same tests while preserving Python isolation.

## Review
n/a — awaiting human review.
