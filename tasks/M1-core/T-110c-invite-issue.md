---
id: T-110c
title: Issue and resend identity invites
milestone: M1
status: blocked
depends_on: [T-103, T-110b]
spec_refs: [IDN-07, STM-21, AUTH-06]
estimated_size: M
---

## Goal
Create a single-use invite token for a person and role, expire any earlier pending invite for that pair, and expose the link channel result to the authorized caller.

## Context
- T-111 defines create_invite(tx, actor, person_id, role, channel), a 32-byte random token with SHA-256 stored, expiry from IdentityPolicy.invite_expires_days, membership status invited (insert if missing), event identity.user_invited.v1 and audit identity.invite.created. TD-135 adds the raw token to the server-side outbox payload so Notify can construct the specified invite URL.
- The invite URL uses RITMA_PUBLIC_URL and the person's locale, falling back to the tenant locale. Channel link returns the invite URL; email/whatsapp do not return it because Notify delivers it in M5.
- Only one pending invite for a person/role remains valid after resend. Token material is S3 and is never logged or returned except as the link-channel URL.

## Scope
### In
- Implement invite issuance/resend as an Identity service callable by T-112, including the invite persistence, membership transition through the Core-owned port, outbox event, audit and link-channel result.
- Keep the exact event name; apply the payload correction in TD-135.
### Out
- Public invite preview and acceptance routes (T-110d).
- People administration routes (T-112).
- Sending email or WhatsApp (M5 Notify).

## Files
| Path | Action | Purpose |
|---|---|---|
| crates/eq-identity/src/invites.rs | create | Invite issuance service and secret-safe result types. |
| crates/eq-identity/src/lib.rs | modify | Export the service contract consumed by T-112. |
| crates/eq-identity/src/repo.rs | modify | Tenant-scoped invite persistence. |
| crates/eq-core/src/auth/directory.rs | modify | Core-owned invited-membership upsert through the existing port. |
| crates/eq-identity/tests/api_invites.rs | create | Resend behavior acceptance test. |
| tasks/M1-core/T-111-invites-reset-me-totp.md | modify | Record the aggregate split and final report. |
| tasks/M1-core/T-110d-invite-acceptance.md | create | Invite acceptance part of the TD-134 split. |
| tasks/M1-core/T-110e-password-reset.md | create | Password reset part of the TD-134 split. |
| tasks/M1-core/T-110f-profile-and-sessions.md | create | Profile and sessions part of the TD-134 split. |
| tasks/M1-core/T-110g-totp-enrolment.md | create | TOTP enrollment part of the TD-134 split. |
| tasks/M1-core/T-110h-recovery-code-login.md | create | Recovery-code login part of the TD-134 split. |
| tasks/M1-core/T-110c-invite-issue.md | modify | Record plan, checks and report. |
| tasks/DECISIONS.md | modify | Record TD-134 and any task decisions. |
| tasks/INDEX.md | generate | Regenerate task status and dependency index. |

## Interfaces
- Provide the T-111 create_invite service behavior using the shipped Core Tx, ActorCtx, PersonId and Role types, and a closed channel value link/email/whatsapp.
- Return invite_url only for channel link. Preserve the event identity.user_invited.v1 name.

## Business rules
- Invite expiry comes only from IdentityPolicy.invite_expires_days.
- Revoke prior unaccepted invites for the same tenant/person/role by setting expires_at to the injected Clock time.

## Security
- Triggers: T1, T3, T4.
- Data classes touched: S3 invite token hash; S1 identity, membership and event ids.
- S/E: tenant or role spoofing → caller permission and actor tenant scope; membership write goes through Core owner port (SBD-40/41/42, CA-30).
- I: token disclosure → persist only SHA-256, redact Debug, never log token or URL, and return URL only for the link channel (SBD-10/21/23, SBD-50).
- Abuse-case acceptance criterion: resend_revokes_previous proves the prior token is no longer valid after a resend.

## Implementation notes
Use a cryptographic random source and the existing Clock, IdentityPolicy, Audit and Outbox APIs. Keep SQL in the Identity repository; do not query Core-owned memberships from Identity SQL.

## Acceptance criteria
- AC1. Given an existing pending invite, when another invite is issued for the same person and role, then the old invite is expired and only the link channel exposes the new URL → resend_revokes_previous.

## Tests to write
| File | Test names |
|---|---|
| crates/eq-identity/tests/api_invites.rs | resend_revokes_previous |

