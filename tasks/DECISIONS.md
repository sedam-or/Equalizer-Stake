# Task decisions and ratification register

This register maps each recorded self-resolution to a stable decision ID. Unless explicitly marked otherwise, the implementer selected the option needed to keep work moving and the architect still needs to ratify it. Source task Questions retain their original text and history. Reports point back to these IDs under Deviations.

| ID | Task | Source | Self-resolution |
|---|---|---|---|
| TD-001 | T-002 | `tasks/M0-foundation/T-002-core-config-clock-errors.md:275` | OPS-01 says the trusted-proxy default includes private ranges in Docker but supplies no CIDRs or Docker-detection contract → when no explicit proxy list is configured, use only OPS-01's exact loopback CIDRs and require explicit `RITMA_TRUSTED_PROXIES` for additional proxies, because SBD-20's deny-by-default reading is safer than trusting an unspecified range. |
| TD-002 | T-002 | `tasks/M0-foundation/T-002-core-config-clock-errors.md:281` | `RITMA_PUBLIC_URL` must parse as HTTP(S), while Config classifies its non-S3 fields as S0/S1 and SBD-50 treats credential-bearing URLs as secrets → reject a public URL containing username or password because a public origin is not specified as an S3 secret and must not leak credentials through Config Debug. |
| TD-003 | T-002 | `tasks/M0-foundation/T-002-core-config-clock-errors.md:282` | the three optional CAPTCHA environment values map to `Option<CaptchaConfig>`, but no partial-configuration behavior is given → reject a partial provider/site-key/secret tuple because the option represents the complete configuration and silent disabling would hide invalid operator input. |
| TD-004 | T-003 | `tasks/M0-foundation/T-003-core-db-and-roles.md:201` | `ActorCtx::system(kind)` requests a kind but the task's required minimal struct and the actor schema define no kind field → accept the signature but do not store or serialize the kind; set role to `System` and keep the transaction actor id empty because the task requires the empty system actor id and spec 06/29 define no kind column. |
| TD-005 | T-003 | `tasks/M0-foundation/T-003-core-db-and-roles.md:202` | T-003 requires Testcontainers integration tests but omits their path and manifest from `## Files`; the pinned workspace Testcontainers 0.23 graph fails enabled security audits → add only the relevant `Cargo.toml`, `crates/eq-core/Cargo.toml` test-only workspace dependency and the already-listed `crates/eq-core/tests/db.rs`, and use Testcontainers 0.28 because its compatible current Docker client graph avoids those advisories while keeping the task's stated test framework and audit gates intact. |
| TD-006 | T-003 | `tasks/M0-foundation/T-003-core-db-and-roles.md:203` | the test note calls the RLS probe a temporary table but requires reading it through a second `eq_app` connection → use a regular probe table inside each disposable Testcontainers database, because PostgreSQL temporary tables are visible only to their creating session and cannot be read by the separate app pool. |
| TD-007 | T-004 | `tasks/M0-foundation/T-004-server-cli-http-skeleton.md:81` | the task's Interfaces, test table and acceptance criteria require direct framework/runtime dependencies and the listed integration-test files, but `## Files` omits their manifests and test paths → add only the three relevant Cargo manifests and the two named test files because these paths are required to compile and verify the task's specified interfaces. |
| TD-008 | T-004 | `tasks/M0-foundation/T-004-server-cli-http-skeleton.md:82` | SQLx creates `_sqlx_migrations` before `0001_core_bootstrap.sql` sets default privileges, so `eq_app` cannot read migration status and `/readyz` reports `migrations_pending` after a successful migration → after migrations, grant `eq_app` only `SELECT` on the SQLx metadata table using the already specified DML grant boundary; this exposes migration versions only and is required by OPS-06. |
| TD-009 | T-004 | `tasks/M0-foundation/T-004-server-cli-http-skeleton.md:83` | the task's startup sequence initializes tracing before Config, but the repository's Config boundary forbids reading environment variables in telemetry (`cargo clippy` `disallowed_methods`, OPS-01) → load and validate Config first, then initialize tracing from `Config.log` and `Config.log_format`, preserving the single configuration source required by the workspace lint. |
| TD-010 | T-005 | `tasks/M0-foundation/T-005-web-scaffold-and-embed.md:156` | The implementation note could be read to add ETags to every embedded file, but CACHE-20 specifies no validator for hashed assets and CACHE-21 specifies ETags for the index/service-worker/manifest shell files → add ETags and honor `If-None-Match` only for those shell files because the normative cache inventory is specific to each response class. |
| TD-011 | T-005 | `tasks/M0-foundation/T-005-web-scaffold-and-embed.md:157` | The Files table omitted required package manifests/lockfile and the existing router modules needed to wire the fallback inside the request-ID/trace stack → add the directly affected paths to the Files table because Scope explicitly requires dependency setup and fallback routing while T-004's Follow-ups require preserving middleware. |
| TD-012 | T-006 | `tasks/M0-foundation/T-006-ci-makefile-devdb.md:172` | The Files table omitted `CONTRIBUTING.md`, which the implementation notes require, and root Cargo files needed to keep the mandated license allow-list green → add those paths; switch the existing SQLx/Reqwest Rustls features to native OS roots so the dependency graph no longer includes the unallowed CDLA-Permissive-2.0 webpki-roots bundle, while preserving TLS validation and the task's Rustls requirement. |
| TD-013 | T-006 | `tasks/M0-foundation/T-006-ci-makefile-devdb.md:173` | The task schedules Cargo and pnpm audits daily/manual, while TST-12 and SBD-75 require dependency advisory checks on every PR as well → add `cargo audit` to the Rust PR job and `pnpm audit --prod` to the web PR job, retaining the daily/manual workflow, because the normative spec requires both frequencies. |
| TD-014 | T-006 | `tasks/M0-foundation/T-006-ci-makefile-devdb.md:174` | Task prose says `unmaintained = "warn"`, but cargo-deny 0.20.2 accepts only scope values (`all`, `workspace`, `transitive`, `none`) and `all` is the strict default → use `unmaintained = "all"` to preserve the strictest permitted check because cargo-deny does not have a warning-only mode. |
| TD-015 | T-007 | `tasks/M0-foundation/T-007-testkit.md:168` | The interface sketch stores credential-bearing PostgreSQL URLs as public `String` fields and offers no redaction contract → use `secrecy::SecretString`, hand-written redacting `Debug`, and no `Clone`/`Serialize`/derived `Debug` for secret-bearing structs; test that formatting leaks neither fake credential, because SBD-50 governs secret storage. |
| TD-016 | T-007 | `tasks/M0-foundation/T-007-testkit.md:169` | The task's Tests section requires `crates/eq-testkit/tests/isolation.rs`, the implementation depends on Cargo manifests, but Files omits those paths → add only the directly required workspace/testkit manifests and test file to Files; this follows the test/interface contract and keeps the manifest edits to dependencies needed for the listed code. |
| TD-017 | T-007 | `tasks/M0-foundation/T-007-testkit.md:170` | The task's container interface specifies `testcontainers-modules` 0.11, which requires the incompatible Testcontainers 0.23 API, while T-003 pins Testcontainers 0.28 to keep enabled dependency audits clean; the newest module release also selects an incompatible exact `bollard-stubs` version → use Testcontainers 0.28 `GenericImage` with the exact `postgres:16-alpine` image because TST-03 requires PostgreSQL 16 through Testcontainers and the workspace pin is the established dependency contract. |
| TD-018 | T-008 | `tasks/M0-foundation/T-008-m0-integration.md:142` | T-008 Scope requires Playwright files, Makefile and CI workflow changes but the task has no Files section; adding Playwright also requires its web manifest and lockfile → add only those directly required paths because they are explicitly needed to satisfy Scope and the mapped Playwright tool. |
| TD-019 | T-008 | `tasks/M0-foundation/T-008-m0-integration.md:143` | The Playwright config is required at `web/e2e/playwright.config.ts`, but a bare `pnpm -C web exec playwright test` searches from `web` and will not load that nested config → pass `--config=e2e/playwright.config.ts` in CI and `make e2e`, keeping the required config location and tests while making execution deterministic. |
| TD-020 | T-008 | `tasks/M0-foundation/T-008-m0-integration.md:144` | Playwright writes generated JS/HTML reports and traces under `web/`, which Biome's recursive `includes` otherwise treats as source after E2E → exclude only `test-results` and `playwright-report` from Biome inputs, preserving the task-required failure artifact and existing lint rules. |
| TD-021 | T-101 | `tasks/M1-core/T-101-core-schema.md:156` | The Tests section requires `crates/eq-core/tests/schema.rs`, but the Files table omitted it → add that exact test path because the named acceptance tests cannot be implemented without it. |
| TD-022 | T-101 | `tasks/M1-core/T-101-core-schema.md:157` | `resolve_single_tenant()` had no declared return shape while the slug resolver returns `(id, slug, status)` → use the same row shape so the tenant resolver contract is consistent and can expose the status required by tenant resolution. |
| TD-023 | T-101 | `tasks/M1-core/T-101-core-schema.md:158` | The SECURITY DEFINER resolver's search path was unspecified → pin it to `pg_catalog` and qualify `public.tenants`, because SBD-01 requires fail-closed security and prevents a shadow object from changing what the definer reads. |
| TD-024 | T-102 | `tasks/M1-core/T-102-money-and-policy-types.md:228` | Currency's derive-only deserialization sketch and unspecified helper signatures do not state CAN-10 lowercase-input normalization or the `iso_currency` lookup shape -> deserialize through `TryFrom<String>`, serialize uppercase canonical codes, expose `Option` for code/exponent lookup, and format minor units with the ISO exponent because CAN-10 and OSS-02 define the wire form and metadata source. |
| TD-025 | T-102 | `tasks/M1-core/T-102-money-and-policy-types.md:229` | `LocalTime` requires `from_const` while CAN-10 permits `24:00` only as a `TimeInterval.end` -> cap standalone `LocalTime` at 23:59 and store minute 1440 only in the private interval-end representation, with `TimeInterval::until_midnight` for construction, because canonical local times must remain valid outside interval context. |
| TD-026 | T-102a | `tasks/M1-core/T-102a-field-kinds-and-validated-extractors.md:264` | AC2 requires source body-key ordering, while the specified DTO plus `Collector::field` interface retains only the adapter's later call order → define the returned error order as `Collector` invocation order and test that contract; source key order cannot be recovered after DTO deserialization without changing the interface (VAL-04). |
| TD-027 | T-102a | `tasks/M1-core/T-102a-field-kinds-and-validated-extractors.md:265` | The generic `Raw<K>`/`Password` path carries a secret although only the profile behavior is specified → use secrecy-backed input/output wrappers with hand-written redacted `Debug` and no `Clone`/`Serialize`/plain `Debug` derive (SBD-50). |
| TD-028 | T-102a | `tasks/M1-core/T-102a-field-kinds-and-validated-extractors.md:266` | The task lists only `crates/eq-core/Cargo.toml` for new dependencies, while CQ-27 requires a single workspace dependency declaration → add those declarations to the workspace `Cargo.toml` and reference them with `.workspace = true` in `eq-core` (CQ-27). |
| TD-029 | T-102a | `tasks/M1-core/T-102a-field-kinds-and-validated-extractors.md:268` | VAL-11/VAL-21 classify `Key` under the identifier profile, which normally trims input, while normative CAN-10 accepts no alternate `Key` form and says no trimming → require input to already match the canonical `Key` pattern and reject surrounding whitespace as `invalid_format`; CAN-10 §1 says its representation table wins (CAN-10, VAL-21). |
| TD-030 | T-102a | `tasks/M1-core/T-102a-field-kinds-and-validated-extractors.md:269` | CAN-32 requires unknown query parameters to return 400, while the specified generic `ValidatedQuery<T>` delegates to `axum_extra::Query<T>` and cannot discover allowed keys from `DeserializeOwned` alone → require each query DTO to use `#[serde(deny_unknown_fields)]` and document that contract; a generic allow-list would add an interface not named by the task or spec (CAN-32). |
| TD-031 | T-102a | `tasks/M1-core/T-102a-field-kinds-and-validated-extractors.md:270` | T-102a names `eq_core::errors::FIELD_CODES`, while T-002 shipped the shared registry as `eq_core::errors::FIELD_ERROR_CODES` → use the dependency's exported name and reuse its registered field codes (T-002, ERR-21). |
| TD-032 | T-103 | `tasks/M1-core/T-103-core-crypto.md:172` | The Scope lists field decrypt and stream helpers without DB transaction/keyring context, while the declared `FieldCipher` trait requires `&mut Tx`; it also omits source, manifest and test paths from a Files table → expose operations as `Keyring` methods using the caller's transaction, keep `tenant_keys` SQL in private `crypto/repo.rs` per CA-30, and include the paths required by Scope/Tests/architecture, because key lookup needs the caller transaction and no global keyring is specified. |
| TD-033 | T-103 | `tasks/M1-core/T-103-core-crypto.md:173` | The cache is described as per-tenant, but AC3 requires an old ciphertext to decrypt after a new active DEK version is inserted → cache each `(tenant_id, key_version)` entry under that tenant so old and new keys coexist for the stated TTL. |
| TD-034 | T-103 | `tasks/M1-core/T-103-core-crypto.md:174` | The interface's `Zeroizing` key arrays need a secret-safe public boundary → wrap key material in `secrecy::SecretBox` (and DEKs in `SecretBox<Zeroizing<[u8; 32]>>`) with hand-written redacting `Debug`, following the pre-authorised SBD-50 correction. |
| TD-035 | T-104 | `tasks/M1-core/T-104-tenant-context-and-db-begin.md:184` | T-104 names `RITMA_BASE_DOMAIN` for multi-mode Host routing, but the shipped T-002 `Config` has no field for it → add an optional validated Config field, required only in multi mode, using the existing missing-config error; single mode keeps no base domain because the tenant is resolved from the database function. |
| TD-036 | T-104 | `tasks/M1-core/T-104-tenant-context-and-db-begin.md:185` | The task has no Files or Security section despite naming source/test work and triggering SBD-30 → add the paths derived from Scope/Tests and the controls directly required by SBD-20/42/81, without adding permissions or new behavior. |
| TD-037 | T-104 | `tasks/M1-core/T-104-tenant-context-and-db-begin.md:186` | The task requires six fields in `TenantInfo`, but T-101's resolvers expose only three and `tenants` has no RLS → use a function-only `0009_core_tenant_resolver.sql` in ERD-06's reserved migration gap, returning exactly the existing `id`, `slug`, `status`, `timezone`, `locale`, and `currency` columns through a pinned `SECURITY DEFINER` function granted to the already-authorized `eq_app` resolver role; this follows spec/06 §4.1 and spec/29 ERD-02/06, adds no table or column, and applies the user's authorization to handle this blocker in the relevant task files. |
| TD-038 | T-105 | `tasks/M1-core/T-105-module-system.md:203` | The normative Module trait is in eq-core but its route signature needs AppState, which T-004 placed in eq-api, and T-105 requires BTreeSet<ModuleId> while the normative derives omit Ord → move the shared AppState definition to eq-core and re-export it from eq-api, and derive Ord/PartialOrd on ModuleId, preserving the normative route signature and required set API while keeping the dependency direction. |
| TD-039 | T-105 | `tasks/M1-core/T-105-module-system.md:204` | T-105 omits Files and Security sections despite requiring module state changes, tenant-scoped caching and source/test additions → add the exact paths from Scope/Tests and the controls from MOD-01/06, CA-30/31 and SBD-42/73/81. |
| TD-040 | T-105 | `tasks/M1-core/T-105-module-system.md:205` | The Module signature refers to ChecklistItem, EventHandler, JobHandler, CronSpec, PortRegistry and sink interfaces without declaring all their shapes; T-107/T-108 define event and job handlers, T-314 defines checklist fields, and ARC-08 defines typed port registration → use those contracts for the scaffolding, keep preset resolution behind PolicyInit, and use T-113's exact `required_by` response extension because its API shape is more precise than T-105's `(+ missing)` note. |
| TD-041 | T-106 | `tasks/M1-core/T-106-policy-store.md:193` | T-106's patch-taking `ensure_initial` sketch differs from the shipped T-105 `PolicyInit` signature → preserve the no-patch port signature and keep a patch-taking `PolicyStore` method for setup callers, because T-105 is the shipped code contract and spec 05 §2.2 step 7 initializes every module's policy during setup. |
| TD-042 | T-106 | `tasks/M1-core/T-106-policy-store.md:194` | T-106 requires policy-change audit/event effects through T-105 sink traits, whose shipped methods only cover module toggles → add policy-specific methods to those same typed traits using the task's exact audit action and event name, because the task explicitly selects these ports and POL-03/SBD-53 require the effects. |
| TD-043 | T-107c | `tasks/M1-core/T-107c-retry-dead-letter.md:94` | T-107 asks for retry/dead timestamps absent from the exhaustive schema → keep the mapped DDL and encode retry due as `ritma-retry-v1:<due_unix_ms>:handler_failed`, dead as `ritma-dead-v1:handler_failed`, and the predicate as `attempts >= 5`; the fifth backoff is unused at the specified terminal threshold. |
| TD-044 | T-107a | `tasks/M1-core/T-107a-event-emission.md:164` | CAN-60 lists event roles as `admin \| system`, while T-105 ships a closed `ActorRole` with additional roles → carry actor/request context in the event envelope and serialize the shipped role values rather than collapse distinct actors. |
| TD-045 | T-107c | `tasks/M1-core/T-107c-retry-dead-letter.md:95` | STM-80 names audited owner Retry/Discard actions without an authorized permission/audit contract in this task → create a high-severity `outbox_dead_letter` attention item with an event-id-only payload and stable dedupe key, and defer owner mutations until the authorized audited surface exists. |
| TD-046 | T-106 | `tasks/M1-core/T-106-policy-store.md:200` | Missing-row policy fallback needs the tenant's stored preset identity, but `eq_app` cannot select `tenants` → resolve only `(preset_id, preset_version)` with a least-privilege function and keep embedded patch lookup behind `PresetPatchSource`, preserving `PolicyStore` APIs and avoiding cross-module SQL joins. |
| TD-047 | T-106 | `tasks/M1-core/T-106-policy-store.md:201` | A prior resolver sketch included a transaction-context guard, while the delegated directive specifies an exact SQL SECURITY DEFINER function → implement the delegated body and pinned `search_path`, return existing identity/version only, and grant execution to `eq_app`; the store validates the supplied tenant against its transaction context. |
| TD-048 | T-106 | `tasks/M1-core/T-106-policy-store.md:202` | T-106 has no migration row, but its delegated resolver needs a function migration and `0010`/`0012` are assigned → use unused slot `0011_core_tenant_preset_resolver.sql`; it creates no table or column and leaves `spec/29` unchanged. |
| TD-049 | T-106 | `tasks/M1-core/T-106-policy-store.md:203` | `jsonschema` 0.30 pulls MIT-0 `borrow-or-share` through `referencing` even with default features disabled, failing the unchanged deny gate → use `jsonschema` 0.17.1 with default features off and `draft202012` enabled; the same crate supports the required draft and `cargo deny check` passes. |
| TD-050 | T-106 | `tasks/M1-core/T-106-policy-store.md:204` | The typed policy audit operation would exceed CQ-101 guidance if passed as positional fields → group module, version, previous document and new document in `PolicyChange`; hand-write `Debug` to redact document values without changing the audit payload. |
| TD-051 | T-106 | `tasks/M1-core/T-106-policy-store.md:205` | Missing-row fallback must return `Versioned<P>` but has no persisted `effective_from` → use synthetic version 0 and the lookup's requested instant, avoiding an invented date or out-of-range timestamp. |
| TD-052 | T-106 | `tasks/M1-core/T-106-policy-store.md:206` | The cache needs a bounded technical capacity → cap it at 1,024 tenant/module keys as a process-local memory guard without truncating policy history. |
| TD-053 | T-106 | `tasks/M1-core/T-106-policy-store.md:207` | AC1 names `policy::tests::effective_dating`, but the test needs the database fixture from the integration test harness → nest the test under the exact `policy::tests` module path in the integration file. |
| TD-054 | T-107 | `tasks/M1-core/T-107-outbox-dispatcher.md:173` | The T-107 prototype exceeded CQ-101's approximate 400-line task guide → split schema/emission, handler delivery, and retry/dead-letter lifecycle into T-107a/b/c; keep each original AC/test in one part and point each consumer to the part that provides its contract. |
| TD-055 | T-107a | `tasks/M1-core/T-107a-event-emission.md:165` | The task sketch uses `String` for `Event.event_type`, while shipped T-105 uses `&'static str` → retain the shipped shape and the mapped PostgreSQL `text` column. |
| TD-056 | T-107b | `tasks/M1-core/T-107b-dispatcher-delivery.md:93` | The dispatcher contract does not define an event with no registered subscriber → treat the empty handler set as successful delivery and mark the event dispatched, following FLW-03's “all handlers succeeded” transition. |
| TD-057 | T-107b | `tasks/M1-core/T-107b-dispatcher-delivery.md:94` | FLW-03 requires sequential delivery per aggregate, but `FOR UPDATE SKIP LOCKED` over a bounded batch can let a second dispatcher claim later rows beyond the first 100 rows → claim only the oldest pending event per non-null `(tenant_id, aggregate_id)` until that event is marked dispatched, preserving ordering without a schema change; leave null-aggregate rows independently claimable. |

