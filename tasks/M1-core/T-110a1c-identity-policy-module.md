---
id: T-110a1c
title: Register identity policy schema and summaries
milestone: M1
status: todo
depends_on: [T-110a1b, T-106]
spec_refs: [IDN-01, IDN-02, IDN-03, IDN-04, IDN-06, POL-01, POL-06, POL-07, POL-10, CAN-64]
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
| `tasks/M1-core/T-110a1c-identity-policy-module.md`, `tasks/INDEX.md` | modify/generate | Report and index. |

## Interfaces
Implement the existing `eq_core::module::Module` interface. `policy_schema`, `default_policy`, `validate_policy`, and `summarize_policy` delegate to the T-110a1b policy contract. Module ID is Identity and schema version is 1.

## Business rules
Do not add permission keys, grants, policy fields or routes. All settings use the exact parent/spec defaults and summary keys.

## Security
- Triggers: T1, T3 (SBD-30).
- The module exposes only validated policy configuration and its specified summary values; credentials remain outside policy output.
- Unknown and out-of-range policy documents fail validation; no authentication endpoint is added here.

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
- [ ] The named policy/module test passes or is explicitly Not run under the fixed-target resource restriction.
- [ ] `cargo fmt --all --check`, `cargo deny check`, applicable guards and task checks pass.
- [ ] No route, permission, grant, schema or business value is added.
- [ ] Report includes Workflow steps 0–12 and resource snapshots.
- [ ] Signed commit `feat(identity): register policy schema [T-110a1c]` pushed to `dev`.

## Report
**Plan** — Implement the existing module contract for Identity, verify schema metadata/defaults/summaries, then run formatting, dependency, guard and task checks.
**Assumptions** — The task's existing manifest identity (`identity`, always on) and the T-110a1b `PolicyDoc` contract are authoritative.
**Test plan** — Compare generated schema ranges and x-ui metadata, default policy values, module metadata and both summary keys.

## Questions
None.

## Review
n/a — pending implementation.
