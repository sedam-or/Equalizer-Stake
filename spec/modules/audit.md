# Module: audit (always on) — includes privacy / data-subject requests

Purpose: an immutable, tamper-evident record of every significant change, with retention and anonymisation, plus data-subject rights (access, export, deletion).
Policy: `AuditPolicy` (04 §6.9) and `CorePolicy.privacy`. Tables: `audit.entries`, `deletion_requests` (06 §4.9).
Legacy reference: `v0-capstone:srs/07-audit-governance.md`, `backend/app/services/audit.py`, `audit_retention.py`, `privacy.py`.

## 1. Rules

- **AUD-01. What is audited.**
  - Every state-changing service call in every module.
  - Every login success or failure, lockout, refresh reuse, role switch, password or TOTP change.
  - Every policy and module change.
  - Every owner override.
  - Every PII read of another person's decrypted fields (`identity.people.read_pii`, category `compliance`), throttled to one entry per (actor, subject, day).
  - Every export.
  - Reads otherwise are **not** audited, except owner dashboard access (v0 rule, one entry per actor per day).
- **AUD-02. Recording API.**
  ```rust
  pub async fn record(tx: &mut Tx, e: AuditEvent<'_>) -> Result<()>;
  pub struct AuditEvent<'a> {
      pub action: &'a str,            // "<module>.<entity>.<verb>", e.g. "billing.invoice.issued"
      pub category: Category,         // Operational | Security | Compliance
      pub entity_type: &'a str, pub entity_id: Option<Uuid>,
      pub before: Option<serde_json::Value>, pub after: Option<serde_json::Value>,
      pub actor: &'a ActorCtx,        // id, role, ip, request_id (system jobs: ActorCtx::system(job_kind))
  }
  ```
  - It is called inside the business transaction (ARC-05). `before`/`after` are **redacted snapshots**: services pass DTO-like structs that derive `Auditable`.
  - `#[audit(redact)]` fields become `"[redacted]"`. Encrypted PII is never put in the audit log; only `display_name` and ids are.
- **AUD-03. Immutability.** The DB role `eq_app` has `INSERT, SELECT` on `audit.entries`. UPDATE is possible **only** through `SECURITY DEFINER` functions `audit.seal(tenant, upto_seq)` and `audit.anonymize(tenant, before_ts)`, which touch only `prev_hash`/`hash`/`sealed_at` and the redaction of `before`/`after`/`ip`. DELETE is only through `audit.purge_expired(tenant, before_ts)`. A test asserts that a plain UPDATE/DELETE as `eq_app` fails.
- **AUD-04. Hash chain (sealing).**
  - The job `audit.seal` runs every 10 s. For each tenant with unsealed rows it takes `pg_advisory_xact_lock(hash(tenant))` and walks the unsealed rows by `seq`:
    - `hash = SHA-256( prev_hash || canonical_json({id, tenant_id, at, actor_id, actor_role, action, entity_type, entity_id, category, before, after, ip, request_id}) )`
    - `prev_hash` is the previous row's hash, or 32 zero bytes for the first row.
  - Canonical JSON uses sorted keys and no whitespace (RFC 8785 JCS via the `serde_jcs` crate).
  - **Verification:** `GET /audit/verify?from_seq&to_seq` (owner) recomputes the chain and reports the first mismatch. A nightly job verifies the last 24 h, and a mismatch creates a security alert to the owner.
  - **Anonymisation** keeps the chain verifiable: when anonymising, the original hash is kept, and a separate `anonymized_at` plus `redaction_proof = SHA-256(original before||after)` is stored. Verification of anonymised rows checks `redaction_proof` instead. *(Add the column `redaction_proof bytea` to `audit.entries`.)*