| TD-058 | T-107c | `tasks/M1-core/T-107c-retry-dead-letter.md` | Apply ERR-52's retryable/permanent classification to event handlers and dead-letter permanent failures immediately. |
| TD-059 | T-108 | `tasks/M1-core/T-108-jobs-and-cron.md` | Use the spec 24 `cron` crate and adapt the five-field schedule syntax. |
| TD-060 | T-108 | `tasks/M1-core/T-108-jobs-and-cron.md` | Use ERR-51's typed retryable/permanent `JobError` variants. |
| TD-061 | T-108 | `tasks/M1-core/T-108-jobs-and-cron.md` | Bound worker claims with a technical batch constant of 100. |
| TD-062 | T-108 | `tasks/M1-core/T-108-jobs-and-cron.md` | Accept trusted active `TenantInfo` metadata; do not add a cross-tenant resolver or grant. |
| TD-063 | T-110, T-112, T-113, T-117, T-302 | Split oversized task contracts into dependent parts while preserving all listed ACs and tests. |
| TD-064 | T-108 | `tasks/M1-core/T-108-jobs-and-cron.md` | Reuse T-107c's OPS-08 attention-item shape for tenant job dead letters. |
| TD-065 | T-108 | `tasks/M1-core/T-108-jobs-and-cron.md` | Do not invent platform tenancy/schema for attention; retain ids-only logging and report the unmet platform alert row. |
| TD-066 | todo/draft tasks listed in `tasks/HYGIENE.md` | Add or complete spec-backed SBD-30 Security sections; use `No triggers.` only after review. |
| TD-067 | todo/draft tasks listed in `tasks/HYGIENE.md` | Fill required task-template headings from each task's existing contract without adding behavior. |
| TD-068 | T-110 | `tasks/M1-core/T-110-identity-login-refresh.md` | Keep the spec 06/ERD refresh-session schema; detect reuse through `revoked_at` and a newer row in the same `family_id` rather than adding `revoked_reason`. |
| TD-069 | T-209, T-210 | `tasks/M2-scheduling/T-209-series.md` | Make T-209 the provider of the reusable materialisation core; T-210 extends it with jobs, cycle caps and closure behavior. |
| TD-070 | T-804, T-908 | `tasks/M8-desktop/T-804-secure-tunnel.md` | Keep a mockable broker-client seam in M8 and make the M9 broker service depend on and implement that contract. |
| TD-071 | T-109 | `tasks/M1-core/T-109-audit.md` | Add `reason` to recursive AUD-05 redaction because ERD-05 requires free-text keys that may describe a minor to be anonymized. |
| TD-072 | T-109 | `tasks/M1-core/T-109-audit.md` | Use the `audit.checkpoints.at DEFAULT now()` from spec 06 §4.11, which is absent from the abbreviated task sketch. |
| TD-073 | T-109 | `tasks/M1-core/T-109-audit.md` | Supply the existing `PolicyStore` to audit jobs through an optional `JobCtx` dependency because `audit.retention` must use it and T-108's shipped context does not carry it. |
| TD-084 | T-109e | `tasks/M1-core/T-109e-audit-chain.md:30` | Treat `from_seq` and `to_seq` as inclusive endpoints because the named bounds should both be checked rather than silently omitted. |
| TD-085 | T-109e | `tasks/M1-core/T-109e-audit-chain.md:23` | Keep audit-chain SQL in the existing repository adapter because CA-11/CQ-24 require persistence queries to stay in the module's `repo.rs`. |
| TD-086 | T-109e | `tasks/M1-core/T-109e-audit-chain.md:45` | Fail sealing without writes when a later entry/checkpoint is already sealed after the first unsealed sequence, because extending that chain would violate sequence order and append-only integrity. |
| TD-087 | T-109g | `tasks/M1-core/T-109g-audit-jobs.md` | Add the omitted repository paths and SQLx metadata because CA-11/CA-30/CQ-24/CQ-42 require owner-only table access and checked-in descriptions for compile-time queries. |
| TD-088 | T-109g | `tasks/M1-core/T-109g-audit-jobs.md` | Use the descriptive attention kind `audit_chain_mismatch`, sequence-only payload, and tenant-scoped dedupe key because the task requires a high-severity audit mismatch item but supplies no exact stable identifier. |
| TD-089 | T-109g | `tasks/M1-core/T-109g-audit-jobs.md` | Implement the core-owned `attention::raise` API named by STM-14 because the task's required item targets core-owned `attention_items` and no API is shipped. |
| TD-090 | T-109h | `tasks/M1-core/T-109h-audit-module.md` | Schedule `audit.verify_recent` daily at 03:30 tenant-local after the parent-specified 03:00 retention job, because the parent requires a nightly verifier schedule but omits its exact time. |
| TD-091 | T-109h | `tasks/M1-core/T-109h-audit-module.md` | Classify module-toggle and policy-change audit records as Security per SBD-53 and include their module identity/state in snapshots, because the sink contract names actions but does not provide this context and ModuleId has no UUID form. |
| TD-092 | T-109h | `tasks/M1-core/T-109h-audit-module.md` | Register the implemented AuditSink port and add explicit integration tests for caller-transaction writes and tenant mismatch rejection because the task's original security test only checked module registration. |
| TD-093 | T-109 | `tasks/M1-core/T-109-audit.md` | Acquire the shared tenant advisory transaction lock before each audit record insert and test writer/sealer serialization so a slower lower-sequence transaction cannot commit after a higher-sequence row is sealed. |
| TD-094 | T-109 | `tasks/M1-core/T-109-audit.md` | Make the writer/sealer regression test observe the sealer blocked on the PostgreSQL advisory lock before committing the writer; bound test-only observation with a 10-second timeout and 10-millisecond polling interval. |
| TD-095 | T-110 | `tasks/M1-core/T-110-identity-login-refresh.md` | Align the tracker-only parent with its child task frontmatter and Scope: list T-110a as depending on T-109d, mark both children todo, and replace the implementation migration path with the parent task/index files. |
| TD-096 | T-110a | `tasks/M1-core/T-110a-identity-schema,-login-and-credential-validation.md` | The child combines schema/policy, password/JWT, login persistence, HTTP login and actor/role-switch implementation and would exceed CQ-101's review guide → split those concerns into five ordered parts, preserve each original AC/test exactly once, and retain T-110a as a final aggregate task. |
| TD-097 | T-110a1b | `tasks/M1-core/T-110a1b-validated-identity-policy-model.md` | The parent policy sketch exposes bounded values as public fields → use private fields and validating constructors/deserialization per CQ-23/CAN-10 while keeping the invariant-free generic RoleMap fields public. |
| TD-098 | T-110a1 | `tasks/M1-core/T-110a1-identity-schema-and-policy.md` | The policy value and module implementation combined with the mapped schema would exceed CQ-101's review guide → keep T-110a1 to DDL/RoleMap, then split policy values and module integration into ordered tasks. |
| TD-099 | T-110a1 | `tasks/M1-core/T-110a1-identity-schema-and-policy.md` | T-110a1 lists SBD-30 trigger T1 although its scope adds no endpoint → remove T1 and retain T3/T4 for S3 columns and tenant RLS. |
| TD-100 | T-110a1 | `tasks/M1-core/T-110a1-identity-schema-and-policy.md` | The parent split/index included T-110a3–T-110a5 while their task files were not named by T-110a1 → include the successor contracts so the generated index and task files are committed atomically. |
| TD-129 | T-110a | `tasks/M1-core/T-110a-identity-schema,-login-and-credential-validation.md` | The original DoD requires a `feat(identity)` commit while the post-split Scope owns aggregate verification only → use a signed `docs(identity): report login aggregate [T-110a]` commit because the task changes only its report, decision register and generated index. |
| TD-130 | T-110a | `tasks/M1-core/T-110a-identity-schema,-login-and-credential-validation.md` | The Report says the disk rule prohibits all caches and databases, but the current policy permits reuse of the fixed Cargo target and existing PostgreSQL container → use those fixed resources and one disposable `ritma_test_T-110a` database, dropping the database after checks. |
| TD-147 | T-110f | `tasks/M1-core/T-110f-profile-and-sessions.md` | API-07/CAN-23 require a versioned `/me` response and the spec-defined `version_conflict` error, but the reused login DTO has no version and Core has no registered symbol → flatten `PersonMe` into a self-profile response with `version`/ETag and register `VERSION_CONFLICT` in Core without changing login JSON. |
| TD-148 | T-110f | `tasks/M1-core/T-110f-profile-and-sessions.md` | T-103 phone normalization needs tenant country, while the tenant contract has no country field → accept validated canonical E.164 for this request and compute its existing tenant blind index without inventing a country default. |
| TD-149 | T-110f | `tasks/M1-core/T-110f-profile-and-sessions.md` | Self-service mutation has no separate write-self permission in the spec → require the existing `identity.people.read` self permission and hard-bind every read/write to the authenticated Actor's own person or session. |
| TD-150 | T-110f | `tasks/M1-core/T-110f-profile-and-sessions.md` | The general state-change rule asks for an outbox event but the event catalog has no profile/password/session event → use only the specified audit actions and do not invent an event name. |
| TD-151 | T-110f | `tasks/M1-core/T-110f-profile-and-sessions.md` | The task DoD names `ritma_test_T-111`, but the explicit operator resource boundary allows only a new `ritma_test_T-110d` database → use a fresh `ritma_test_T-110d` and drop it after checks. |

| TD-152 | T-110f | `tasks/M1-core/T-110f-profile-and-sessions.md` | API-15 requires per-user and per-IP limits, but no shared limiter is wired into the API yet and a route-local limiter would be incomplete → flag shared middleware as a required follow-up; do not mark this control as implemented. |
| TD-153 | T-110f | `tasks/M1-core/T-110f-profile-and-sessions.md` | The profile route updates Core-owned `persons`, but the task does not name its audit action → use `core.person.updated`, following STM-04's `<module>.<entity>.<verb_past>` form and ERD-01's Core ownership. |
| TD-154 | T-110f | `tasks/M1-core/T-110f-profile-and-sessions.md` | T-110f implements five API behaviors in one existing task contract and exceeds CQ-101's approximately 400 non-generated-line guidance → complete the assigned contract in one task commit as instructed, disclose the 2,066-line implementation diff, and request architect review of the task sizing. |

**Total mapped self-resolutions:** 99.

## Detailed entries added in run 6

The delegated resolver directive calls itself TD-001; this registry already assigned TD-001 to T-002, so the resolver decisions are recorded under the next available unique local IDs, TD-046–TD-048.

### TD-046 — Preset patch source boundary

- **Question:** How does the policy store find the correct embedded patch for a tenant when `eq_app` cannot read `tenants` directly?
- **Options:** Change the normative `PolicyStore`/`PolicyInit` signatures; add direct table access; or return only the stored preset id/version and delegate patch lookup to an injected source.
- **Decision:** Use `resolve_tenant_preset(uuid)` plus `PresetPatchSource`.
- **Why:** This preserves the policy API, least privilege and module ownership boundaries.
- **Changed:** Added the source port and uses it only with the resolver's stored identity/version.
- **Ratify:** Yes.

