# ADR-0012: One ordered migration set; schema always complete regardless of compiled features

- Status: Accepted (2026-10-25)

## Decision
- All migrations live in the top-level `migrations/`, named `NNNN_<module>_<desc>.sql`, and are run by sqlx.
- Every table exists in every build. Cargo features and runtime toggles gate **code**, never schema.

## Consequences
- ✅ Turning a module on or off never needs a migration. Backups restore across builds.
- ❌ Minimal builds carry unused tables (negligible).
