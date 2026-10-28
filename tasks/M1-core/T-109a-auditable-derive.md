---
id: T-109a
title: Add the Auditable derive and safe audit snapshot trait
milestone: M1
status: review
depends_on: [T-003, T-005, T-105, T-106, T-108]
spec_refs: [AUD-02, SBD-10, SBD-30, SBD-73, SBD-81, CQ-27]
estimated_size: S
---

## Goal
Provide the `Auditable` snapshot contract and derive macro used by the audit module. Fields marked `redact` serialize as `[redacted]`; fields marked `skip` are absent.

## Context
The parent contract is `T-109` Context → Record API. Use the task-authorized `syn`, `quote` and `proc-macro2` dependencies. The derive supports named and tuple structs; enums and unions return a compile error.

## Scope
### In
- Add `eq-audit-derive`, the `Auditable` trait/re-export and its private serde re-exports.
- Implement `#[audit(redact)]` and `#[audit(skip)]` with malformed or repeated modifiers rejected.
### Out
- Database, record, policy, sealer and job implementation (T-109b–T-109h).

## Files
| Path | Action | Purpose |
|---|---|---|
| `Cargo.toml`, `Cargo.lock`, `crates/eq-audit/Cargo.toml`, `crates/eq-audit/src/lib.rs` | modify | Register and re-export the macro/trait. |
| `crates/eq-audit-derive/Cargo.toml`, `src/lib.rs`, `tests/derive.rs` | create | Proc macro and exact redaction tests. |
| `tasks/M1-core/T-109a-auditable-derive.md` | modify | Report and status. |

## Interfaces
`pub trait Auditable { fn audit_snapshot(&self) -> serde_json::Value; }`; derive `Auditable`, with field attributes `#[audit(redact)]` and `#[audit(skip)]`.

## Business rules
None; this part implements only the stated snapshot behavior.

## Security
- Triggers: sensitive audit snapshots (SBD-30 T4).
- Data classes: S1/S2 per ERD-04/05.
- Threat: snapshot disclosure → redact marked fields and omit skipped fields (AUD-02, SBD-10, SBD-73).
- Abuse case: the fake password value must never appear in the serialized snapshot → `redact_attr`.

## Implementation notes
Use the package-scoped AGPL license entry already added for this Ritma-owned crate. Keep exact test names. Do not serialize a marked field to obtain its value.

## Acceptance criteria
- AC1. A redacted field becomes `[redacted]` and the fake value is absent → `redact_attr`.
- AC2. A skipped field is absent for named and tuple structs → `skip_attr`.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-audit-derive/tests/derive.rs` | `redact_attr`, `skip_attr` |

## Definition of done
- [x] `cargo fmt --all --check` and `cargo test -p eq-audit-derive --test derive` pass.
- [x] `cargo deny check` passes without changing the global allowed license list or advisory policy.
- [x] No new `#[allow]`; signed commit `feat(audit): add Auditable derive [T-109a]`.

