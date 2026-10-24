# 10 — Security & Privacy

## 1. Threat model (summary)

| Asset | Threat | Primary control |
|---|---|---|
| Children's PII (names, birth dates, photos) | DB dump / backup leak | Field-level AES-256-GCM (SEC-03), encrypted backups (SEC-31), photo retention (ATT-08) |
| Tenant data in cloud | Cross-tenant read via a bug | Postgres RLS, forced (SEC-05) + isolation test (TST-06) |
| Accounts | Credential stuffing / brute force | argon2id, lockout, rate limits, optional TOTP (IDN) |
| Sessions | XSS token theft | Access token in memory only, HttpOnly refresh cookie, strict CSP (SEC-20) |
| Money records | Silent tampering by insiders | Append-only audit with hash chain (AUD-03/04) |
| Fresh self-host install | Stranger claims instance | Setup token (SET-11) |
| Uploaded files | Malware / XSS via SVG / polyglots | Magic sniffing, re-encoding, SVG sanitisation, served with `Content-Disposition` and nosniff (SEC-13/14) |
| Third-party credentials (SMTP, WhatsApp) | Leak via API/logs | Encrypted at rest, redacted in API, never logged (SEC-23) |

## 2. Cryptography

**SEC-01. Master secret.** `RITMA_SECRET_KEY` is 32 random bytes, base64. It is required, and startup fails if it is missing or shorter than 32 bytes. The installer generates it (DST). Sub-keys are derived with HKDF-SHA256 (`info` labels):
- `"eq/jwt/v1"` (JWT HMAC key)
- `"eq/kek/v1"` (key-encryption key)
- `"eq/bidx/v1"` (blind-index key base)
- `"eq/filesig/v1"` (signed-URL key)
- `"eq/backup/v1"` (backup encryption key)
- `"eq/csrf/v1"`
- `"eq/ics/v1"` (calendar feed token key, SCH-16)

`RITMA_SECRET_KEY_PREVIOUS` (optional) is tried for unwrapping and verifying during master-key rotation (`ritma rotate-keys`).

**SEC-02. Envelope encryption.**
- Each tenant has data-encryption keys (DEKs) in `tenant_keys(tenant_id, key_version int, dek_enc bytea, created_at, retired_at, PRIMARY KEY(tenant_id, key_version))`. A DEK is 32 random bytes wrapped with AES-256-GCM under the KEK.
- The field ciphertext format is `0x01 || key_version (u32 BE) || nonce (12 B) || ciphertext||tag`, with **AAD** = `tenant_id || "|" || table || "|" || column || "|" || row_id`, so ciphertexts can't be swapped between rows or columns.
- Library: `aes-gcm` with `zeroize` on key material.
- DEK rotation: a new version is used for writes. The background job `crypto.reencrypt` rewrites old rows in batches of 500.

**SEC-03. Encrypted fields.**
- `persons.{legal_name, email, phone, birth_date, address}_enc`
- `registrations.applicant_enc`
- `credentials.totp_secret_enc`
- `channel_configs.config_enc`
- File contents for purposes `checkin_photo`, `payment_proof`, `registration_doc` and `export` (the same envelope applied to the file bytes in 1 MiB chunks, using STREAM construction via `aead::stream`)

**SEC-04. Blind indexes and plaintext display names.**
- `email_bidx` / `phone_bidx` = `HMAC-SHA256(HKDF(bidx_base, tenant_id), normalized_value)[0..16]`.
- Normalisation: email is trimmed, lowercased and NFC. Phone is E.164 via `phonenumber` with the tenant country as default region.
- **Trade-off (documented, accepted):** `display_name` is plaintext so that search, sorting and lists stay practical. It holds the name the school uses day to day (e.g. "Ayu S."). The full legal name, contacts and birth date are encrypted. The privacy notice states this.

**SEC-05. Database roles and RLS.**
- `eq_owner` runs migrations and owns the objects.
- `eq_app` is used by the server: no BYPASSRLS, and DML only on tenant tables.
- `eq_platform` is used only by cloud platform jobs, with BYPASSRLS, never by request handlers.
- Every tenant table has `FORCE ROW LEVEL SECURITY`. `core::db::begin` always sets `app.tenant_id`. A connection returned to the pool is `RESET ALL` (sqlx `after_release` hook).
- The test TST-06 proves that an unset `app.tenant_id` yields 0 rows and that a cross-tenant id yields 0 rows.

**SEC-06. Transport.**
- The binary speaks plain HTTP on `RITMA_LISTEN`. TLS is terminated by Caddy (docker), the platform (1-click), or the tunnel (desktop).
- When `RITMA_PUBLIC_URL` is https, the server sets HSTS (`max-age=31536000; includeSubDomains`), marks cookies `Secure`, and honours `X-Forwarded-Proto` only from `RITMA_TRUSTED_PROXIES`.
- When `RITMA_PUBLIC_URL` is http and not localhost or a private IP, startup logs a loud warning and the setup wizard shows a red banner.

