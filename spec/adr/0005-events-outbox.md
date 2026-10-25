# ADR-0005: Modules communicate through a transactional outbox + ports

- Status: Accepted (2026-10-25)

## Decision
- Optional modules never import each other.
- Asynchronous reactions go through versioned domain events written to `outbox` in the same transaction and dispatched at-least-once to idempotent handlers.
- Synchronous cross-module reads go through **ports** (traits in `eq-core::ports`) that return empty results when the owning module is disabled.

## Consequences
- ✅ Toggling a module can't break another one at compile time or runtime.
- ✅ No lost events. There are no dual-write bugs.
- ❌ Eventual consistency between modules, typically under 2 s. The UI shows derived states (e.g. the "frozen" badge) after the event is processed.
