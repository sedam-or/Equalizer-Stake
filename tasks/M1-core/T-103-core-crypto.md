---
id: T-103
title: Add core crypto (HKDF sub-keys, tenant DEK envelope, field encryption with AAD, blind index, file streams)
milestone: M1
status: review
depends_on: [T-101, T-102]
spec_refs: [SEC-01, SEC-02, SEC-03, SEC-04]
estimated_size: M
---

## Goal
Any module can encrypt and decrypt PII fields and files for a tenant, and compute blind indexes, with key versioning in place for rotation.

## Context
- SEC-01: sub-keys via HKDF-SHA256 from `RITMA_SECRET_KEY` with `info` labels `"eq/jwt/v1"`, `"eq/kek/v1"`, `"eq/bidx/v1"`, `"eq/filesig/v1"`, `"eq/backup/v1"`, `"eq/csrf/v1"`, `"eq/ics/v1"`. `RITMA_SECRET_KEY_PREVIOUS` is tried when unwrapping fails.
- SEC-02: one row per tenant per DEK version in `tenant_keys(tenant_id, key_version, dek_enc, created_at, retired_at)`.
  - The DEK is 32 random bytes, wrapped with AES-256-GCM under the KEK (nonce || ct).
  - Field ciphertext = `0x01 || key_version u32 BE || nonce(12) || ct+tag`.
  - AAD = `"{tenant_id}|{table}|{column}|{row_id}"`.
  - The current version is the max with `retired_at IS NULL`.
- SEC-03: file encryption uses the same DEK and STREAM construction (`aead::stream::EncryptorBE32` with XChaCha20Poly1305 or AES-GCM-SIV; choose `chacha20poly1305::XChaCha20Poly1305` STREAM BE32) with 1 MiB chunks. The header is `0x02 || key_version || nonce_prefix(19)`.
- SEC-04: `bidx(tenant, kind, normalized) = HMAC-SHA256(key = HKDF(bidx_base, info = tenant_id bytes), msg = kind || 0x00 || normalized)[0..16]`. `kind` is `"email"` or `"phone"`. Normalisation: email is trimmed, lowercased and NFC (crate `unicode-normalization`). Phone is parsed with `rlibphonenumber` (`PHONE_NUMBER_UTIL.parse(input, Some(region))`, `region` from the tenant country, ISO alpha-2) and formatted as E.164. An invalid phone is a validation error.

## Scope
### In
- `eq_core::crypto::{Keys, Keyring, FieldCipher, Aad, bidx_email, bidx_phone, normalize_email, normalize_phone}`.
- `Keyring` is cached per tenant (DEKs unwrapped in memory as `Zeroizing<[u8;32]>`, `moka` cache with 10-min TTL). It creates the tenant's first DEK lazily, inside the provided tx.
- `encrypt_field(tx, tenant, aad, plaintext: &[u8]) -> Vec<u8>`, `decrypt_field(tenant, aad, ct) -> Zeroizing<Vec<u8>>`, plus string helpers.
- `encrypt_stream(reader, writer)` / `decrypt_stream` for files.

### Out
- Key rotation CLI (T-705). The re-encrypt job (T-705).

## Files
| Path | Action |
|---|---|
| `Cargo.toml`, `Cargo.lock`, `crates/eq-core/Cargo.toml` | add task-named crypto, normalization and cache dependencies |
| `crates/eq-core/src/crypto.rs`, `crates/eq-core/src/lib.rs` | create the crypto seam and export it |
| `crates/eq-core/tests/crypto.rs` | create the six named acceptance tests |

