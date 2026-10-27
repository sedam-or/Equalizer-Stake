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

**Plan** — Write the six named acceptance tests first; implement HKDF-derived keys and a tenant keyring over T-101's Tx; add normalized blind indexes and bounded STREAM encryption. Use rlibphonenumber 2 and leave deny.toml unchanged.

**Assumptions** — None. Test fixtures are synthetic. SEC-01..04 define the crypto formats and technical constants.

**Test plan** —
- AC1: Field encrypt/decrypt returns the original bytes: crypto::tests::roundtrip.
- AC2: Changing row_id returns CryptoError::Integrity: crypto::tests::aad_binding.
- AC3: Old ciphertext decrypts and new writes use version 2: crypto::tests::key_versioning.
- AC4: Master B with previous=A unwraps the DEK from A: crypto::tests::previous_master_unwraps.
- AC5: Normalized emails match within a tenant and differ across tenants; phone normalizes to E.164: crypto::tests::bidx_normalized_and_tenant_scoped.
- AC6: A 3.5 MiB random stream round-trips: crypto::tests::stream_roundtrip.
- Abuse coverage: tenant/table/column/row AAD binding, tampering, transaction tenant mismatch, debug redaction, rollback-safe key cache.

**Summary** — Added eq_core::crypto with HKDF-derived redacted subkeys, AES-256-GCM tenant DEK envelopes, AAD-bound field encryption, and XChaCha20-Poly1305 STREAM file encryption in 1 MiB chunks. Added tenant-scoped email/phone blind indexes, phone normalization using the approved parser, previous-master unwrapping, and version-aware key caching.

**Spec read** — spec/10-security-privacy.md §2 (SEC-01..04); spec/18-security-by-design.md §§2, 5.3, 5.5, 8 (SBD-48..51, SBD-81); spec/24-open-source-stack.md §§1–2; spec/16-code-quality.md §§9, 12; spec/17-clean-architecture.md §§1, 4, 7; spec/29-erd.md §§4–5, 7; WORKFLOW.md §§3–4.

**Files** —
- Cargo.lock and crates/eq-core/Cargo.toml: crypto dependencies; workspace Cargo.toml already declared the required crates.
- crates/eq-core/src/crypto.rs: crypto seam, keyring, field/file encryption, blind indexes, normalization, errors.
- crates/eq-core/src/lib.rs: export eq_core::crypto.
- crates/eq-core/tests/crypto.rs: six AC tests and security/cache regressions.
- .sqlx/: five query metadata files generated by cargo sqlx prepare --workspace.
- tasks/M1-core/T-103-core-crypto.md and tasks/INDEX.md: report, status, generated index.
- No workspace Cargo.toml change was needed.

**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | crypto::tests::roundtrip | PASS |
| AC2 | crypto::tests::aad_binding | PASS |
| AC3 | crypto::tests::key_versioning | PASS |
| AC4 | crypto::tests::previous_master_unwraps | PASS |
| AC5 | crypto::tests::bidx_normalized_and_tenant_scoped | PASS |
| AC6 | crypto::tests::stream_roundtrip | PASS |

**Checks run** —
- ✅ cargo fmt --all --check.
- ✅ cargo clippy --workspace --all-targets -- -D warnings.
- ✅ cargo test --workspace; all unit, integration, and doc tests passed. T-103 crypto integration: 12 passed, 0 failed.
- ✅ cargo deny check; advisories, bans, licenses, and sources passed. Duplicate-package warnings were emitted; deny.toml was unchanged.
- ✅ scripts/check-deps.sh.
- ✅ scripts/check-literals.sh.
- ✅ scripts/check-web-boundaries.sh.
- ✅ scripts/check-layers.sh.
- ✅ cargo sqlx prepare --workspace against migrated disposable PostgreSQL 16.15; generated metadata is included.
- ✅ make e2e against disposable PostgreSQL 16.15; all 6 Playwright checks passed after adding one synthetic tenant. The first run returned setup_required for four page checks because the fresh DB had no tenant; both health checks passed. The database container was removed afterward.
- ✅ scripts/tasks-index.sh --check.
- ✅ git diff --check.
- Not run: pnpm web lint/typecheck/test/i18n checks; no web source changed. E2E did run web build and Playwright.
- Not run: make api; the Makefile documents OpenAPI generation as a T-114 addition and the current target exits nonzero. SQLx metadata was prepared separately.
- Not run: hosted CI.

