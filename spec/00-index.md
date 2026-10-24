# Spec Index — Equalizer v2

> Codename: **Equalizer** (product display name TBD, see `brand/BRAND.md` §4).
> Everything brand-visible reads from one place (`crates/eq-core/src/brand.rs` and `web/src/brand.ts`). A rename touches only those files plus docs.

This folder is the **single source of truth** for v2. It defines *what* and *why*.
`tasks/` defines *how, step by step*. Codex implements **only** from `tasks/**/T-*.md`. Every task inlines the parts of this spec it needs.

## 1. Reading order

| # | File | Purpose |
|---|---|---|
| 01 | `01-product.md` | Vision, personas, the 3-step setup contract, non-goals |
| 02 | `02-architecture.md` | Modular monolith, workspace layout, request lifecycle, events, errors |
| 03 | `03-module-system.md` | `Module` trait, dependency graph, compile-time and runtime gating |
| 04 | `04-policy-engine.md` | Business rules as typed, versioned, effective-dated data, plus every policy struct |
| 05 | `05-presets-and-setup.md` | Presets (YAML) and the 3-step setup wizard |
| 06 | `06-domain-model.md` | Entities, invariants, state machines, full DDL |
| 07 | `07-api-conventions.md` | REST conventions, auth, errors, pagination, idempotency |
| 08 | `08-frontend.md` | SPA architecture, routing, module-aware UI, PolicyForm |
| 09 | `09-design-system.md` | Tokens, theming, components, a11y |
| 10 | `10-security-privacy.md` | Crypto, PII, audit chain, data-subject rights |
| 11 | `11-ops.md` | Config, migrations, jobs, backups, observability |
| 12 | `12-testing.md` | Test pyramid, concurrency proof, e2e |
| 13 | `13-roadmap.md` | Milestones → `tasks/INDEX.md` |
| 14 | `14-distribution.md` | One artifact, four channels (cloud, 1-click, desktop, docker) |
| 15 | `15-cloud.md` | Managed multi-tenant hosting |
| — | `modules/*.md` | One spec per module (identity, scheduling, enrollment, rental, billing, attendance, notify, audit, reports, feedback) |
| — | `adr/*.md` | Architecture Decision Records |

Business and OSS docs live outside `spec/`: `brand/BRAND.md`, `brand/GTM.md`, and the governance files at the repo root.

## 2. Requirement ID scheme

Each requirement has a stable ID: `<AREA>-<NN>`. Tasks reference them in their `spec_refs`. IDs are never reused.

| Prefix | Area | Defined in |
|---|---|---|
| PRD | Product | 01 |
| ARC | Architecture | 02 |
| MOD | Module system | 03 |
| POL | Policy engine | 04 |
| SET | Presets and setup | 05 |
| DOM | Domain model | 06 |
| API | API conventions | 07 |
| FE | Frontend | 08 |
| DS | Design system | 09 |
| SEC | Security and privacy | 10 |
| OPS | Operations | 11 |
| TST | Testing | 12 |
| DST | Distribution | 14 |
| CLD | Cloud | 15 |
| IDN, SCH, ENR, RNT, BIL, ATT, NTF, AUD, RPT, FBK | Module specs | `modules/*.md` |

Normative keywords follow RFC 2119: **MUST**, **MUST NOT**, **SHOULD**, **MAY**.

## 3. Glossary

| Term | Meaning |
|---|---|
| **Tenant** | One school/studio business. Self-hosted installs have exactly one; the cloud has many. |
| **Location** | A physical site of a tenant. It has an address, timezone override and GPS point. |
| **Resource** | A bookable physical thing at a location, usually a room/studio. It has a capacity. |
| **Person** | Any human known to the tenant (owner, admin, instructor, guardian, student). PII is encrypted. |
| **Membership** | A Person's role in a Tenant. A person may hold several roles (e.g. an instructor who is also a guardian). |
| **Offering** | A sellable thing in the catalog: a course (recurring lessons) or a rental product (hourly studio use). |
| **Level** | An ordered progression step within an Offering's track (e.g. Grade 1…6). |
| **Price plan** | How an Offering is charged: monthly fee, per-session, per-hour, plus a one-time registration fee. |
| **Enrollment** | A student taking an Offering at a level, with a recurring schedule pattern and a lifecycle. |
| **Session** | One concrete lesson occurrence: time range, resource, instructor(s), participants. |
| **Reservation** | One concrete rental booking of a resource by a booker. |
| **Calendar hold** | Core's conflict ledger row. Every session/reservation writes holds, and a DB exclusion constraint makes double booking impossible. |
| **Payer** | The person who receives invoices (the guardian for minors, otherwise the student). |
| **Policy** | A typed, versioned, effective-dated document holding one module's business rules for a tenant. |
| **Preset** | A YAML bundle (modules + policy overrides + terminology + sample catalog) used in setup step 1. |
| **Module** | A feature unit (crate) that can be compiled in or out and enabled or disabled per tenant at runtime. |
| **Minor units** | Money as an `i64` in the currency's ISO 4217 minor unit (IDR and USD exponent 2, JPY 0). Rp100.000 = `10_000_000`. See POL-12. |
| **Outbox** | A table of domain events written in the same transaction as the state change and dispatched afterwards. |

## 4. Source of business knowledge

The v0 capstone (archived as `legacy/v0-capstone.zip`, git tag `v0-capstone`) was the source for every business rule in this spec. **Do not open the zip.** Section 4.9 of `04-policy-engine.md` has the full legacy→policy translation table, including how each contradiction found in v0 was resolved.