- **AUD-05. Anonymisation.** The daily job (03:00) runs `audit.anonymize(tenant, now - anonymize_pii_after_years)`. It replaces `before`/`after` values under keys marked as PII in the registry (`display_name`, `email`, `phone`, `name`, `address`, `note`, `comment`) with `"REDACTED-" || hex(sha256(value))[0..16]`, and nulls `ip`.
- **AUD-06. Retention.** The daily job runs `audit.purge_expired(tenant, now - retention_years)` and deletes entries older than that. The chain restarts from the first remaining row, whose `prev_hash` is recorded in a `audit.checkpoints(tenant_id, seq, hash, at)` row written before the purge.
- **AUD-07. Access.**
  - Owner: all categories.
  - Admin: `admin_categories` only.
  - Others: none.
  - Filters: date range, actor, action prefix, entity, category.
  - Export is CSV, and the export itself is audited.

## 2. Privacy and data-subject rights (UU PDP / GDPR-ready)

- **AUD-10. Access and export.** `POST /privacy/export {subject_id}` (staff for anyone; guardian for self and linked students; person for self) runs a job that produces a ZIP:
  - `person.json` (decrypted PII), enrollments, sessions attended, invoices, payments, attendance marks, notes and ratings, consents.
  - It is stored as a `files` row with purpose `export`, downloadable through a signed URL for 24 h, then deleted.
- **AUD-11. Deletion (right to erasure).**
  - The subject or their guardian submits `POST /privacy/deletion-requests {subject_id, reason}`. `due_by = received + deletion_sla_business_days` business days (Mon–Fri, excluding tenant closures).
  - If `block_deletion_with_open_invoices` and the subject (or, as payer, their household) has open invoices, the request is rejected with the reason "open invoices" (v0 rule).
  - An admin verifies identity (`identity_verified`), then the job `privacy.erase(subject)`:
    1. Ends active enrollments (`ended_reason=admin`).
    2. Deletes future participations and holds.
    3. Deactivates memberships and revokes sessions.
    4. Sets `persons.display_name = "Deleted user " || short_hash`, nulls all `*_enc` and `*_bidx`, sets `anonymized_at`.
    5. Hard-deletes avatar, check-in photos and payment-proof files of the subject.
    6. Anonymises audit entries referencing the subject immediately (AUD-05 function scoped to the subject).
    7. Keeps invoices and payments with the anonymised payer name, because accounting records must be retained.
    8. Writes audit `privacy.subject.erased` (compliance).
  - Status `completed` and a confirmation message to the requester's last known contact, captured before erasure.
  - SLA watch: an attention item 3 business days before `due_by`.
- **AUD-12. Consent records.** Registration consent (`registrations.consent_at` + the consent text version hash) and portal acceptance of terms (`consents(person_id, kind, text_hash, at)`) are stored. *(Add the table `consents(tenant_id, person_id, kind text, text_hash bytea, at timestamptz)`.)*

## 3. API

| Method | Path | Perm |
|---|---|---|
| GET | `/audit/entries?from&to&actor_id&action&entity_type&entity_id&category&cursor` | `audit.read` (category-filtered) |
| GET | `/audit/entries/{id}` | `audit.read` |
| GET | `/audit/export?…` (CSV) | `audit.read` |
| GET | `/audit/verify?from_seq&to_seq` | owner |
| POST | `/privacy/export` / GET `/privacy/exports/{id}` | see AUD-10 |
| POST | `/privacy/deletion-requests` / GET list / POST `/{id}/verify` / `/{id}/reject` | requester / admin |

## 4. Acceptance criteria

1. **Rollback.** Given an audit insert forced to fail (trigger in the test), When issuing an invoice, Then the invoice isn't created.
2. **Tamper evidence.** After sealing, a manual `UPDATE … SET after='{}'` as superuser in the test makes `/audit/verify` report that seq.
3. **Immutability.** `UPDATE audit.entries` as `eq_app` → permission denied.
4. **Anonymisation.** An entry older than 2 years has PII keys redacted, and verify still passes.
5. **Deletion blocked.** Given open invoices, Then the deletion request is rejected. Without them, after verify: the person's PII is null, photos are deleted from disk, invoices are kept with the "Deleted user" name, and the audit has a compliance entry.
6. **Admin category scope.** An admin can't see `security` entries unless the policy grants it.