**Manual checks** — Verified PostgreSQL 16.15, applied four migrations, and exercised field round-trip, AAD binding, key versioning, previous-master unwrap, phone E.164, tenant mismatch, rollback recovery, and a 3.5 MiB stream. The E2E server became ready and six browser smoke checks passed with one synthetic tenant.

**Debug log** — Tests-first compilation failed because the crypto module and crate dependencies were absent; implementation made the named suite compile. A rollback regression exposed uncommitted DEK cache poisoning; entries now cache only after commit and the regression passes. Clippy flagged redundant else branches and fixture expect calls; both were corrected without lint suppression. The first E2E run found setup_required on an empty database; a synthetic tenant fixture made the rerun pass. The first migration command lacked RITMA_PUBLIC_URL; rerunning with E2E settings applied all four migrations.

**Performance** — No benchmark was required or taken. STREAM uses bounded 1 MiB chunks and zeroizing buffers; the 3.5 MiB round-trip passed.

**Security** — Trigger T4. Derived keys and DEKs use secrecy SecretBox plus Zeroizing and redact Debug. Field encryption binds tenant, table, column, and row id. tenant_keys queries filter by tenant and validate the transaction tenant because the table has no RLS. Current DEK selection locks the active row. The cache is keyed by tenant/version and excludes uncommitted rows. No plaintext, key material, or credentials are logged. Abuse tests cover AAD swaps, tampering, transaction-tenant mismatch, and rollback cache poisoning.

**Deviations** — Implemented operations as Keyring methods over the caller's Tx because key lookup needs that context; keyed the cache by (tenant_id, key_version) so old ciphertext remains decryptable; wrapped key material in SecretBox per SBD-50. These three self-resolved decisions remain pending architect review. Added transaction-tenant validation and excluded uncommitted rows from the cache for isolation and rollback safety. The task specifies kind || 0x00 in the blind-index HMAC message while SEC-04 describes the normalized value; the task's explicit separator is implemented and its interpretation remains pending architect review.

**Dependencies affected** — T-110 depends on T-103.

**Rollback plan** — Revert the T-103 commit. No migration or persistent schema change was made; the disposable E2E database was removed. Revert the generated SQLx metadata with the same commit.

**Follow-ups** — Architect review of the three self-resolved API/cache/secret-boundary decisions and the blind-index domain separator. T-110 can use the exported crypto seam.

**Step 0 — Start** — Resumed on clean milestone/m1-core at 513576f. T-103 was todo; T-101 and T-102 were review with their commits on this branch. The operator directed T-103 after D-56 resolved Q1.

**Step 1 — Task and spec** — Read the complete task and cited SEC-01..04, including key labels, envelope formats, tenant key schema, 1 MiB STREAM size, and rlibphonenumber API.

**Step 2 — Dependencies** — Read T-101 and T-102 Reports, tenant_keys migration, Tx, config secret types, and dependency manifests. tenant_keys has no RLS, so queries filter tenant_id and verify the transaction tenant.

**Step 3 — Design reference** — n/a; backend crypto task.

**Step 4 — Plan and test plan** — Recorded above before implementation. The six ACs and added abuse/cache regressions map to named tests.

**Step 5 — Implement** — Wrote the named acceptance tests first, confirmed they failed before implementation, then added the crypto seam and generated SQLx metadata.

**Step 6 — Run locally** — Ran the crypto tests with testcontainers PostgreSQL 16, workspace tests, and browser E2E on disposable PostgreSQL 16.15 with a synthetic tenant.