## Interfaces
```rust
pub struct Keys { pub jwt: Zeroizing<[u8;32]>, pub kek: Zeroizing<[u8;32]>, pub bidx_base: Zeroizing<[u8;32]>, pub filesig: Zeroizing<[u8;32]>, pub backup: Zeroizing<[u8;32]>, pub csrf: Zeroizing<[u8;32]>, pub ics: Zeroizing<[u8;32]>, pub kek_previous: Option<Zeroizing<[u8;32]>> }
impl Keys { pub fn derive(master: &[u8], previous: Option<&[u8]>) -> Keys; }
pub struct Aad<'a> { pub tenant: TenantId, pub table: &'a str, pub column: &'a str, pub row_id: Uuid }
#[async_trait] pub trait FieldCipher {
  async fn encrypt(&self, tx: &mut Tx, aad: &Aad<'_>, pt: &[u8]) -> Result<Vec<u8>, CryptoError>;
  async fn decrypt(&self, tx: &mut Tx, aad: &Aad<'_>, ct: &[u8]) -> Result<Zeroizing<Vec<u8>>, CryptoError>;
}
pub fn bidx_email(keys: &Keys, tenant: TenantId, email: &str) -> Result<[u8;16], CryptoError>;
pub fn bidx_phone(keys: &Keys, tenant: TenantId, phone: &str, default_region: &str) -> Result<([u8;16], String /*E.164*/), CryptoError>;
```

## Implementation notes
- `tenant_keys` has no RLS (T-101). Access it only through this module, with queries filtered by `tenant_id`.
- Decryption with a wrong AAD must fail (tag mismatch) → `CryptoError::Integrity`.
- Never log key material or plaintext. Implement `Debug` for key structs as `"[redacted]"`.

