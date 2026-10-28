---
id: T-110a1b
title: Add validated identity policy values
milestone: M1
status: review
depends_on: [T-110a1, T-106]
spec_refs: [IDN-01, IDN-02, IDN-03, IDN-04, IDN-05, IDN-06, IDN-12, POL-01, POL-10, CQ-23, CAN-10, CAN-64]
estimated_size: M
---

## Goal
Implement the validated `IdentityPolicy`, `Lockout`, and `TotpPolicy` values used by the identity module.

## Context
Defaults, numeric bounds and enum variants come from `spec/04-policy-engine.md` §6.2 and the T-110a parent contract. Use `RoleMap<u16>` from T-110a1. Wire enums use externally tagged snake_case forms per CAN-64. The public parent sketch has bounded public fields; CQ-23/CAN-10 require invalid value types to have private fields and validated construction/deserialization.

## Scope
### In
- Add private-field policy values, validating constructors/accessors, defaults and serde validation.
- Implement `PolicyDoc` validation identity/version and the two specified summary keys.
### Out
- Module registration/schema `x-ui` wiring (T-110a1c), login behavior, endpoints, and schema changes.

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-identity/src/policy.rs`, `crates/eq-identity/src/lib.rs` | create/modify | Validated policy values and exports. |
| `crates/eq-identity/Cargo.toml`, `Cargo.toml`, `Cargo.lock` | modify | Direct serde/schemars dependencies, if needed. |
| `tasks/M1-core/T-110a1b-validated-identity-policy-model.md`, `tasks/INDEX.md`, `tasks/DECISIONS.md` | modify/generate | Report, index and self-resolution record. |

## Interfaces
`IdentityPolicy` implements `PolicyDoc` (`ModuleId::Identity`, schema version 1). Its accessors supply each value without permitting invalid mutation. Defaults and ranges are exactly those in spec 04 §6.2. Summaries use `policy.identity.summary.sessions` and `policy.identity.summary.lockout`.

## Business rules
All default values and range bounds come only from spec 04 §6.2. TOTP variants serialize as `off`, `optional_for_staff`, and `required_for_owner_admin`.

## Security
- Triggers: T4 (SBD-30; authentication policy).
- Policy fields are S1; no credentials or tokens are held in these values.
- Invalid bounds, unknown enum variants and unknown fields are rejected at deserialization (CQ-23/CAN-10/CAN-64).

## Implementation notes
1. Keep constructors and serde on the same validation path so no invalid policy value can be stored or consumed.
2. Use the existing registered `must_be_gte`/`must_be_lte` validation codes for range failures; add no error code.
3. Keep enum serialization in canonical snake_case form and reject unknown variants.

## Acceptance criteria
- AC1. Every specified minimum/maximum is accepted and values outside the range are rejected at construction and serde boundaries → `out_of_range_timeout_rejected`, `invalid_values_rejected_on_deserialize`.
- AC2. Defaults and `PolicyDoc` summaries match the task contract → verified with the module integration test in T-110a1c.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-identity/src/policy.rs` | `invalid_values_rejected_on_deserialize`, `out_of_range_timeout_rejected`, `constructors_reject_out_of_range_values` |

## Definition of done
- [x] All three task tests pass, or are explicitly Not run under the fixed-target resource restriction.
- [x] All bounds, defaults and serde rejection follow spec 04/CAN-64; fields remain private.
- [x] `cargo fmt --all --check`, `cargo deny check`, applicable guards and task checks pass.
- [x] Report includes Workflow steps 0–12 and resource snapshots.
- [x] Signed commit `feat(identity): add validated policy values [T-110a1b]` pushed to `dev`.

