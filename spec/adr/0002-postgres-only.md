# ADR-0002: PostgreSQL is the only stateful dependency (no Redis)

- Status: Accepted (2026-10-25)

## Context
v0 used Redis for locks, rate limiting and queues. Every extra service makes self-hosting and desktop installs harder.

## Decision
Use **PostgreSQL ≥ 16 only**:
- Exclusion constraints (`btree_gist`) for conflicts.
- `SELECT … FOR UPDATE SKIP LOCKED` for the job queue.
- A transactional outbox with LISTEN/NOTIFY for events.
- Advisory locks for singleton work.
- In-process memory for rate limiting and caches.

The desktop app embeds PostgreSQL (`postgresql_embedded`) instead of switching to SQLite.

## Consequences
- ✅ One backup target. Transactional consistency between state, audit and events.
- ✅ The same SQL dialect everywhere (no SQLite variant to test).
- ❌ In-memory rate limits are per instance. They are acceptable for single instances; the cloud runs a few instances, so the limits are effectively N× (documented).
- ❌ The desktop bundle grows by about 30 MB for the PG binaries.
