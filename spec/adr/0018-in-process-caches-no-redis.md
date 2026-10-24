# ADR-0018: In-process caches invalidated by PostgreSQL NOTIFY; no Redis

- Status: Proposed (2026-10-25), accepted on merge of the PR that adds `spec/26-containers-and-caching.md`

## Context
Hot paths read small, rarely changing data on every request: tenant resolution, the enabled modules, policies, permission sets, unwrapped tenant keys. Most installs are one process (desktop, Docker, 1-click); the cloud runs a few. PostgreSQL is the only stateful dependency (OSS-05), and a school PC can't be asked to run Redis.

Options considered:
1. **No caching.** Simple, but several queries per request for data that changes a few times a year.
2. **Redis or Memcached.** Shared across instances, but one more service to install, secure, back up and monitor, against ADR-0001/0002.
3. **In-process `moka` caches with short TTLs, invalidated by `LISTEN/NOTIFY` after commit.**

## Decision
Option 3, inventoried in spec 26 §6:
- Every server cache is a `moka` cache keyed by tenant first, listed with key, TTL, invalidation and data class.
- Data whose staleness matters (modules, policies) is evicted by `NOTIFY` after commit, with a 30 s TTL as the safety net. On listener reconnect, all NOTIFY-backed caches are cleared.
- Authenticated API responses are never cached by HTTP intermediaries (`no-store`, D-10).
- Release builds never restore CI caches (D-68).

## Consequences
- ✅ No extra service on any channel. Cache hits cost nanoseconds.
- ✅ Multi-instance correctness comes from PostgreSQL, which already coordinates everything else (outbox, jobs, cron).
- ❌ Rate limits and report caches are per instance. Accepted for v1; the cloud sizes per-instance limits (spec 15).
- ❌ A process can serve stale tenant status or permissions for up to the TTL (60 s / 30 s). Accepted and documented (D-65, D-66).
