---
id: T-302
title: Add the setup API (status, presets, countries, school, modules, rules, complete, reset) and the first-run setup token
milestone: M3
status: todo
depends_on: [T-301]
spec_refs: [SET-02, SET-10, SET-11, SET-12, SET-13, MOD-06, POL-09, PRD-04]
estimated_size: M
---

## Goal
The server side of "set up your school in 3 steps". One transaction per step, resumable after a crash, protected on fresh self-hosted installs by a one-time token.

## Context
- SET-11 first-run token:
  - On `serve` start in single mode with an **empty** `tenants` table: use `RITMA_SETUP_TOKEN` if set, else generate 24 random bytes (base64url).
  - Write it to `RITMA_DATA_DIR/setup-token.txt` (mode 0600) and log `info!("Open {RITMA_PUBLIC_URL}/setup?token={token}")`. **This is the one place a secret may be logged**, because the operator needs it. Allow-list it in `PiiGuardLayer` by using the field name `setup_url`.
  - Store only `sha256(token)` in memory (`SetupGuard`). Compare in constant time.
  - After step 1 succeeds, delete the file and forget the token.
- API (spec 05 §3), all under `/api/v1/setup`:

| Method | Path | Auth | Body → Result |
|---|---|---|---|
| GET | `/status` | none | `{completed, current_step: 1..4, needs_token, tenant?: {name, preset_id}}` |
| GET | `/presets` | none | `[{id, name, description, icon, modules, order, ask_room_count, default_room_count}]` (I18nText resolved per `Accept-Language`) |
| GET | `/countries` | none | `[{code, name (localized via `Accept-Language`, fallback en), locale, currency, timezone}]` from `countries.json` (T-113) |
| POST | `/school` | setup token (single mode, no tenant) **or** owner (multi/cloud) | `{name, country, locale, currency, timezone, preset_id, room_count, owner?: {name, email, password}, setup_token?}` → `{tenant: {id, slug, name}, access_token?, next_step: 2}` + `eq_rt` cookie |
| PATCH | `/school` | owner, before completion | `{name?, country?, locale?, currency?, timezone?, room_count?}` → 204 |
| PUT | `/modules` | owner | `{enabled: [ModuleId]}` → `/meta/modules` body; moves to step 3 |
| GET | `/rules` | owner | like `GET /settings/policies`, but in the step-3 module order: scheduling, enrollment, billing, attendance, rental, notify, feedback, reports, identity, audit (core excluded) |
| POST | `/complete` | owner | `{policies: {module: document}}` → `{redirect: "/dashboard?welcome=1"}` |
| POST | `/reset` | owner | → 204; only while no sessions, enrollments or invoices exist (check via each module's `has_data`), else 409 `setup_reset_blocked` |

- `POST /school`, one transaction (SET-12 / spec 05 §2.2):
  1. Create the tenant: `slug` from the name (lowercase ASCII, non-alnum → `-`, trimmed to 40, a numeric suffix on collision); `country, preset_id, preset_version`, and `locale/currency/timezone`.
  2. Create the owner person (display_name = owner name; email encrypted + bidx), credentials (password per **IdentityPolicy after preset merge**) and the owner membership.
  3. Create the default location (`name` = school name, timezone null).
  4. Create N resources from `name_pattern` with `{n}` = 1..N in the tenant locale, capacity from the preset.
  5. Seed the catalog: level tracks + levels (names from `presets/i18n` for both locales), offerings and price plans (`effective_from` = today local; currency handling per T-301), then call `Module::seed_from_preset(tx, tenant, &catalog_json)` for every compiled module.
  6. Write `tenant_modules` from `preset.modules` (validated with `plan_toggle`; if the preset itself violates dependencies, that's a bug → 500, covered by TST-08).
  7. Policies v1 for every compiled module: core from the step-1 fields (+ preset terminology); the rest `policy_for(preset, module)`. **Skip modules whose policy is still a stub** (`policy_schema()` has no properties). They get v1 lazily through `ensure_initial` when implemented (POL-04 fallback).
  8. Audit `setup.school_created`; set `tenants.setup_step = 2`.
  9. Issue a login (access token + refresh cookie) for the owner.
  - Single mode with an existing tenant → 409 `already_set_up`.
- `PUT /modules`: delegates to `ModuleService::set_enabled` (MOD-06 semantics and errors), then `setup_step = 3`.
- `POST /complete`:
  - For each module in `policies` whose document differs (JSON equality after a normalising round-trip through the struct) from the current version, `PolicyStore::put` with `effective_from = now` and note `"setup"`.
  - Validation errors → 422 with pointers prefixed by `/<module>` (e.g. `/billing/cycle/due_day`), and nothing is saved.
  - Then `setup_step = 4`, `setup_completed_at = now`, and audit `setup.completed`.
- Going back (SET-12): `PATCH /school` edits name, country fields and room count (adding rooms appends `{n}`; reducing archives the highest-numbered rooms only if they have no sessions, else 409 `resource_in_use`). The preset can't change.
- Redirect rule (SET-10): while `setup_completed_at IS NULL`, non-setup API routes other than auth/meta/setup return 503 `setup_required` for **non-owner** callers. The owner may use everything (the wizard needs the settings APIs).
- In multi mode (cloud), `POST /school` is called by an authenticated platform account. **That flow is M9.** For now, multi mode returns 501 from `POST /school` (no token path).

## Scope
### In
- `eq_core::setup` service + handlers in `eq-api`, the `SetupGuard`, the startup token logic, the slug helper, and the step-gating middleware.

### Out
- The SPA wizard (T-310). Cloud signup (M9). Sample data (`sample: true`, M7 demo).

## Acceptance criteria
- AC1. Fresh install: `/setup/status` → `{completed:false, current_step:1, needs_token:true}`. `POST /school` without a token → 401, with a wrong token → 401, with the right token → 201 + an access token, and the token file is deleted. → `api_setup::first_run_token`
- AC2. After step 1 with `music_school` + 3 rooms: 3 resources "Ruang 1..3" (locale id), 4 offerings, 2 level tracks, the price plans in IDR, the modules from the preset enabled, the scheduling policy v1 equal to the preset values, and the identity policy `password_min_length = 8`. → `school_seeds_music_preset`
- AC3. Currency SGD with music_school → price plans 0 and `price_needs_review = true`. → `missing_currency_needs_review`
- AC4. Step 2 with `[billing]` alone → 422 `module_dependency`. → `modules_step_dependency`
- AC5. `POST /complete` with a changed buffer → scheduling v2 (`note = "setup"`), unchanged modules stay v1, and `setup_completed_at` is set. → `complete_writes_only_changed`
- AC6. `POST /complete` with an invalid billing doc → 422 with pointer `/billing/...` (use scheduling if billing is still a stub: `/scheduling/buffer_minutes`) and nothing saved. → `complete_validation_atomic`
- AC7. Reset after a session exists → 409 `setup_reset_blocked`. → `reset_blocked_with_data`
- AC8. A second `POST /school` in single mode → 409 `already_set_up`. → `cannot_setup_twice`
- AC9. "Next → Next → Finish" with **every** preset (no edits) succeeds. → `next_next_finish_all_presets` (SET-13)

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-api/tests/api_setup.rs` | AC names above |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Rust checks pass; `make api` regenerated
- [ ] Commit `feat(core): setup wizard API [T-302]`, signed off

## Report

## Questions
