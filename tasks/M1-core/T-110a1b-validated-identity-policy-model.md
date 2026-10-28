---
id: T-110a1b
title: Add validated identity policy values
milestone: M1
status: todo
depends_on: [T-110a1, T-106]
spec_refs: [IDN-01, IDN-02, IDN-03, IDN-04, IDN-06, POL-01, POL-10, CQ-23, CAN-10, CAN-64]
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
- Triggers: T1, T3 (SBD-30).
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
| `crates/eq-identity/src/policy.rs` | `invalid_values_rejected_on_deserialize`, `out_of_range_timeout_rejected` |

## Definition of done
- [ ] Both named tests pass, or are explicitly Not run under the fixed-target resource restriction.
- [ ] All bounds, defaults and serde rejection follow spec 04/CAN-64; fields remain private.
- [ ] `cargo fmt --all --check`, `cargo deny check`, applicable guards and task checks pass.
- [ ] Report includes Workflow steps 0–12 and resource snapshots.
- [ ] Signed commit `feat(identity): add validated policy values [T-110a1b]` pushed to `dev`.

## Report
**Plan** — Add the closed policy values and ranges, route serde through constructors, implement the typed policy contract, then run formatting, dependency and source checks.
**Assumptions** — The existing `PolicyDoc` trait and `RoleMap<T>` shipped by T-110a1 are authoritative.
**Test plan** — Exercise inclusive boundaries and min−1/max+1 for every numeric field; reject unknown enum values and unknown fields.

## Questions
- **Self-resolved (implementer, run 7, needs architect ratification; TD-097):** the parent policy sketch exposes bounded `IdentityPolicy`/`Lockout` fields directly, while CQ-23 and CAN-10 require invalid value types to use private fields and validating construction/deserialization → keep those fields private, validate through constructors, and deserialize through the same validation path; retain public `RoleMap<T>` role fields because that container has no independent invariant.

## Review
n/a — pending implementation.
