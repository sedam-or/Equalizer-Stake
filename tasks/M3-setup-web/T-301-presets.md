---
id: T-301
title: Add the preset files (5 YAML), preset.schema.json, embedded loader and preset validation tests
milestone: M3
status: todo
depends_on: [T-213]
spec_refs: [SET-01, SET-02, POL-09, POL-13, TST-08]
estimated_size: M
---

## Goal
A school picks a template, and the whole configuration comes from data. Non-programmers can add a template by writing YAML, and CI proves every template produces valid rules.

## Context
- SET-01: presets are YAML in `presets/`, embedded with `include_str!` through a build-script-generated `presets.rs` (`crates/eq-core/build.rs` lists `presets/*.yaml` and `presets/preset.schema.json`). Data only.
- File schema (normative, spec 05 §1.1): `id, version, name (I18nText), description (I18nText), icon, order, modules [optional ModuleIds], terminology {term_key: I18nText}, policies {module: merge patch}, catalog {resources {ask_count, default_count, name_pattern (I18nText with {n}), capacity}, level_tracks [{key, name, levels: [key]}], offerings [{key, kind: course, name, description?, level_track?, durations, capacity {min, max}, min_age_months?, max_age_months?, internal?, color?, price_plans [{key, kind, sample_price {CUR: minor}, sessions_included?}]}], rate_cards? (rental, opaque here)}, sample: bool`.
- Level names: preset `levels` are keys only. The display name for each level key is the i18n key `preset.level.<key>` from `presets/i18n/{en,id}.json` (add these files; include all level keys used).
- Prices (spec 05 §1.1): `sample_price` maps currency → minor units. If the tenant currency is missing → seed `0` and set `offerings.price_needs_review = true`.
- **The five presets**:
  1. `presets/music_school.yaml`: **copy verbatim** from `spec/05-presets-and-setup.md` §1.3.
  2. `presets/dance_martial_arts.yaml`, `presets/tutoring.yaml`, `presets/studio_rental.yaml` and `presets/blank.yaml`: write them complete from the bullet specs in spec 05 §1.4, quoted here:
     - **dance_martial_arts:**
       - modules `[enrollment, billing, attendance, notify, reports]`; terminology instructor Coach/Pelatih, offering Class/Kelas.
       - scheduling: Mon–Fri 15:00–21:00, Sat–Sun 08:00–14:00; buffer 0; durations [60, 90]; tiers []; no_show forfeit.
       - billing: cycle 1/7, grouping per_payer, late_fee enabled false, overdue action restrict, unfreeze auto_on_full_payment.
       - attendance: instructor_checkin enabled false; student_marks enabled, marked_by either.
       - enrollment: placement trial, approval auto.
       - catalog: track `dance_levels` [beginner, intermediate, advanced, competition]; offerings ballet, hip_hop, karate with capacity 4–20, durations [60, 90]; sample prices IDR 35000000 / USD 4500 monthly, registration IDR 15000000 / USD 2000; resources ask_count true, default 2, pattern "Studio {n}", capacity 20.
     - **tutoring:**
       - modules `[enrollment, billing, attendance, notify, reports, feedback]`; terminology instructor Tutor/Tutor, offering Subject/Mapel.
       - scheduling: Mon–Sat 13:00–20:00; buffer 5; durations [60, 90]; tiers [{min_hours_before 24, makeup_credit_bps 10000}]; no_show forfeit.
       - billing: cycle 1/10, late fee disabled, overdue restrict.
       - attendance: checkin disabled; student_marks enabled.
       - enrollment eligibility enabled false.
       - catalog: track `grades` [grade_1 … grade_12]; offerings mathematics, english, science with capacity 1–5, durations [60, 90]; sample prices IDR 50000000 / USD 6000 monthly; resources default 3 "Class {n}" / "Kelas {n}" capacity 5.
     - **studio_rental:**
       - modules `[rental, billing, notify, reports]`.
       - scheduling: Mon–Sun 09:00–21:00; buffer 15.
       - rental patch exactly as described in spec 05 §1.4 (`booking_types` rehearsal and recording, payment deposit_to_confirm 3000 bps hold 60 min, who_can_book members_and_guests). **The `rental` policy struct doesn't exist until M6.** The merge-patch test for `rental` is skipped when the module isn't compiled or its policy is the stub `{}`; mark that with a `// TODO(M6)` in the test.
       - billing: late fee disabled, overdue restrict.
       - catalog: resources ask_count true, default 2, `name_pattern {en: "Studio {n}", id: "Studio {n}"}`, capacity 8; `rate_cards` with hourly sample prices IDR 15000000 / USD 2000 for rehearsal and IDR 40000000 / USD 5000 for recording.
     - **blank:** modules `[notify]`, no policies, catalog: 1 resource "Room {n}" ask_count true default 1, capacity 1.
  - If a field name in §1.4 doesn't match the policy struct of a module that already exists (scheduling, identity, core), **stop and ask** (status blocked). For modules not yet implemented (enrollment, billing, attendance, notify, feedback, rental), write the patches exactly as the spec names them. They are validated when those modules land (each module task re-runs TST-08).