### TD-047 — Resolver authorization contract

- **Question:** Should the resolver enforce tenant transaction context internally or follow the delegated function contract exactly?
- **Options:** Keep the earlier context guard; alter the policy APIs; or implement the delegated SQL body and validate context in the store.
- **Decision:** Implement the specified `STABLE SECURITY DEFINER`, pinned search path, two-column projection and `eq_app` execution grant; verify caller transaction context in the store.
- **Why:** The delegated objective defines the exact function behavior and it returns no secret or table-wide data.
- **Changed:** Added the function migration and app-role probes; no table/column or direct `tenants` grant.
- **Ratify:** Yes.

### TD-048 — Resolver migration slot

- **Question:** Which migration file owns the new function when T-106 has no `spec/29` table row?
- **Options:** Change the ERD (out of scope); attach it to a migration owned by another task; or use the next unassigned function-only slot.
- **Decision:** Use `0011_core_tenant_preset_resolver.sql` between assigned `0010` and `0012`.
- **Why:** The delegated objective authorizes the resolver and the migration adds no table or column.
- **Changed:** Added one SQL function migration and tested it against the mapped DDL.
- **Ratify:** Yes.

### TD-049 — Deny-compatible JSON Schema dependency

- **Question:** Which `jsonschema` dependency configuration validates the required draft without violating the repository license gate?
- **Options:** Keep 0.30 and allow MIT-0; disable its default features; or use an older version of the same crate with only the required draft feature.
- **Decision:** Use version 0.17.1 with `default-features = false` and `draft202012` enabled.
- **Why:** Version 0.30 still resolves the rejected MIT-0 dependency with default features off; 0.17.1 supports Draft 2020-12 and passes the unchanged deny gate.
- **Changed:** Updated the workspace dependency, lockfile and `JSONSchema` API usage.
- **Ratify:** Yes.

### TD-050 — Typed policy audit payload

- **Question:** How should the policy audit sink accept its task-defined audit fields while respecting CQ-101 argument guidance?
- **Options:** Leave a long positional signature; add a generic map; or use a task-specific typed value.
- **Decision:** Pass a `PolicyChange` containing module, version, previous document and new document.
- **Why:** The audit content stays explicit and typed while the method shape stays small; custom `Debug` redacts both documents.
- **Changed:** Added the value type and updated the store plus sink double.
- **Ratify:** Yes.

### TD-051 — Missing-row effective timestamp

- **Question:** What effective timestamp belongs in `Versioned<P>` when the requested policy has no persisted row?
- **Options:** Use an artificial minimum timestamp, make `effective_from` optional and change the API, or use the lookup instant with a synthetic version marker.
- **Decision:** Return version `0` and the caller's requested `at` instant; do not persist this fallback.
- **Why:** POL-04 requires the fallback, while the normative `Versioned` type requires a timestamp. The lookup instant is already defined by the caller and avoids a business-date choice or an out-of-range timestamp.
- **Changed:** Updated the fallback and added `missing_policy_fallback_uses_requested_instant`.
- **Ratify:** Yes.

### TD-052 — Policy cache capacity

- **Question:** How many tenant/module policy histories may the process-local cache retain?
- **Options:** Leave the cache unbounded or set a technical entry capacity.
- **Decision:** Keep at most 1,024 tenant/module keys in the process-local cache; retain each selected key's full policy history.
- **Why:** POL-04 requires a cache but specifies no process-memory bound; this is a technical capacity, not a policy rule or truncation of persisted history.
- **Changed:** Added `POLICY_CACHE_CAPACITY` with an inline technical rationale.
- **Ratify:** Yes.

### TD-053 — Preserve AC1's qualified test name

- **Question:** How can AC1 keep its required `policy::tests::effective_dating` test path while exercising the shared PostgreSQL fixture defined in the integration test?
- **Options:** Move database fixture setup into a separate library unit-test harness, change the task's listed name, or nest the existing integration test under the required module path.
- **Decision:** Nest the integration test under `policy::tests`.
- **Why:** It preserves the exact listed test name and reuses the task's existing PostgreSQL fixture with the fewest changes.
- **Changed:** Moved only the test function into nested modules in `crates/eq-core/tests/policy_store.rs`.
- **Ratify:** Yes.

### TD-054 — Split the outbox task under CQ-101

- **Question:** How should the T-107 implementation that exceeded CQ-101's approximate 400-line guide be decomposed while preserving its contracts and consumers?
- **Options:** Keep one oversized task; or split it into dependent tasks for the exact schema/event producer, handler delivery and receipts, then retry/dead-letter lifecycle.
- **Decision:** Use the three-part chain T-107a → T-107b → T-107c, assigning each original acceptance criterion and test name to exactly one part.
- **Why:** CQ-101 requires oversized work to be split before implementation; the sequence keeps each commit reviewable and preserves dependency order without changing the schema or event names.
- **Changed:** Converted T-107 into a draft split parent, created the three part files, changed T-108 to depend on T-107a (the task that creates `jobs`), and changed T-116a to depend on T-107c (the complete dispatcher contract). The delegated directive called this TD-002, but TD-002 is already assigned to T-002 in this local register.
- **Ratify:** Yes.

### TD-055 — Preserve the shipped event type shape

- **Question:** Should T-107a change the shipped `Event.event_type` field from `&'static str` to `String` to match its interface sketch?
- **Options:** Change the public handler contract; preserve the shipped static string shape while storing the event type in the mapped text column; add a second event envelope type.
- **Decision:** Preserve `&'static str` on the existing `Event` and keep `outbox.event_type` as PostgreSQL `text`.
- **Why:** T-105's event-handler contract is already shipped; the difference is a representation shape, and changing it is outside this part's minimal producer scope.
- **Changed:** T-107a task interface and implementation use the shipped shape; no schema change.
- **Ratify:** Yes.

### TD-056 — Complete events with no subscribers

- **Question:** Should a pending event with no registered handler subscription remain pending or be considered complete?
- **Options:** Keep it pending indefinitely for a future handler; treat the empty handler set as successful; or return an error and leave it pending.
- **Decision:** Treat the empty handler set as successful and mark the event dispatched.
- **Why:** FLW-03 marks the event dispatched after all subscribed handlers succeed; with no subscriber there are no handler effects to run. This also avoids repeatedly claiming an event that has no consumer.
- **Changed:** T-107b records this behavior and adds a focused integration test; no schema or event name is introduced.
- **Ratify:** Yes.

### TD-057 — Preserve aggregate order across dispatchers

- **Question:** Can a second dispatcher claim later events for an aggregate while the first dispatcher holds its oldest events in a batch?
- **Options:** Rely on batch row locks; serialize all dispatchers globally; or expose only each aggregate's oldest pending event to the claim query.
- **Decision:** For non-null aggregate IDs, claim only the oldest pending event for each `(tenant_id, aggregate_id)` while that event remains pending. Keep null-aggregate rows independently claimable.
- **Why:** FLW-03 requires sequential delivery per aggregate, while a null aggregate has no sequence to preserve. Row locks alone do not protect rows beyond the first dispatcher's 100-row batch; materialized aggregate-head selection preserves non-null aggregate order without a new lock table, advisory-lock scheme or schema change.
- **Changed:** T-107b adds a materialized aggregate-head CTE to the claim query and a PostgreSQL test that holds the oldest handler while a second dispatcher attempts to drain 150 same-aggregate events.
- **Ratify:** Yes.

### TD-058 — Classify event-handler failures per ERR-52

- **Question:** Should every event-handler error use the retry schedule, or should event handlers distinguish transient and permanent failures?
- **Options:** Treat all failures as retryable until five attempts; classify failures as retryable or permanent and dead-letter permanent failures immediately.
- **Decision:** Use a typed `EventHandlerError` with retryable and permanent variants. Apply the specified retry schedule only to retryable errors and dead-letter permanent errors immediately.
- **Why:** ERR-51 and ERR-52 explicitly require the same retryable/permanent split for event handlers; permanent failures do not retry.
- **Changed:** T-107c's handler contract and tests implement the split; no schema change.
- **Ratify:** Yes.


### TD-059 — Use the approved cron parser

- **Question:** The task sketch names `croner`, while spec 24 lists the pinned Rust crate `cron`.
- **Options:** Add the unlisted crate; or use the mapped `cron` crate and adapt its seconds/year fields.
- **Decision:** Use the workspace `cron` dependency and preserve five-field `CronSpec.schedule` values through an adapter.
- **Why:** OSS-01/02 makes the spec stack authoritative; the adapter changes no module-facing schedule contract.
- **Changed:** `crates/eq-core/Cargo.toml`, `Cargo.lock`, `crates/eq-core/src/jobs.rs`, and T-108.
- **Ratify:** Yes.

### TD-060 — Use typed job failures

- **Question:** The task's handler sketch uses `anyhow::Result`, but ERR-51 specifies retryable and permanent failures.
- **Options:** Keep the generic result; or return the ERR-51 `JobError` variants.
- **Decision:** Use `Result<(), JobError>` with `Retryable(anyhow::Error)` and `Permanent(anyhow::Error)`.
- **Why:** The error-handling spec defines the state transition and the handler classification.
- **Changed:** Job handler contract, runner, tests and T-108.
- **Ratify:** Yes.

### TD-061 — Bound the worker claim batch

- **Question:** ARC-09 uses a claim limit `n` without a value.
- **Options:** Use an unbounded claim; or choose a conservative technical batch limit.
- **Decision:** Use `JOB_CLAIM_BATCH_SIZE = 100` as a technical bound, matching the shipped outbox worker.
- **Why:** This limits one worker claim without affecting any business rule or persisted history.
- **Changed:** `crates/eq-core/src/jobs.rs` and `tasks/M1-core/T-108-jobs-and-cron.md`.
- **Ratify:** Yes.

### TD-062 — Keep tenant enumeration behind the caller

- **Question:** T-108 requires planning active tenants, but the shipped app role has no allowed active-tenant list resolver.
- **Options:** Add a new cross-tenant function/grant; use an owner pool; or have the trusted integration caller pass `TenantInfo` values.
- **Decision:** Keep existing database privileges unchanged and accept a trusted `&[TenantInfo]` in the planner/runner.
- **Why:** This avoids broad access and does not add a schema object or grant; active tenant sourcing remains an integration responsibility.
- **Changed:** Planner and runner interfaces, tests and T-108.
- **Ratify:** Yes.

### TD-063 — Split oversized task contracts

- **Question:** Several `todo`/`draft` task files exceeded CQ-101 size limits.
- **Options:** Retain oversized tasks; or split the listed acceptance criteria and named tests into a dependent chain.
- **Decision:** Split T-110, T-112, T-113, T-117 and T-302 into parent contracts and implementation parts; preserve every original test name exactly once and point consumers at the part that provides their contract.
- **Why:** CQ-101 requires oversized work to be split before implementation.
- **Changed:** The five parent tasks, their part files and dependent task references; see `tasks/HYGIENE.md` and `tasks/INDEX.md`.
- **Ratify:** Yes.

### TD-064 — Record tenant job dead letters

- **Question:** ERR-51 requires an attention row for dead-lettered jobs, but T-108 does not define the row's kind or payload.
- **Options:** Add a schema object; or follow T-107c's existing OPS-08 pattern with a job-id-only reference, payload and dedupe key.
- **Decision:** Use `kind='job_dead_letter'`, `ref_type='job'`, the job id as `ref_id`, a `{job_id}` payload, high severity and `job_dead_letter:<id>` dedupe.
- **Why:** It reuses the mapped `attention_items` table and the already shipped dead-letter alert pattern without adding schema or PII.
- **Changed:** `crates/eq-core/src/jobs/repo.rs`, `crates/eq-core/tests/jobs.rs` and T-108.
- **Ratify:** Yes.

### TD-065 — Preserve the platform-job boundary

- **Question:** A platform job has `tenant_id = NULL`, but the mapped `attention_items.tenant_id` is NOT NULL.
- **Options:** Invent a tenant/schema for platform alerts; or retain the platform job's ids-only alert log and report that it cannot create a tenant-scoped attention row.
- **Decision:** Do not invent a tenant or schema. Keep the fixed ids-only alert log and report the missing platform attention row as an ERR-51 limitation.
- **Why:** Spec 06/29 provide no platform attention table, and tenant isolation forbids attributing the alert to an unrelated tenant.
- **Changed:** T-108 Report and Questions; no schema change.
- **Ratify:** Yes.

### TD-066 — Add spec-backed security sections

- **Question:** The task audit found `todo`/`draft` tasks with SBD-30 triggers but no Security section, and keyword candidates that may be false positives.
- **Options:** Add one generic section to every task; or review each candidate and describe only its applicable data classes, controls and abuse tests from specs 18/25/22, using `No triggers.` for confirmed false positives.
- **Decision:** Review each task and apply the spec-backed section only; affected task IDs are listed in the security portion of `tasks/HYGIENE.md`.
- **Why:** SBD-30 requires threat controls for real triggers and does not justify adding permissions or behavior.
- **Changed:** Security sections and Questions/Reports on the affected todo/draft tasks.
- **Ratify:** Yes.

### TD-067 — Complete required task headings

- **Question:** The audit found missing headings required by `tasks/_TEMPLATE.md`.
- **Options:** Leave tasks structurally incomplete; or fill the required headings using each task's existing contract and mark inapplicable content `n/a`/`No triggers.`.
- **Decision:** Complete only the missing headings from each task's current Scope, interfaces, ACs and tests; do not add implementation behavior.
- **Why:** WORKFLOW §3 requires the standard report/task structure, while AGENTS §0 forbids designing new scope.
- **Changed:** Required headings in the affected todo/draft task files; task ID evidence is in `tasks/HYGIENE.md`.
- **Ratify:** Yes.


### TD-068 — Preserve the mapped refresh-session schema

- **Question:** STM-22/D-37 and T-110 describe `revoked_reason`, but spec 06 §4.2 and ERD §7 define `refresh_sessions` without that column.
- **Options:** Add an unmapped column; or keep the canonical DDL and identify reuse from the revoked token plus a newer row in the same `family_id`.
- **Decision:** Do not add `revoked_reason`; detect a presented revoked token with a newer family row, revoke the family, write the existing security audit, and return `session_expired`.
- **Why:** The task's schema map and the explicit Run 6 no-new-column rule prohibit schema drift; IDN-04 already defines reuse by the family having a newer row.
- **Changed:** T-110 parent, T-110a and T-110b task contracts; no migration or table change.
- **Ratify:** Yes.


### TD-069 — Keep materialisation core with its earlier provider

- **Question:** T-209 first says it consumes the materialiser core from T-210, then says the core belongs in T-209 and T-210 extends it.
- **Options:** Make T-209 depend on a later task; or keep the reusable planner in T-209 and implement the job, cycle cap and closure extension in T-210.
- **Decision:** T-209 owns the base `plan_occurrences`/`apply` core; T-210 extends it with jobs and the remaining scheduling rules.
- **Why:** The task's own later ownership statement and dependency order identify T-209 as the provider; T-210 already depends on T-209.
- **Changed:** T-209 wording and both task Reports/Questions; no interface outside the task contract is added.
- **Ratify:** Yes.

### TD-070 — Keep the tunnel broker dependency pointed forward

- **Question:** M8 T-804 uses a broker implemented by M9 T-908, but CA-10 requires a testable external-I/O adapter and M8 cannot depend backward on M9.
- **Options:** Call an unspecified M9 endpoint directly from T-804; or define the client seam in T-804 and make T-908 depend on that seam for its service implementation.
- **Decision:** T-804 owns the mockable client boundary; T-908 depends on T-804 and implements the broker service. Endpoint and rate-limit details remain governed by DST-22 and the T-908 contract when that task is promoted.
- **Why:** CA-10 requires a mockable adapter for external I/O, and stacked milestone order requires M9 to follow M8.
- **Changed:** T-804/T-908 Reports and Questions; T-908 dependency metadata; no new wire path or business limit was selected.
- **Ratify:** Yes.

### TD-071 — Include the ERD-05 `reason` key in audit anonymization