## Acceptance criteria
- AC1. Round trip: encrypt, then decrypt with the same AAD → equal. → `crypto::tests::roundtrip`
- AC2. Decrypt with a different `row_id` in the AAD → `Integrity` error. → `aad_binding`
- AC3. After inserting a new DEK version, old ciphertexts still decrypt and new ones use version 2. → `key_versioning`
- AC4. With master key B and previous = A, DEKs wrapped under A unwrap. → `previous_master_unwraps`
- AC5. `bidx_email(" Ana@Mail.COM ")` equals `bidx_email("ana@mail.com")`, and differs across tenants. → `bidx_normalized_and_tenant_scoped`
- AC6. File stream round trip of 3.5 MiB of random bytes. → `stream_roundtrip`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-core/tests/crypto.rs` | the AC names above |

## Definition of done
- [x] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [x] Rust checks pass
- [x] Commit `feat(core): envelope encryption and blind indexes [T-103]`, signed off

## Report

**Plan** — (1) Write the six named acceptance tests and database fixture first. (2) Implement HKDF-derived keys and the tenant keyring over T-101's caller transaction, keeping `tenant_keys` SQL in the owner adapter. (3) Implement field/file envelopes, blind-index normalization and phone parsing with the architect-approved `rlibphonenumber` 2. Keep the repository `cargo deny` gate unchanged.

**Assumptions** — None. SEC-01..04 define the key labels, envelope formats, AAD, stream chunk size and normalization. Fixtures use synthetic values only.

**Test plan** —
- AC1: Given a tenant transaction and AAD, when a field is encrypted and decrypted, then the bytes match → `crypto::tests::roundtrip`.
- AC2: Given valid field ciphertext, when `row_id` changes, then decryption returns `CryptoError::Integrity` → `crypto::tests::aad_binding`.
- AC3: Given ciphertext under version 1, when version 2 becomes active, then old and new ciphertexts both decrypt and new writes use version 2 → `crypto::tests::key_versioning`.
- AC4: Given a DEK wrapped by master A, when master B is configured with previous=A, then it unwraps → `crypto::tests::previous_master_unwraps`.
- AC5: Given equivalent email inputs, when indexes are computed, then normalization is stable within a tenant and differs across tenants; phone is E.164 → `crypto::tests::bidx_normalized_and_tenant_scoped`.
- AC6: Given 3.5 MiB of random bytes, when streamed through encrypt/decrypt, then bytes match → `crypto::tests::stream_roundtrip`.
- Edges/errors: empty and chunk-boundary streams, malformed/truncated headers, tampered fields and streams, invalid phone/region, missing key, previous-master unwrap and rollback.
- Abuse: tenant/table/column/row AAD swaps, transaction-tenant mismatch, secret `Debug`, cache rollback, and tenant-scoped blind indexes.

**Summary** — Added the `eq_core::crypto` seam for HKDF subkeys, versioned tenant DEKs, AAD-bound field encryption, bounded XChaCha20-Poly1305 STREAM encryption, and tenant-scoped email/phone blind indexes. Secrets use `SecretBox` and zeroization with hand-written redacted `Debug`; PostgreSQL access to `tenant_keys` is isolated in the private crypto repository adapter.

**Spec read** — `spec/10-security-privacy.md` §2 (SEC-01..04); `spec/18-security-by-design.md` §§2, 5.3, 5.5, 8 (SBD-48..51, SBD-81); `spec/24-open-source-stack.md` §§1–2; `spec/16-code-quality.md` §§9, 12; `spec/17-clean-architecture.md` §§1, 3, 4 (CA-11/30/31); `spec/29-erd.md` §§4–5, 7 (owner/class/migration map); `WORKFLOW.md` §§3–4.

**Files** —
- Modified `Cargo.lock` and `crates/eq-core/Cargo.toml` for the task's crypto/parser dependencies; workspace `Cargo.toml` already contains `rlibphonenumber = "2"` per the architect resolution.
- Added `crates/eq-core/src/crypto.rs` and `crates/eq-core/src/crypto/repo.rs` for the crypto seam and owner-scoped SQL adapter; exported the module from `crates/eq-core/src/lib.rs`.
- Added `crates/eq-core/tests/crypto.rs` with the six named acceptance tests and abuse/cache regressions.
- Added five `.sqlx/query-*.json` files through `cargo sqlx prepare --workspace`.
- Updated this task Report/status and regenerated `tasks/INDEX.md`.
- No migration, API, web source or `deny.toml` change.

**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `crypto::tests::roundtrip` | Passed. |
| AC2 | `crypto::tests::aad_binding` | Passed; row, tenant, table and column binding regressions also passed. |
| AC3 | `crypto::tests::key_versioning` | Passed; version 1 remained readable and new writes used version 2. |
| AC4 | `crypto::tests::previous_master_unwraps` | Passed. |
| AC5 | `crypto::tests::bidx_normalized_and_tenant_scoped` | Passed; email normalization, tenant separation, E.164 and invalid-phone cases passed. |
| AC6 | `crypto::tests::stream_roundtrip` | Passed; 3.5 MiB, empty stream and tamper/header cases passed. |

**Checks run** —
- ✅ `SQLX_OFFLINE=true cargo fmt --all --check`.
- ✅ `SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings` after the repository-adapter move.
- ✅ `SQLX_OFFLINE=true cargo test --workspace` after the repository-adapter move; all unit, API, CLI, database integration, crypto integration and doc tests passed. T-103 crypto cases: 12/12.
- ✅ `cargo deny check`; advisories, bans, licenses and sources passed. It reported the existing duplicate `base64` versions and unmatched `CC0-1.0` allowance warning; `deny.toml` was unchanged.
- ✅ `scripts/check-deps.sh && scripts/check-literals.sh && scripts/check-web-boundaries.sh && scripts/check-layers.sh`.
- ✅ `env -u SQLX_OFFLINE DATABASE_URL=postgres://eq_app:test-secret-app@localhost:5434/ritma RITMA_DATABASE_OWNER_URL=postgres://eq:test-secret-owner@localhost:5434/ritma cargo sqlx prepare --workspace`; generated the five committed query metadata files against a fresh PostgreSQL 16.15 container.
- ✅ `env -u SQLX_OFFLINE DATABASE_URL=postgres://eq_app:test-secret-app@localhost:5436/ritma RITMA_DATABASE_OWNER_URL=postgres://eq:test-secret-owner@localhost:5436/ritma cargo sqlx prepare --workspace --check` after moving the same queries into `crypto/repo.rs`.
- ✅ `E2E_SKIP_DB_START=1 E2E_PLAYWRIGHT_CONTAINER=1 DATABASE_URL=postgres://eq_app:test-secret-app@localhost:5436/ritma RITMA_DATABASE_OWNER_URL=postgres://eq:test-secret-owner@localhost:5436/ritma RITMA_SECRET_KEY=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA= RITMA_PUBLIC_URL=http://localhost:8080 RITMA_LISTEN=127.0.0.1:8080 RITMA_BACKUP_SCHEDULE=off make e2e`; 6/6 Chromium and mobile WebKit cases passed on the final code.
- ✅ `scripts/tasks-index.sh --check`, `python3 scripts/erd.py --check`, and `git diff --check`.
- n/a `python3 scripts/erd.py --migrations`; no migration changed.
- n/a `make api` and web lint/typecheck/unit/i18n checks; no API or web source changed. `make e2e` did run the production web build and Playwright.
- Not run: hosted GitHub Actions; it requires the hosted CI environment.

