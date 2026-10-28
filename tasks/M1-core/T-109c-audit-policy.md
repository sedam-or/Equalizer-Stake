---
id: T-109c
title: Add the versioned audit retention policy
milestone: M1
status: review
depends_on: [T-109b]
spec_refs: [AUD-07, POL-01, POL-06, POL-07, POL-10, CQ-23, CAN-10]
estimated_size: S
---

## Goal
Implement `AuditPolicy` and `Category` with the exact defaults, ranges, cross-field validation, schema metadata and summaries from parent T-109.

## Context
`AuditPolicy` has `retention_years` (1..=10, default 7), `anonymize_pii_after_years` (1..=10, default 2 and no greater than retention), and `admin_categories` (default `[Operational]`). It implements `PolicyDoc` for `ModuleId::Audit`, schema version 1.

## Scope
### In
- Validated value type, serde deserialization through validation, policy schema with `x-ui: advanced`, validation code `must_be_lte`, and summary keys.
### Out
- Policy persistence, module wiring and jobs (T-109g/T-109h).

## Files
| Path | Action | Purpose |
|---|---|---|
| `Cargo.lock` | modify | Lock the audit policy schema dependency. |
| `crates/eq-audit/Cargo.toml`, `crates/eq-audit/src/lib.rs`, `crates/eq-audit/src/policy.rs` | modify/create | Policy contract and exports. |
| `tasks/M1-core/T-109c-audit-policy.md` | modify | Report and status. |
| `tasks/DECISIONS.md` | modify | Record T-109c self-resolutions. |

## Interfaces
`eq_audit::policy::AuditPolicy`; fields stay private, the constructor and deserializer validate individual field ranges and closed enum values, and `PolicyDoc::validate` checks cross-field policy rules after deserialization (POL-10). Preserve the task's exact summary keys and validation pointer.

## Business rules
Use only the policy defaults and numeric ranges stated above and in parent T-109.

## Security
- Triggers: audit retention and data lifecycle (SBD-30 T4).
- Data classes: S1/S2 per ERD-04/05.
- Threat: invalid retention policy → private fields, validating constructor, validated deserialization and cross-field validation.
- Abuse cases: out-of-range and retention/anonymization order are rejected by the listed policy tests.

## Implementation notes
Follow `spec/16` CQ-23 and the policy type patterns already shipped in `eq-core`; do not add fields or defaults.

