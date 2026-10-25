---
id: T-101
title: Add the core schema migrations (tenants, locations, resources, persons, memberships, links, modules, policies, closures, files, drafts, attention, consents, keys, idempotency)
milestone: M1
status: todo
depends_on: [T-003, T-007]
spec_refs: [DOM-01, ARC-06, POL-03, SEC-02, API-08]
estimated_size: M
---

## Goal
The core tables exist exactly as specified, with RLS on every tenant table, and with Rust row structs and enums for them in `eq-core`.

## Context
- The normative DDL is in `spec/06-domain-model.md`:
  - §4.1: `tenants`, `locations`, `resources`, `persons`, `memberships`, `person_links`, `tenant_modules`, `closures`, `files`, `drafts`, `outbox`, `event_receipts`, `jobs`. **Leave `outbox`/`event_receipts`/`jobs` out of this task**; T-107/T-108 add them.
  - §2 of `spec/04-policy-engine.md`: `policies`.
  - §4.11: `attention_items`, `consents`, `tenant_keys`, `idempotency_keys`.
- **Copy the DDL verbatim.** Column names, types, checks and indexes are the contract for later tasks.
- **Addition to the spec DDL (approved):** `tenants` also gets `locale text NOT NULL DEFAULT 'en'`, `currency char(3) NOT NULL DEFAULT 'USD'` and `timezone text NOT NULL DEFAULT 'UTC'`. These are denormalised copies of `CorePolicy` for fast tenant resolution, kept in sync by T-113.
- RLS: call `SELECT eq_enable_rls('<table>');` (from T-003) for every table with `tenant_id`, **except** `tenants` and `tenant_keys`. Those get no RLS: `tenants` is read through `resolve_tenant`, and `tenant_keys` only through core crypto.
- `tenants` has no RLS. Add `CREATE FUNCTION resolve_tenant_by_slug(p_slug text) RETURNS TABLE(id uuid, slug text, status text) SECURITY DEFINER …` and `resolve_single_tenant()`, which returns the only row or none (errors if more than one).
- DOM-01: enums are `text` + CHECK. Rust maps them with `#[derive(sqlx::Type)] #[sqlx(type_name = "text", rename_all = "snake_case")]`.
- `policies` is append-only: `REVOKE UPDATE, DELETE ON policies FROM eq_app` (inside an exception block if the role is missing).

## Scope
### In
- `migrations/0002_core_tenancy.sql`: tenants, locations, resources, persons, memberships, person_links, tenant_modules, closures.
- `migrations/0003_core_policies_files.sql`: policies, files, drafts, attention_items, consents, tenant_keys, idempotency_keys.
- `eq_core::model::{Tenant, Location, Resource, Person (without decrypted fields), Membership, Role, PersonLink, Closure, FileRow, AttentionItem}` structs (`sqlx::FromRow`, serde) and enums `Role`, `TenantStatus`, `ResourceKind`, `MembershipStatus`, `FilePurpose`, `ClosureKind`, `Severity`.
- `eq_core::ids`: add `LocationId`, `ResourceId`, `FileId` newtypes.

### Out
- Repositories and services (later tasks create them where they're used).

## Files
| Path | Action |
|---|---|
| `migrations/0002_core_tenancy.sql`, `migrations/0003_core_policies_files.sql` | create |
| `crates/eq-core/src/model/*.rs`, `ids.rs` | create/edit |

## Acceptance criteria
- AC1. Migrations apply on a fresh DB. → `eq-core/tests/schema.rs::migrations_apply`
- AC2. Every table with a `tenant_id` column except `tenant_keys` has `relrowsecurity` and `relforcerowsecurity` true (query `pg_class`). → `rls_enabled_on_tenant_tables`
- AC3. `person_links_one_primary`: inserting a second primary guardian for the same student fails. → `one_primary_guardian`
- AC4. `resolve_single_tenant()` errors when two tenants exist. → `single_tenant_guard`

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-core/tests/schema.rs` | `migrations_apply`, `rls_enabled_on_tenant_tables`, `one_primary_guardian`, `single_tenant_guard`, `policies_append_only_for_app_role` (connect as `eq_app` with a password set in the test, expect UPDATE to fail) |

## Definition of done
- [ ] `scripts/erd.py --migrations` passes (ERD-03): every table and column this task migrates matches the `spec/06` DDL and the task's row in `spec/29-erd.md` §7
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Rust checks pass; DDL matches spec §4.1/§4.11 (reviewer diff)
- [ ] Commit `feat(core): tenancy and policy schema [T-101]`, signed off

## Report

## Questions