**Manual checks** — The crypto integration suite used ten fresh PostgreSQL 16 `TestDb::new()` containers for its DB-backed tests; the two pure tests did not need a database. SQLx metadata was generated on a separate fresh PostgreSQL 16.15 container at port 5434. The final browser run used fresh PostgreSQL 16.15 container `ritma-t103-final-20261027` at port 5436, applied the current migrations, seeded one synthetic `Test School` tenant using the same fixture shape as the repository's existing tenant tests, and passed 6/6 cases. That container was stopped and removed. `EXPLAIN (ANALYZE, BUFFERS)` on synthetic, rolled-back rows measured the active-key lookup at 0.195 ms, next-version lookup at 0.071 ms, and exact-version lookup at 0.033 ms.

**Debug log** — The first rollback regression run exposed that a DEK created inside an uncommitted transaction could enter the shared cache; cache insertion now checks `xmin` against the current transaction id, and the rollback regression passes. An early offline Clippy attempt lacked SQLx metadata; regenerated metadata from a disposable migrated database, verified it with `cargo sqlx prepare --workspace --check`, then offline Clippy passed. The first `make e2e` run on a fresh but unprovisioned DB passed both health checks and got 503 `setup_required` for four SPA requests; after seeding the existing synthetic tenant fixture shape, the rerun passed 6/6. The first migration launches also exposed required fake `RITMA_SECRET_KEY` and `RITMA_PUBLIC_URL` configuration; the migration succeeded after supplying the full Makefile E2E settings.

**Performance** — `EXPLAIN (ANALYZE, BUFFERS)` on synthetic, rolled-back rows measured active-key lookup 0.195 ms, next-version lookup 0.071 ms, and exact-version lookup 0.033 ms. STREAM buffers are bounded to the task's 1 MiB chunks and zeroized; the 3.5 MiB test passed.

**Security** — Triggers T3 (S2/S3) and T4 (crypto/keys). SBD-81: (1) no HTTP handler or permission chain is added; (2) tenant comes from the caller's `Tx`, `app.tenant_id` is verified, and every repository query filters `tenant_id`; `tenant_keys` has no RLS by ERD-04 and is accessed through `eq_core::crypto`; (3) S3 master-derived keys and DEKs use `SecretBox<Zeroizing<_>>`, have manual redacted `Debug`, and are never serialized, logged or returned; (4) phone and region input are validated, stream framing is bounded, and malformed input is rejected; (5) no UI/API output is introduced; (6) AES-GCM field AAD, XChaCha STREAM and HMAC blind indexes stay inside `eq_core::crypto`; (7) no external call; (8) no business aggregate transition is added; first-DEK provisioning remains inside the caller transaction; (9) key rotation is owned by T-705 and no event name was invented here; (10) abuse tests cover AAD swaps, tampering, tenant mismatch, secret formatting and rollback cache poisoning. Test fixtures contain only synthetic values.

