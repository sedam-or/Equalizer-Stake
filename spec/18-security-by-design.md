# 18 — Security by Design

`10-security-privacy.md` lists the security **controls** the product has: crypto, RLS, headers, uploads, privacy. This document is the **standard for building every change securely**. It covers how features are designed, which coding rules apply, how security is tested and reviewed, and how vulnerabilities are handled.

It applies to every task, human or agent, whether or not the task cites it. Tags: **[tool]** means CI enforces it, and **[review]** means the reviewer checks it (§8).

Why this matters more than usual: Ritma stores **children's data**, **money records** and **many schools in one cloud database**. It's installed by non-technical owners who will never harden a server themselves. So the software has to be secure **by default, without configuration**.

## 1. Principles

- **SBD-01. Deny by default, fail closed.** No permission means no access. A missing tenant, an unknown role, a disabled module or an unparsable token is rejected. Errors in security checks deny; they never fall through to "allow".
- **SBD-02. Least privilege.**
  - DB roles: `eq_app` has no BYPASSRLS, and `eq_platform` is used only by platform jobs (SEC-05).
  - Default role grants are minimal (IDN-10), and the owner is read-mostly (IDN-11).
  - A new permission is granted only to the roles the spec names.
- **SBD-03. Complete mediation.** Every request is authenticated (or explicitly public), tenant-resolved, module-gated and permission-checked, in that order (ARC-04). There's no "internal" endpoint that skips the chain.
- **SBD-04. Defense in depth.** Each critical guarantee has at least two independent layers:
  - Tenancy: query scoping + RLS (SEC-05).
  - Double booking: domain check + `EXCLUDE` constraint.
  - Money: state machine + append-only + audit hash chain.
- **SBD-05. Secure defaults.** A fresh install is safe without touching settings: a required secret key, a setup token, HTTPS warnings, strict CSP, telemetry off, TOTP offered to owner and admin (§6).
- **SBD-06. Minimise data and surface.** Collect only what a policy needs (spec 10 §4). Expose only the fields a role needs, and only the endpoints the task specifies. Every new public endpoint is a design decision recorded in the spec.
- **SBD-07. One way to do security things.** Auth, crypto, file handling, signed URLs, HTML rendering and outbound HTTP each have **one** implementation in `eq-core` or `eq-identity`. Modules reuse it and never re-implement it (CA-31).
- **SBD-08. Never trust the client.** The browser, the desktop app, the CSV import and the webhook sender are all untrusted. Prices, totals, tenant ids, roles and timestamps that matter are computed or looked up server-side.

## 2. Data classification

**SBD-10.** Every stored field, file purpose, log field and API field has a class. A task that adds one **states its class**. If the class is S2 or S3 and the task doesn't say how it's protected, the task is blocked (AGENTS.md §0).

| Class | Examples | Storage | API and UI | Logs |
|---|---|---|---|---|
| **S3 Secret** | `RITMA_SECRET_KEY` and derived keys, DEKs, password hashes, refresh/invite/reset tokens, TOTP secrets, channel credentials | Hashed (tokens, passwords) or encrypted (SEC-02/03); key material zeroized in memory | **Never** returned. Credentials are redacted (SEC-23) | **Never**, not even partially |
| **S2 Sensitive PII** | Legal name, email, phone, birth date, address, check-in photos, payment proofs, registration documents, any field about a minor beyond the display name | Field or file encryption (SEC-03), blind index for lookup (SEC-04) | Only with the `read_pii` permission, scoped. Exports are audited (AUD-01) | Never. Log ids only (SEC-21) |
| **S1 Internal** | Display names, schedules, invoices and amounts, attendance marks, policies | Plaintext, tenant RLS | Role-scoped (IDN-10) | Ids and counts only |
| **S0 Public** | Public catalog, tenant branding, the public registration form schema | Plaintext | `auth: none` endpoints, explicitly listed in the spec | Allowed |

## 3. Trust boundaries

**SBD-20.** Data crossing a boundary **inward** is parsed into typed values at the boundary ("parse, don't validate"). Inner code only ever sees typed, validated data.

