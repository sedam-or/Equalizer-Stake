# ADR-001: FastAPI over Django

## Status

Accepted

## Context

Equalizer-Stake needs a Python backend for a music school management system with 15 functional requirements, 20 use cases, 11 domain entities. The system handles real-time attendance (GPS + face verification), async notification dispatch (WhatsApp + email), and scheduled billing (monthly cron). Team has Python experience but limited Django experience.

## Decision

Use **FastAPI 0.110+** with SQLAlchemy 2.0 async, Pydantic v2, and Uvicorn/ASGI.

## Rationale

| Factor | FastAPI | Django |
|--------|---------|--------|
| Async support | Native async/await, asyncpg | Django 4.x async is partial (ORM still sync) |
| API-first | Auto OpenAPI/Swagger, Pydantic validation built-in | DRF needed for API, separate validation layer |
| Performance | ~3-5x faster than Django for I/O-bound (async DB, HTTP) | Thread-per-request, sync ORM bottleneck |
| WebSockets | Native support (future: real-time notifications) | Channels addon required |
| Dependency injection | Built-in `Depends()` — clean service layer | No native DI (use third-party or manual) |
| Pydantic v2 | First-class integration, Rust-core validation | Need separate serializer layer (DRF serializers) |
| Learning curve | Minimal for async Python developers | Heavier: ORM, admin, middleware conventions |
| Admin panel | Not needed (Next.js admin dashboard) | Django admin is killer feature but irrelevant here |
| ORM | SQLAlchemy 2.0 — mature, flexible, async | Django ORM — simpler but less powerful, no async |

### Key factors:

1. **Async is critical**: Attendance checks involve GPS validation (DB query) + notification dispatch (HTTP to WhatsApp) + audit logging. Async handles concurrent requests without thread pool exhaustion.

2. **Pydantic v2 validation**: All 15 FRs have strict input validation (email format, GPS coordinates, UUID references). Pydantic v2 (Rust-core) validates at wire speed, generates JSON Schema, integrates with FastAPI auto-docs.

3. **Cron jobs**: APScheduler runs inside FastAPI lifespan — no separate Celery/Redis queue needed for billing cron (5th/11th of month). Django would need Celery for async task scheduling.

4. **No admin panel needed**: Equalizer-Stake has a custom Next.js admin dashboard. Django admin would be unused overhead.

## Consequences

- No built-in admin panel → build custom admin pages in Next.js
- No Django ORM migrations → use Alembic (more verbose but more control)
- Need to manage own session/auth middleware → FastAPI has OAuth2 scheme, JWT is straightforward
- Smaller ecosystem of reusable apps vs Django → build notification, billing from scratch (acceptable for this scope)

## Alternatives Considered

- **Django + DRF**: Strong ecosystem, but sync ORM is a dealbreaker for async notification dispatch
- **Litestar**: Similar to FastAPI, smaller community, less production maturity
- **Express.js (Node)**: Would split the stack; team prefers Python