**Step 7 — Debug loop** — Resolved compiler API mismatches, Clippy findings, and the cache poisoning bug exposed by rollback regression. Added a synthetic E2E tenant after the empty database correctly returned setup_required.

**Step 8 — Self code review** — Reviewed tenant scoping, AAD components, zeroization/redaction, tenant_keys filters, error mapping, parameterized SQL, rollback cache behavior, and STREAM bounds. No #[allow] attributes or out-of-scope source files were added.

**Step 9 — Optimize** — n/a; no performance tuning or benchmark was warranted.

**Step 10 — Security audit** — Followed SBD-81 for crypto and tenant controls. No secrets or real personal data were introduced; all fixtures are synthetic.

**Step 11 — Revise and verify** — Rust, dependency, layer, literal, web-boundary, SQLx, workspace test, cargo-deny, E2E, task-index, and whitespace checks passed as recorded above.

**Step 12 — Report, commit** — Report complete and status set to review. The task index was regenerated; include source, tests, SQLx metadata, and report in one signed-off T-103 commit on milestone/m1-core.
## Questions
- **Self-resolved (implementer, run 6, needs architect ratification):** The Scope lists field decrypt and stream helpers without DB transaction/keyring context, while the declared `FieldCipher` trait requires `&mut Tx`; it also omits source, manifest and test paths from a Files table → expose operations as `Keyring` methods using the declared transaction shape and list only paths directly required by Scope/Tests, because key lookup must use the caller's transaction and no global keyring is specified.
- **Self-resolved (implementer, run 6, needs architect ratification):** The cache is described as per-tenant, but AC3 requires an old ciphertext to decrypt after a new active DEK version is inserted → cache each `(tenant_id, key_version)` entry under that tenant so old and new keys coexist for the stated TTL.
- **Self-resolved (implementer, run 6, needs architect ratification):** The interface's `Zeroizing` key arrays need a secret-safe public boundary → wrap key material in `secrecy::SecretBox` (and DEKs in `SecretBox<Zeroizing<[u8; 32]>>`) with hand-written redacting `Debug`, following the pre-authorised SBD-50 correction.
- **Resolved (architect, 2026-10-27):** Q1 — option (c), as recommended: `phonenumber` is replaced by `rlibphonenumber` 2 (Apache-2.0, maintained libphonenumber port). Probed with the repository `deny.toml`: advisories, bans and licenses pass; the gate is unchanged and no exception is added. Spec 10, 22 and 24 are updated. In the workspace `Cargo.toml` remove `phonenumber` and add `rlibphonenumber = "2"`. API: `rlibphonenumber::PHONE_NUMBER_UTIL.parse(input, Some(region))` returns a `PhoneNumber`; `is_valid()`; `format_as(PhoneNumberFormat::E164)`. `region` is `rlibphonenumber::enums::Region`, built from the tenant's ISO alpha-2 country (a `Region` has no string constructor, so convert through a small private mapping from the alpha-2 code; an unknown code is a validation error, not a panic). Letters (vanity numbers) stay rejected, as before.
- **Q1 (resolved by architect; historical):** How should T-103 use SEC-04's required phone parser while keeping the repository's `cargo-deny` unmaintained-advisory gate green, given the current `phonenumber` release pulls an unmaintained transitive crate?
  - Context: `cargo info phonenumber` reports current release `0.3.10+9.0.33`; its manifest depends on `postcard` with default features, which enables `heapless-cas` and pulls `heapless 0.7.17` → `atomic-polyfill 1.0.3` (RUSTSEC-2023-0089). `deny.toml` sets `unmaintained = "all"`. This dependency is required by SEC-04/spec 24 and cannot be introduced without making the security gate fail.
  - Options: (a) add an advisory exception or relax the gate, which violates AGENTS §4 and SEC-22; (b) vendor/patch the parser or its transitive dependency to disable the unused default feature, which changes the approved dependency source and requires an explicit maintenance/security decision; (c) update spec 24 to name a maintained parser or wait for a published `phonenumber` release with a clean dependency graph.
  - Recommendation: (c), because it preserves the security gate and the task's specified dependency contract.
