---
id: T-303
title: Add the core files service (upload, sniff, re-encode, sanitize SVG, encrypt, local/S3 storage, signed URLs) and the drafts API
milestone: M3
status: todo
depends_on: [T-002, T-101, T-102a, T-103, T-115, T-302b]
spec_refs: [SEC-13, SEC-14, SEC-15, API-12, FE-30, VAL-15, VAL-16, CAN-81, FLW-54, STM-15, OSS-02]
estimated_size: M
---

## Goal
One safe path for every uploaded file (logos and avatars now; photos, payment proofs and imports later), plus the server half of the form-draft autosave.

## Context
- Table `files` (T-101): `id, tenant_id, purpose avatar|checkin_photo|payment_proof|registration_doc|export|import|logo, content_type, size_bytes, sha256, storage_key, encrypted, created_by, created_at, deleted_at`.
- SEC-13: per-purpose allow-list + size limit:

| Purpose | MIME allow-list | Max |
|---|---|---|
| avatar | image/jpeg, image/png, image/webp | 2 MB |
| logo | image/png, image/svg+xml | 512 KB |
| checkin_photo | image/jpeg, image/webp | from `AttendancePolicy.photo.max_bytes` (passed in by the caller) |
| payment_proof | image/jpeg, image/png, application/pdf | 5 MB |
| registration_doc | image/jpeg, image/png, application/pdf | 5 MB |
| import | text/csv, application/vnd.openxmlformats-officedocument.spreadsheetml.sheet | 10 MB |
| export | (server-generated only) | — |

  - Technical consts, except checkin_photo. The MIME type is detected by magic bytes (`infer`); the client `Content-Type` is ignored. Mismatch → 422 `validation /file unsupported_type`.
  - Images are decoded and re-encoded with the `image` crate (strips EXIF): JPEG quality 85, PNG lossless, WebP as-is re-encoded lossless. Max dimension 4096 px (downscale).
  - PDFs are stored as-is and always served as `attachment`.
