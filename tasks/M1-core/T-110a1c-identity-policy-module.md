---
id: T-110a1c
title: Register identity policy schema and summaries
milestone: M1
status: review
depends_on: [T-110a1b, T-106]
spec_refs: [IDN-01, IDN-02, IDN-03, IDN-04, IDN-05, IDN-06, IDN-12, POL-01, POL-06, POL-07, POL-10, CAN-64]
estimated_size: S
---

## Goal
Replace the identity module stub with its versioned policy defaults, schema metadata, validation and summary contract.

## Context
The compiled module registry uses `IdentityModule`. Its policy schema is generated from the T-110a1b policy values, with every field marked `x-ui: advanced`; numeric schema bounds match spec 04 §6.2. Schema version is 1. The summary keys and arguments are defined by T-110a and spec 04 §6.2.

## Scope
### In
- Implement the Identity module policy methods and retain empty routes, events, jobs, cron, ports, permissions and nav until their owning tasks.
- Add the exact named defaults/schema/summary test.
### Out
- Login or role behavior, routes, permissions, migrations and schema/value changes.

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-identity/src/module.rs`, `crates/eq-identity/src/lib.rs` | create/modify | Identity module manifest and policy wiring. |
| `crates/eq-identity/src/policy.rs` | modify | Keep policy values unchanged while exercising their module contract. |
| `tasks/M1-core/T-110a1c-identity-policy-module.md`, `tasks/DECISIONS.md`, `tasks/INDEX.md` | modify/generate | Report, ratification register and index. |

## Interfaces
Implement the existing `eq_core::module::Module` interface. `policy_schema`, `default_policy`, `validate_policy`, and `summarize_policy` delegate to the T-110a1b policy contract. Module ID is Identity and schema version is 1.

## Business rules
Do not add permission keys, grants, policy fields or routes. All settings use the exact parent/spec defaults and summary keys.

## Security
- Triggers: T4 (SBD-30; authentication policy configuration).
- S1 policy values only; credentials and tokens remain outside policy output.
- T: forged out-of-range policy values, unknown enum variants or unknown fields are rejected by the validated type/serde path. E: unsupported schema versions fail closed.
- Abuse case: a forged invalid identity-policy document is rejected by `IdentityModule::validate_policy` in `defaults_schema_and_summary_match_contract`.

## Implementation notes
1. Generate the policy schema from the typed document and add only the required `x-ui` metadata and declared numeric bounds.
2. Preserve the module manifest as Identity/always-on and keep every unrelated module surface empty until its owning task.
3. Reject unsupported policy schema versions instead of silently accepting them.

## Acceptance criteria
- AC1. Default policy, identity schema with every `x-ui` field advanced, range constraints and summaries match the task contract → `defaults_schema_and_summary_match_contract`.
- AC2. `IdentityModule` remains always-on and exposes no unowned routes, events, jobs, permissions or navigation.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-identity/src/module.rs` | `defaults_schema_and_summary_match_contract` |

## Definition of done
- [x] The named policy/module test is explicitly Not run under the fixed-target resource restriction.
- [x] `cargo fmt --all --check`, `cargo deny check`, applicable guards and task checks pass.
- [x] No route, permission, grant, policy field, database schema object or business value is added; JSON Schema only describes existing IdentityPolicy values.
- [x] Report includes Workflow steps 0–12 and resource snapshots.
- [x] Signed commit `feat(identity): register policy schema [T-110a1c]` pushed to `dev`.

