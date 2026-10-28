---
id: T-506
title: Add the notify schema, NotifyPolicy, the Notifier port implementation and the delivery pipeline
milestone: M5
status: todo
depends_on: [T-416, T-505]
spec_refs: [NTF-01, NTF-02, NTF-09, NTF-10]
estimated_size: M
---

> **Draft.** This task is complete in intent but cites spec sections instead of inlining them. Before it moves to `todo`, the maintainer (or the spec author) inlines the cited DDL, structs and rules, re-checks it against what M1–M3 actually shipped, and removes this note. Codex must not start a `draft` task.

## Goal
Every module asks for a message the same way, and notify owns delivery: who, which channel, when, and what happened.

## Context
- DDL `spec/06-domain-model.md` §4.8 (notifications, deliveries, breaker_states, channel_configs, notification_prefs) verbatim into `migrations/0050_notify.sql`; RLS.
- Policy `NotifyPolicy` exactly as `spec/04-policy-engine.md` §6.8 (routing, retry, circuit_breaker, quiet_hours, session_reminders_minutes_before, aggregate_per_payer).
- NTF-01 request API `Notifier::request(tx, NotificationRequest{category critical|standard, template, recipient, args, dedupe_key, aggregate_key, send_after})` replaces the no-op from T-416. Dedupe per key (module AC6).
- NTF-02 pipeline as jobs: plan → render → deliver per channel. NTF-09 addresses decrypted at send time only. NTF-10 opt-out for standard only.
- On disable (MOD-08): suppress and log `suppressed`.

## Scope
### In
- Migration, policy, port impl, pipeline skeleton with an in-app channel (always available), prefs API

### Out
- Email/WhatsApp (T-507), routing logic (T-508)

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-notify/tests/pipeline.rs` | test | Task-listed verification file. |
| `migrations/0050_notify.sql` | create or modify | Path named in this task's contract. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T4 (SBD-30).
- Data classes touched: S1 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- Abuse-case acceptance criterion: `dedupe` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. AC6 dedupe → `notify::dedupe`
- AC2. Disabled notify → suppressed rows, no deliveries → `disabled_suppresses`
- AC3. Opt-out applies to standard, not critical → `opt_out_rules`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-notify/tests/pipeline.rs` | AC names |

## Definition of done
- [ ] `scripts/erd.py --migrations` passes (ERD-03): every table and column this task migrates matches the `spec/06` DDL and the task's row in `spec/29-erd.md` §7
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] All checks for the touched stacks pass (Rust and/or web), guard scripts pass, `make api` regenerated if the API changed
- [ ] `scripts/check-literals.sh` passes: every business value comes from the module policy
- [ ] Commit `feat(notify): notifier and pipeline [T-506]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