- SEC-14: SVG logos are parsed with `usvg` and re-serialised (removes scripts, foreignObject, event attributes and external refs). If that fails, rasterise to PNG with `resvg` at 512 px.
- Encryption: avatar and logo are **not** encrypted (public within the tenant). All other purposes are encrypted with the T-103 stream cipher (`encrypted = true`).
- Storage: `FileStore` trait with `LocalFileStore` (`RITMA_DATA_DIR/files/{tenant}/{yyyy}/{mm}/{file_id}`) and `S3FileStore` (`RITMA_FILES_BACKEND=s3`, `RITMA_S3_*` env vars; crate `object_store` with the aws feature). Add the config fields to T-002's config if missing: `RITMA_FILES_BACKEND=local|s3`, `RITMA_S3_BUCKET`, `RITMA_S3_REGION`, `RITMA_S3_ENDPOINT?`, `RITMA_S3_ACCESS_KEY_ID`, `RITMA_S3_SECRET_ACCESS_KEY`.
- SEC-15 signed URLs:
  - `GET /api/v1/files/{id}?exp=<unix>&sig=<base64url(HMAC-SHA256(Keys.filesig, tenant|id|exp|actor))>`, valid ≤ 5 min (technical const), bound to the actor (the actor's person id or `anon`).
  - Avatars and logos skip the signature: `GET /api/v1/files/{id}` is enough when purpose is avatar|logo, with `Cache-Control: private, max-age=3600` for avatars and `public, max-age=3600` for the logo (spec 26 CACHE-24/25, D-67).
  - Others require a valid sig and an authenticated actor that matches. Responses get `X-Content-Type-Options: nosniff` and `Content-Disposition: attachment` for PDFs and imports.
- API:
  - `POST /api/v1/files` multipart (`file` + `json {purpose}`), perm by purpose: avatar → any authenticated (own); logo → `core.policy.write`; others → used internally by module endpoints, **not** exposed here (module endpoints call `FileService::store`). Returns `{file_id, url}`.
  - Rate limit: upload layer (T-115).
- Drafts (FE-30): `GET/PUT/DELETE /api/v1/me/drafts/{form_key}`. Body `{payload: object}` ≤ 256 KB. Table `drafts(tenant_id, person_id, form_key, payload, updated_at)` (T-101). `form_key` pattern `^[a-z0-9_.:-]{3,80}$`. A daily job `core.drafts_cleanup` (cron `45 3 * * *`) deletes drafts older than 7 days (technical const).
- `PATCH /me` gains `avatar_file_id`. `CorePolicy.branding.logo_file_id` is set via the normal policy PUT; `validate_policy_in_context` for core checks that the file exists with purpose `logo`.
- Update `meta/app.brand.logo_url` to `/api/v1/files/{id}` when set.

- **Alignment with spec 20–24** (added with the canonical-behaviour specs):
  - Add `eq_core::http::ValidatedMultipart` (the multipart counterpart of T-102a's `ValidatedJson`; file-part errors point at `/$files/<part>`).
  - Upload rules per purpose are the VAL-15 table (sniffed type only, `image` decode limits ≤ 4096×4096 px and ≤ 64 MiB, re-encode, EXIF strip, `spawn_blocking` with a 5 s timeout). Display filenames per VAL-16 (`sanitize-filename`). Downloads set `Content-Disposition` per CAN-81.
  - Storage goes through `object_store` (local FS and S3 behind one API, OSS §2). Physical deletes run after commit as job `core.files.delete` (STM-15, D-35).

## Scope
### In
- `eq_core::files::{FileService, FileStore, LocalFileStore, S3FileStore, Purpose, sign_url, verify_sig}`, the endpoints above, the drafts API, and the jobs.

### Out
- Module-specific upload endpoints (M4–M6 call `FileService`).

## Files
| Path | Action | Purpose |
|---|---|---|
| `crates/eq-api/tests/api_drafts.rs` | test | Task-listed verification file. |
| `crates/eq-core/tests/files.rs` | test | Task-listed verification file. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
- Triggers: T3, T4, T5, T6 (SBD-30).
- Data classes touched: S2/S3 per ERD-04/ERD-05; do not create a data class outside `spec/29-erd.md` §5.
- I: S2/S3 disclosure or secret leakage → encrypt/hash as specified, require `read_pii` for sensitive reads, and never log personal data or secret values (SBD-10/21/23, SEC-02/03/21).
- T/D: hostile content or remote response → validate bounds at the boundary; uploads are sniffed/limited and external calls use allow-listed hosts, TLS verification and timeouts (SBD-20/44/47, VAL-52).
- I: unsafe file or rendered/exported content → re-encode uploads, escape HTML, guard CSV formulas, and use safe attachment headers (SBD-45/46, SEC-13/14).
- S/T/I: forged or replayed external events → verify signatures in constant time before parsing, deduplicate provider event IDs, and never log raw bodies (SBD-63).
- Abuse-case acceptance criterion: `encrypted_at_rest` must exercise the task's highest-risk forbidden, replay or cross-tenant path and demonstrate rejection without its control (SBD-31/73).

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. Uploading a JPEG with EXIF GPS → the stored file has no EXIF (parse it and assert). → `files::tests::exif_stripped`
- AC2. A PNG renamed `.jpg` with Content-Type `image/jpeg` is stored as `image/png`. A text file posing as PNG → 422. → `magic_bytes_win`
- AC3. An SVG with `<script>` and `onload` → the stored SVG contains neither. → `svg_sanitized`
- AC4. A signed URL works for its actor, fails for another actor, and fails after expiry (TestClock). → `signed_url_bound_and_expiring`
- AC5. An encrypted file on disk doesn't contain the plaintext bytes, and download returns the original. → `encrypted_at_rest`
- AC6. Drafts PUT, GET, DELETE round trip, and the cleanup job removes an 8-day-old draft. → `drafts_roundtrip_and_cleanup`
- AC7. The `S3FileStore` passes the same `FileStore` conformance test against MinIO (testcontainers) — `#[ignore]` locally, run in CI. → `s3_conformance`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-core/tests/files.rs` | AC1–AC5, AC7 |
| `crates/eq-api/tests/api_drafts.rs` | `drafts_roundtrip_and_cleanup` |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Rust checks pass; `cargo deny` passes with the new crates; `make api` regenerated
- [ ] Commit `feat(core): files and drafts [T-303]`, signed off

## Report

**Deviations** — RATIFY: TD-063, dependency references now target the final split part that provides the referenced contract.

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions
- **Self-resolved (implementer, run 6, needs architect ratification; TD-063):** this task depended on the original parent contract after its CQ-101 split → depend on the final implementation part that provides the contract, because the parent now owns only the split map.


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
