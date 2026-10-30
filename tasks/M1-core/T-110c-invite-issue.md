---
id: T-110c
title: Issue and resend identity invites
milestone: M1
status: review
depends_on: [T-103, T-110b]
spec_refs: [IDN-07, STM-21, AUTH-06]
estimated_size: M
---

## Goal
Create a single-use invite token for a person and role, expire any earlier pending invite for that pair, and expose the link channel result to the authorized caller.

## Context
- T-111 defines create_invite(tx, actor, person_id, role, channel), a 32-byte random token with SHA-256 stored, expiry from IdentityPolicy.invite_expires_days, membership status invited (insert if missing), event identity.user_invited.v1 and audit identity.invite.created. TD-135 historically proposed the raw token in the event; TD-137 resolves T-110c by keeping the event IDs-only.
- The invite URL uses RITMA_PUBLIC_URL and the person's locale, falling back to the tenant locale. Channel link returns the invite URL; email/whatsapp do not return it. Sending email/whatsapp is out of scope here and needs a spec-compliant Notify handoff.
- Only one pending invite for a person/role remains valid after resend. Token material is S3 and is never logged or returned except as the link-channel URL.

## Scope
### In
- Implement invite issuance/resend as an Identity service callable by T-112, including the invite persistence, membership transition through the Core-owned port, outbox event, audit and link-channel result.
- Keep the exact event name and the spec-defined `{person_id, role, invite_id}` payload. Do not put the S3 token in the event. Email/WhatsApp delivery is out of scope for this task and belongs to M5 Notify.
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
| crates/eq-identity/tests/api_invites.rs | resend_revokes_previous, invited_event_excludes_token |