| # | Boundary | Inbound control | Code location |
|---|---|---|---|
| B1 | Browser / desktop / phone → API | TLS, auth chain (ARC-04), CSRF on cookie endpoints (IDN-05), body limits, rate limits (API-15), DTO parsing | `http.rs` adapters, `eq-api` middleware |
| B2 | Tenant ↔ tenant | `TenantCtx` from host/config only, RLS forced (SEC-05) | `eq_core::db::begin` |
| B3 | API → database | Compile-checked parameterised SQL (CQ-42), `eq_app` role | `repo.rs` |
| B4 | Module ↔ module | Events carry ids and minimal facts. Ports return only what the consumer needs, and no S2/S3 unless the spec says so (CA-12) | `eq_core::ports`, `events.rs` |
| B5 | Server → external providers (SMTP, WhatsApp, payment gateways, tunnel broker) | Allow-listed hosts from config, timeouts (CQ-25), TLS verification on, no redirects across hosts | Adapter clients |
| B6 | External → server (webhooks, imports, uploads) | Signature verification, size and type limits, sniffing and re-encoding (SEC-13/14), decompression limits | Adapter + `eq_core::files` |
| B7 | Operator → binary (env, CLI, desktop shell) | Config validated at startup, fail-fast on weak secrets (SEC-01), dev-only commands refuse outside `RITMA_ENV=dev` | `eq_core::config`, `eq-server` |
| B8 | Cloud platform → tenants | `eq_platform` used only in platform jobs, every action audited (CLD) | `eq-platform` |

The layers in spec 17 line up with these boundaries:
- `http.rs`: B1 parsing.
- `repo.rs`: B3.
- ports and events: B4.
- adapter clients: B5 and B6.
- `domain/`: enforces invariants on values that are already typed.

## 4. Threat modelling in tasks

**SBD-30. Security triggers.** A change triggers security design if it does any of these:

| Code | Trigger |
|---|---|
| T1 | Adds or changes an endpoint that is unauthenticated, public, or cookie-authenticated |
| T2 | Adds a permission, role, scope rule or anything about tenant resolution |
| T3 | Adds or reads an **S2/S3** field or file purpose (SBD-10) |
| T4 | Touches auth, sessions, tokens, crypto, keys, RLS, DB roles or audit |
| T5 | Handles uploads, imports, exports, HTML or PDF rendering, or CSV output |
| T6 | Talks to an external system, inbound (webhook) or outbound |
| T7 | Moves money or changes booking state |
| T8 | Runs across tenants (platform jobs, cloud admin) |

**SBD-31. The `## Security` section.** Every task with a trigger has a `## Security` section in its task file (see `tasks/_TEMPLATE.md`). The spec author writes it, using a short STRIDE pass:
- the triggers that apply (T1–T8);
- the threats in one line each (**S**poofing, **T**ampering, **R**epudiation, **I**nformation disclosure, **D**enial of service, **E**levation of privilege), naming only the relevant letters;
- the control for each threat, with the spec ID or SBD rule;
- at least one **abuse-case acceptance criterion** with a test name. Examples: "a guardian requests another family's invoice → 404", "a replayed webhook → ignored".

**Rules for draft and older tasks:**
- Tasks written before this standard (M0–M3) carry their controls inline. When implementing one, the implementer lists the triggers that applied and the controls added in the `## Report`.
- A `draft` task with triggers can't be promoted to `todo` until its `## Security` section is written (`tasks/README.md`).

**SBD-32.** A module spec (`spec/modules/*.md`) that adds a public endpoint, an S2 field or an external integration updates the threat table in spec 10 §1 in the same change.

## 5. Secure coding rules

### 5.1 Authentication, authorisation, tenancy

- **SBD-40.** Every handler declares its auth mode in its `#[utoipa::path]` security block: bearer, cookie or none. Every handler other than `auth: none` calls `actor.require(perm::…)` before any data access. [review; TST-04 checks every endpoint × allowed and forbidden roles]
- **SBD-41. Scope lives in the query.** `list`/`get` services take the `Actor` and apply a `Scope` in SQL (IDN-10), never by filtering in Rust after loading.
  - An object outside the actor's scope returns **404 `not_found`**, so its existence isn't revealed.
  - A visible object with a forbidden action returns **403 `forbidden`** (API-04).
- **SBD-42. The tenant comes from context only.** `tenant_id` is never read from a request body, query string or header other than the resolved `Host`/config (ARC-04 step 3). DTOs don't have a `tenant_id` field. [review]
- **SBD-43.** Login, reset and invite flows don't reveal whether an account exists:
  - the same response and status for an unknown user and a wrong password;
  - a dummy argon2 verification for unknown users, to equalise timing;
  - reset requests always answer "if the account exists, we've sent a link" (IDN-01/08).

### 5.2 Input and output

