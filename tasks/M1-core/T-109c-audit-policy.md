---
id: T-109c
title: Add the versioned audit retention policy
milestone: M1
status: todo
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
| `crates/eq-audit/Cargo.toml`, `crates/eq-audit/src/lib.rs`, `crates/eq-audit/src/policy.rs` | modify/create | Policy contract and exports. |
| `tasks/M1-core/T-109c-audit-policy.md` | modify | Report and status. |

## Interfaces
`eq_audit::policy::AuditPolicy`; fields stay private and deserialize through a validating constructor (CQ-23/CAN-10). Preserve the task's exact summary keys and validation pointer.

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
- [ ] Policy tests and Rust format/clippy checks pass.
- [ ] No public policy field can be constructed/deserialized invalidly.
- [ ] Signed commit `feat(audit): add audit retention policy [T-109c]`.

## Report
**Plan** — Implement the constrained policy value type and exact `PolicyDoc` behavior; test defaults, serde and cross-field errors.
**Assumptions** — Parent T-109 specifies every business value; no new default is inferred.
**Test plan** — AC1–AC3 map to the three named unit tests above.
**Summary** — Pending implementation.
**Spec read** — `spec/16-code-quality.md` §12; `spec/23-contracts-and-canonical-forms.md` CAN-10; parent T-109 policy contract.
**Files** — Pending.
**Acceptance criteria → tests** — Pending.
**Checks run** — Pending.
**Manual checks** — n/a; policy unit tests exercise the boundary.
**Debug log** — Pending.
**Performance** — n/a.
**Security** — SBD-81 pending.
**Deviations** — None beyond parent T-109 RATIFY items.
**Dependencies affected** — T-109d and later parts.
**Rollback plan** — Revert this part commit; no stored policy migration is added.
**Follow-ups** — None identified.

## Questions
None.

## Review
n/a until review.