- **Question:** T-109's explicit AUD-05 key list omits `reason`, while ERD-05 requires `reason` values to be included in AUD-05 anonymization.
- **Options:** Follow only the narrower task list; or include the additional key required by the schema classification.
- **Decision:** Recursively redact `reason` in addition to the listed keys.
- **Why:** ERD-05 is normative for open free text that may describe a minor and explicitly points to AUD-05.
- **Changed:** T-109 and the recursive audit JSON redactor.
- **Ratify:** Yes.

### TD-072 — Preserve the checkpoint timestamp default

- **Question:** T-109's abbreviated checkpoint DDL omits a default for `at`, while spec 06 §4.11 defines `DEFAULT now()`.
- **Options:** Use the abbreviated sketch; or match the mapped spec DDL.
- **Decision:** Use the spec default `now()`.
- **Why:** The schema map requires migrations to match spec 06; the row is mapped to T-109 in ERD §7.
- **Changed:** T-109 migration and task Report.
- **Ratify:** Yes.

### TD-073 — Inject PolicyStore into job context

- **Question:** T-109's `audit.retention` must read policy through `PolicyStore`, but the shipped T-108 `JobCtx` carries only tenant, pool, clock and time.
- **Options:** Read `policies` directly in the audit module; construct a separate store inside each job; or let the shared runner supply its existing `PolicyStore` through an optional job-context field.
- **Decision:** Add an optional shared `PolicyStore` to `JobCtx` via a builder on `JobRunner`; audit retention requires the supplied store, while existing jobs and callers remain unchanged.
- **Why:** The task explicitly requires the core policy service, and the additive context path preserves module ownership without direct cross-module SQL or per-job listener construction.
- **Changed:** `crates/eq-core/src/jobs.rs`, T-109 task contract and audit job implementation.
- **Ratify:** Yes.

### TD-074 — Keep the first-party audit derive license exception package-scoped

- **Question:** `cargo deny` rejects the workspace-inherited `AGPL-3.0-only` license for the Ritma-owned `eq-audit-derive` crate unless that package is explicitly recognized.
- **Options:** Change the workspace-wide allowed-license policy; or add the exact first-party package to the existing license exception list without changing advisory or ban policy.
- **Decision:** Retain the package-scoped `eq-audit-derive` exception in `deny.toml`; the crate inherits `AGPL-3.0-only` from the workspace and repository license.
- **Why:** OSS-03 permits the repository's own AGPL-licensed crates. The workspace package license and repository `LICENSE` agree. TD-010 is already assigned to T-005, so this decision uses the next available ID rather than duplicating it.
- **Changed:** `deny.toml`, `Cargo.toml`, `crates/eq-audit-derive/Cargo.toml`, `tasks/M1-core/T-109-audit.md`, `tasks/M1-core/T-109a-auditable-derive.md`, and this register. The architect's `deny.toml` exception was committed in `f49eb66`.
- **Ratify:** Yes.

### TD-075 — Split the oversized audit task

- **Question:** The complete T-109 implementation measured 2,102 changed lines, above CQ-101's approximately 400-line review guide.
- **Options:** Keep one oversized implementation task; or split its scope into a dependent sequence while preserving each original acceptance criterion and test name exactly once.
- **Decision:** Split implementation into T-109a through T-109h by code ownership, then keep T-109 as the final aggregate verification task.
- **Why:** CQ-101 requires oversized work to be split before implementation; the sequence preserves the parent contract and lets each implementation part be reviewed independently.
- **Changed:** T-109 parent and part task files, consumer dependency references, `tasks/INDEX.md`, and this register.
- **Ratify:** Yes.

### TD-076 — Replace the stale SBD-21 reference

- **Question:** T-109a cites SBD-21, but the current `spec/18-security-by-design.md` defines no SBD-21 rule.
- **Options:** Keep an invalid citation; or retain the AUD-02 redaction contract and cite the current SBD controls that govern data classification, triggers, abuse-case tests and review.
- **Decision:** Remove SBD-21 from T-109a and cite SBD-10, SBD-30, SBD-73 and SBD-81 while preserving AUD-02's exact behavior.
- **Why:** Current spec 18 defines the applicable classification, trigger, abuse-case and security-review rules; AUD-02 itself defines which marked fields are redacted or skipped.
- **Changed:** `tasks/M1-core/T-109a-auditable-derive.md` and this register; implementation behavior is unchanged.
- **Ratify:** Yes.

### TD-077 — Declare audit schema test dependencies

- **Question:** T-109b requires an integration test against `TestDb::app_pool` and SQLx but does not list `crates/eq-audit/Cargo.toml` among its files.
- **Options:** Avoid a direct manifest change and leave the listed test unable to compile; or add the required crates as dev dependencies and include the manifest in the task's file list.
- **Decision:** Add workspace `eq-testkit`, `sqlx` and `tokio` dependencies under `[dev-dependencies]` in `crates/eq-audit/Cargo.toml`.
- **Why:** Rust integration tests need direct dependencies, and TST-20 requires privilege checks through the non-superuser `TestDb::app_pool`; dev dependencies do not expand production dependencies.
- **Changed:** `Cargo.lock`, `crates/eq-audit/Cargo.toml`, `crates/eq-audit/tests/audit.rs`, and `tasks/M1-core/T-109b-audit-schema.md`.
- **Ratify:** Yes.

### TD-078 — Use isolated unittest discovery for the task gate tests

- **Question:** The prescribed `python3 -I -m unittest scripts/tests/test_tasks_lint.py` invocation cannot import `scripts` because Python isolated mode excludes the current directory from module lookup.
- **Options:** Drop the gate fixture tests; or use unittest discovery rooted at `scripts/tests` to execute the same file under isolated mode.
- **Decision:** Run `python3 -I -m unittest discover -s scripts/tests -p 'test_tasks_lint.py'`.
- **Why:** The discovery runner explicitly adds the test start directory while preserving Python isolated mode; all eight fixture tests pass without changing the gate or tests.
- **Changed:** `tasks/M1-core/T-109b-audit-schema.md` and this decision register.
- **Ratify:** Yes.

### TD-079 — Preserve the policy validation error path

- **Question:** A broad constructor-validation reading could reject an audit policy's cross-field violation during deserialization, before `PolicyDoc::validate` can return the specified `must_be_lte` field pointer.
- **Options:** Reject the relation in `new`/Deserialize and accept a generic root serde error; or validate individual field bounds in the constructor/deserializer and apply cross-field policy validation after deserialization.
- **Decision:** Keep range and enum validation in `new`/Deserialize; keep `anonymize_pii_after_years <= retention_years` in `PolicyDoc::validate`.
- **Why:** POL-10 prescribes schema → serde → cross-field validation, and parent T-109 requires `must_be_lte` at `/anonymize_pii_after_years`.
- **Changed:** `tasks/M1-core/T-109c-audit-policy.md` and `crates/eq-audit/src/policy.rs`.
- **Ratify:** Yes.

### TD-080 — Order audit policy schema fields

- **Question:** The policy contract requires `x-ui: advanced`, but the `XUi` builder also requires an order that the task does not specify.
- **Options:** Choose an independent order; or follow the declared policy field order.
- **Decision:** Use order values 1, 2 and 3 for `retention_years`, `anonymize_pii_after_years` and `admin_categories`, respectively.
- **Why:** This is deterministic presentation metadata following the contract's field order; it does not change a business rule or field default.
- **Changed:** `crates/eq-audit/src/policy.rs` and `tasks/M1-core/T-109c-audit-policy.md`.
- **Ratify:** Yes.

### TD-081 — Reject unmapped anonymous audit actors

- **Question:** `ActorRole` includes `Anonymous`, while the mapped `audit.entries.actor_role` values list owner, admin, instructor, guardian, student, system and platform only.
- **Options:** Persist an additional `anonymous` value not listed in the audit data dictionary; map it to another role; or reject the event before insertion.
- **Decision:** Reject `Anonymous` audit events with the existing internal error; do not map it to another actor role.
- **Why:** The audit role values in spec 06 §4.9 and ERD-04 are the schema contract. Rejecting the unmapped role is the fail-closed choice and invents no persisted role value.
- **Changed:** `crates/eq-audit/src/record.rs`, `tasks/M1-core/T-109d-audit-record-api.md`, and this register.
- **Ratify:** Yes.

### TD-082 — Include task-owned SQLx offline metadata

- **Question:** T-109d adds compile-time SQLx queries but its Files table omits `.sqlx/` and `Cargo.lock`.
- **Options:** Avoid query macros and lose compile-time query validation; or include the exact generated metadata and dependency edge required for the offline build.
- **Decision:** Use the existing `SQLX_OFFLINE=true` query-macro path and include only the matching metadata and lockfile changes for T-109d.
- **Why:** `.cargo/config.toml` makes offline query metadata part of the build contract, and AGENTS.md requires metadata updates when a query changes.
- **Changed:** `.sqlx/`, `Cargo.lock`, `crates/eq-audit/Cargo.toml`, `crates/eq-audit/src/repo.rs`, `tasks/M1-core/T-109d-audit-record-api.md`, and this register.
- **Ratify:** Yes.

### TD-083 — Keep audit SQL in the repository adapter

- **Question:** T-109d's Files table omits `repo.rs`, while CA-11/CQ-24 require SQL persistence to live in that adapter.
- **Options:** Put SQL in `record.rs` without the repository adapter; or keep the public contract in `record.rs` and delegate persistence to the module's private `repo.rs`.
- **Decision:** Keep `AuditEvent` and `record` in `record.rs`; put transaction-context checks, role conversion and SQL in `repo.rs`.
- **Why:** CA-11/CQ-24 define adapter placement, and the private repository preserves the exact public signature while keeping SQL inside the audit module's own tables.
- **Changed:** `crates/eq-audit/src/lib.rs`, `crates/eq-audit/src/record.rs`, `crates/eq-audit/src/repo.rs`, `tasks/M1-core/T-109d-audit-record-api.md`, and this register.
- **Ratify:** Yes.

### TD-084 — Verify both named audit sequence endpoints

- **Question:** The `verify(tenant, from_seq, to_seq)` interface names both bounds but does not state whether the range includes them.
- **Options:** Include both endpoints or use a half-open range that excludes `to_seq`.
- **Decision:** Treat the range as inclusive and query `seq >= from_seq AND seq <= to_seq`.
- **Why:** This checks both explicitly named boundary sequences instead of silently omitting the final one; it changes no business, schema, security or tenant-isolation rule.
- **Changed:** `crates/eq-audit/src/sealer.rs`, `tasks/M1-core/T-109e-audit-chain.md`, and this register.
- **Ratify:** Yes.

### TD-085 — Keep audit-chain SQL in the repository adapter

- **Question:** T-109e lists `sealer.rs` but omits the existing repository adapter, while the implementation needs parameterized SQL for audit tables.
- **Options:** Put the queries in the sealer orchestration module or add task-owned query helpers to `repo.rs`.
- **Decision:** Keep SQL in `repo.rs`; leave hash construction and chain orchestration in `sealer.rs`.
- **Why:** CA-11/CQ-24 require table access to stay in the module's repository adapter, and this preserves the existing audit module boundary.
- **Changed:** `crates/eq-audit/src/repo.rs`, `crates/eq-audit/src/sealer.rs`, `tasks/M1-core/T-109e-audit-chain.md`, and this register.
- **Ratify:** Yes.

### TD-086 — Fail closed on an out-of-order sealed chain

- **Question:** The parent requires sequence-ordered hashing but does not state how to handle an unsealed sequence before a later already-sealed row or checkpoint.
- **Options:** Continue from the latest sealed hash, rewrite later immutable hashes, or stop without persisting new hashes.
- **Decision:** Return an internal error and make no seal updates when the existing state cannot be extended in sequence order.
- **Why:** Continuing or rewriting would violate AUD-03/04 chain order and append-only integrity; failing closed preserves the stored evidence without changing schema or business behavior.
- **Changed:** `crates/eq-audit/src/repo.rs`, `crates/eq-audit/src/sealer.rs`, task-owned `.sqlx` metadata, `tasks/M1-core/T-109e-audit-chain.md`, and this register.
- **Ratify:** Yes.

### TD-087 — Keep audit-job queries in the repository adapter

- **Question:** T-109g's Files table omits the repository paths and SQLx offline metadata needed for compile-time queries.
- **Options:** Place SQL in job orchestration code or add private audit repository helpers and the checked-in query metadata.
- **Decision:** Keep audit query helpers in `crates/eq-audit/src/repo.rs`; route the core-owned table write through `eq_core::attention::raise` and its repository; include the four exact metadata files for compile-time queries in T-109g's file contract.
- **Why:** CA-11/CA-30/CQ-24/CQ-42 assign table access to the owning module's repository, and SQLX_OFFLINE requires descriptions for compile-time query macros.
- **Changed:** `crates/eq-audit/src/jobs.rs`, `crates/eq-audit/src/repo.rs`, `crates/eq-core/src/lib.rs`, `crates/eq-core/src/attention.rs`, `crates/eq-core/src/attention/repo.rs`, the four T-109g `.sqlx` metadata files, and this register.
- **Ratify:** Yes.

### TD-088 — Use an idempotent audit-chain attention item

- **Question:** T-109g asks for a high-severity audit mismatch attention item but does not define its stable kind, payload or dedupe shape.
- **Options:** Store only the failure category, add audit-entry content, or store the sequence that failed verification.
- **Decision:** Use module `audit`, kind `audit_chain_mismatch`, payload containing only the mismatched sequence, severity `high`, and dedupe key `audit_chain_mismatch:<sequence>`.
- **Why:** The mismatch sequence is the minimum diagnostic needed to locate the failure, and deduping by tenant/sequence is safe across job retries without storing PII or changing schema, permissions or events.
- **Changed:** `crates/eq-audit/src/repo.rs`, `crates/eq-audit/src/jobs.rs`, `crates/eq-audit/tests/audit.rs`, and this register.
- **Ratify:** Yes.

### TD-089 — Use the core-owned attention port

- **Question:** T-109g requires `eq-audit` to raise a row in `attention_items`, but ERD-04 assigns that table to core and the task provides no shared API.
- **Options:** Write the core table from `eq-audit`, emit a new cross-module event, or add the `attention::raise` API named by STM-14.
- **Decision:** Implement `eq_core::attention::raise` with SQL in the core-owned repository and invoke it inside the caller's tenant transaction.
- **Why:** STM-14 names `attention::raise` as the any-module creation path; CA-30 prohibits `eq-audit` from directly writing a core-owned table. This adds no schema, event, permission or role grant.
- **Changed:** `crates/eq-core/src/lib.rs`, `crates/eq-core/src/attention.rs`, `crates/eq-core/src/attention/repo.rs`, `crates/eq-audit/src/jobs.rs`, `tasks/M1-core/T-109g-audit-jobs.md`, and this register.
- **Ratify:** Yes.

### TD-090 — Schedule recent audit verification after retention

- **Question:** T-109h asks for both audit cron schedules from parent T-109, but the parent specifies only `audit.retention` at `0 3 * * *` tenant-local.
- **Options:** Run `audit.verify_recent` at the same 03:00 instant; choose a second tenant-local nightly time after the maintenance job.
- **Decision:** Run `audit.verify_recent` at `30 3 * * *` tenant-local.
- **Why:** This keeps the check nightly as T-109g describes and avoids overlapping the audit anonymization/purge job that starts at 03:00. The job remains tenant-local as required by `CronSpec`.
- **Changed:** `crates/eq-audit/src/module.rs`, `tasks/M1-core/T-109h-audit-module.md`, and this register.
- **Ratify:** Yes.

### TD-091 — Preserve audit context for core actions

- **Question:** The `AuditSink` port identifies toggle/policy actions but does not prescribe their category or snapshot context; `ModuleId` cannot be stored in the UUID-only `entity_id` field.
- **Options:** Omit the module identifier from snapshots; add a new persisted identifier/schema value; or keep `entity_id` absent and place the module id and state/document in the JSON snapshots.
- **Decision:** Use `Category::Security` for module toggles and policy changes, `entity_type` `module`/`policy`, no `entity_id`, and snapshots carrying the module slug with enabled state or policy version/document.
- **Why:** SBD-53 explicitly lists module toggles and policy changes as security-relevant audit events. The snapshot carries enough context without inventing a UUID, table column, permission or event name.
- **Changed:** `crates/eq-audit/src/module.rs`, `crates/eq-audit/src/lib.rs`, `tasks/M1-core/T-109h-audit-module.md`, and this register.
- **Ratify:** Yes.