## Acceptance criteria
- AC1. Defaults, schema metadata and summaries match the contract → `defaults_schema_and_summary_match_the_policy_contract`.
- AC2. Invalid input cannot bypass validation through serde → `deserialization_validates_value_ranges_and_rejects_unknown_fields`.
- AC3. Anonymization after retention returns `must_be_lte` at the specified pointer → `cross_field_limit_reports_the_contract_pointer_and_code`.

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-audit/src/policy.rs` | `defaults_schema_and_summary_match_the_policy_contract`, `deserialization_validates_value_ranges_and_rejects_unknown_fields`, `cross_field_limit_reports_the_contract_pointer_and_code` |

## Definition of done
- [x] Policy tests and Rust format/clippy checks pass.
- [x] No public policy field can be constructed/deserialized outside its declared range or from an unknown enum/field; cross-field policy rules return their specified error from `PolicyDoc::validate`.
- [x] Signed commit `feat(audit): add audit retention policy [T-109c]`.

## Report
**Plan** — (1) Confirm the parent audit-policy values and the shipped `PolicyDoc`/`XUi` contracts → verify defaults and schema extension points. (2) Implement the private validated policy type and closed category enum → verify the three named unit tests. (3) Add schema bounds, advanced UI metadata and exact summary keys → verify the schema and summary assertions. (4) Run the Rust, dependency, guard, task and secret checks → record every result and any disk-limited rerun.
**Assumptions** — Parent T-109 supplies every business value; no additional default or policy field is introduced. `PolicyDoc::validate` is the cross-field validation stage specified by POL-10.
**Test plan** — AC1: Given the policy default, when its schema and summary are read, then defaults, numeric bounds, `x-ui: advanced`, field order and summary keys match `defaults_schema_and_summary_match_the_policy_contract`. AC2: Given an out-of-range value or unknown field, when JSON is deserialized, then it is rejected by `deserialization_validates_value_ranges_and_rejects_unknown_fields`. AC3: Given individually valid values where anonymization follows retention, when `PolicyDoc::validate` runs, then it reports `must_be_lte` at `/anonymize_pii_after_years` in `cross_field_limit_reports_the_contract_pointer_and_code`. Edge cases cover 1, 10, 0 and 11; error cases cover `must_be_gte`/`must_be_lte` and unknown input; abuse cases cover invalid ranges, unknown fields and the cross-field limit.
**Summary** — Added `AuditPolicy` and `Category` with closed serde forms, private fields, validated construction/deserialization, exact defaults, schema bounds and advanced UI metadata. `PolicyDoc::validate` reports the contract's cross-field error pointer/code, and `summarize` emits the two required keys. The three task tests and the previously run workspace checks passed; final Rust build checks are not repeated because the mandated fixed Cargo target cache is absent and creating a replacement is prohibited by the disk rule.
**Spec read** — `spec/16-code-quality.md` §§9/12 (CQ-23, CQ-101); `spec/17-clean-architecture.md` §§1/4 (CA-01/30); `spec/18-security-by-design.md` §§4/5.2/8 (SBD-30/44/73/81); `spec/19-error-handling.md` §§3/5/10 (ERR-21/31); `spec/23-contracts-and-canonical-forms.md` §§1/2 (CAN-10); parent `T-109` policy contract; dependency `T-109b` Report and shipped `eq-audit` exports.
**Files** — Created `crates/eq-audit/src/policy.rs`; modified `crates/eq-audit/Cargo.toml`, `crates/eq-audit/src/lib.rs`, `Cargo.lock`, this task Report/status, `tasks/DECISIONS.md`, and generated `tasks/INDEX.md`.
**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `defaults_schema_and_summary_match_the_policy_contract` | Pass (1/1). |
| AC2 | `deserialization_validates_value_ranges_and_rejects_unknown_fields` | Pass (1/1). |
| AC3 | `cross_field_limit_reports_the_contract_pointer_and_code` | Pass (1/1). |

**Checks run** — `cargo fmt --all --check` ✅; `SQLX_OFFLINE=true cargo test -p eq-audit --lib` ✅ (3/3); `SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings` ✅; `env -u RITMA_TEST_DATABASE_URL SQLX_OFFLINE=true cargo test --workspace -- --test-threads=1` ✅ (209 passed, 0 failed); `cargo deny check` ✅ (advisories, bans, licenses and sources pass; existing duplicate-version and unmatched-CC0 warnings remain); `scripts/check-deps.sh`, `scripts/check-literals.sh`, `scripts/check-web-boundaries.sh`, `scripts/check-layers.sh` ✅; `python3 -I scripts/erd.py --check` ✅; `python3 -I scripts/tasks-lint.py` ✅; `python3 -I -m unittest discover -s scripts/tests -p 'test_tasks_lint.py'` ✅ (8/8); `scripts/tasks-index.sh --check` ✅; `git diff --check` ✅; `gitleaks dir --redact --no-banner` run separately for each of the seven changed paths ✅. A multi-path invocation returned one known repository-wide false positive in `Makefile:80-85`; the individual changed-path scans were clean. The compile, clippy and workspace test checks completed before the disk-hygiene instruction arrived; the code has not changed since. `cargo fmt --all --check` was rerun during this continuation and passed. Final clippy/test reruns are **Not run** because `$HOME/.cache/ritma/target` and repository `target/` are absent, and the disk rule prohibits creating a new target directory. `cargo sqlx prepare --workspace` **Not run** (no SQL query or macro changed). Web lint/typecheck/test/i18n, `make dev`, browser E2E and `make e2e` **Not run / n/a** because this pure policy contract adds no web, API, database or runtime behavior. ERD `--migrations` **Not run / n/a** because no migration changed.
**Manual checks** — Read the policy implementation and contract against the parent T-109 policy stanza and existing `PolicyDoc`/`XUi` definitions; confirmed fields are private, the category enum is closed, numeric bounds are enforced before construction, and cross-field validation remains in `PolicyDoc::validate`. No server behavior or UI flow exists in this task.
**Debug log** — The first policy test compile failed because a test lacked the schema-field constant import; the import was added and the three policy tests passed. Clippy then reported `manual_let_else`; the serialization schema handling was changed to `let-else`, after which workspace clippy passed. The full workspace suite later passed 209/209. No check was retried without a code change.
**Performance** — n/a; pure policy value validation and schema construction, with no database query or runtime endpoint.
**Security** — Trigger: audit retention/data lifecycle (SBD-30 T4). The task adds no S2/S3 field storage or endpoint. Private fields, a validating constructor, closed `Category` deserialization, `deny_unknown_fields`, and cross-field validation enforce the policy contract. Abuse cases for range violations, unknown fields, and invalid retention/anonymization ordering are covered by the named policy tests. No SQL, external call, secret, PII logging, permission or role grant is added. SBD-81 reviewed: auth/tenant/data output/crypto/external-call/integrity/audit checklist items are n/a for this pure type; input validation and abuse-test items are satisfied.
**Deviations** — RATIFY TD-079: preserve POL-10's schema→serde→cross-field validation order so `must_be_lte` retains its field pointer. RATIFY TD-080: assign schema order 1, 2, 3 in the policy field declaration order because `XUi` requires an order and the task gives none. Disk rule: Cargo build checks are not rerun because the fixed target cache is absent; no replacement target, container, image, volume or test database was created in this continuation. The exact earlier Testcontainers activity and cleanup is recorded below.
**Dependencies affected** — T-109d consumes `AuditPolicy`; later audit parts use its `Category` and policy contract. No dependency interface is changed.
**Rollback plan** — Revert this task commit; no migration or persisted policy is introduced.
**Follow-ups** — T-109d onward implement policy persistence and module wiring. Hosted CI remains a maintainer-run check.

**Workflow steps 0–12** — Step 0: continued the existing T-109c work on `dev`; T-109b was `review`, this was the sole `in_progress` task, and the task index was checked. Step 1: read the complete T-109c contract, parent T-109 policy stanza and cited specs. Step 2: read T-109b's Report/deviations/follow-ups and its shipped manifest, migration and public crate surface. Step 3: n/a, no UI. Step 4: wrote the plan and AC/edge/error/abuse test map before implementation. Step 5: implemented the policy value type, closed category enum, validation, schema metadata, summaries and named tests. Step 6: n/a for `make dev`; exercised behavior through the policy tests. Step 7: fixed the test import and clippy finding; all named tests and clippy passed. Step 8: reviewed against CQ-23/CQ-101, CA-01/30, ERR-21/31 and the task scope. Step 9: n/a; no measurable hot path. Step 10: reviewed SBD-81; no auth, tenant SQL, secrets, PII output or external call was added. Step 11: Rust and task/guard checks passed before the disk rule arrived; final non-build checks are recorded in the run log; Rust build reruns are Not run due the absent fixed cache. Step 12: completed this Report, regenerated the task index and prepared the single signed commit.

**Disk hygiene** — The required pre-task disk snapshot was not taken before the earlier workspace tests; the first available snapshot was after those tests and before this continuation. At that snapshot, `du -sh "$HOME/.cache/ritma/target"` produced no output (exit 1; path absent); `docker system df` reported Images 21 / 20 active / 19.1GB / 3.595GB reclaimable; Containers 77 / 8 active / 761.2MB / 750.7MB reclaimable; Local Volumes 434 / 60 active / 73.49GB / 61.08GB reclaimable; Build Cache 0. After validation, `du -sh "$HOME/.cache/ritma/target"` again produced no output (exit 1; path absent). `docker system df` reported Images 20 / 7 active / 15.5GB / 2.073GB reclaimable; Containers 8 / 8 active / 10.48MB / 0B reclaimable; Local Volumes 434 / 5 active / 73.85GB / 67.08GB reclaimable; Build Cache 0. A concurrent external maintenance process had removed unrelated exited containers and one image during this interval; volume usage rose by 0.36GB, and the cause is not attributable to this task. No cleanup of non-owned volumes was attempted. The earlier workspace suite had created 12 Testcontainers containers (`jovial_dewdney`, `interesting_shirley`, `gifted_kepler`, `gifted_maxwell`, `compassionate_heyrovsky`, `thirsty_lederberg`, `jolly_grothendieck`, `inspiring_cori`, `sad_banach`, `strange_perlman`, `romantic_bose`, `pensive_hoover`) using the pre-existing `postgres:16-alpine` image; all 12 containers and their anonymous volumes were removed with `docker rm -f -v` after the disk rule arrived. No container, image, volume, target directory or test database was created in this continuation. The 84 per-test databases created inside those temporary containers were: `ritma_test_01a182eb6c24742389bc39afd5113eae`, `ritma_test_01a182eb6e77758ba76a5cd23d58782c`, `ritma_test_01a182eb7399770ba6e2c848b1b1b07e`, `ritma_test_01a182eb7bef750b84e3706a820f893c`, `ritma_test_01a182eb7dec74eca5ba95a0f9991ef1`, `ritma_test_01a182ec017b74b6a7daf6e8e51bf23f`, `ritma_test_01a182ec042d75c29586f4d6133bf7fd`, `ritma_test_01a182eca5cb7015a133ef4cec7984b9`, `ritma_test_01a182ed524a76689808fa84c216e0b3`, `ritma_test_01a182ed553170639ebcb7b4ed594d50`, `ritma_test_01a182ed5a8070b7a15e3ae114515496`, `ritma_test_01a182ed5da77379b94914af7efce15d`, `ritma_test_01a182ed6033772cb2f55c12e4014e0e`, `ritma_test_01a182ed648575e2bb57f45ce0e98337`, `ritma_test_01a182ed6d8a76cca1dba65c90a7e136`, `ritma_test_01a182ed6fd5768b9cc6c836d6fd1df0`, `ritma_test_01a182ed74877137a14c343e5556f308`, `ritma_test_01a182ed7dd072d1ac088c3e887efdff`, `ritma_test_01a182edfd5f73d8bc178786288c9e38`, `ritma_test_01a182ee02257375994f0f6c8cc6e4d2`, `ritma_test_01a182ee091a73fd9a7136880718bbe6`, `ritma_test_01a182eea653714f93c2ce3f390e9ea4`, `ritma_test_01a182eea8c0777d8874e70e71772478`, `ritma_test_01a182eeb2247109bf2d98ed620a5e80`, `ritma_test_01a182eeb9847186930f6b9852f433aa`, `ritma_test_01a182eebbaf7439a939d9ac963bf631`, `ritma_test_01a182eec0a173f3935887ced0397824`, `ritma_test_01a182eec5817525b99c335ac7c953c2`, `ritma_test_01a182eec901732f90012bf7a1e3d44a`, `ritma_test_01a182eecda0702db788dc6f8ae4de7b`, `ritma_test_01a182eed01970918fbad27e6ebb42ca`, `ritma_test_01a182ef753c72e28a2396e6814c2a46`, `ritma_test_01a182ef774e7384856ff8a4ca9e7ab2`, `ritma_test_01a182ef83ca7063a292e2125c8ee69b`, `ritma_test_01a182ef8703719d8bcfca1143037654`, `ritma_test_01a182f01d39732d9facbc15e4698ec1`, `ritma_test_01a182f01f357552be3cade3863f8c5d`, `ritma_test_01a182f024a67160ba008c57af5a8a30`, `ritma_test_01a182f0280c738a93be08fbb0fe74a0`, `ritma_test_01a182f02a417068889d1d66fd27a557`, `ritma_test_01a182f02f4770d49c73681b50d31689`, `ritma_test_01a182f030ca70649426ba6220f5e7e6`, `ritma_test_01a182f0334e7018bec2bd6064494e8a`, `ritma_test_01a182f05e61710a824bb622b4ee3c6f`, `ritma_test_01a182f060aa737e80238e120bed58ba`, `ritma_test_01a182f0b80475f1a2d63caffe037a59`, `ritma_test_01a182f0bd8a72b5988a333a5a66480f`, `ritma_test_01a182f0bfb876ceb5312d37f001599e`, `ritma_test_01a182f0c22876f6b636c745ab7579a4`, `ritma_test_01a182f0c5747383b45dc04e675dbe16`, `ritma_test_01a182f0ed7d737e8661797464d16fe4`, `ritma_test_01a182f1400c75b0b153d6a7db51e3c5`, `ritma_test_01a182f144e675768c7295982ecbd725`, `ritma_test_01a182f1a43f719b98797c17cec1304b`, `ritma_test_01a182f1a63171f4b29a38424b2a467a`, `ritma_test_01a182f1a9347602b5664b00b27e6ecc`, `ritma_test_01a182f1b0dd7766b7b1c5eed2a2de0d`, `ritma_test_01a182f1b30674eda54a578dfd9e2235`, `ritma_test_01a182f1b625741ca4831c1a08b17cb7`, `ritma_test_01a182f1bbd576438c3d1f7e69a7798b`, `ritma_test_01a182f1bdd075ccaa005f8bea242a1e`, `ritma_test_01a182f1c0c3710bb690060768a2df49`, `ritma_test_01a182f1c6257780b90d7dab2cf5905e`, `ritma_test_01a182f1c8aa749a89544e03799615ca`, `ritma_test_01a182f1cd5b73909ab97a92ae5db3ac`, `ritma_test_01a182f1d0ca71508c436de05120a378`, `ritma_test_01a182f1db00762b9ec9e8d6bc2054e0`, `ritma_test_01a182f1de15716f8a66c89bcb16097e`, `ritma_test_01a182f1e207766eb90413b181621d64`, `ritma_test_01a182f27832717cab75a92ba1c9ef6b`, `ritma_test_01a182f31289750b8ec9bbe0bce834d3`, `ritma_test_01a182f315d47316b9c3b1ea19f2d40b`, `ritma_test_01a182f3a48e742182dce7768c51910b`, `ritma_test_01a182f3a69f773a8893540cb344676d`, `ritma_test_01a182f3a8ae742182dce7768c51910b`, `ritma_test_01a182f477b37613a45e2a834761b907`, `ritma_test_01a182f479667736bbf7e25bee533c9c`, `ritma_test_01a182f4c630724ba3103cdbc031195b`, `ritma_test_01a182f4c7f271dba66a3e5f21ac0f1c`, `ritma_test_01a182f61c957151b868e79c0eaf498e`, `ritma_test_01a182f61e4874f0a273d2446177cc92`, `ritma_test_01a182f61e4874f0a273d245c57d3c93`, `ritma_test_01a182f623dc70068d07e5ee2df538bf`, `ritma_test_01a182f6260471599722a0f30ff8f9e3`. No real credentials or user data were involved.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification; TD-079):** a broad CQ-23/CAN-10 reading could reject `anonymize_pii_after_years > retention_years` in `new`/Deserialize, but that would make POL-10's deserialization stage return root `invalid_type` instead of the contract's `must_be_lte` pointer → keep range validation in `new`/Deserialize and check the cross-field rule in `PolicyDoc::validate` after deserialization, as POL-10 and parent T-109 specify.
- **Self-resolved (implementer, run 6, needs architect ratification; TD-080):** the task requires `x-ui: advanced` but gives no schema order while `XUi::new` requires one → use the fields' declaration order (1, 2, 3) because this is deterministic presentation metadata and does not add a business rule.

## Review
n/a until review.