**SEC-07. Append-only tables.** `policies`, `audit.entries`, `payments` (no UPDATE except the status/verify columns through a function), `checkins` (review columns only through a function) and `consents`. Enforcement is by column-level GRANTs and/or `BEFORE UPDATE` triggers that raise on forbidden column changes.

## 3. Application security

- **SEC-10.** Password hashing and policy: IDN-02. TOTP: IDN-12.
- **SEC-11.** Rate limits: API-15. Lockout: IDN-06. Public forms: honeypot plus optional captcha (ENR-03).
- **SEC-12.** The system never emails or messages passwords. Invites and resets use single-use tokens (hash stored) with short TTLs.
- **SEC-13. Uploads.**
  - Each purpose has an allow-list of MIME types confirmed by magic bytes (`infer` crate) and a size limit: avatar 2 MB, photo per policy, payment proof 5 MB, logo 512 KB, import 10 MB.
  - Images are decoded and **re-encoded** (EXIF stripped).
  - PDFs (payment proof) are stored as-is but always served with `Content-Disposition: attachment` and `Content-Type: application/pdf`.
  - Files are stored under `RITMA_DATA_DIR/files/{tenant}/{yyyy}/{mm}/{file_id}` (local) or S3 with the same key. They are never served from a static directory.
- **SEC-14. SVG logos** are sanitised with an allow-list (no script, foreignObject, event attributes or external refs) using `usvg` parsing and re-serialisation, or rasterised to PNG if sanitising fails.
- **SEC-15. Signed file URLs.** `/api/v1/files/{id}?exp=<unix>&sig=<base64url(HMAC(filesig, tenant|id|exp|actor))>`, valid for 5 min, bound to the actor. Avatars and the logo are exempt (public within the tenant, cache 1 h).
- **SEC-20. HTTP security headers** (all responses):
  - `Content-Security-Policy: default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob: <tile-host>; connect-src 'self'; font-src 'self'; worker-src 'self' blob:; frame-ancestors 'none'; base-uri 'self'; form-action 'self'`
  - `X-Content-Type-Options: nosniff`
  - `Referrer-Policy: strict-origin-when-cross-origin`
  - `Permissions-Policy: camera=(self), geolocation=(self), microphone=()`
  - `Cross-Origin-Opener-Policy: same-origin`
  - `'unsafe-inline'` for styles is needed for the tenant theme style tag and Radix positioning. Scripts stay strict.
- **SEC-21. Logging hygiene.**
  - `tracing` fields named `email`, `phone`, `password`, `token`, `legal_name`, `address` or `birth_date` are rejected by a custom layer in debug builds (panic in tests) and redacted in release builds.
  - Request bodies are never logged.
  - TST-15 includes a log-capture test.
- **SEC-22. Supply chain.**
  - `cargo-deny` (licenses: allow MIT, Apache-2.0, BSD-2/3, ISC, Zlib, MPL-2.0, Unicode-3.0, AGPL-3.0-only for own crates; advisories: deny), `cargo-audit` daily, `pnpm audit --prod`, Renovate weekly.
  - Release artefacts are signed with cosign (keyless, GitHub OIDC) and published with an SBOM (CycloneDX for Rust and npm).
- **SEC-23. Secrets in config.** Channel credentials are encrypted (SEC-03) and returned redacted. Changing them requires re-entering the current password (step-up) for non-owner roles.

## 4. Privacy (UU PDP No. 27/2022, GDPR-ready)

| Requirement | How |
|---|---|
| Lawful basis and consent | Registration consent text versioned (`consents.text_hash`). Contract basis for operational processing. Consent for optional photos |
| Data minimisation | Address and photo optional by policy. Birth date only if age rules are used. No mandatory profile photos (legacy rule 38 changed) |
| Purpose limitation | Telemetry off by default and anonymous (DST-40) |
| Rights: access, rectification, erasure, portability | AUD-10/11, profile editing, export ZIP (machine-readable JSON + CSV) |
| Retention | Policy-driven: audit 7 y / anonymise 2 y, photos `photo_retention_days`, drafts 7 d, exports 24 h |
| Security of processing | §2–3 |
| Breach notification (UU PDP Art. 46: 3×24 h to subjects and authority) | `docs/runbooks/breach.md`: detection sources (audit verify failure, anomaly alerts), notification templates (EN/ID), timeline checklist. The cloud runbook adds the platform contact |
| Data residency | Cloud region selectable at signup: `ap-southeast-3` (Jakarta) default for ID, Singapore otherwise (CLD-05) |
| Processor agreements | Cloud: a DPA template in `docs/legal/` |

**SEC-31. Backups** made by `ritma backup` are encrypted with the backup key (XChaCha20-Poly1305, streaming). The UI warns: "Keep your secret key safe. Backups cannot be restored without it." The desktop app stores the key in the OS keychain and offers "Export recovery kit" (a PDF/QR with the secret key).