## Report
**Plan** — (1) Implement the `Module` policy methods against the shipped `IdentityPolicy`/`PolicyDoc` contract → verify module ID/version, defaults and summary keys. (2) Generate the policy schema and add only its declared x-ui metadata and spec ranges → verify all top-level and nested fields and every numeric bound. (3) Preserve the existing always-on identity manifest and empty unowned surfaces → verify the manifest and routes/events/jobs/cron/permission/navigation surfaces. (4) Run formatting, cargo-deny, guards, task checks and targeted secret scans → record the Rust test/build checks as Not run because creating a Cargo target is prohibited.
**Assumptions** — The existing Identity manifest (`identity`, always on) and T-110a1b's shipped `PolicyDoc`/validated policy values are authoritative. Spec 04 §6.2 supplies every range/default; no value, permission, route, grant or schema field is invented.
**Test plan** — AC1: Given `IdentityPolicy::default()`, when the module contract is exercised, then defaults, ID/version, both summary keys/arguments, all x-ui metadata and every schema bound match spec → `defaults_schema_and_summary_match_contract`. AC2: Given an `IdentityModule`, when its manifest and owned surfaces are inspected, then it is always-on and exposes no unowned routes, events, jobs, permissions or navigation. Edge cases: nested `$defs` for RoleMap/Lockout and every lower/upper schema bound. Error/abuse cases: out-of-range values, unknown enum variants and unknown fields are rejected, invalid documents produce no partial summary, and unsupported versions fail closed.
**Summary** — Exported `IdentityModule` from `eq-identity` and implemented the policy schema, defaults, validation, summary and version contract using the shipped validated `IdentityPolicy`. The schema records the declared x-ui order, all spec bounds and closed TOTP variants. Routes, events, jobs, cron, permissions and navigation remain empty.
**Spec read** — `spec/03-module-system.md` §§1–4; `spec/04-policy-engine.md` §§4, 6.2; `spec/16-code-quality.md` §§9, 12; `spec/17-clean-architecture.md` §§1, 4, 7; `spec/18-security-by-design.md` §§4, 5, 8; `spec/19-error-handling.md` §§5, 10; `spec/23-contracts-and-canonical-forms.md` §6; `spec/25-auth-and-access-control.md` §§2–3. Read T-106 and T-110a1b Reports and their shipped policy/module interfaces. T-106's `PresetPatchSource` follow-up does not change this module contract; T-110a1b assigns this task the schema/default/summary integration test and its policy values are unchanged.
**Files** —
- `crates/eq-identity/src/module.rs` — created the Identity `Module` implementation and `defaults_schema_and_summary_match_contract`.
- `crates/eq-identity/src/lib.rs` — exported `module` and `IdentityModule` for the compiled registry.
- `tasks/M1-core/T-110a1c-identity-policy-module.md` — recorded the implementation report and review status.
- `tasks/DECISIONS.md` — registered TD-103 and TD-104 for architect ratification.
- `tasks/INDEX.md` — regenerated from task status.
**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `defaults_schema_and_summary_match_contract` | Not run: the fixed Cargo target directory is absent and this run prohibits creating a target. Source assertions cover defaults, schema metadata and ranges, validation, migration version handling and summary keys/arguments. |
| AC2 | `defaults_schema_and_summary_match_contract` | Not run for the same target restriction. Source assertions cover manifest identity/version, empty OpenAPI paths, events, jobs, cron, permissions and navigation. |
**Checks run** —
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo fmt --all --check`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo deny check` — advisories, bans, licenses and sources passed; existing duplicate-`base64` and unmatched-`CC0-1.0` warnings remain. No deny policy was changed.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo metadata --offline --locked --format-version 1 --no-deps`.
- ✅ `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh`.
- ✅ `scripts/tasks-index.sh --check`, `python3 -I scripts/tasks-lint.py`, and `python3 -I scripts/tests/test_tasks_lint.py` — task hygiene passed; 8/8 task-lint fixtures passed.
- ✅ `python3 -I scripts/erd.py --check` — ERD matches the DDL.
- ✅ `git diff --check`.
- ✅ Per-file `gitleaks dir --redact=100 --no-banner <path>` scans for `crates/eq-identity/src/module.rs`, `crates/eq-identity/src/lib.rs`, this task, `tasks/DECISIONS.md` and `tasks/INDEX.md` — no findings.
- Not run: `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings`, `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --workspace`, and the named module test — the fixed target directory is absent and the run's disk rule prohibits creating any target directory.
- n/a: web lint/typecheck/tests/i18n, `make api`, `cargo sqlx prepare --workspace`, `scripts/erd.py --migrations` and database tests — no web/API/query/migration changed.
- n/a: `make dev` — this task adds no runtime route or user flow and database/container creation is prohibited by the run's resource rule.
- n/a: `make e2e` — this is a module policy contract task, not an integration/journey task; database used: none; E2E pass count: 0/0.
- Not run: hosted GitHub Actions; a hosted runner is unavailable in this session.
**Manual checks** — Compared all defaults, numeric bounds and TOTP values to spec 04 §6.2 and the shipped `IdentityPolicy`; checked the two summary keys/arguments against `PolicyDoc::summarize`; inspected the manifest, empty surfaces, private validated policy types, serde validation and version rejection. No database or runtime endpoint was involved.
**Debug log** — The first Python fixture command used module import syntax with isolated mode and failed to import `scripts`; running `python3 -I scripts/tests/test_tasks_lint.py` directly passed all 8 fixtures. No implementation test failure was observed. The Rust test could not be attempted without violating the fixed-target restriction.
**Performance** — n/a; this task adds no request path, query or measured performance target.
**Security** — T4 only under SBD-30 (RATIFY TD-103): authentication policy configuration changes, with no endpoint or S2/S3 field. SBD-81: (1) no handler or auth chain; (2) no tenant input, table or RLS change; (3) only S1 policy values, no secrets or PII; (4) numeric bounds, closed TOTP enum and unknown-field rejection use the validated type/serde path; (5) no rendered output; (6) no credentials, tokens or cryptographic material; (7) no external calls; (8) no persisted state or money mutation; (9) no security event is emitted by this module contract; (10) the named test contains invalid-range, unknown-enum and unknown-field abuse assertions, but execution is Not run under the target restriction. Targeted Gitleaks scans passed.
**Deviations** — RATIFY TD-103: corrected the task's trigger list to T4 because the scope changes authentication policy but adds no endpoint or S2/S3 data. RATIFY TD-104: added IDN-05 and IDN-12 to `spec_refs` because the task uses invite-expiry and TOTP settings defined by those contracts. RATIFY TD-105: clarified that the DoD prohibits new persisted schema or policy fields, while this task generates JSON Schema for the already-defined values per POL-06. No policy value, field, permission, grant, error code, route or migration was added.
**Dependencies affected** — T-110a2 now consumes the exported `IdentityModule` and this schema/default/summary contract. No other task's shipped behavior changed.
**Rollback plan** — Revert the signed task commit. No migration or persisted data change is involved.
**Follow-ups** — Run `defaults_schema_and_summary_match_contract`, the workspace Clippy checks and workspace tests when the run's fixed-target restriction permits build artifacts. The current Report marks those checks Not run; the source has not been represented as runtime-verified.
**Resource report** — Before (2026-10-29T01:15:35Z): `/home` had 81G free, `$HOME/.cache/ritma/target` was absent, and `timeout 20s docker system df` timed out; the last recorded inventory was 8 containers, 11 images and 434 volumes. After (2026-10-29T01:28:11Z): `/home` had 81G free, the fixed target remained absent, inventory was 8 containers, 11 images and 434 volumes, and `timeout 20s docker system df` again exited 124 without output. No database, container, image, volume or Cargo target was created or removed.
**Workflow steps 0–12** — 0: resumed the preserved in-progress T-110a1c on `dev`, retained unrelated WIP and checked task ordering/index. 1: read the full task, cited specs and cross-cutting rules above. 2: read T-106 and T-110a1b Reports, deviations/follow-ups and shipped policy/module code; their interfaces match this task. 3: n/a, backend-only. 4: recorded the plan, assumptions and AC/edge/error/abuse test plan before implementation. 5: implemented only the policy/module wiring and exact named test. 6: n/a, no server route or runtime flow was introduced. 7: corrected the isolated Python fixture invocation; no implementation failure remains, and Rust execution was unavailable under the target restriction. 8: reviewed the diff against spec 16 §9, spec 17 §7 and spec 19; no secret or out-of-scope behavior was found. 9: n/a, no measurable performance path. 10: completed the applicable SBD-81 checklist above. 11: ran the applicable checks listed above and explicitly recorded Rust checks as Not run. 12: completed this Report, set `review`, regenerated/checked the index, and made one signed task commit pushed to `dev`.

## Questions
- **Self-resolved (implementer, run 7, needs architect ratification; TD-103):** the task lists T1/T3 although it adds no endpoint and only exposes S1 policy values → use T4 only because identity authentication policy changes; SBD-30 defines T1 for endpoints, T3 for S2/S3 data and T4 for authentication.
- **Self-resolved (implementer, run 7, needs architect ratification; TD-104):** the task uses invite-expiry and TOTP settings but omits IDN-05 and IDN-12 from `spec_refs` → add those references because spec 04 §6.2 defines the settings and spec 25 assigns TOTP the IDN-12 contract.
- **Self-resolved (implementer, run 7, needs architect ratification; TD-105):** the DoD says no schema is added, while the Goal and AC1 require `policy_schema` → interpret the restriction as prohibiting new persisted schema objects or policy fields; generate JSON Schema only for existing IdentityPolicy values because POL-06 and the task Scope require exposing that contract.

## Review
n/a — awaiting maintainer review.