**Deviations** — RATIFY: (1) the task's free-function sketches omit database/keyring context and its Files table is missing; implement the operations as `Keyring` methods over the caller's `Tx`, place table SQL in private `crypto/repo.rs` per CA-30, and include the manifest/test/adapter paths required by Scope and architecture; (2) key the cache by `(tenant_id, key_version)` so AC3 keeps old and current DEKs readable; (3) use `SecretBox` with hand-written redacting `Debug` for key-bearing structs per SBD-50. The run-6 phone-parser question is resolved by the architect's `rlibphonenumber` 2 decision; no advisory exception was added. The initial key bootstrap does not create a domain event; SBD-53 explicitly assigns key rotation audit to the rotation flow, and no creation event is specified.

**Dependencies affected** — T-110 can use `bidx_email`, `bidx_phone`, `normalize_email`, `normalize_phone` and the derived `Keys`; T-705 owns key rotation and must account for DEKs that still unwrap only with the previous KEK.

**Rollback plan** — Revert the single T-103 commit, which removes the crypto source/repository adapter/tests/dependencies and SQLx metadata. No migration or persistent schema change was introduced; disposable databases were removed.

**Follow-ups** — Architect review of the three RATIFY choices above. Review whether lazy first-DEK initialization needs a named audit event; no event name was invented because SEC-02/SBD-53 do not define one for initial creation. T-101's default privileges let `eq_app` access `tenant_keys` directly even though ERD-04 says access only through `core::crypto`; assess a database-level restriction in the T-101-owned policy/migration scope. Reconcile the generic `phonenumber` mention in `spec/22` with the resolved `rlibphonenumber` contract without editing the spec in this task. Verify T-705 rewraps previous-master DEKs before retiring the old KEK. Review the approximate CQ-101 PR-size overage (1,090 lines across the new crypto source, SQL adapter and acceptance tests; lockfiles are excluded). Retain the known full-history gitleaks false positive at `Makefile:80` for maintainer review; the staged scan is recorded after final staging.

**Step 0 — Start** — Resumed on `milestone/m1-core` at `513576f` with T-103 in progress and its implementation WIP present; T-101 and T-102 were `review`. Read the current Run Log, checked the index, and kept all task work on the milestone branch.

**Step 1 — Task and spec** — Read the complete T-103 task, its resolved parser contract and historical Questions, SEC-01..04, SBD-48..51/81, CQ-11/12/42/101, CA-11/30/31 and the ERD owner/class map.

**Step 2 — Dependencies** — Read the T-101 and T-102 Reports (including RATIFY items and Follow-ups), T-101's `tenant_keys` DDL/privileges, T-102's validated types, the caller `Tx`, and the shipped config secret wrappers. The table is core-crypto-owned, has no RLS, and the app role's default DML grant is a human follow-up.

**Step 3 — Design reference** — n/a; backend crypto task.

**Step 4 — Plan and test plan** — Recorded above before implementation. Each AC maps to its exact named test; error, edge and abuse cases are listed.

**Step 5 — Implement** — Added HKDF key derivation, redacted secret wrappers, a versioned tenant keyring, AAD-bound field crypto, STREAM file crypto, blind indexes and phone normalization. Kept all production `tenant_keys` SQL in the private `crypto/repo.rs` adapter and added the AC/abuse tests.

**Step 6 — Run locally** — Ran the final workspace suite and `make e2e` against disposable PostgreSQL 16 databases. The crypto suite passed 12/12; final browser run passed 6/6 after seeding the synthetic tenant fixture.

**Step 7 — Debug loop** — Corrected the rollback cache bug, refreshed SQLx metadata for offline compilation, and seeded the expected fake tenant after the first fresh-database E2E correctly returned `setup_required`. No application code or security gate was weakened.

**Step 8 — Self code review** — Reviewed CA-30 repository placement, tenant scoping, AAD components, secret zeroization/redaction, parameterized SQL, rollback behavior, error mapping and bounded stream framing. No `#[allow]`, schema change or out-of-scope source path was added.

