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

**Total mapped self-resolutions:** 56.

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
