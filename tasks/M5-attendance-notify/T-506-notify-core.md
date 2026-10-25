---
id: T-506
title: Add the notify schema, NotifyPolicy, the Notifier port implementation and the delivery pipeline
milestone: M5
status: draft
depends_on: [T-505]
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

## Questions