### TD-092 — Verify the audit sink's transaction and tenant boundary

- **Question:** T-109h's named abuse-case test checked only module registration and did not exercise the `AuditSink` tenant boundary; the `Module` interface also has an explicit port-registration hook.
- **Options:** Leave the port implementation unregistered and rely on future manual construction; retain only the schedule-registration test; or register `AuditSink` and test same-transaction writes plus rejection when the supplied tenant differs from the transaction context.
- **Decision:** Register the stateless `AuditModule` as the existing `AuditSink` port and add database-backed checks for both audit actions in one caller transaction, rollback behavior, and mismatched tenant rejection.
- **Why:** MOD-04/ARC-08 provide the module port-registration hook, SBD-73 requires an abuse-case test for the tenant boundary, and the test preserves the existing `AuditSink` signatures and audit transaction semantics.
- **Changed:** `crates/eq-audit/src/module.rs`, `crates/eq-audit/tests/audit.rs`, `tasks/M1-core/T-109h-audit-module.md`, and this register.
- **Ratify:** Yes.

### TD-093 — Serialize audit writers with chain sealing

- **Question:** `audit.entries.seq` is allocated before transaction commit, but the sealer's tenant lock is not acquired by `record`; a slower lower-sequence writer can commit after a higher-sequence row has already been sealed.
- **Options:** Leave writer transactions outside the sealer lock and rely on the next verification to expose the broken ordering; acquire the existing per-tenant advisory transaction lock before inserting each audit row.
- **Decision:** Acquire the same `audit.seal:<tenant_id>` transaction lock in the audit repository before inserting, and add `concurrent_writer_and_sealer_preserve_sequence_order` to hold a record transaction open while sealing starts and verify the resulting chain.
- **Why:** The task requires an append-only, sequence-ordered tamper-evident chain. A PostgreSQL sequence reflects allocation order, not transaction commit order. Sharing the existing lock serializes record insertion with sealing/anonymization/purge without changing schema, wire values, permissions or business behavior; the lock remains held by the caller's transaction so an audit failure still rolls back with the business change.
- **Changed:** `crates/eq-audit/src/repo.rs`, `crates/eq-audit/tests/audit.rs`, `tasks/M1-core/T-109-audit.md`, `tasks/INDEX.md`, and this register.
- **Ratify:** Yes.

### TD-094 — Synchronize the writer/sealer regression test

- **Question:** The test spawned the sealer and immediately committed the writer, so the sealer could start only after commit and fail to exercise lock contention.
- **Options:** Commit after spawning and rely on scheduling; wait until PostgreSQL reports a sealer session blocked on the advisory lock before releasing the writer transaction.
- **Decision:** Poll `pg_stat_activity` for the sealer's advisory-lock wait before committing the writer. Use a 10-second test-only timeout and a 10-millisecond poll interval.
- **Why:** The regression test must demonstrate that sealing starts while the writer transaction still owns the lock. The bounded observation is technical test synchronization only; it does not change application behavior, database policy, or a user-visible deadline.
- **Changed:** `crates/eq-audit/tests/audit.rs`, `tasks/M1-core/T-109-audit.md`, and this register.
- **Ratify:** Yes.

### TD-095 — Align the identity split tracker with its children

- **Question:** The T-110 tracker says its Scope is split tracking only, but its Files table names the implementation migration and its split table lists T-110a as depending on T-109 with both children still draft; the child frontmatter is todo and T-110a depends on T-109d.
- **Options:** Keep the stale parent sketch; update only T-110's tracker rows to match its stated Scope and the child task frontmatter/dependency graph.
- **Decision:** List only the parent task and generated task index as files, set both child tracker statuses to todo, and show T-109d as T-110a's dependency.
- **Why:** This changes no identity behavior, schema, permission or dependency. It makes the tracking contract reflect the already-shipped child tasks and current index.
- **Changed:** `tasks/M1-core/T-110-identity-login-refresh.md`, `tasks/INDEX.md`, and this register.
- **Ratify:** Yes.

### TD-096 — Split T-110a into ordered implementation parts

- **Question:** T-110a combines five independently testable identity concerns whose implementation and tests are expected to exceed CQ-101's approximately 400 changed-line review guide.
- **Options:** Keep the task combined and review a large change; split schema/policy, password/JWT, login service, HTTP login, and actor/role switching into ordered parts while retaining the original as final aggregate verification.
- **Decision:** Add T-110a1 through T-110a5 with explicit Files, AC/test ownership, and dependency edges; leave T-110a draft until every part is review, then promote T-110a for aggregate verification.
- **Why:** CQ-101 requires larger work to be split before implementation, while each original AC and named test can keep a single owner. The split introduces no behavior, business value, permission, schema, or migration number.
- **Changed:** T-110a, the five new T-110a part task files, `tasks/DECISIONS.md`, and generated `tasks/INDEX.md`.
- **Ratify:** Yes.

### TD-097 — Validate identity policy values at construction and deserialization

- **Question:** The parent task sketches public bounded policy fields, while CQ-23/CAN-10 require invalid value types to reject invalid construction and deserialization.
- **Options:** Preserve public fields and rely on call-site validation; use private fields with validating constructors and deserialize through those constructors.
- **Decision:** Use private fields for `IdentityPolicy` and `Lockout`, add validating constructors/accessors, and validate every deserialized value; keep the generic `RoleMap<T>` fields public because it has no value invariant.
- **Why:** The CQ-23/CAN-10 value-type invariant applies to bounded policy values; `RoleMap<T>` only names the five closed roles and imposes no independent value constraint.
- **Changed:** `crates/eq-identity/src/policy.rs`, the T-110a1b task Report/Questions, this register, and generated `tasks/INDEX.md`.
- **Ratify:** Yes.

### TD-098 — Split identity schema and policy work by concern

- **Question:** T-110a1 combined the mapped identity migration, shared `RoleMap`, validated policy values, policy schema metadata and module integration, exceeding CQ-101's approximate review-size guidance.
- **Options:** Keep the combined change; retain the migration/RoleMap task and add ordered policy-model and module-integration tasks.
- **Decision:** T-110a1 owns the mapped DDL and shared RoleMap; T-110a1b owns validated policy values; T-110a1c owns Identity module registration, schema metadata and summaries.
- **Why:** Each part has a small, independently verifiable contract and dependency edge, and no schema or policy behavior is changed by the decomposition.
- **Changed:** T-110a parent split table, T-110a1/1b/1c task files, T-110a2 dependency, this register, and generated `tasks/INDEX.md`.
- **Ratify:** Yes.

### TD-099 — Align T-110a1 security triggers with its scope

- **Question:** T-110a1 lists SBD-30 trigger T1 even though its scope adds no endpoint.
- **Options:** Keep T1 as a conservative overclassification; remove T1 and retain the triggers that match the migration's S3 fields and RLS behavior.
- **Decision:** Remove T1 and keep T3/T4.
- **Why:** SBD-30 defines T1 for endpoint changes, while T3 covers S2/S3 data and T4 covers RLS/database security touched by the task.
- **Changed:** `tasks/M1-core/T-110a1-identity-schema-and-policy.md` and this register.
- **Ratify:** Yes.

### TD-100 — Keep the identity task split and index consistent

- **Question:** The parent split and generated index include T-110a3–T-110a5, but their task files were not named in T-110a1's `Files` list.
- **Options:** Commit an index that refers to task files absent from the commit; include the ordered successor contracts and generated index with the task split.
- **Decision:** Include T-110a3, T-110a4 and T-110a5 task files in T-110a1's scope and `Files` list, and commit them with the parent split and index.
- **Why:** CQ-15 requires a generated index that reflects the task files present in the repository. The current parent split already assigns each original behavior/test to one successor; committing the file set atomically preserves that contract without implementing successor behavior. The complete task-definition change is 805 lines, above CQ-101's approximate 400-line guide; this is disclosed for maintainer ratification because the files define one ordered identity split and the implementation itself stays within the narrow schema/RoleMap scope.
- **Changed:** `tasks/M1-core/T-110a1-identity-schema-and-policy.md`, the three successor task files, the parent tracker, `tasks/DECISIONS.md` and generated `tasks/INDEX.md`.
- **Ratify:** Yes.

### TD-101 — Match identity policy security triggers to the scope

- **Question:** T-110a1b lists T1/T3 although it adds no endpoint and its policy values are S1.
- **Options:** Keep the broad trigger list; or apply the SBD-30 definitions to this task's actual scope.
- **Decision:** Mark T4 only because the task changes authentication policy values.
- **Why:** SBD-30 defines T1 for endpoint changes, T3 for S2/S3 data, and T4 for authentication.
- **Changed:** `tasks/M1-core/T-110a1b-validated-identity-policy-model.md` and this register.
- **Ratify:** Yes.

### TD-102 — Record the identity policy spec references

- **Question:** T-110a1b implements the invite-expiry and TOTP policy fields but omits IDN-05 and IDN-12 from `spec_refs`.
- **Options:** Leave the field references implicit; or include the IDs that define those fields and variants.
- **Decision:** Add IDN-05 and IDN-12 to the task's `spec_refs`.
- **Why:** Spec 04 §6.2 includes invite expiry and TOTP; spec 25 assigns TOTP the IDN-12 contract.
- **Changed:** `tasks/M1-core/T-110a1b-validated-identity-policy-model.md` and this register.
- **Ratify:** Yes.

### TD-103 — Match identity module security triggers to scope

- **Question:** T-110a1c lists T1/T3 although it adds no endpoint and exposes only S1 policy settings.
- **Options:** Keep the broader trigger list; or apply the SBD-30 definitions to the actual module-policy scope.
- **Decision:** Mark T4 only because identity authentication policy changes.
- **Why:** SBD-30 defines T1 for endpoints, T3 for S2/S3 data and T4 for authentication.
- **Changed:** `tasks/M1-core/T-110a1c-identity-policy-module.md` and this register.
- **Ratify:** Yes.

### TD-104 — Record the identity module policy spec references

- **Question:** T-110a1c includes invite-expiry and TOTP settings but omits IDN-05 and IDN-12 from `spec_refs`.
- **Options:** Keep those references implicit; or include the IDs defining the fields and variants used by the task.
- **Decision:** Add IDN-05 and IDN-12 to `spec_refs`.
- **Why:** Spec 04 §6.2 defines the policy fields and spec 25 assigns TOTP IDN-12.
- **Changed:** `tasks/M1-core/T-110a1c-identity-policy-module.md` and this register.
- **Ratify:** Yes.

### TD-105 — Clarify policy-schema DoD boundary

- **Question:** The task DoD says no schema is added, while the Goal and AC1 require exposing `IdentityModule::policy_schema`.
- **Options:** Treat the DoD literally and omit the required policy schema; or interpret it as no new persisted schema or policy fields, while generating schema for existing values.
- **Decision:** Clarify the DoD to prohibit new persisted schema objects and policy fields; retain the required JSON Schema for the existing `IdentityPolicy` values.
- **Why:** POL-06 and the task Scope/AC require the existing policy contract to be exposed, and this choice adds no migration, field or business value.
- **Changed:** `tasks/M1-core/T-110a1c-identity-policy-module.md` and this register.
- **Ratify:** Yes.

### TD-106 — Match password/JWT security triggers to scope

- **Question:** T-110a2 lists T1/T3 although it adds no endpoint and implements S3 authentication material.
- **Options:** Keep the broader trigger list; or apply the SBD-30 trigger definitions to the actual scope.
- **Decision:** Mark T3 and T4 only.
- **Why:** SBD-30 assigns T3 to S2/S3 fields and T4 to authentication, sessions, tokens and crypto; T1 is for endpoint changes.
- **Changed:** `tasks/M1-core/T-110a2-password-and-jwt-primitives.md` and this register.
- **Ratify:** Yes.

### TD-107 — Register `password_weak` as an Identity application code

- **Question:** T-110a2 requires a registered `password_weak` validation error, but ERR-31's shared field-code set is closed and excludes this code; the task's Files table omits the module error registry.
- **Options:** Add `password_weak` to the shared field-error set; or register it as the Identity-owned 422 application code named in the identity endpoint contract.
- **Decision:** Create the Identity `ErrorCodeDef` in `crates/eq-identity/src/errors.rs` and keep `password_weak` out of the shared ERR-31 field-code set.
- **Why:** ERR-21 requires owning crates to register their application codes, ERR-31 keeps the shared field codes closed, and the identity endpoint contract names `password_weak` as a 422 response.
- **Changed:** `tasks/M1-core/T-110a2-password-and-jwt-primitives.md`, `crates/eq-identity/src/errors.rs`, `crates/eq-identity/src/lib.rs` and this register.
- **Ratify:** Yes.

### TD-108 — Keep access tokens and password hashes in redacted wrappers

- **Question:** T-110a2 does not define the return types for issued access tokens or stored password hashes, both of which carry credential material.
- **Options:** Return plain strings; or wrap the values in secrecy-backed types with a redacted Debug implementation.
- **Decision:** Use private secrecy-backed wrappers with handwritten redacting Debug and no Clone/Serialize/plain Debug derives.
- **Why:** SBD-50 requires secret-bearing values to use secrecy types and forbids Clone, Serialize and derived/plain Debug.
- **Changed:** `tasks/M1-core/T-110a2-password-and-jwt-primitives.md`, `crates/eq-identity/src/password.rs`, `crates/eq-identity/src/jwt.rs` and this register.
- **Ratify:** Yes.

### TD-109 — Preserve AC1's common-password rejection

- **Question:** T-110a2 AC1 requires `password123` to be rejected at policy minimum 10, but the selected SecLists 10k source snapshot does not contain that entry.
- **Options:** Ignore the named AC and accept the password because it is absent from the source; or preserve the upstream list and add the one AC-named password as a supplement.
- **Decision:** Add `password123` as the only documented supplement in the bundled list.
- **Why:** The task explicitly classifies this value as common and requires the existing `common_rejected` test; a narrow supplement is stricter than the source snapshot and adds no composition rule.
- **Changed:** `crates/eq-identity/data/common-passwords.txt`, `tasks/M1-core/T-110a2-password-and-jwt-primitives.md` and this register.
- **Ratify:** Yes.

### TD-110 — Resume T-110a3 under the delegated check policy

- **Question:** T-110a3 was blocked because a fixed Cargo target and disposable PostgreSQL fixture were treated as unavailable under a prior resource rule, while the current run delegates technical decisions and says unavailable checks are `Not run` rather than grounds to block.
- **Options:** Keep T-110a3 blocked pending another permission change; or resume it, provision the task-required fresh PostgreSQL 16 fixture, and report only genuinely unavailable checks as `Not run`.
- **Decision:** Resume T-110a3 and attempt the implementation, generated SQLx metadata, and named tests with the current toolchain and a fresh PostgreSQL 16 fixture.
- **Why:** Current run TD-004 requires continuation when checks cannot run, and the task requires generated metadata and database-backed tests. This decision does not change business rules, security controls, schema, or `cargo deny` policy.
- **Changed:** `tasks/M1-core/T-110a3-login-service-and-lockout.md`, `tasks/INDEX.md`, and `RUN_LOG.md`.
- **Ratify:** Yes.

### TD-111 — Use the spec-mapped minimal TOTP verifier features

- **Question:** The task names `totp-rs` and fixes RFC 6238 step/skew behavior, but the dependency map does not pin a release or feature set.
- **Options:** Use the crate's default features, including URI/migration helpers; or pin the current 6.x API with only `std` and `zeroize` for verification of the encrypted raw secret.
- **Decision:** Use `totp-rs` 6.x with default features disabled and `std` plus `zeroize` enabled; do not enable QR/URI or migration features in this service task.
- **Why:** OSS-24 maps TOTP to `totp-rs`; this task only verifies secrets, while the selected features provide RFC verification and zeroization. The dependency graph will be checked by unchanged `cargo deny` rules.
- **Changed:** workspace and identity manifests, `Cargo.lock`, and `crates/eq-identity/src/service.rs`.
- **Ratify:** Yes.

### TD-112 — Keep login identity lookup and anonymous audit attribution within mapped contracts