## Definition of done
- [ ] The exact acceptance test passes on the existing PostgreSQL 16 container using the task-scoped database ritma_test_T-111.
- [ ] Rust formatting, Clippy and focused tests pass; cargo-deny and guard scripts pass.
- [ ] Code quality and architecture follow spec/16 §12 and spec/17, with no new #[allow].
- [ ] OpenAPI generation is attempted and any T-114 placeholder is recorded as Not run; SQLx metadata is updated if queries change.
- [ ] The implementation and report are committed with sign-off.

## Report
**Plan** — (1) Read T-103 and T-110b Reports and shipped code → verified crypto, Clock, audit/outbox, tenant and transaction interfaces. (2) Read Identity's ERD and invite DDL → verified ownership and exact columns. (3) Add `resend_revokes_previous` first → verify it is red before implementation. (4) Reconcile the event payload against CAN-60 and FLW-60 before writing the service. (5) Stop implementation when the existing TD-135 contract conflicts with the security specs; report the blocking handoff question and preserve the task split.

**Assumptions** — None about how Notify receives a usable invite token: the task and its TD-135 decision conflict with the security contract, so that handoff needs an architect decision. A resend would reuse the transaction, tenant, actor, clock, and policy supplied by the caller. The fixed Cargo target and existing ritma-db-1 PostgreSQL 16 container were reused; no task database was created.

**Test plan** — Given one pending invite for a person/role, when a second invite is created at a later injected time, then the first is expired and the second URL is returned only for link channel → `resend_revokes_previous`. Edge/error cases: no previous invite, accepted invite history, tenant mismatch, email/WhatsApp channel response contains no URL, and secret Debug redaction. Abuse case: replay of the first invite token must be rejected by the invite lookup owned by T-110d. No behavior test can run until the task's outbox-token conflict is resolved.

**Design reference (Step 3)** — n/a; this is a backend-only task.

**Implementation and runtime (Steps 5–7)** — Blocked before implementation by Q1. The test-first compile was attempted, then the temporary test file was removed. No service or runtime request was added.

**Self-review and optimization (Steps 8–9)** — Reviewed the proposed contract against spec/16 §9, spec/17 and spec/19; the CAN-60/FLW-60 conflict blocks implementation, and there is no code to optimize.

**Deviations** — RATIFY TD-134 split the oversized parent contract into six ordered parts. TD-135 proposes adding the raw token to the invite outbox event, but this task does not adopt it because CAN-60 and FLW-60 prohibit that payload; Q1 asks for a compliant handoff. RATIFY TD-136 assigns part ids T-110c–T-110h so they sort before aggregate T-111. No new event name or schema object is introduced.

**Summary** — T-110c is blocked before implementation. TD-135 requires the raw S3 invite token in `identity.user_invited.v1`, but the event schema and SBD-10/CAN-60/FLW-60 prohibit secret tokens in events. No source code, migration, test database or Docker resource was created.

**Spec read** — `spec/03-module-system.md` §6; `spec/06-domain-model.md` §§4.1–4.2; `spec/18-security-by-design.md` §§5, 8; `spec/19-error-handling.md` §10; `spec/20-state-machines.md` STM-20/21; `spec/21-data-flows.md` FLW-12/60; `spec/23-contracts-and-canonical-forms.md` CAN-60/80; `spec/25-auth-and-access-control.md` AUTH-06 and §4.5; `spec/29-erd.md` Identity and Core dictionaries, §5 and §7; `spec/16-code-quality.md` §12; `spec/17-clean-architecture.md` CA-30..33.

**Files** — `tasks/M1-core/T-110c-invite-issue.md` (blocked Report and Q1); `tasks/M1-core/T-111-invites-reset-me-totp.md` (aggregate split from earlier work); `tasks/M1-core/T-110d-invite-acceptance.md`, `T-110e-password-reset.md`, `T-110f-profile-and-sessions.md`, `T-110g-totp-enrolment.md`, and `T-110h-recovery-code-login.md` (ordered parts from the TD-134 split); `tasks/DECISIONS.md` (TD-134–136); `tasks/INDEX.md` (generated). The temporary red test file was removed before blocking; no code file remains changed.

**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1. Resend expires the previous pending invite and only link returns the URL | `resend_revokes_previous` | Not run: blocked before implementation by Q1. |

