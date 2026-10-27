---
id: T-103
title: Add core crypto (HKDF sub-keys, tenant DEK envelope, field encryption with AAD, blind index, file streams)
milestone: M1
status: todo
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
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Rust checks pass
- [ ] Commit `feat(core): envelope encryption and blind indexes [T-103]`, signed off

## Report

**Plan** — (1) Add the exact task-named crypto, phone-normalization, Unicode-normalization and cache dependencies, and the six AC tests → verify their APIs against the pinned crates and T-101's `Tx`/`tenant_keys`. (2) Implement derived keys with redacted formatting and a version-aware tenant keyring → verify labels, field AAD, rotation and previous-master unwrap. (3) Add blind-index normalization and bounded STREAM framing → verify tenant separation, E.164 output and the 3.5 MiB stream case. This plan is not executed because the required phone-parser dependency currently makes the unchanged supply-chain gate fail.

**Assumptions** — None. The dependency contract, schema, cryptographic formats and technical constants are stated by the task and cited specs.

**Test plan** —
- AC1: Given a tenant with no DEK and an `Aad`, when a field is encrypted and decrypted in tenant transactions, then the original bytes are returned: `crypto::tests::roundtrip`.
- AC2: Given valid ciphertext, when only `row_id` changes, then authentication fails as `CryptoError::Integrity`: `aad_binding`.
- AC3: Given first-version ciphertext, when version 2 is inserted, then old ciphertext remains readable and new ciphertext uses version 2: `key_versioning`.
- AC4: Given a DEK wrapped under master A, when master B is configured with A as the previous key, then it unwraps: `previous_master_unwraps`.
- AC5: Given equivalent normalized emails, when blind indexes are computed, then same-tenant values match and cross-tenant values differ: `bidx_normalized_and_tenant_scoped`; phone E.164 and invalid input are included.
- AC6: Given 3.5 MiB of random bytes, when file streaming encrypts and decrypts it, then the bytes match: `stream_roundtrip`.
- Edges: empty and chunk-boundary streams; old/current versions; malformed headers; missing key; invalid UTF-8; invalid phone; Unicode NFC email.
- Errors: authentication, framing, missing-key, database and phone-validation errors remain distinguishable.
- Abuse: row/column/tenant ciphertext swaps, tampering, key debug formatting, and bounded streaming memory.

**Summary** — Blocked before implementation. The task requires the `phonenumber` crate for SEC-04, but the currently published version's dependency graph includes an unmaintained advisory rejected by the repository's `cargo-deny` policy. No code, dependency or policy change was made.

**Spec read** — `spec/10-security-privacy.md` §2 (SEC-01..04); `spec/18-security-by-design.md` §§2, 5.3, 8 (SBD-48..51, SBD-81); `spec/24-open-source-stack.md` §§1–2 (OSS-02/phone parser/crypto); `spec/16-code-quality.md` §§9, 12; `spec/17-clean-architecture.md` §§1, 4, 7; `spec/29-erd.md` §§4–5, 7 (tenant_keys owner and migration); `WORKFLOW.md` §§3–4.

**Files** — Updated only this task file and regenerated `tasks/INDEX.md`; no implementation or dependency files were changed.

**Acceptance criteria → tests**
| AC | Test | Result |
|---|---|---|
| AC1 | `crypto::tests::roundtrip` | Not run: blocked before implementation. |
| AC2 | `aad_binding` | Not run: blocked before implementation. |
| AC3 | `key_versioning` | Not run: blocked before implementation. |
| AC4 | `previous_master_unwraps` | Not run: blocked before implementation. |
| AC5 | `bidx_normalized_and_tenant_scoped` | Not run: blocked before implementation. |
| AC6 | `stream_roundtrip` | Not run: blocked before implementation. |

**Checks run** —
- ✅ `cargo search phonenumber --limit 5` and `cargo info phonenumber` — current crates.io release is `0.3.10+9.0.33`.
- ✅ `cargo deny check advisories` — passes for the unchanged workspace graph; `phonenumber` is not yet used by a crate and is therefore absent from that graph.
- ✅ `scripts/tasks-index.sh --check` and `git diff --check` after the blocked status/index update.
- Not run: Rust implementation/tests, E2E and crypto abuse tests; implementation stopped at the dependency gate.
- Not run: `cargo sqlx prepare --workspace`; no query was added.
- Not run: hosted CI; requires GitHub Actions.

**Manual checks** — Read the current `phonenumber` crate manifest and `postcard` feature declarations. `phonenumber` depends on `postcard` with default features; `postcard` defaults to `heapless-cas`, which brings `heapless 0.7.17` and `atomic-polyfill 1.0.3`. `deny.toml` sets `unmaintained = "all"`, so introducing that graph conflicts with the enforced security gate. `cargo info phonenumber` confirms no newer release is currently published.

**Debug log** — Blocker analysis found no permitted dependency configuration that removes the transitive advisory without changing the pinned parser's source, selecting an unlisted parser, or suppressing the repository's all-unmaintained policy. The listed alternatives need an architect/spec decision; implementation was not started.

**Performance** — n/a; no implementation was written.

**Security** — Trigger T4: cryptographic keys and S2/S3 data. No crypto code was written and no security control was weakened. No abuse test can run until the dependency contract is resolved. The existing advisory policy remains unchanged.