- **SBD-44. Parse at the boundary.** The full rules (pipeline stages, sanitising profiles, forbidden characters, field kinds, per-request matrix, file and import limits) are `22-validation-and-sanitization.md`.
  - DTO fields have explicit bounds: string length, list length, numeric range.
  - Enum inputs are closed. An unknown value is a 422, never a silent default.
  - Identifiers are normalised (NFC, trimmed, case rules per SEC-04) before comparison or lookup.
  - Dynamic sort and filter fields map through an allow-list `enum` and are never spliced into SQL (CQ-42, API-06).
- **SBD-45. Encode on output.**
  - JSON goes only through `serde`.
  - Server-rendered HTML (printables RPT-04, email templates) uses a template engine with **autoescape on** (`askama` or `minijinja`). Raw-HTML filters are forbidden.
  - In React, `dangerouslySetInnerHTML` is forbidden [tool: Biome `noDangerouslySetInnerHtml`].
  - `href`/`src` values built from data accept only `https:`, `mailto:`, `tel:` or same-origin relative URLs.
- **SBD-46. CSV and spreadsheet exports.** A cell value that starts with `=`, `+`, `-`, `@`, a tab or a carriage return gets a leading `'`, which prevents formula injection. Exports respect `read_pii` and are audited (AUD-01).
- **SBD-47. Outbound requests (SSRF).**
  - Server-side HTTP goes only to hosts from operator config or provider constants, through the one shared outbound client `eq_core::outbound` (rustls, TLS verification on, timeouts, redirects disabled, host allow-list). T-507 creates it with the first channel.
  - A URL supplied by a tenant or user is never fetched server-side. If a future feature needs that, its spec must add an allow-list and private-IP blocking (RFC 1918, loopback, link-local, metadata IPs) after DNS resolution.

### 5.3 Crypto, secrets, tokens

- **SBD-48. Crypto only through `eq_core::crypto`** (SEC-01..04).
  - No custom constructions, and no direct use of `aes-gcm`, `hmac` or `rand` outside `eq-core`/`eq-identity` [review; CODEOWNERS].
  - A new HKDF label, encrypted field or key use means updating spec 10 in the same change.
- **SBD-49. Tokens.**
  - Generate them with `OsRng`, with at least 128 bits of entropy, encoded base64url.
  - Store only their hash (SHA-256). Compare in constant time with `subtle::ConstantTimeEq`, never `==`, for tokens, signatures and HMACs.
  - Single-use tokens are consumed in the same transaction that checks them.
- **SBD-50. Secrets in memory.**
  - S3 values are wrapped in `secrecy::SecretString`/`SecretBox`, whose `Debug` redacts the value, and are zeroized on drop.
  - A struct holding a secret has no `Serialize` derive unless its spec requires it, and never derives a plain `Debug`.
- **SBD-51. Secrets in the repo.**
  - No secret, real credential or production URL with a token is ever committed. Test fixtures use obviously fake values (`test-secret-…`). [tool: gitleaks on every PR]
  - `.env` is git-ignored, and `.env.example` holds only placeholders.

### 5.4 Errors, logging, audit

- **SBD-52.** Client-facing errors expose only the problem `code`, a safe detail and field errors. Internal errors return a generic 500 with the `request_id` (ARC-12, ERR-05/30). Stack traces, SQL and file paths never reach the client.
- **SBD-53.** Security-relevant events are **audited**, not just logged (AUD-01):
  - login success and failure;
  - lockout, password and TOTP changes, role switch, role grants;
  - permission-denied bursts;
  - exports, key rotation, module toggles, policy changes and overrides.
- **SBD-54.** Logs follow SEC-21 and CQ-26: ids only, and no S2/S3 values. A security event in the logs carries `request_id`, `tenant_id`, `actor_id` and a stable `event` name, so alerts (OPS-08) can match on it.

### 5.5 Files, imports, resource limits

- **SBD-55.** Uploads follow SEC-13/14/15.
  - The client filename is untrusted. It's stored as display metadata only, never used in a path.
  - Imports (CSV/XLSX) are capped by size, row count and decompressed size. XLSX is a zip, so guard against zip bombs. They're parsed in `spawn_blocking`.
- **SBD-56. Bound everything.**
  - Every list is paginated, with a server-side maximum page size (API-05).
  - Every queue and batch has a maximum size.
  - Every external call has a timeout (CQ-25).
  - Every user-triggered expensive operation (export, report, import) is rate-limited and runs as a job.

### 5.6 Business-logic integrity

- **SBD-57.**
  - Money and booking creation are idempotent (API-08).
  - Concurrent edits use optimistic locking (API-07).
  - State changes go through aggregate methods (CA-23).
  - The database constraints are the last line of defence and are never removed to "make it work" (SBD-04).
  - Amounts the client sends are recomputed server-side. The server never trusts a client total, price or discount.