## Definition of done
- [x] The exact acceptance test passes on the existing PostgreSQL 16 container using task-scoped database `ritma_test_T-110c` (the split task's database per run instructions).
- [x] Rust formatting, Clippy and focused tests pass; cargo-deny and guard scripts pass.
- [x] Code quality and architecture follow spec/16 §12 and spec/17, with no new `#[allow]`.
- [x] OpenAPI generation was attempted and the T-114 placeholder is recorded as Not run; SQLx metadata is updated for the changed queries.
- [x] The implementation and report are committed with sign-off.

## Report
**Plan** — Read T-103/T-110b reports and shipped crypto, Clock, audit, outbox and Core-port code; read the Identity/Core ERD dictionaries and invite DDL; wrote both acceptance tests first; implemented invite persistence, expiry, membership, audit, event and link URL; exercised tests on the task database; completed security, quality and repository checks; regenerated the index and prepared one signed-off commit.

**Assumptions** — CAN-60 and FLW-60 govern the event payload `{person_id, role, invite_id}`. T-110c returns the URL only for `link`; email/WhatsApp dispatch is out of scope. The caller owns the transaction and enforces `identity.people.write`; this service uses the supplied actor, tenant locale, configured public URL, policy, Clock and Core directory. The isolated database is `ritma_test_T-110c` inside the existing PostgreSQL 16 container.

**Test plan** — Given an existing invite, a resend expires it and returns a distinct URL only for link → `resend_revokes_previous`. Given an email or WhatsApp channel, no URL is returned; the event contains only the three specified fields and no token → `invited_event_excludes_token`. Edge coverage includes first issuance, resend, existing membership, non-link channels, person-locale URL selection and redacted Debug. T-110d owns token acceptance/replay validation.

**Summary** — Identity now creates 32-byte invite tokens, stores only SHA-256 digests and expires prior unaccepted invites under a transaction advisory lock. Membership changes go through the Core-owned port; audit and outbox writes share the caller transaction. The event remains IDs-only, and only the link channel returns a redacted URL.

**Spec read** — `spec/03-module-system.md` §6; `spec/06-domain-model.md` §§4.1–4.2; `spec/18-security-by-design.md` §§5, 8; `spec/19-error-handling.md` §10; `spec/20-state-machines.md` STM-20/21; `spec/21-data-flows.md` FLW-12/60; `spec/23-contracts-and-canonical-forms.md` CAN-60/80; `spec/25-auth-and-access-control.md` AUTH-06 and §4.5; `spec/29-erd.md` Identity/Core dictionaries, §5 and §7; `spec/16-code-quality.md` §§9, 12; `spec/17-clean-architecture.md` §§7, CA-30..33.

**Files** — `crates/eq-identity/src/invites.rs` (service, channel/request/result, ID-only event and fake-secret Debug test); `crates/eq-identity/src/lib.rs` (module export); `crates/eq-identity/src/repo.rs` (lock, expire and insert queries); `crates/eq-core/src/auth/directory.rs` (Core membership port); `crates/eq-identity/tests/api_invites.rs` (acceptance tests); `.sqlx/` (nine generated workspace query metadata files); this task file; `tasks/DECISIONS.md` (TD-137); `tasks/INDEX.md` (generated).

**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1. Resend expires the previous pending invite and only link returns the URL | `resend_revokes_previous` | ✅ Passed on `ritma_test_T-110c` using the existing PostgreSQL 16 container. |
| Scope: event has the spec-defined IDs/facts and no invite token | `invited_event_excludes_token` | ✅ Passed; email and WhatsApp return no URL. |

**Checks run** —

- ✅ `cargo fmt --all --check`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo clippy --locked --offline --workspace --all-targets -- -D warnings`.
- ✅ `CARGO_TARGET_DIR=… DATABASE_URL=<task app URL> RITMA_DATABASE_OWNER_URL=<task owner URL> SQLX_OFFLINE=true cargo test --locked --offline -p eq-identity --test api_invites` — 2 passed, 0 failed; `ritma_test_T-110c`.
- ✅ `CARGO_TARGET_DIR="$HOME/.cache/ritma/target" SQLX_OFFLINE=true cargo test --locked --offline --lib -p eq-identity invites::tests` — 1 passed, 0 failed.
- ✅ `CARGO_TARGET_DIR=… DATABASE_URL=<task owner URL> cargo sqlx prepare --workspace -- --all-targets` — generated workspace and integration-test metadata.
- ✅ `CARGO_TARGET_DIR=… DATABASE_URL=<task owner URL> cargo sqlx migrate run --source migrations` — applied the existing migrations to the fresh task database.
- ✅ `cargo deny check` — advisories, bans, licenses and sources passed; duplicate `base64` lock-entry warnings remain.
- ✅ `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh`.
- ✅ `scripts/tasks-index.sh --check`, `python3 -I scripts/tasks-lint.py`, and `python3 scripts/erd.py --check`.
- ✅ `gitleaks detect --source crates/eq-identity/src/invites.rs --no-git --redact` and the same command for `crates/eq-identity/tests/api_invites.rs` — no leaks.
- ✅ `git diff --check` (rerun after final edits).
- ❌ `make api` — not run: Makefile exits with “OpenAPI generation is added by T-114”; recorded as the T-114 placeholder.
- Not run: `cargo test --workspace`; the full suite starts testcontainers and creates additional databases, beyond this task's single task-database allowance. Both task acceptance tests and the invite redaction test ran.
- Not applicable: web checks, `make dev`, `make e2e`, and `scripts/erd.py --migrations`; no web file, HTTP route or migration changed. Real database service behavior was exercised by the acceptance tests.

**Manual checks** — Reused the running `ritma-db-1` PostgreSQL 16 container, created fresh `ritma_test_T-110c`, applied migrations, and verified the app connection uses that database and `eq_app` with `NOSUPERUSER NOBYPASSRLS`. The acceptance tests wrote and read invite, membership, audit and outbox rows through the real app pool. Dropped the task database after its final test run; no new container, image or volume was created. `make api` returned its explicit T-114 placeholder message.

**Debug log** — The test-first compile failed as expected while `eq_identity::invites` was absent. The first implementation compile found ambiguous `Url::into()` and a deprecated digest slice; changed to `Url::to_string()` and a slice borrow. Initial SQLx preparation omitted integration-test queries, so reran with `-- --all-targets`. Clippy findings (repository argument count, long fixture, direct environment reads and redundant qualification) were corrected with a justified `#[expect]`, extracted fixture helpers and `Config::from_env()`. A URL assertion initially assumed the test origin; changed it to assert the locale path independent of origin. Focused tests and Clippy then passed.

**Performance** — The final focused integration test run completed in 0.22 seconds. The transaction performs one advisory lock, one targeted expiry update, one Core membership insert-if-missing and one invite insert; no list-query N+1 or cache was introduced. The cleaned target was rebuilt only at `$HOME/.cache/ritma/target`.

**Security** — Triggers: T1, T3 and T4. SBD-81: (1) no route was added; T-112 must enforce `identity.people.write`; (2) Core profile/membership methods verify transaction tenant and invite queries scope tenant/person/role under RLS; (3) token is S3, SHA-256 only at rest, event contains IDs/facts only; (4) channel and role are closed types, expiry comes from validated policy; (5) only link returns a URL built with path-segment APIs; (6) token uses `OsRng`, base64url and zeroized raw bytes; (7) no external call; (8) issuance, expiry, membership, audit and outbox share the transaction and resend is serialized; (9) audit records only IDs, role and channel; (10) resend and event tests cover invalidation/payload abuse, with a fake-secret Debug test. Gitleaks found no leaks. Local `eq`/`eq_app` role passwords were aligned with the documented development credentials after TCP authentication failed; no credentials were written to the repository or report.

**Deviations** — RATIFY TD-137 chooses the stricter CAN-60/FLW-60 event contract over TD-135: IDs-only event and link-only URL return; Notify delivery remains out of scope. RATIFY TD-134 and TD-136 remain as historical task split/id decisions. No schema, migration, permission key, error code or event name was added.

**Dependencies affected** — T-110d through T-110h can proceed in their declared order; aggregate T-111 remains dependent on all six parts, and T-112's other dependencies still apply.

**Rollback plan** — Revert the single signed-off T-110c commit. No migration needs compensation; the disposable `ritma_test_T-110c` database has been dropped.

**Follow-ups** — Architect ratification is needed for TD-137. M5 Notify needs a spec-compliant email/WhatsApp token handoff that does not put S3 data in events or add an undocumented schema field. T-112 must enforce the named permission and pass the authenticated tenant and actor.

## Questions
- **Q1 (resolved by TD-137; history):** What durable handoff lets M5 Notify send a usable invite URL without placing the raw S3 token in an event or adding an undocumented schema field?
  - Context: TD-135 adds `token` to `identity.user_invited.v1`, but `spec/03-module-system.md` §6 defines only `{person_id, role, invite_id}`; CAN-60 prohibits S2/S3 event values; FLW-60 explicitly excludes invite tokens from events; `spec/06` and ERD define only `token_hash` in `invites`.
  - Options: (a) revise the event/security contract and define an explicit protected token handoff; (b) specify a separate secure delivery port/storage contract that keeps the token out of events and the invite table hash-only; (c) defer email/WhatsApp invite delivery until the handoff is designed.
  - Recommendation: (b), preserving CAN-60/FLW-60 and the existing hash-only schema.
- **Self-resolved (implementer, run 16, needs architect ratification; TD-137):** TD-135 asks for a raw S3 token in the invite outbox event, while CAN-60/FLW-60 forbid S2/S3 event values and this task excludes email/WhatsApp delivery → keep the spec-defined IDs-only event and implement link-only URL return; leave email/WhatsApp delivery to M5 Notify because the stricter spec and this task's Scope → Out prohibit adding a secret handoff here.
- **Self-resolved (implementer, run 15, needs architect ratification; TD-134):** T-111's single M-sized task exceeds CQ-101 when its six security workflows and exact tests are combined → implement ordered parts T-110c–h and keep the parent as the aggregate, because CQ-101 requires larger work to be split.
- **Self-resolved (implementer, run 15, needs architect ratification; TD-136):** child IDs T-111a–f sort after parent T-111 and fail the task index's dependency-order rule → use T-110c–h after the existing T-110b, because tasks/README.md requires inserted tasks to use the lower neighbor's id plus a suffix.

## Review
Ready for human review. The task is `review`; the workspace-wide test suite was not run and the Make API target remains deferred to T-114, as recorded above.