- `preset.schema.json` (JSON Schema 2020-12) validates the file shape (not the policy contents).
- Loader API:
```rust
pub struct Preset { pub id: String, pub version: u32, pub name: I18nText, pub description: I18nText, pub icon: String, pub order: i32,
  pub modules: Vec<ModuleId>, pub terminology: BTreeMap<TermKey, I18nText>, pub policies: BTreeMap<ModuleId, serde_json::Value>,
  pub catalog: PresetCatalog, pub sample: bool }
pub fn all() -> &'static [Preset];            // sorted by order
pub fn get(id: &str) -> Option<&'static Preset>;
pub fn policy_for(preset: &Preset, module: &dyn Module) -> serde_json::Value; // default ⊕ patch (POL-09)
```
- TST-08 (`cargo test -p eq-core presets`), for every preset:
  - The file validates against `preset.schema.json`, and `id` equals the filename.
  - For every **compiled** module whose policy struct is real, `policy_for` passes `validate_policy`.
  - `scheduling.recurrence.extra_occurrence.substitute.offering_key` (if any) exists in the preset catalog.
  - Every level key has an `en` and `id` name in `presets/i18n`.
  - The YAML parses identically under `serde_yaml` round-trip (no duplicate keys).

## Scope
### In
- The 5 YAML files, the schema, the i18n files, the build script, the loader and the tests.
- Update `CONTRIBUTING.md`, section "Add a preset" (create the section if missing): a 6-step guide (copy `blank.yaml`, edit, run `cargo test -p eq-core presets`, add translations, open a PR with the `preset` label).

### Out
- Applying presets (T-302).

## Files
| Path | Action | Purpose |
|---|---|---|
| `CONTRIBUTING.md` | create or modify | Path named in this task's contract. |
| `crates/eq-core/build.rs` | create or modify | Path named in this task's contract. |
| `crates/eq-core/tests/presets.rs` | test | Task-listed verification file. |
| `presets/*.yaml` | create or modify | Path named in this task's contract. |
| `presets/preset.schema.json` | create or modify | Path named in this task's contract. |

## Interfaces
Use only the exact domain signatures, DTOs, SQL DDL, endpoints and wire values stated in this task's Context and Scope. Do not add a public contract.

## Business rules
Apply only the policy keys, defaults, ranges and state transitions stated in this task's inline contract. If a value is absent, do not infer one.

## Security
No triggers. The task changes only its listed domain, presentation or static configuration behavior.

## Implementation notes
Follow the ordered behavior already stated in Context, Scope and Acceptance criteria; keep the task's listed test names unchanged.

## Acceptance criteria
- AC1. `presets::tests::all_presets_valid` passes for all 5.
- AC2. A deliberately broken fixture preset (in `crates/eq-core/tests/fixtures/bad_preset.yaml`) with `buffer_minutes: 999` fails with the pointer `/buffer_minutes` → `bad_preset_rejected`.
- AC3. `music_school` merged scheduling policy equals the spec values (opening hours, buffer 10, durations [30, 45], cycle 4 + theory_class substitute) → `music_school_scheduling_values`.
- AC4. `scripts/check-literals.sh` excludes `presets/` (business values are allowed in data).

## Tests to write
| File | Test names |
|---|---|
| `crates/eq-core/tests/presets.rs` | `all_presets_valid`, `bad_preset_rejected`, `music_school_scheduling_values`, `level_names_translated` |

## Definition of done
- [ ] Code quality and architecture: `spec/16-code-quality.md` §12 and `spec/17-clean-architecture.md` followed (decisions in `domain/`, own tables only, no new `#[allow]`); `check-layers.sh` and `check-web-boundaries.sh` pass once T-006 has created them
- [ ] Rust checks pass
- [ ] Commit `feat(core): school presets [T-301]`, signed off

## Report

**Deviations** — RATIFY: TD-067, required task-template headings filled from existing task text only.
**Security deviation** — RATIFY: TD-066, SBD-30 trigger classification and corresponding spec-backed Security section.

## Questions


- **Self-resolved (implementer, run 6; TD-067):** required template sections were missing → complete them only from the existing task contract, Scope and tests without inventing behavior.
- **Self-resolved (implementer, run 6; TD-066):** classify SBD-30 triggers from the task behavior and add the corresponding spec-backed Security section; use `No triggers.` for a reviewed false positive.

## Review
n/a — pending review after implementation.