## Report
**Plan** — Add the proc-macro crate, public trait and private serde path; verify redaction, skip behavior and package license gate.
**Assumptions** — Derive output is a JSON object keyed by field name or tuple index, as required by AUD-02.
**Test plan** — AC1→`redact_attr`; AC2→`skip_attr`; use obvious fake secrets and assert they do not leak.
**Summary** — Added `eq-audit-derive` with named and tuple field support for `#[audit(redact)]` and `#[audit(skip)]`, and exposed the `Auditable` trait and macro from `eq-audit`. Redacted and skipped values are not serialized; the existing `AuditModule` stub remains available until T-109h.
**Spec read** — `spec/16-code-quality.md` CQ-27/28/101 and §12; `spec/17-clean-architecture.md` §§1/4; `spec/18-security-by-design.md` SBD-10/30/73/81; `spec/19-error-handling.md` §§1/10; `spec/24-open-source-stack.md` OSS-03; parent T-109 contract.
**Files** — Added `crates/eq-audit-derive/{Cargo.toml,src/lib.rs,tests/derive.rs}`; changed workspace manifests/lockfile and `crates/eq-audit/src/lib.rs`; created T-109b–T-109h contracts; updated this task, parent T-109, decisions/index and consumer dependency references.
**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `redact_attr` | Pass; `[redacted]` is present and the fake password is absent from JSON. |
| AC2 | `skip_attr` | Pass; named and tuple skipped fields are absent. |
**Checks run** — `cargo fmt --all --check` ✅; `SQLX_OFFLINE=true cargo test -p eq-audit-derive --test derive` ✅ (2/2); `SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings` ✅; `env -u RITMA_TEST_DATABASE_URL SQLX_OFFLINE=true cargo test --workspace -- --test-threads=1` ✅; `cargo deny check` ✅ (advisories, bans, licenses and sources pass; existing duplicate-version and unmatched-CC0 allowance warnings remain); `scripts/check-deps.sh`, `scripts/check-literals.sh`, `scripts/check-web-boundaries.sh`, `scripts/check-layers.sh`, `python3 -I scripts/erd.py --check`, `python3 -I scripts/tasks-lint.py`, `scripts/tasks-index.sh --check`, and `git diff --check` ✅. `gitleaks dir --redact --no-banner` run once per changed path ✅. Not run: `scripts/erd.py --migrations` (no migration in T-109a); E2E (no runtime behavior in scope).
**Manual checks** — n/a; macro behavior is covered by the two integration tests.
**Debug log** — The first workspace Clippy run compiled the full audit integration test before its APIs were assigned to later split parts; retained the test source as `audit.rs.hold` and isolated it from this part. The next run found `eq-api` still requires the existing `AuditModule` stub, so the stub remains until T-109h. Final workspace Clippy and tests passed. An initial multi-path Gitleaks invocation scanned the full checkout and found one unrelated repository finding; rerunning each changed path separately found no leaks.
**Performance** — n/a; no performance target was specified.
**Security** — SBD-30 trigger: audit snapshot disclosure. The derive inserts the fixed `[redacted]` value without serializing a marked field, omits skipped fields, and the tests assert a fake secret is absent. No database field, handler, tenant access, external call or new permission is added. Targeted Gitleaks scans of changed paths were clean.
**Deviations** — RATIFY TD-074: retain the architect-approved package-scoped AGPL exception for this first-party crate; the workspace license matches the repository license and global license/advisory policy is unchanged. RATIFY TD-076: replace the missing SBD-21 citation with current spec 18 references; behavior remains defined by AUD-02.
**Dependencies affected** — T-109b–T-109h consume the public trait/macro or complete audit crate; T-110/T-110a now depend on T-109d, module/settings consumers on T-109h, and T-610 on T-109b.
**Rollback plan** — Revert this part commit; it introduces no migration or persistent state.
**Follow-ups** — T-109b adds the mapped audit schema and database grants/functions. The audit integration-test source held during this part is split among the later task owners and restored under its declared test path before those checks run.

**Workflow steps 0–12** — Step 0: resumed the existing T-109 work on `milestone/m1-core`; T-109a is the single WIP and its five dependencies are `review`; regenerated and checked the index. Step 1: read the full task, parent contract and cited standards. Step 2: read dependency Reports, deviations/follow-ups and shipped code for T-003/T-005/T-105/T-106/T-108. Step 3: n/a, backend macro task. Step 4: recorded this plan and AC test map before final verification. Step 5: implemented only the macro and snapshot trait while preserving the module stub. Step 6: n/a, no server or endpoint. Step 7: recorded the staging-related Clippy findings and fixes above. Step 8: reviewed against CQ-28, CA and ERR; no unsafe code, `#[allow]`, logging or error-code changes. Step 9: n/a, no measured performance concern. Step 10: completed the applicable SBD-81 abuse review and fake-secret tests. Step 11: reran formatting, Clippy, workspace tests, dependency/guard/task checks and targeted secret scans. Step 12: this Report and index are complete; signed commit and push follow.

## Questions
- **Self-resolved (implementer, run 6; TD-074):** the objective calls this license resolution TD-010, but TD-010 already identifies T-005's ETag decision → record it as TD-074 and retain the architect's single-package AGPL exception because OSS-03 permits AGPL for Ritma's own crate and the package inherits the repository license.
- **Self-resolved (implementer, run 6; TD-076):** T-109a cites SBD-21, which is not defined in the current security spec → keep AUD-02's exact redaction behavior and cite the current SBD-10/30/73/81 classification, trigger, abuse-test and review controls.

## Review
n/a until review.