- **Question:** T-110a3 requires a core-owned directory port injected through shared state, while the shipped audit API rejects `ActorRole::Anonymous`, STM-23 supplies the exact failure action, and `Email::parse` requires a `FieldCtx` that the task does not supply.
- **Options:** Let identity query core-owned tables directly, attempt audit with an unmapped anonymous role, and invent a default field context; or use a core-owned typed directory port/adapter, the mapped `system` audit role without actor id, STM-23's action name, and the caller's existing `FieldCtx`.
- **Decision:** Inject a `CoreIdentityDirectory` port implemented by the core PostgreSQL adapter through redacted `AuthState`; use `ActorRole::System` with no actor id for login audit records while preserving request IP/id, retain known people only as audit entities, use `security.login.failed` per STM-23, and pass the caller's `FieldCtx` into `Email::parse`.
- **Why:** CA-30/31 and ERD-02 prohibit identity SQL joining core tables; spec 06 §4.9 and ERD-04 exclude `anonymous` from persisted audit roles, while `system` is mapped; STM-23 supplies the failure action; VAL-03 requires caller context for field parsing, so fabricating a default would invent validation policy.
- **Changed:** `crates/eq-core/src/auth.rs`, `crates/eq-core/src/auth/directory.rs`, `crates/eq-core/src/app_state.rs`, `crates/eq-identity/src/service.rs`, and this task's Report.
- **Ratify:** Yes.

### TD-113 — Represent absent country context for context-independent login fields
- **Question:** T-110a3's caller-supplied `FieldCtx` requires a country, but the HTTP tenant context does not provide one and the login fields are Email and Password, which do not consume it.
- **Options:** (a) invent a country default for login; or (b) make the country context optional and have only country-dependent fields reject its absence.
- **Decision:** Make `FieldCtx.country` optional, have `Phone::parse` return its existing format validation error when country is absent, and construct login context from the resolved tenant's currency/locale and its timezone-local date with no password minimum.
- **Why:** VAL-20/21 define context as data consumed by a field kind; Email ignores it and Password only consumes the policy minimum, which IDN-06 does not apply on login. This preserves canonical validation without a fabricated tenant-country value or any change to phone behavior when country is supplied.
- **Changed:** `crates/eq-core/src/fields/mod.rs`, `crates/eq-core/src/fields/kinds.rs`, existing `FieldCtx` construction sites, `crates/eq-identity/src/api_auth.rs`, this task and generated `tasks/INDEX.md`.
- **Ratify:** Yes.

### TD-114 — Inject shared login runtime dependencies at API composition
- **Question:** T-110a3's `LoginService` requires the shared `PolicyStore`, while the shipped `AppState` carries no policy store and API router initialization is synchronous.
- **Options:** (a) create a new policy store/listener per login request or bypass the service; or (b) initialize the store once in the API composition root, make router construction async, and inject a shared login runtime containing the compiled module registry and enabled-module service.
- **Decision:** Use (b), with the current `AuditModule` and `OutboxEventSink`; until T-301 supplies the embedded preset loader, a missing persisted policy fails closed instead of silently applying an unverified preset/default patch. The initial eager async-router construction is superseded by TD-116 so health routes remain available when PostgreSQL is down.
- **Why:** IDN-06 and T-110a3 require the existing `PolicyStore` boundary, POL-04 requires its bounded cache/listener, and CA-30/31 keep composition outside module-owned table access. The fallback does not add or select a policy value.
- **Changed:** `crates/eq-api/src/lib.rs`, `crates/eq-api/src/modules.rs`, `crates/eq-server/src/main.rs`, `crates/eq-testkit/src/api.rs`, existing API test callers, `crates/eq-identity/src/api_auth.rs`, this task and generated `tasks/INDEX.md`.
- **Ratify:** Yes.

### TD-116 — Lazily initialize the shared login policy store

- **Question:** `PolicyStore::new` needs an async PostgreSQL listener, but eagerly creating it while building the API router prevents `/healthz` and `/readyz` from starting when the database is unavailable.
- **Options:** (a) make router construction async and fail server startup if the policy listener cannot connect; or (b) inject one shared `OnceCell` runtime at composition and initialize its policy store on the first login request.
- **Decision:** Use (b); after first initialization, all login requests share the same `PolicyStore`, cache and listener. Router construction stays synchronous.
- **Why:** POL-04 requires a shared bounded cache and listener, while OPS-06 requires health/readiness routes to remain usable during a database outage. Lazy initialization preserves both without changing login policy behavior.
- **Changed:** `crates/eq-identity/src/api_auth.rs`, `crates/eq-api/src/lib.rs`, `crates/eq-api/src/modules.rs`, this task and generated `tasks/INDEX.md`.
- **Ratify:** Yes.

### TD-115 — Complete the login `PersonMe` contract without adding permission grants
- **Question:** The login response must contain the spec's full `PersonMe.permissions` snapshot, while T-110a5 owns shared Actor extraction and role switching.
- **Options:** (a) emit a placeholder/empty profile until T-110a5; or (b) compute only the login response snapshot from the already-compiled, enabled module registry and read the person's profile through the core-owned directory port.
- **Decision:** Use (b); apply active-role grants and the IDN-11 owner-without-admin rule exactly, expose S2 email/phone only when the resulting set contains the existing `identity.people.read_pii`, and let T-110a5 reuse the same permission calculation for Actor extraction. Add no permission key or role grant.
- **Why:** `spec/modules/identity.md` §2 fixes the `PersonMe` wire contract, IDN-10/11 fix the derivation, and CA-30/ERD-02 require core-owned person reads through a port. SBD-10/ERD-04 prohibit returning PII without `read_pii`.
- **Changed:** `crates/eq-core/src/auth/directory.rs`, `crates/eq-api/src/modules.rs`, `crates/eq-identity/src/api_auth.rs`, `crates/eq-identity/src/module.rs`, this task and generated `tasks/INDEX.md`.
- **Ratify:** Yes.

### TD-117 — Split Actor authentication and cookie role switching
- **Question:** T-110a5 combines verified Actor extraction, permission caching, and a cookie-authenticated role-switch endpoint, which together are larger than CQ-101's approximately 400-line review guide.
- **Options:** Keep both concerns in one task and exceed the review-size guide; or split Actor/permission work and role switching into two ordered child tasks, then retain T-110a5 as the aggregate verification task.
- **Decision:** Create T-110a4a for verified Actor authentication and permission derivation, T-110a4b for role switching/CSRF, and keep T-110a5 as a draft aggregate until both are reviewed.
- **Why:** CQ-101 directs larger concerns to be split before implementation, while preserving each original acceptance criterion and named test once.
- **Changed:** T-110a5, the two child task files, and generated task index.
- **Ratify:** Yes.

### TD-118 — Register the already-specified identity write permission
- **Question:** T-110a5 AC2 requires `identity.people.write`, but the task says not to add permission keys/grants and the shipped `IdentityModule` currently declares none.
- **Options:** Leave the permission unregistered and make the acceptance criterion impossible; or register the existing spec key with its exact Admin grant and use IDN-11's owner fallback only when no active admin exists.
- **Decision:** Register only the already-specified `identity.people.write` definition and its existing Admin grant; do not add any other permission.
- **Why:** `spec/modules/identity.md` §3 and `spec/25` §6 already define the key and grant, and IDN-11 defines the conditional owner behavior. This adds no business permission or role grant beyond the spec.
- **Changed:** `crates/eq-identity/src/module.rs`, T-110a4a and this decision register.
- **Ratify:** Yes.

### TD-119 — Require Origin on role-switch requests
- **Question:** T-110a5's Security section said to match Origin only "where supplied", but cookie endpoints require a configured-origin match.
- **Options:** Accept requests without Origin if their CSRF header matches; or reject missing/mismatched Origin along with a missing/invalid CSRF header.
- **Decision:** Require `Origin` to equal `RITMA_PUBLIC_URL`'s origin and require `X-EQ-CSRF: 1`.
- **Why:** `spec/25` §3.3 and `spec/23` CAN-34 define both controls; the stricter spec rule is mechanical and doesn't weaken security.
- **Changed:** T-110a5 and T-110a4b.
- **Ratify:** Yes.

### TD-120 — Keep split-task dependencies in index order
- **Question:** The task-index validator rejects a task that depends on a higher-sorting task ID, so T-110a5 cannot depend on child IDs T-110a5a/T-110a5b.
- **Options:** Weaken or change the task-index DAG validation; or assign the children the free lower IDs immediately after their T-110a4 dependency.
- **Decision:** Use T-110a4a for Actor authentication/permissions and T-110a4b for role switching, with T-110a5 as the aggregate task.
- **Why:** `scripts/tasks-index.sh --check` is a required guard and task ordering compares IDs; choosing the available ordered suffixes preserves the dependency DAG without changing implementation scope.
- **Changed:** T-110a4a, T-110a4b, T-110a5 and generated task index.
- **Ratify:** Yes.

### TD-121 — Keep extracted bearer tokens in secret-safe storage

- **Question:** The Identity middleware receives the access token as header text, but using a plain `String` beyond parsing would retain authentication material without the secret wrapper required by SBD-50.
- **Options:** Keep the parsed token as an ordinary string until verification; or wrap it immediately in `SecretString` and verify only through its explicit exposure API.
- **Decision:** Return `SecretString` from the canonical Bearer parser, expose it only for the JWT verifier call, and test that formatting the wrapper omits a fake token.
- **Why:** SBD-50 requires secret values to use secrecy types and redact Debug output; this preserves CAN-30's exact wire header without retaining a plain token string.
- **Changed:** `crates/eq-identity/src/api_auth.rs`, `tasks/M1-core/T-110a4a-actor-auth-and-permissions.md`, and this decision register.
- **Ratify:** Yes.

### TD-122 — Keep split Actor task dependencies in index order

- **Question:** T-110a4a exceeded CQ-101 and must become an aggregate depending on smaller ordered subtasks, but every dependency ID must sort lower than its dependent.
- **Options:** Rename the existing lowercase aggregate and repoint its downstream tasks; or preserve it and use uppercase-suffix child IDs that sort before it.
- **Decision:** Preserve T-110a4a as the draft aggregate and create T-110a4A, T-110a4B and T-110a4C, ordered A → B → C → aggregate.
- **Why:** `scripts/tasks-index.sh` fixes `LC_ALL=C` and compares IDs bytewise; uppercase suffixes sort before the lowercase aggregate. This keeps the DAG guard unchanged and avoids renumbering T-110a4b and T-110a5.
- **Changed:** the aggregate and split task files, the dependent T-110a5 task contract, and generated `tasks/INDEX.md`.
- **Ratify:** Yes.

### TD-123 — Keep the Identity surface test aligned with the shipped login endpoint
- **Question:** `defaults_schema_and_summary_match_contract` still asserts that Identity has no routes, but T-110a4 has shipped the spec-defined login route.
- **Options:** Keep the stale empty-route assertion and leave the named test failing; or assert the exact `POST /api/v1/auth/login` path while preserving the remaining empty-surface checks.
- **Decision:** Assert that the manifest exposes `/api/v1/auth/login`.
- **Why:** `spec/modules/identity.md` §2 and shipped T-110a4 define this route; replacing only the obsolete assertion keeps the named test and changes no business or security behavior.
- **Changed:** `crates/eq-identity/src/module.rs`, `tasks/M1-core/T-110a4B-identity-permissions.md`, and this register.
- **Ratify:** Yes.

### TD-124 — Use CACHE-14 auth cache keys

- **Question:** T-110a4C specifies 30/60-second auth caches but does not repeat the cache key shapes.
- **Options:** Key permissions by tenant and role only; or use the exact CACHE-14 keys for both auth caches.
- **Decision:** Keep credential versions keyed by `(tenant, person)` and permission sets keyed by `(tenant, person, role)`.
- **Why:** `spec/26-containers-and-caching.md` §6.1 (CACHE-14) defines those keys and their 60/30-second TTLs.
- **Changed:** `crates/eq-identity/src/api_auth.rs`, this task, and this register.
- **Ratify:** Yes.

### TD-125 — Register the role-switch route in the Identity module
- **Question:** T-110a4b requires a new HTTP handler but its `## Files` table omits the Identity module route registry.
- **Options:** Add the handler without registering it, leaving the endpoint unreachable; or include the module route registry as a task file.
- **Decision:** Modify `crates/eq-identity/src/module.rs` to register `POST /api/v1/auth/switch-role` and keep the module surface test aligned.
- **Why:** The task's Scope requires a working HTTP route, and this is the existing module composition point. The file-list omission is mechanical and the added file is limited to exposing the requested handler.
- **Changed:** `crates/eq-identity/src/module.rs`, T-110a4b, and generated `tasks/INDEX.md` if needed.
- **Ratify:** Yes.

### TD-126 — Use the specified session controls for cookie role switching
- **Question:** SBD-40 says every non-public handler calls `actor.require(perm)`, but the cookie role-switch endpoint has no Actor before refresh-cookie authentication and no self-switch permission is specified.
- **Options:** Require an unrelated registered permission, which would prevent ordinary active roles from switching; or enforce the refresh-session authentication, exact CSRF/Origin checks and active target membership defined for this endpoint.
- **Decision:** Use a valid, unexpired refresh session plus an active target membership as the authorization controls for role switching; do not add an `actor.require` permission or grant.
- **Why:** IDN-05/IDN-09 and `spec/25` §4.4 define the cookie session, CSRF/Origin, and active-membership checks for this operation. No role-switch permission exists, and T-110a4b explicitly excludes new grants.
- **Changed:** `crates/eq-identity/src/api_auth.rs`, `crates/eq-identity/src/service.rs`, T-110a4b, and generated `tasks/INDEX.md` if needed.
- **Ratify:** Yes.

### TD-127 — Split role switching to stay within CQ-101

- **Question:** The T-110a4b implementation was approximately 900 changed lines, above CQ-101's approximately 400-line review guide.
- **Options:** Keep the service, HTTP boundary and acceptance tests in one task; or split them into ordered persistence/service, HTTP/CSRF and HTTP-test parts while preserving the original aggregate contract.
- **Decision:** Split into T-110a4a1, T-110a4a2 and T-110a4a3; keep T-110a4b as a draft aggregate until all three are reviewed.
- **Why:** CQ-101 calls for larger work to be split first, and the task order preserves all original acceptance criteria and test names without changing any business, schema or security rule.
- **Changed:** the T-110a4b aggregate, three new child task files, this decision register and generated `tasks/INDEX.md`.
- **Ratify:** Yes.

### TD-128 — Check role-switch CSRF before tenant resolution

- **Question:** The API-wide `tenant_guard` resolves the request tenant before the role-switch handler can validate CSRF, which can query PostgreSQL on an invalid cross-site request.
- **Options:** Keep the handler-only CSRF check and allow tenant lookup first; or add a path-specific CSRF/Origin guard outside the tenant resolver.
- **Decision:** Add `role_switch_csrf_guard` in Identity and install it in `health::router` outside `tenant_guard`, while retaining the handler check as defense in depth.
- **Why:** T-110a4a2, IDN-05 and `spec/25` §3.3 require invalid CSRF/Origin to fail before database access; `health::router` is the composition point that orders the middleware.
- **Changed:** `crates/eq-identity/src/api_auth.rs`, `crates/eq-api/src/health.rs`, T-110a4a2, T-110a4b, this register and generated `tasks/INDEX.md`.
- **Ratify:** Yes.

### TD-129 — Use a documentation commit for aggregate-only T-110a

- **Question:** The original DoD asks for a `feat(identity)` implementation commit, while the split parent Scope now says to verify the completed parts and report their results only.
- **Options:** Keep the feature commit type despite no product-code change; or use a documentation commit that matches the aggregate-only Scope.
- **Decision:** Use `docs(identity): report login aggregate [T-110a]`.
- **Why:** The parent Scope expressly excludes reimplementation and owns only final verification; the signed commit changes the report, this decision register and the generated index.
- **Changed:** `tasks/M1-core/T-110a-identity-schema,-login-and-credential-validation.md`, this decision register and generated `tasks/INDEX.md`.
- **Ratify:** Yes.

### TD-130 — Reuse fixed build and database resources for T-110a verification