### 5.7 Frontend

- **SBD-58.**
  - The access token lives in memory only (IDN-03), and the refresh token is an HttpOnly cookie (IDN-04).
  - Tokens are never written to `localStorage`, `sessionStorage` or IndexedDB.
  - PII may be kept in browser storage only where the spec allows it: FE-30 drafts and the offline check-in queue. **All `eq:*` storage and the offline queue for the user are cleared on logout** and when the auth broadcast says `logout`.
  - No third-party scripts, fonts or CDNs at runtime (the CSP is `'self'`, SEC-20).
  - External links use `rel="noopener noreferrer"`.

### 5.8 Dependencies and build

- **SBD-59.**
  - A new dependency is reviewed for maintenance, license and build scripts (CQ-27). Avoid crates whose `build.rs` downloads anything.
  - Lockfiles (`Cargo.lock`, `pnpm-lock.yaml`) are committed, and CI installs with them frozen.
  - `unsafe` is forbidden (CQ-21).

## 6. Secure defaults and configuration

**SBD-60.** These defaults are normative. Changing any of them requires a spec change:
- the startup failures and warnings in SEC-01/06;
- the setup token (SET-11);
- the strict CSP and headers (SEC-20);
- telemetry off (DST-40);
- signed file URLs (SEC-15);
- lockout (IDN-06);
- TOTP offered at setup for owner and admin (IDN-12).

**SBD-61. Dev-only behaviour fails closed.**
- Seed commands, demo mode and debug endpoints refuse to run unless `RITMA_ENV=dev` or `RITMA_DEMO=1` is set **explicitly**.
- Debug-only routes are compiled out of release builds (`#[cfg(debug_assertions)]`).
- `RITMA_DEMO=1` also forces data resets and disables outbound channels.

**SBD-62.** New modules and endpoints are **off until declared**:
- a route that isn't registered through its module's `routes()` with a permission is unreachable;
- a new permission has no role grants beyond those the spec lists;
- a newly compiled optional module stays disabled for existing tenants until an owner enables it (no `tenant_modules` row means off, MOD-06).

## 7. Security testing

| ID | What | When | Tool / location |
|---|---|---|---|
| SBD-70 | Permission matrix: every endpoint × allowed role and forbidden role | Every PR | TST-04, `axum-test` |
| SBD-71 | Tenant isolation over every tenant table | Every PR | TST-06 |
| SBD-72 | Log hygiene: no S2/S3 in captured logs | Every PR | TST-16, SEC-21 |
| SBD-73 | Abuse-case tests from each task's `## Security` section | Every PR | named in the task |
| SBD-74 | Secret scanning | Every PR | `gitleaks` (CI job `security`) |
| SBD-75 | Dependency advisories | Every PR + daily | `cargo deny check advisories`, `cargo audit`, `pnpm audit --prod` (SEC-22) |
| SBD-76 | Property and fuzz tests for parsers of untrusted input: CSV/XLSX import, multipart, preset YAML, ICS tokens, signed URLs | Property tests per PR. `cargo-fuzz` targets nightly from M7 | `proptest`, `fuzz/` |
| SBD-77 | DAST baseline against the release build | Release branches, from M7 | OWASP ZAP baseline in the e2e job |
| SBD-78 | Independent penetration test | Before cloud GA (M9) and yearly | External; findings tracked as private advisories |

## 8. Review and lifecycle

**SBD-80. Security review.**
- A PR whose task has triggers (SBD-30), or that touches a path owned by `@security` in CODEOWNERS, gets the `security-review` label.
- It needs approval from a maintainer with the **Security** area in `MAINTAINERS.md`. The author can't approve their own security review.
- The reviewer records findings in the task's `## Review` section (`WORKFLOW.md` §5).

**SBD-81. Security review checklist** (in addition to `16-code-quality.md` §9):
1. **Auth chain:** auth mode declared, `require(perm)` before data access, scope in SQL, 404 versus 403 correct (SBD-40/41).
2. **Tenant:** no client-supplied tenant id, RLS on new tables, no `eq_platform` outside platform jobs (SBD-42, SEC-05).
3. **Data class:** new fields classified, S2/S3 encrypted or hashed, not returned without `read_pii`, not logged (SBD-10).
4. **Input:** bounds on DTOs, closed enums, allow-listed sort and filter, files sniffed and limited (SBD-44/55).
5. **Output:** autoescaped HTML, no raw HTML in React, CSV formula guard, safe URLs (SBD-45/46).
6. **Crypto and tokens:** through `eq_core::crypto`, constant-time compares, hashed tokens, `SecretString` (SBD-48..50).
7. **External calls:** shared client, allow-listed host, timeout; webhooks verified and idempotent (SBD-47, SBD-63).
8. **Integrity:** idempotency, If-Match, server-side totals, constraints intact (SBD-57).
9. **Audit:** security events audited (SBD-53).
10. **Tests:** abuse-case tests present and failing without the control (SBD-73).