**Checks run** — ✅ `docker ps --format '{{.Names}} {{.Image}} {{.Status}}'` confirmed the existing `ritma-db-1` PostgreSQL 16 container was already running. ✅ `docker exec ritma-db-1 pg_isready -U eq -d ritma`. ✅ `docker system df` before and after: 20 images / 15.5GB, 24 containers / 10.51MB, 450 volumes / 75.28GB, build cache 0. ✅ `du -sh /home/kepolu/.cache/ritma/target` before and after: 14G. ✅ Database inventory before and after was unchanged (`postgres`, `ritma`, `ritma_test_T-110a4b`). ❌ Expected RED: `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --locked --offline -p eq-identity --test api_invites --no-run` did not compile because `eq_identity::invites` is not implemented yet. ✅ `scripts/tasks-index.sh` regenerated `tasks/INDEX.md` (170 tasks). ❌ `scripts/tasks-index.sh --check` and `python3 -I scripts/tasks-lint.py` reject the required `blocked` status (`status blocked is not allowed`); neither guard was weakened. ✅ `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh`. ✅ `scripts/erd.py --check`. ✅ `cargo deny check` (advisories, bans, licenses and sources passed; it emitted duplicate lock-entry warnings). ✅ `gitleaks detect --source tasks/M1-core/T-110c-invite-issue.md --no-git --redact` (no leaks). ✅ `git diff --check`. Not run: Rust format/Clippy/full tests, the exact PostgreSQL acceptance test, `make dev`, `make api` (T-114 placeholder; blocked before endpoint implementation), `make e2e`, and SQLx prepare (no query changed). No task database or Docker resource was created.

**Manual checks** — Read-only source review confirmed the existing outbox contract has `{person_id, role, invite_id}`; no runtime request could be exercised without implementing the service. No `make e2e` run: this service-only task is blocked before runtime implementation.

**Debug log** — The test-first compile stopped at the expected unresolved import `eq_identity::invites`; no implementation attempt followed. Independent contract review then identified Q1, so this was not treated as an implementation failure.

**Performance** — n/a; no runtime implementation. Fixed Cargo target remained 14G. Docker system totals and database inventory were unchanged before/after.

**Security (Step 10)** — Triggers: T1, T3, T4. S3 invite tokens must remain hash-only at rest and must not enter logs, audit or events (SBD-10/49, FLW-60). CAN-60 limits `identity.user_invited.v1` to minimal IDs/facts and prohibits S2/S3 values. SBD-81: (1) auth chain — no handler added; (2) tenant — no table/query added; (3) data class — token remains S3 and the proposed event token is not adopted; (4) input — no DTO added; (5) output — no URL/API response added; (6) crypto — no token implementation added; (7) external calls — none; (8) integrity — no state mutation added; (9) audit — no event emitted; (10) abuse tests — `resend_revokes_previous` not run because the compliant token handoff is unresolved. No PII or secret was written to the repository.

**Deviations** — RATIFY TD-134 and TD-136 describe the prior task split. RATIFY TD-135 proposes adding a raw S3 token to the invite event; this task did not implement that proposal because it conflicts with CAN-60 and FLW-60. Q1 asks the architect to choose a compliant handoff. No schema, event name or error code was changed.

**Dependencies affected** — T-110d, T-110e, T-110f, T-110g, T-110h and aggregate T-111 are ineligible while T-110c is blocked. T-112, T-112a and T-115 depend on T-111 and are consequently ineligible.

**Rollback plan** — No code or database state changed. Revert the signed blocked-task commit after the architect resolves Q1 and the task returns to `todo`.

**Follow-ups** — Define a durable, tenant-safe, S3-safe handoff from Identity to the future Notify consumer that preserves CAN-60/FLW-60 and the hash-only invite schema. Until then, email/WhatsApp delivery cannot be completed safely from the event contract.

## Questions
- **Q1 (blocking):** What durable handoff lets M5 Notify send a usable invite URL without placing the raw S3 token in an event or adding an undocumented schema field?
  - Context: TD-135 adds `token` to `identity.user_invited.v1`, but `spec/03-module-system.md` §6 defines only `{person_id, role, invite_id}`; CAN-60 prohibits S2/S3 event values; FLW-60 explicitly excludes invite tokens from events; `spec/06` and ERD define only `token_hash` in `invites`.
  - Options: (a) revise the event/security contract and define an explicit protected token handoff; (b) specify a separate secure delivery port/storage contract that keeps the token out of events and the invite table hash-only; (c) defer email/WhatsApp invite delivery until the handoff is designed.
  - Recommendation: (b), preserving CAN-60/FLW-60 and the existing hash-only schema.
- **Self-resolved (implementer, run 15, needs architect ratification; TD-134):** T-111's single M-sized task exceeds CQ-101 when its six security workflows and exact tests are combined → implement ordered parts T-110c–h and keep the parent as the aggregate, because CQ-101 requires larger work to be split.
- **Self-resolved (implementer, run 15, needs architect ratification; TD-136):** child IDs T-111a–f sort after parent T-111 and fail the task index's dependency-order rule → use T-110c–h after the existing T-110b, because tasks/README.md requires inserted tasks to use the lower neighbor's id plus a suffix.

## Review
n/a — blocked pending the architect's answer to Q1 and implementation of the compliant handoff.