**Step 9 — Optimize** — Measured the three tenant-key queries with `EXPLAIN (ANALYZE, BUFFERS)` over synthetic rolled-back rows; results are recorded under Performance. STREAM remains bounded by the required 1 MiB chunk.

**Step 10 — Security audit** — Completed the SBD-81 crypto/tenant review above. Tests cover AAD swaps, tampering, transaction tenant mismatch, secret formatting and rollback cache poisoning. No real secrets or personal data were added.

**Step 11 — Revise and verify** — Final Rust format, Clippy and workspace tests passed after the repository-adapter move; SQLx check, cargo-deny, guard scripts, ERD/index checks, staged gitleaks and E2E are listed above.

**Step 12 — Report, commit** — Set status to `review`, regenerated the index, and committed the implementation, tests, SQLx metadata and Report in one signed-off T-103 commit on `milestone/m1-core`.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification):** The Scope lists field decrypt and stream helpers without DB transaction/keyring context, while the declared `FieldCipher` trait requires `&mut Tx`; it also omits source, manifest and test paths from a Files table → expose operations as `Keyring` methods using the caller's transaction, keep `tenant_keys` SQL in private `crypto/repo.rs` per CA-30, and include the paths required by Scope/Tests/architecture, because key lookup needs the caller transaction and no global keyring is specified.
- **Self-resolved (implementer, run 6, needs architect ratification):** The cache is described as per-tenant, but AC3 requires an old ciphertext to decrypt after a new active DEK version is inserted → cache each `(tenant_id, key_version)` entry under that tenant so old and new keys coexist for the stated TTL.
- **Self-resolved (implementer, run 6, needs architect ratification):** The interface's `Zeroizing` key arrays need a secret-safe public boundary → wrap key material in `secrecy::SecretBox` (and DEKs in `SecretBox<Zeroizing<[u8; 32]>>`) with hand-written redacting `Debug`, following the pre-authorised SBD-50 correction.
- **Resolved (architect, 2026-10-27):** Q1 — option (c), as recommended: `phonenumber` is replaced by `rlibphonenumber` 2 (Apache-2.0, maintained libphonenumber port). Probed with the repository `deny.toml`: advisories, bans and licenses pass; the gate is unchanged and no exception is added. Spec 10, 22 and 24 are updated. In the workspace `Cargo.toml` remove `phonenumber` and add `rlibphonenumber = "2"`. API: `rlibphonenumber::PHONE_NUMBER_UTIL.parse(input, Some(region))` returns a `PhoneNumber`; `is_valid()`; `format_as(PhoneNumberFormat::E164)`. `region` is `rlibphonenumber::enums::Region`, built from the tenant's ISO alpha-2 country (a `Region` has no string constructor, so convert through a small private mapping from the alpha-2 code; an unknown code is a validation error, not a panic). Letters (vanity numbers) stay rejected, as before.
- **Q1 (resolved by architect; historical):** How should T-103 use SEC-04's required phone parser while keeping the repository's `cargo-deny` unmaintained-advisory gate green, given the current `phonenumber` release pulls an unmaintained transitive crate?
  - Context: `cargo info phonenumber` reports current release `0.3.10+9.0.33`; its manifest depends on `postcard` with default features, which enables `heapless-cas` and pulls `heapless 0.7.17` → `atomic-polyfill 1.0.3` (RUSTSEC-2023-0089). `deny.toml` sets `unmaintained = "all"`. This dependency is required by SEC-04/spec 24 and cannot be introduced without making the security gate fail.
  - Options: (a) add an advisory exception or relax the gate, which violates AGENTS §4 and SEC-22; (b) vendor/patch the parser or its transitive dependency to disable the unused default feature, which changes the approved dependency source and requires an explicit maintenance/security decision; (c) update spec 24 to name a maintained parser or wait for a published `phonenumber` release with a clean dependency graph.
  - Recommendation: (c), because it preserves the security gate and the task's specified dependency contract.