**Deviations** — RATIFY: (1) complete the `FieldCipher` API through a `Keyring` and caller `Tx` because the Scope's free-function sketches lack the DB/key context required by its declared trait; add the file and manifest paths derived from Scope/Tests because the task omitted a Files table; (2) key the cache by `(tenant_id, key_version)` so AC3 can retain old and current DEKs; (3) use `SecretBox` with hand-written redacting `Debug` for all key-bearing structs per SBD-50. The task remains blocked independently on the phone-parser dependency question below.

**Dependencies affected** — T-110 depends on T-103. Other tasks remain ordered by their declared dependencies.

**Rollback plan** — Revert the task-status/question/report commit and regenerate the index. No migration or source change exists.

**Follow-ups** — Architect must resolve the phone-parser/supply-chain conflict before T-103 or dependent tasks can introduce `phonenumber` while keeping `cargo-deny` green.

**Step 0 — Start** — Started on clean `milestone/m1-core` at `5ea488f`; `scripts/tasks-index.sh --check` passed. T-101 and T-102 were both `review`, so T-103 was the lowest-ID eligible task. Set T-103 to `in_progress` and regenerated the index.

**Step 1 — Task and spec** — Read the complete task and the cited SEC-01..04 requirements, including cryptographic formats, key labels, 1 MiB stream size and required parser. Read mandatory CQ-16, CA-17 and SBD-81 guidance.

**Step 2 — Dependencies** — Read the full T-101 and T-102 Reports, including their RATIFY items and Follow-ups, plus the shipped tenant key schema, `Tx`, configuration secrets, validated types and dependency manifests. T-101's tenant_keys DDL is S3, has no RLS and is owned by `core::crypto`; T-102 contributes no crypto API. The prior reports' unrelated follow-ups do not change this contract.

**Step 3 — Design reference** — n/a; backend crypto task.

**Step 4 — Plan and test plan** — Recorded above before implementation. Tests are mapped to every AC, edge, error and abuse case.

**Step 5 — Implement** — n/a; blocked before implementation by the required dependency conflict.

**Step 6 — Run locally** — n/a; no runtime behavior was implemented.

**Step 7 — Debug loop** — n/a; no implementation failure. The dependency tree was checked against the repository's enforced advisory policy.

**Step 8 — Self code review** — n/a; no source diff.

**Step 9 — Optimize** — n/a; no code or query.

**Step 10 — Security audit** — Reviewed the SBD-81 crypto and supply-chain controls. No crypto change exists to audit; the advisory gate was not weakened.

**Step 11 — Revise and verify** — n/a; no implementation to revise. The task index and whitespace checks pass after recording the blocker.

**Step 12 — Report, commit** — Blocker question recorded with options and recommendation; status set to `blocked`; index regenerated. The signed-off task blocker commit is pushed on `milestone/m1-core`.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification):** The Scope lists field decrypt and stream helpers without DB transaction/keyring context, while the declared `FieldCipher` trait requires `&mut Tx`; it also omits source, manifest and test paths from a Files table → expose operations as `Keyring` methods using the declared transaction shape and list only paths directly required by Scope/Tests, because key lookup must use the caller's transaction and no global keyring is specified.
- **Self-resolved (implementer, run 6, needs architect ratification):** The cache is described as per-tenant, but AC3 requires an old ciphertext to decrypt after a new active DEK version is inserted → cache each `(tenant_id, key_version)` entry under that tenant so old and new keys coexist for the stated TTL.
- **Self-resolved (implementer, run 6, needs architect ratification):** The interface's `Zeroizing` key arrays need a secret-safe public boundary → wrap key material in `secrecy::SecretBox` (and DEKs in `SecretBox<Zeroizing<[u8; 32]>>`) with hand-written redacting `Debug`, following the pre-authorised SBD-50 correction.
- **Resolved (architect, 2026-10-27):** Q1 — option (c), as recommended: `phonenumber` is replaced by `rlibphonenumber` 2 (Apache-2.0, maintained libphonenumber port). Probed with the repository `deny.toml`: advisories, bans and licenses pass; the gate is unchanged and no exception is added. Spec 10, 22 and 24 are updated. In the workspace `Cargo.toml` remove `phonenumber` and add `rlibphonenumber = "2"`. API: `rlibphonenumber::PHONE_NUMBER_UTIL.parse(input, Some(region))` returns a `PhoneNumber`; `is_valid()`; `format_as(PhoneNumberFormat::E164)`. `region` is `rlibphonenumber::enums::Region`, built from the tenant's ISO alpha-2 country (a `Region` has no string constructor, so convert through a small private mapping from the alpha-2 code; an unknown code is a validation error, not a panic). Letters (vanity numbers) stay rejected, as before.
- **Q1 (blocking):** How should T-103 use SEC-04's required phone parser while keeping the repository's `cargo-deny` unmaintained-advisory gate green, given the current `phonenumber` release pulls an unmaintained transitive crate?
  - Context: `cargo info phonenumber` reports current release `0.3.10+9.0.33`; its manifest depends on `postcard` with default features, which enables `heapless-cas` and pulls `heapless 0.7.17` → `atomic-polyfill 1.0.3` (RUSTSEC-2023-0089). `deny.toml` sets `unmaintained = "all"`. This dependency is required by SEC-04/spec 24 and cannot be introduced without making the security gate fail.
  - Options: (a) add an advisory exception or relax the gate, which violates AGENTS §4 and SEC-22; (b) vendor/patch the parser or its transitive dependency to disable the unused default feature, which changes the approved dependency source and requires an explicit maintenance/security decision; (c) update spec 24 to name a maintained parser or wait for a published `phonenumber` release with a clean dependency graph.
  - Recommendation: (c), because it preserves the security gate and the task's specified dependency contract.