## Report
**Plan** — (1) Compare defaults, numeric bounds and TOTP variants against spec 04 §6.2 and the parent task → verify the RoleMap field set and all policy values. (2) Keep bounded values private and validate constructors and deserialization through one path → verify direct and serde boundary tests. (3) Implement the Identity PolicyDoc module/version and the two contract summary keys → verify exact keys/args and leave schema metadata to T-110a1c. (4) Run formatting, cargo-deny, guards, task checks, schema drift check and targeted secret scans → verify every applicable local check; record build/test restrictions explicitly.
**Assumptions** — Spec 04 §6.2 supplies every default and numeric bound. T-110a1's `RoleMap<T>` is the shipped closed role container; T-106's `PolicyDoc` is authoritative for module/version, validation and summaries. No new business value, permission, error code, schema field or credential behavior is needed.
**Test plan** — AC1: Given every numeric field at min/max, when deserialized, then it is accepted; given min−1/max+1, then it is rejected → `invalid_values_rejected_on_deserialize`. Given every constructor field at the inclusive boundaries and outside them, when constructed, then only in-range values are accepted → `out_of_range_timeout_rejected` for each role and `constructors_reject_out_of_range_values` for the remaining fields and Lockout. Edge cases include all 13 numeric ranges, unknown TOTP variants and unknown fields. AC2: Given default identity values, when the module contract is exercised, then defaults, schema and summaries match → `defaults_schema_and_summary_match_contract` in T-110a1c. Abuse case: forged unknown policy fields and TOTP modes cannot be accepted. No API error code is introduced.
**Summary** — Added validated identity policy and lockout values with private fields, accessor methods, exact spec defaults/ranges and canonical TOTP variants. Serialization is unchanged; deserialization rejects out-of-range values, unknown enum variants and unknown fields. The PolicyDoc uses Identity/version 1 and the two summary contracts.
**Spec read** — `spec/04-policy-engine.md` §6.2; `spec/16-code-quality.md` §§1–3, 9, 12; `spec/17-clean-architecture.md` §§1, 4; `spec/18-security-by-design.md` §§2, 4–5, 8 (SBD-30/81); `spec/19-error-handling.md` §§5, 10; `spec/23-contracts-and-canonical-forms.md` §6 (CAN-64); `spec/25-auth-and-access-control.md` §§2–3 (IDN-12). Also read the full T-110a1b task, T-110a parent contract, and dependency reports/code for T-110a1 and T-106.
**Files** —
- `crates/eq-identity/src/policy.rs` — added validated `Lockout`, `TotpPolicy`, `IdentityPolicy`, PolicyDoc summaries and three unit tests.
- `crates/eq-identity/src/lib.rs` — exported the policy module and removed a duplicated crate doc comment.
- `crates/eq-identity/Cargo.toml`, `Cargo.lock` — declared the crate's direct serde/schema dependencies and updated the lockfile.
- `tasks/M1-core/T-110a1b-validated-identity-policy-model.md` — corrected task security/spec references, recorded the report and decisions.
- `tasks/DECISIONS.md`, `tasks/INDEX.md` — recorded TD-101/102 and regenerated the task index.
**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `invalid_values_rejected_on_deserialize` | Not run: the fixed Cargo target directory is absent and the run rules prohibit creating one. Test covers every min/max and min−1/max+1, unknown enum values and unknown fields. |
| AC1 | `out_of_range_timeout_rejected` | Not run for the same target-directory restriction. Test covers each role timeout at both valid boundaries and both invalid edges through `IdentityPolicy::new`. |
| AC1 | `constructors_reject_out_of_range_values` | Not run for the same target-directory restriction. Test covers remaining IdentityPolicy numeric constructor bounds and Lockout constructor bounds. |
| AC2 | `defaults_schema_and_summary_match_contract` in T-110a1c | Not run; owned by the dependent task. Source review confirms defaults and summary keys/arguments match the parent contract. |
**Checks run** —
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo fmt --all --check`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo deny check` — advisories, bans, licenses and sources pass; existing duplicate-`base64` and unmatched-`CC0-1.0` warnings remain, and `deny.toml` was not changed.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo metadata --offline --locked --format-version 1`.
- ✅ `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh`.
- ✅ `scripts/tasks-index.sh --check && python3 -I -B scripts/tasks-lint.py && python3 -I -B -m unittest discover -s scripts/tests -p 'test_tasks_lint.py'` — task lint passed and 8/8 fixtures passed.
- ✅ `python3 -I scripts/erd.py --check` — the ERD matches the DDL; no migration changed.
- ✅ `git diff --check`.
- ✅ Per-file `gitleaks dir --redact --no-banner --log-level error --report-format json --report-path /dev/null <path>` scans for all seven task-owned paths listed under Files — no findings.
- Not run: `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings` and `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --workspace` — the fixed target directory is absent; creating any target directory is prohibited by the run's disk rule.
- Not run: the three Rust unit tests named above — same target restriction.
- Not run: `make dev` — this task adds no server route or runtime flow, and starting a database/container is prohibited.
- n/a: web lint/typecheck/tests/i18n, `make api`, SQLx prepare, and `scripts/erd.py --migrations` — no web, API, query or migration changed.
- n/a: `make e2e` — a pure policy value task adds no user journey or integration flow; database used: none; E2E pass count: 0/0.
- Not run: hosted GitHub Actions — requires a hosted runner.
**Manual checks** — Compared all defaults and ranges against spec 04 §6.2; compared summary keys/arguments against the T-110a parent; inspected constructors, serde paths, private fields and the `RoleMap<T>` values. No server endpoint or database interaction exists in this scope. No database was used.
**Debug log** — A first multi-path Gitleaks command traversed the full repository and reported one pre-existing `curl-auth-user` match in the untouched Makefile E2E command. Per-file scans of every task-owned path then passed with no findings. No source change was made for this unrelated baseline match.
**Performance** — n/a; no runtime path, query or performance target is added.
**Security** — T4 only per SBD-30 (RATIFY TD-101): authentication policy values change, but there is no endpoint (T1) or S2/S3 field (T3). SBD-81 review: (1) no handler/auth chain; (2) no tenant input or database access; (3) fields are S1 policy settings, with no PII or secrets; (4) numeric bounds, a closed enum and unknown-field rejection are enforced at constructors/serde; (5) no rendered output; (6) no crypto, credentials or token material; (7) no external calls; (8) no state mutation; (9) no audit event; (10) abuse tests reject out-of-range values, unknown enum variants and unknown fields, but tests were not run due the target restriction. Targeted Gitleaks scans passed.
**Deviations** — RATIFY TD-097: use private bounded-value fields and validated construction/deserialization per CQ-23/CAN-10. RATIFY TD-101: correct the task's SBD-30 triggers to T4 only. RATIFY TD-102: add omitted spec refs IDN-05 and IDN-12. No policy values, schema fields, errors or permissions were added.
**Dependencies affected** — T-110a1c consumes this policy contract and owns the default/schema/summary integration test; T-110a2 consumes the validated policy types. No shipped business behavior changed.
**Rollback plan** — Revert the signed commit. No migration or persisted data changes are involved.
**Follow-ups** — Run the three Rust tests, workspace clippy/tests and the T-110a1c module integration test once the fixed target cache is permitted. The staged task diff totals 602 changed lines, exceeding CQ-101's approximate 400-line PR guide; ask the maintainer to review its size and consider a separate follow-up split if needed.
**Resource report** — Before (2026-10-28T17:53:57Z): `/home` had 82G free; `du -sh "$HOME/.cache/ritma/target"` reported no such file (exit 1); `timeout 20s docker system df` timed out (exit 124); inventories: 8 containers, 11 images, 434 volumes. After (2026-10-28T18:00:03Z): same target absence, timeout and inventory counts; `/home` still had 82G free. No database/container/image/volume/target was created or removed.
**Workflow steps 0–12** — 0: resumed the pre-existing in-progress task on `dev`; preserved unrelated WIP, regenerated the stale index and confirmed it was consistent. 1: read the whole task and cited/cross-cutting specs above. 2: read T-110a1 and T-106 reports, deviations/follow-ups and shipped `RoleMap<T>`/`PolicyDoc` code; no inherited item changes this task's contract. 3: n/a, backend-only. 4: wrote the plan, assumptions and test mapping above before completing implementation. 5: implemented only the policy values, exports and direct dependencies named by the task. 6: no runtime route exists; `make dev` and database use were unavailable/not applicable. 7: recorded and resolved the broad Gitleaks scan scope issue by scanning each task-owned path individually; no code failure remained. 8: reviewed values against CQ-23/CAN-10/CAN-64, public docs, module boundaries and the task scope. 9: n/a, no runtime or query optimization target. 10: completed the applicable SBD-81 review above. 11: ran all applicable static checks listed above; Rust tests/build checks are explicitly Not run under the fixed-target restriction. 12: after the report, set `review`, regenerated and checked the index, created one signed task commit and pushed it to `dev`.

## Questions
- **Self-resolved (implementer, run 7, needs architect ratification; TD-097):** the parent policy sketch exposes bounded `IdentityPolicy`/`Lockout` fields directly, while CQ-23 and CAN-10 require invalid value types to use private fields and validating construction/deserialization → keep those fields private, validate through constructors, and deserialize through the same validation path; retain public `RoleMap<T>` role fields because that container has no independent invariant.
- **Self-resolved (implementer, run 7, needs architect ratification; TD-101):** the task lists T1/T3 although it adds no endpoint and contains only S1 policy fields → use T4 only because it changes authentication policy; SBD-30 defines T1 for endpoint changes, T3 for S2/S3 data, and T4 for authentication.
- **Self-resolved (implementer, run 7, needs architect ratification; TD-102):** the task uses invite-expiry and TOTP policy fields but omits their spec references → include IDN-05 and IDN-12, which define those fields and variants in spec 04 §6.2 and spec 25.

## Review
n/a — awaiting maintainer review.
