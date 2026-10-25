# ADR-0006: Every install is multi-tenant-shaped; isolation via forced RLS

- Status: Accepted (2026-10-25)

## Decision
- All tenant data carries `tenant_id`, with Postgres RLS forced and a per-transaction `app.tenant_id`.
- Self-host has exactly one tenant. The cloud has many.
- The same code paths are used in both modes.

## Consequences
- ✅ No "cloud fork". Moving desktop → cloud is a data import.
- ✅ Defence in depth against missing `WHERE tenant_id` bugs, proven by TST-06.
- ❌ Slight overhead per transaction (`SET LOCAL`), and care is needed with connection reuse (RESET on release).
