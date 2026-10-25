# ADR-0013: Pragmatic clean architecture (functional core, concrete repositories, ports only at real seams)

- Status: Accepted (2026-10-25)

## Context
Textbook Clean/Hexagonal Architecture puts every outbound dependency behind an interface, including one repository trait per aggregate. Our situation differs in three ways:
- Postgres is the only database (ADR-0002).
- Several core invariants are **database constraints**: the `EXCLUDE` constraint against double booking, RLS for tenancy, and append-only triggers.
- Contributors range from students to agents, so every indirection has a real cost in understanding.

## Decision
- **Functional core, imperative shell.** All business decisions live in pure `domain/` code. Services only load, call the domain, persist, audit, emit and commit (spec 17 CA-02).
- **The dependency rule is strict** between layers and between modules (CA-01, CA-30).
- **Repositories are concrete** `async fn`s over `&mut Tx` with compile-checked SQL. There are no repository traits (CA-11).
- **Traits only at real seams:** cross-module ports, external IO with several implementations (channels, gateways, storage, tunnel), and time/randomness (CA-10).

## Consequences
- ✅ Business rules are tested fast and exhaustively without a DB. Services are tested against the real constraints that make them correct.
- ✅ Less code and fewer layers of indirection, so a module is readable end to end.
- ✅ Module toggling stays safe, because boundaries are enforced by crates, events and ports rather than by convention.
- ❌ Swapping the database would require rewriting `repo.rs` files. We accept this: Postgres features are part of the design.
- ❌ Service tests need Postgres (testcontainers). This is mitigated by template databases (TST-20) and by keeping the decisions in the pure domain.