**SBD-63. Inbound webhooks**, such as payment gateways (CLD subscription billing) and future channel callbacks:
- The signature is verified with HMAC or the provider's scheme, using a constant-time compare, **before** parsing the body.
- The timestamp must be within the provider's tolerance (5 minutes unless the provider specifies otherwise).
- Events are deduplicated by provider event id.
- The raw body is never logged.

**SBD-64. Exceptions.** A security rule is never waived inside a feature PR. A deviation needs a spec change to this file or to spec 10, approved under SBD-80, with the residual risk written down.

## 9. Vulnerabilities and incidents

- **SBD-90.** External reports follow `SECURITY.md` (private advisory, 48 h acknowledgement, 90-day disclosure).
  - Fixes are developed in a private fork of the advisory.
  - Every fix includes a regression test.
  - The advisory gets a CVE when applicable, and the release notes say "security fix" without exploit detail until most users have upgraded.
- **SBD-91. Runbooks** live in `docs/runbooks/` (from M7):
  - `breach.md` (spec 10 §4, UU PDP 3×24 h);
  - `key-compromise.md` (`ritma rotate-keys`, revoke refresh sessions, rotate channel credentials);
  - `vulnerable-dependency.md` (triage, patch release, advisory).
- **SBD-92.** A failed audit hash-chain verification (AUD-04), an RLS-isolation test failure, or a secret found in the repo is a **stop-the-line** event. It's fixed before any feature work merges.

## 10. OWASP Top 10 (2021) mapping

| OWASP | Ritma controls |
|---|---|
| A01 Broken access control | IDN-10/11, SBD-40..42, SEC-05 RLS, TST-04/06, SBD-62 |
| A02 Cryptographic failures | SEC-01..04, SEC-31, SBD-48..50, SBD-10 |
| A03 Injection | CQ-42 compile-checked SQL, SBD-44..46, SEC-14 SVG sanitisation |
| A04 Insecure design | This document §4 (triggers, STRIDE, abuse cases), ADR-0005/0006, SBD-04 |
| A05 Security misconfiguration | SEC-06/20, SBD-60/61, the installer defaults (DST-30) |
| A06 Vulnerable and outdated components | SEC-22, SBD-59, SBD-75, Renovate |
| A07 Identification and authentication failures | IDN-02..08/12, SBD-43, SBD-49, API-15 |
| A08 Software and data integrity failures | cosign + SBOM (SEC-22), AUD-03/04 hash chain, SBD-63 webhooks, SEC-07 append-only |
| A09 Logging and monitoring failures | SEC-21, SBD-53/54, AUD-01, OPS-08 alerts |
| A10 SSRF | SBD-47 |

## 11. Enforcement

| Rule | How | Set up by |
|---|---|---|
| SBD-40 | TST-04 permission matrix + review | T-112/T-113 (M1) |
| SBD-41/42 | TST-06 + review | T-116 |
| SBD-45 (React) | Biome `noDangerouslySetInnerHtml` at `error` | T-005 |
| SBD-48 | CODEOWNERS on `crates/eq-core/src/crypto*` and `crates/eq-identity/`; review | this change |
| SBD-49/50 | `subtle` and `secrecy` pinned in `[workspace.dependencies]`; review | T-001 |
| SBD-51, SBD-74 | `gitleaks` job in CI | T-006 |
| SBD-75 | `cargo deny` (PR), scheduled `security.yml` daily: `cargo audit`, `pnpm audit --prod` | T-006 |
| SBD-46, SBD-58 | Named in the implementing tasks: T-607 (exports), T-306 (logout clears storage), T-511 (offline queue cleanup) | — |
| SBD-47, SBD-50 | `eq_core::outbound` + `SecretString` for channel credentials | T-507 |
| SBD-63 | Webhook verification before parsing, deduplication by event id | T-905 |
| SBD-76/77 | `cargo-fuzz` targets, ZAP baseline | T-711 (M7) |
| Everything else | SBD-81 checklist, `security-review` label | — |