- **Question:** The old T-110a Report says the disk rule prohibits creating any Cargo target cache, database or container, while the current user policy allows the fixed shared target and one task database inside the existing PostgreSQL container.
- **Options:** Leave all database-backed ACs Not run; or reuse `$HOME/.cache/ritma/target` and `ritma-db-1`, create one `ritma_test_T-110a` database, then drop it.
- **Decision:** Use the existing target and container, create only `ritma_test_T-110a` for the aggregate checks, and drop it after the tests.
- **Why:** The current disk-hygiene instruction explicitly requires reuse of those fixed resources and permits one task-named database while forbidding additional containers, images, volumes and target directories.
- **Changed:** `tasks/M1-core/T-110a-identity-schema,-login-and-credential-validation.md`, this decision register and generated `tasks/INDEX.md`.
- **Ratify:** Yes.

### TD-131 — Require Origin on refresh requests
- **Question:** T-110b says Origin must match only when present, while the cookie-authenticated refresh contract requires both the CSRF header and configured Origin.
- **Options:** Permit a missing Origin if `X-EQ-CSRF: 1` is present; or require the exact Origin and CSRF header.
- **Decision:** Require `X-EQ-CSRF: 1` and an Origin equal to `RITMA_PUBLIC_URL`'s origin before refresh handling.
- **Why:** `spec/25` §3.3 defines both controls for cookie-authenticated routes. This is the stricter spec reading and matches TD-119/TD-128.
- **Changed:** T-110b and `crates/eq-identity/src/api_auth.rs`.
- **Ratify:** Yes.

### TD-132 — Correct T-110b's implementation file list
- **Question:** T-110b lists only its API test and `migrations/0007_identity.sql`, but its in-scope refresh route requires implementation changes and T-110a already shipped that migration.
- **Options:** Leave the file list stale and implement the route outside its declared Files; or list the existing Identity HTTP, service, repository, module and error-registry files, with no migration edit.
- **Decision:** Update T-110b's Files table to name the implementation files and omit the already-shipped migration.
- **Why:** The route and behavior are explicitly in Scope, the migration is present from T-110a, and the mapped DDL already contains the required columns (spec 06 §4.2; ERD §7).
- **Changed:** T-110b Files table.
- **Ratify:** Yes.

### TD-133 — Use the specified refresh success response
- **Question:** T-110b specifies refresh and rotation behavior but omits the successful response body shape.
- **Options:** Invent a new refresh DTO; or use the existing Identity endpoint contract.
- **Decision:** Return the same response shape as login, with a new access token and rotated `eq_rt` cookie.
- **Why:** `spec/modules/identity.md` §2 explicitly defines `POST /auth/refresh` as returning the login response; no new public contract is needed.
- **Changed:** T-110b Interfaces and `crates/eq-identity/src/api_auth.rs`.
- **Ratify:** Yes.

### TD-134 — Split the T-111 identity workflows

- **Question:** T-111 combines invite issuance/acceptance, password reset, profile/session management and TOTP/recovery login in one task, exceeding CQ-101's approximate 400 changed-line review limit.
- **Options:** (a) keep all six security workflows in one task; (b) split them into ordered implementation parts and retain T-111 as an aggregate.
- **Decision:** Split implementation into T-110c through T-110h, chained by dependencies; T-111 depends on all parts and becomes an aggregate verification/report after they are review.
- **Why:** CQ-101 and the task's exact acceptance names require smaller independently reviewable changes; the split preserves all listed behavior and tests.
- **Changed:** T-111, T-110c through T-110h, and generated tasks/INDEX.md.
- **Ratify:** Yes.

### TD-135 — Carry invite token to the Notify consumer

- **Question:** identity.user_invited.v1 lists person_id, role and invite_id, while FLW-12 requires Notify to build an invite URL and the database stores only token_hash.
- **Options:** (a) add the raw token as a secret field in the existing server-side outbox payload; (b) keep the declared payload unchanged, leaving Notify unable to create a usable invite link.
- **Decision:** Add token to the existing identity.user_invited.v1 payload, keep it confined to the server-side outbox and never log or expose it.
- **Why:** FLW-12 requires an invite URL containing the token, and a SHA-256 hash cannot recover the raw token. The password-reset flow already keeps its raw token in the server-side outbox for Notify. The event name and database schema remain unchanged.
- **Changed:** T-111 and T-110c task contract; T-110c will implement the Identity invite event.
- **Status:** Not implemented; superseded for T-110c by TD-137. Email/WhatsApp delivery remains outside T-110c pending a spec-compliant Notify handoff.
- **Ratify:** Yes.

### TD-136 — Keep split task ids before the aggregate

- **Question:** T-111a through T-111f sort after T-111, so a dependency from T-111 to those parts fails the task index's topological-order guard.
- **Options:** (a) keep the later child ids and weaken the task index ordering rule; (b) use the next unused suffixes after T-110b, which sort before T-111.
- **Decision:** Rename the six parts to T-110c through T-110h and keep T-111 as their aggregate.
- **Why:** tasks/README.md requires inserted tasks to use the lower neighbor's id plus a suffix; scripts/tasks-index.sh enforces that dependencies sort before dependants.
- **Changed:** T-111, T-110c through T-110h, this register and generated tasks/INDEX.md.
- **Ratify:** Yes.

### TD-137 — Keep invite tokens out of the event payload

- **Question:** TD-135 adds a raw S3 invite token to `identity.user_invited.v1`, but the normative event contract and security lineage prohibit S2/S3 values in events, while T-110c excludes notification delivery.
- **Options:** (a) include the token in the server-side event as TD-135 proposed; (b) add a durable protected token handoff or schema field; (c) keep the spec-defined IDs-only payload, return the URL only for link channel, and leave email/WhatsApp delivery to M5 Notify.
- **Decision:** Use option (c). `identity.user_invited.v1` carries only `{person_id, role, invite_id}`. T-110c returns the URL only for `link`; it does not persist or emit the S3 token for email/WhatsApp delivery.
- **Why:** CAN-60 says payloads contain minimal IDs/facts and no S2/S3 values; FLW-60 explicitly excludes invite tokens from events; `spec/03-module-system.md` §6 declares the exact payload; T-110c places email/WhatsApp sending out of scope. This chooses the stricter spec-compliant behavior without adding a schema object or weakening a security control.
- **Changed:** `tasks/M1-core/T-110c-invite-issue.md`, this register, and the invite event implementation.
- **Ratify:** Yes.

### TD-138 — Expose tenant display name to the invite preview through Core

- **Question:** T-110d's preview requires `tenant_name`, but `tenants.name` is not readable by `eq_app` and the existing resolver `resolve_tenant_info_by_slug` (migration 0009) omits it, so Identity has no approved path to it.
- **Options:** (a) add a Core-owned SECURITY DEFINER function that returns `tenants.name` for the caller's tenant only, and call it through the Core directory port; (b) drop `tenant_name` from the preview; (c) return `slug` as `tenant_name`.
- **Decision:** Option (a). Migration `migrations/0012_core_tenant_name_resolver.sql` defines `public.resolve_tenant_name(p_tenant_id uuid) RETURNS text`, `LANGUAGE sql STABLE SECURITY DEFINER SET search_path = pg_catalog, public`, returning `t.name` only when `t.id = p_tenant_id` AND `t.id = NULLIF(current_setting('app.tenant_id', true), '')::uuid`. `REVOKE ALL FROM PUBLIC`, `GRANT EXECUTE TO eq_app`. Identity calls `CoreIdentityDirectory::tenant_name`, never the table.
- **Why:** Option (b) changes the specified contract; option (c) returns a different value under the same name. Option (a) follows the TD-001 pattern (a named function with a grant), keeps SEC-05 (no direct `tenants` SELECT for `eq_app`), and adds no cross-tenant read because the setting check limits the result to the tenant set by `core::db::begin` (the same setting every RLS policy reads, `migrations/0001_core_bootstrap.sql:28`). The number 0012 is the next free forward-only number; 0008 is unused and is not reused.
- **Changed:** `migrations/0012_core_tenant_name_resolver.sql` (new), `tasks/M1-core/T-110d-invite-acceptance.md` (Files, Context, Security, Questions, status `todo`), this register.
- **Ratify:** Yes. Migration `0012` and the new function are not in `spec/06` DDL; `scripts/erd.py --migrations` must be checked and, if it flags the function, the TD-001 allow-list entry applies with a reference to TD-138.

### TD-139 — Retire the `blocked` task status

- **Question:** `WORKFLOW.md` §§2/4 and the status table define `blocked` as the answer to an open question. `tasks/README.md` says a task never ends `blocked`, and `scripts/tasks-lint.py` (line 175) rejects it. The run that produced T-110d and the T-110c/T-108 blocks hit this conflict, and T-110d therefore stopped with the guard failing.
- **Options:** (a) keep `blocked` in the docs and relax the guard; (b) retire `blocked`: a decision is recorded as a TD and the task continues; the guard stays strict.
- **Decision:** Option (b). `WORKFLOW.md` §§1, 2, 4, the state diagram, the status table and the commit table are updated to match. The guard is not changed. Existing `blocked` tasks are reset to `todo` with their open question marked resolved by a TD.
- **Why:** The maintainer requires that no task stay blocked. The lint already encodes this rule, and `tasks/README.md` agrees. Option (a) would weaken the guard the maintainer relies on. Technical questions are decided under the stricter-reading rule of AGENTS.md §0 and recorded here, so they stay traceable. Security, money and schema beyond the spec are still never guessed: the decision records the strictest safe option and is marked Ratify.
- **Changed:** `WORKFLOW.md`, `tasks/M1-core/T-110d-invite-acceptance.md`, this register.
- **Ratify:** Yes.

### TD-140 — Activate invited membership through Core during acceptance

- **Question:** T-110d requires invite acceptance to activate a membership and create a login session in one transaction, while `CoreIdentityDirectory` exposes only `ensure_invited_membership` and Identity must not query the Core-owned memberships table.
- **Options:** (a) add an Identity SQL update of `memberships`; (b) add a Core-owned port operation that transitions the invited role to active and returns the active roles needed for the login session; (c) accept the invite without activating membership.
- **Decision:** Use option (b). Add `activate_invited_membership(tx, tenant, person, role) -> Result<Option<LoginIdentity>, AppError>` to `CoreIdentityDirectory`. It changes only the matching `invited` row to `active`, then returns the person's active roles through Core. Return `None` if no matching invited membership was activated.
- **Why:** STM-20 t3 defines `invited → active` on invite acceptance, STM-21 t2 requires membership activation and login session creation in the same transaction, and CA-30 assigns membership SQL to Core. The returned Core-owned `LoginIdentity` gives the session service only active roles and avoids a second module reading memberships.
- **Changed:** `tasks/M1-core/T-110d-invite-acceptance.md`, Core directory port/adapter, invite service, login-session service, and invite acceptance tests. No schema, business value, or error code is added.
- **Ratify:** Yes.

### TD-141 — Preserve TOTP for an already enrolled invitee

- **Question:** T-110d's accept body contains only a password and places TOTP behavior out of scope, but an invited person may already have a credential with TOTP enabled.
- **Options:** (a) create the acceptance session without verifying the existing TOTP factor; (b) reject acceptance with the existing `totp_required` code and roll back the invite, password, and membership changes; (c) add a TOTP field to the request and expand this task's scope.
- **Decision:** Use option (b). Before changing credentials or creating the refresh session, detect an already enabled TOTP credential and return the spec-defined `totp_required` 401. The handler rolls back the transaction.
- **Why:** AUTH-06/SBD-20 requires preserving enrolled authentication factors, while T-110d does not define a TOTP request contract and explicitly excludes TOTP behavior. The existing login error code avoids inventing a code or bypassing a security control.
- **Changed:** `tasks/M1-core/T-110d-invite-acceptance.md`, `crates/eq-identity/src/service.rs`, and `crates/eq-identity/src/api_invites.rs`.
- **Ratify:** Yes.

### TD-142 — Attribute invite acceptance to the invitee

- **Question:** Public invite acceptance begins with an anonymous transaction actor, but STM-20 identifies the invitee as the actor for `invited → active`; `eq-audit` requires the recorded actor id to match the transaction's `app.actor_id` setting.
- **Options:** (a) record acceptance as anonymous/system; (b) add a generic transaction actor-switch API; (c) resolve the tenant-scoped bearer invite in a read-only transaction, bind the atomic acceptance transaction to that invitee, and revalidate the invite before any mutation.
- **Decision:** Use option (c). Resolve the token and invited role using the existing tenant-scoped, constant-work invite lookup, roll back the lookup transaction, then begin the acceptance transaction with the invitee's `ActorCtx`. Acceptance rechecks pending state and expiry before mutation. Successful login audit retains that authenticated invitee actor; ordinary anonymous login continues to use a system actor.
- **Why:** STM-20 t3 names the invitee as the transition actor; STM-21 t2 requires all acceptance state changes and audit to be atomic; the existing audit adapter enforces actor/transaction equality. A preflight read does not authorize the mutation: the main transaction revalidates token, expiry, and single-use state before setting credentials, activating membership, or creating a session.
- **Changed:** `tasks/M1-core/T-110d-invite-acceptance.md`, `crates/eq-identity/src/api_invites.rs`, `crates/eq-identity/src/invites.rs`, `crates/eq-identity/src/service.rs`, and `crates/eq-identity/tests/api_invites.rs`.
- **Ratify:** Yes.

### TD-143 — Keep reset tokens out of event payloads

- **Question:** T-110e and its T-111 parent require a raw S3 reset token in `identity.password_reset_requested.v1`, while CAN-60 and FLW-60 prohibit S2/S3 values in events and `spec/03-module-system.md` §6 defines this event as `{person_id, token_id}`.
- **Options:** (a) include the raw token in the server-side outbox despite the canonical security rules; (b) add protected token persistence or a handoff, which requires schema and access-control decisions outside this task; (c) emit the spec-defined IDs-only event and defer Notify delivery until a secure handoff is specified.
- **Decision:** Use option (c). Emit `{person_id, token_id}` only. Keep the generated raw token transiently in request memory to compute its digest; never persist, log, return, or emit it.
- **Why:** CAN-60, FLW-60, SBD-10 and the exact event catalogue are stricter and security-normative. No DDL, port, or approved mechanism authorizes a different cross-module token handoff.
- **Changed:** `tasks/M1-core/T-110e-password-reset.md`, reset event implementation and tests. The T-111 aggregate's stale raw-token wording is a follow-up for that task; no other task file is edited here.
- **Ratify:** Yes.

### TD-144 — Use T-110e task-scoped database and SQLx metadata

- **Question:** T-110e's Definition of Done names `ritma_test_T-111`, and its Files list omits `.sqlx` metadata although query changes must be prepared for offline builds.
- **Options:** (a) reuse the aggregate's database name and omit generated metadata; (b) use `ritma_test_T-110e` and list generated `.sqlx/*.json` files.
- **Decision:** Use option (b), create only a fresh `ritma_test_T-110e` in the existing PostgreSQL container, and commit only SQLx metadata generated by this task.
- **Why:** Per-task database isolation prevents stale data and query macros require committed offline metadata (CQ-42).
- **Changed:** `tasks/M1-core/T-110e-password-reset.md` and generated `.sqlx` metadata if query signatures change.
- **Ratify:** Yes.

### TD-145 — Use the system actor for password reset audit

- **Question:** The reset endpoint is a bearer-token operation without an authenticated role but must write `security.password.reset`; the audit adapter rejects `Anonymous` and requires the audit actor to match the transaction actor.
- **Options:** (a) attribute it to the existing `System` actor and identify the affected person as the audit entity; (b) infer an owner role from token possession; (c) add a Core port to infer a membership role from the token owner.
- **Decision:** Use option (a). Begin the tenant transaction with the existing `System` actor and record the reset person's id as the audit entity.
- **Why:** The bearer token proves authority to perform only this reset; it does not prove which active membership role should be recorded. `System` is an existing audit role, while role inference or a new cross-module port is not specified. STM-24 still keeps password update, token consumption, session revocation and audit in one transaction.
- **Changed:** `tasks/M1-core/T-110e-password-reset.md`, `crates/eq-identity/src/api_reset.rs`, service/repository code, and reset tests.
- **Ratify:** Yes.

### TD-146 — Advance the access-token password version on reset

- **Question:** `repo::password_version` floors `password_changed_at` to whole epoch seconds, so setting the same instant during a reset may preserve existing access-token versions.
- **Options:** (a) store only the injected current instant; (b) add a version column, which is outside the mapped schema; (c) store the later of the injected current instant and the previous whole-second timestamp plus one technical second.
- **Decision:** Use option (c) in the existing credentials update.
- **Why:** The shipped IDN-03 token contract compares whole-second versions. A password reset must change that version so existing access tokens stop matching; the monotonic timestamp uses only the mapped column and introduces no business rule or schema change.
- **Changed:** `crates/eq-identity/src/repo.rs`, `crates/eq-identity/tests/api_reset.rs`, and `tasks/M1-core/T-110e-password-reset.md`.
- **Ratify:** Yes.

### TD-147 — Version the self-profile without changing the login DTO

- **Question:** API-07 and CAN-23 require the mutable self-profile response to expose its aggregate version and ETag, but the existing login `PersonMe` DTO has no version and the shared `version_conflict` code was not registered.
- **Options:** (a) change the shared login response DTO; (b) add a self-profile response that preserves the login fields and adds `version`, and register the spec-defined conflict code.
- **Decision:** Use option (b). The `/me` response flattens the existing `PersonMe` data, includes `persons.version`, returns the matching strong ETag, and uses the Core `VERSION_CONFLICT` code. The login JSON remains unchanged.
- **Why:** API-07/CAN-23 require an ETag for this mutable aggregate and the shared stable code is specified; isolating the extra field to `/me` keeps the existing login contract intact.
- **Changed:** `crates/eq-identity/src/api_me.rs`, `crates/eq-core/src/error.rs`, this task's Questions/Report, this register, and generated task index.
- **Ratify:** Yes.

### TD-148 — Accept only canonical E.164 in self-profile updates

- **Question:** T-110f requires phone normalization, but `TenantInfo` has no country and POL-02 prohibits deriving a country from locale, currency or timezone.
- **Options:** (a) guess a default dialing region; (b) add a tenant country field or schema change; (c) accept only the validated canonical E.164 representation and compute the existing tenant-scoped blind index.
- **Decision:** Use option (c). Parse with the existing `Phone` value type, reject non-canonical/non-E.164 values, and blind-index the supplied canonical value through Core crypto.
- **Why:** This follows the existing canonical phone contract and avoids introducing a default or schema field not named by the task/spec.
- **Changed:** `crates/eq-core/src/crypto.rs`, `crates/eq-identity/src/api_me.rs`, this task's Questions/Report, and this register.
- **Ratify:** Yes.

### TD-149 — Scope self-service to the authenticated person

- **Question:** T-110f requires self-service reads and writes but does not name a separate self-write permission; the handlers still must check a permission and must not allow another person's data to be changed.
- **Options:** (a) add a new permission key or role grant; (b) use the existing spec-defined `identity.people.read` permission and bind every route/query to the authenticated Actor's own person/session.
- **Decision:** Use option (b). The module uses the existing permission matrix; operations derive person and tenant only from Actor and session revocation targets only a family owned by that person.
- **Why:** This adds no permission key or role grant and enforces resource ownership at the route and repository boundary.
- **Changed:** `crates/eq-identity/src/api_me.rs`, `crates/eq-identity/src/module.rs`, `crates/eq-identity/src/repo.rs`, this task's Questions/Report, and this register.
- **Ratify:** Yes.

### TD-150 — Do not invent an outbox event for self-profile changes

- **Question:** The repository-wide state-change guideline calls for audit and outbox work, but the event catalogue does not define a profile, password-change or session-revocation event and T-110f names no such event.
- **Options:** (a) invent and emit a new event name/payload; (b) record the required audits transactionally and leave the outbox unchanged.
- **Decision:** Use option (b). The password and session audits use their task/spec-defined actions; the profile audit action follows STM-04 and Core ownership as recorded in TD-153. No outbox event is emitted.
- **Why:** Event names and payloads are contracts and cannot be inferred. Adding a new event would exceed scope and violate the no-invented-event rule.
- **Changed:** `crates/eq-identity/src/api_me.rs`, `crates/eq-identity/src/service.rs`, this task's Questions/Report, and this register.
- **Ratify:** Yes.

### TD-151 — Use the operator-approved T-110d database for T-110f

- **Question:** The task's DoD names `ritma_test_T-111`, while the operator explicitly allows creating only `ritma_test_T-110d` in the existing PostgreSQL container.
- **Options:** (a) create the task-named database; (b) create only the operator-approved fresh database and drop it after the checks.
- **Decision:** Use option (b). All T-110f database checks run against a fresh `ritma_test_T-110d` in the existing `ritma-db-1` container; the database is dropped after checks.
- **Why:** The explicit resource boundary limits created databases, while the fresh database still prevents stale-state results.
- **Changed:** `tasks/M1-core/T-110f-profile-and-sessions.md`, this register, and the task Report.
- **Ratify:** Yes.

### TD-152 — Track the missing shared API rate limiter

- **Question:** API-15 specifies 100 requests/minute per authenticated user and 300/minute per IP with `RateLimit-*` headers, but the current API composition has no shared rate-limiting middleware. T-110f's Files and Scope are limited to Identity's self-profile and session paths.
- **Options:** (a) add a one-off limiter to these handlers, which would not provide uniform API middleware or cover other authenticated routes; (b) leave the rate limit explicitly unimplemented here and require a shared API middleware task before these routes are externally exposed.
- **Decision:** Use option (b). Do not claim API-15 is satisfied by this task. Record the gap as a required follow-up for shared API middleware using the existing `tower_governor` stack and the exact API-15 limits.
- **Why:** A local workaround would create inconsistent limits and omit the shared `RateLimit-*` behavior. A cross-cutting fix needs to cover all authenticated routes and belongs in the API composition, which T-110f does not name or own.
- **Changed:** `tasks/M1-core/T-110f-profile-and-sessions.md` and this register. No rate-limit control is weakened or reported as implemented.
- **Ratify:** Yes; maintainer review is required before deployment.

### TD-153 — Name the Core-owned profile audit action

- **Question:** T-110f updates the Core-owned `persons` aggregate through the Core port, but does not name the profile-update audit action.
- **Options:** (a) use an Identity action because the route lives in Identity; (b) use a Core action because ERD-01 assigns `persons` to Core, while following STM-04's `<module>.<entity>.<verb_past>` format.
- **Decision:** Use option (b), `core.person.updated`.
- **Why:** The updated aggregate is Core-owned; the audit action follows the normative module/entity/verb format without adding an outbox event.
- **Changed:** `crates/eq-identity/src/api_me.rs`, this task's Questions/Report, and this register.
- **Ratify:** Yes.

### TD-154 — Record the T-110f task-size deviation

- **Question:** CQ-101 asks for about 400 changed non-generated lines per task/PR and says larger work should be split, while T-110f already assigns five API behaviors across Core and Identity paths and the operator requires one task commit without editing other tasks.
- **Options:** (a) rewrite the task breakdown and successor contracts before implementation; (b) complete this assigned contract in one task commit and disclose the measured size for architect review.
- **Decision:** Use option (b), as directed by the operator's one-task-per-commit and no-block run instructions.
- **Why:** The existing task contract was already implemented in the active workspace; splitting it now would require changing other task files and separating dependent code beyond this task's authorized edit set. The implementation-only diff is 2,066 lines excluding generated SQLx metadata.
- **Changed:** `tasks/M1-core/T-110f-profile-and-sessions.md`, this register, and the generated task index.
- **Ratify:** Yes; review whether equivalent future task contracts should be split before implementation.

### TD-155 — Preserve the declared TOTP wire contract

- **Question:** T-110g's context notes that `totp-rs`'s mapped `qr` feature returns a PNG, while its Interfaces/Acceptance criteria and the T-111 aggregate require `qr_svg`. T-111's old shorthand for DELETE also says only `password`, while STM-25 requires current password and TOTP code.
- **Options:** (a) change the endpoint response to PNG and omit the code; (b) keep the task's declared `qr_svg` response and STM-25's password-plus-code request, adapting the library output without adding a dependency.
- **Decision:** Use option (b). Embed `totp-rs`'s PNG data URI in a minimal SVG response; require both the current password and a valid current TOTP code for disable.
- **Why:** The task's explicit API and acceptance contracts plus STM-25 give the more specific current behavior. The generated PNG is fixed library output, so embedding it in SVG preserves the wire type without adding a crate or allowing user markup.
- **Changed:** `crates/eq-identity/src/api_me.rs`, `crates/eq-identity/src/service.rs`, `crates/eq-identity/src/module.rs`, `crates/eq-identity/tests/api_me.rs`, and `tasks/M1-core/T-110g-totp-enrolment.md`.
- **Ratify:** Yes.

### TD-156 — Limit the TOTP setup secret disclosure

- **Question:** SBD-10 classifies TOTP seeds as S3 and generally says they are never returned, while IDN-12, FLW-60 and T-110g require the user to receive a setup QR/URI/seed to enroll an authenticator.
- **Options:** (a) suppress the setup material and leave the declared enrollment flow unusable; (b) disclose only the newly generated seed through the authenticated setup response, with no-store handling, redacted Debug, and no logging or later read endpoint.
- **Decision:** Use option (b); setup returns the declared URI/QR/base32 fields only while creating a pending secret for the authenticated Actor. The secret remains encrypted at rest, response caching is disabled, and no endpoint or log exposes it later.
- **Why:** FLW-60 explicitly names the setup response as the one-time TOTP disclosure path. This is the narrow enrollment flow needed by IDN-12 and the task; general S3 reads remain prohibited.
- **Changed:** `crates/eq-identity/src/api_me.rs`, `crates/eq-identity/src/service.rs`, `crates/eq-identity/tests/api_me.rs`, and `tasks/M1-core/T-110g-totp-enrolment.md`.
- **Ratify:** Yes.

### TD-157 — Reuse the operator-approved task database name for T-110g

- **Question:** The existing API test fixture is constrained to a single task database name, while this task must not create any database except `ritma_test_T-110d` in the existing `ritma-db-1` container.
- **Options:** (a) create a second task-named database; (b) run T-110g against a fresh `ritma_test_T-110d` and drop it after the task checks, leaving every other database untouched.
- **Decision:** Use option (b). Confirm `ritma_test_T-110d` is absent before creation; create only that database and drop only it after testing. Do not read, modify or delete `ritma_test_T-110a4b`.
- **Why:** This is the operator's exact resource boundary and preserves the isolation provided by a fresh disposable database.
- **Changed:** `crates/eq-identity/tests/api_me.rs`, `tasks/M1-core/T-110g-totp-enrolment.md`, `tasks/DECISIONS.md`, and the generated task index.
- **Ratify:** Yes.

### TD-158 — Register the spec-defined TOTP transition error

- **Question:** T-110g must return invalid_transition for an invalid enrollment state, and spec/19 ERR-10 already defines that 409 code, but eq_core::error::codes had no corresponding shared symbol.
- **Options:** (a) add a task-local error or reuse an unrelated conflict; (b) register the exact existing spec code in the shared Core registry and test its 409 mapping.
- **Decision:** Use option (b), adding codes::INVALID_TRANSITION with the spec-defined wire value and conflict status.
- **Why:** The value and status are already fixed by spec/19; adding the missing symbol makes the existing interface usable without inventing an error code.
- **Changed:** crates/eq-core/src/error.rs, crates/eq-core/src/error/tests.rs, tasks/M1-core/T-110g-totp-enrolment.md, this register, and the generated task index.
- **Ratify:** Yes.

### TD-159 — Audit the pending TOTP enrollment transition

- **Question:** STM-25 names audit actions for TOTP enable/disable but not the off-to-pending setup transition; SBD requires security state changes to be audited.
- **Options:** (a) omit the setup audit; (b) use the action security.totp.setup_started and record only the pending/enabled booleans, without including the seed, URI, QR or recovery values.
- **Decision:** Use option (b). The action describes the transition and its audit payload contains only state booleans.
- **Why:** This preserves the security audit trail while following the action naming pattern in STM-25 and keeping all enrollment secrets out of audit data. Do not emit an outbox event because the event catalogue defines none; T-110f's TD-150 records that boundary.
- **Changed:** crates/eq-identity/src/service.rs, crates/eq-identity/tests/api_me.rs, tasks/M1-core/T-110g-totp-enrolment.md, this register, and the generated task index.
- **Ratify:** Yes.

### TD-160 — Record T-110g review-size deviation

- **Question:** T-110g's assigned enrollment, confirmation, disable, migration and HTTP security tests exceed CQ-101's approximate 400 non-generated changed-line review guidance, while the operator requires this task to be finished as one signed commit.
- **Options:** (a) split the implementation into new task files and move scope; (b) complete the existing contract in this task, disclose its measured size, and ask the architect to adjust the future breakdown.
- **Decision:** Use option (b), keeping the existing task boundaries and documenting the code review-size deviation for maintainer review.
- **Why:** The task and user instruction assign one task/commit, and moving work would require changing other task contracts beyond this task's scope.
- **Changed:** tasks/M1-core/T-110g-totp-enrolment.md, this register, and the generated task index.
- **Ratify:** Yes; review the task sizing and whether the backlog should split comparable security workflows before implementation.

### TD-161 — Place the TOTP login acceptance test with the auth adapter

- **Question:** The T-110h interface sketch names `crates/eq-identity/tests/api_me.rs`, but the shipped login route and email-backed login fixture are in `api_auth.rs`.
- **Options:** (a) add login identity setup to the unrelated `api_me.rs` fixture; (b) keep the exact `totp_flow` test name and implement it in the auth adapter test file.
- **Decision:** Use option (b), placing `totp_flow` in `crates/eq-identity/tests/api_auth.rs`.
- **Why:** The existing auth fixture exercises the actual login boundary and already seeds the required email identity; this is mechanical drift in the task sketch.
- **Changed:** `tasks/M1-core/T-110h-recovery-code-login.md`, `tasks/DECISIONS.md`, and generated task index.
- **Ratify:** Yes.

### TD-162 — Reuse configured Argon2id parameters for recovery-code verification

- **Question:** T-110h needs to verify a recovery-code secret, but the existing shared verifier accepts only the domain `Password` type.
- **Options:** (a) duplicate Argon2id configuration in the service; (b) add a narrow `SecretString` verifier beside the configured password verifier.
- **Decision:** Use option (b), adding the recovery-code verifier in `password.rs` and reusing its configured Argon2id parameters.
- **Why:** `password.rs` owns the existing Argon2id configuration; this preserves the PHC format used by T-110g without duplicating security-sensitive settings.
- **Changed:** `crates/eq-identity/src/password.rs`, `tasks/M1-core/T-110h-recovery-code-login.md`, `tasks/DECISIONS.md`, and generated task index.
- **Ratify:** Yes.

### TD-163 — Align the T-111 aggregate with IDs-only identity events

- **Question:** T-111's copied context says invite and password-reset events carry raw tokens, while the shipped T-110c/T-110e implementations emit only the spec-defined identifiers.
- **Options:** (a) add S3 tokens to outbox events; (b) keep the IDs-only events and leave secure Notify token delivery as a follow-up.
- **Decision:** Use option (b), and update the T-111 aggregate context to match the shipped event contracts.
- **Why:** CAN-60 and FLW-60 forbid S2/S3 event values; T-110c TD-137 and T-110e TD-143 already implement that stricter contract. No protected token handoff is specified in these task scopes or the schema.
- **Changed:** `tasks/M1-core/T-111-invites-reset-me-totp.md`, this register, and generated `tasks/INDEX.md`.
- **Ratify:** Yes.

### TD-164 — Defer aggregate OpenAPI generation to T-114

- **Question:** T-111's inherited Definition of Done asks for `make api`, but the repository Makefile still defines that target as a T-114 placeholder, while this aggregate task owns no API implementation or generated OpenAPI/client files.
- **Options:** (a) generate or edit API artifacts outside T-111's Files without a working generator; (b) record the check as not run for this aggregate and leave generation to T-114.
- **Decision:** Use option (b); do not claim `make api` passed, and retain the T-114 follow-up.
- **Why:** The target explicitly says OpenAPI generation is added by T-114, and fabricating generated contracts here would exceed T-111's aggregate-only Scope and Files.
- **Changed:** `tasks/M1-core/T-111-invites-reset-me-totp.md`, this register, and generated `tasks/INDEX.md`.
- **Ratify:** Yes.
