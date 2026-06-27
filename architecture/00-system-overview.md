# Equalizer-Stake — System Overview

## Layered Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    PRESENTATION LAYER                     │
│  Next.js 15 PWA (React 18, Tailwind CSS, face-api.js)   │
│  Service Worker │ IndexedDB │ Manifest                   │
└──────────────────────────┬──────────────────────────────┘
                           │ HTTPS / REST + JSON
                           ▼
┌─────────────────────────────────────────────────────────┐
│                    API GATEWAY LAYER                      │
│  FastAPI (Uvicorn/ASGI, Python 3.12+)                    │
│  CORS │ JWT Auth │ Rate Limiting │ Audit Middleware       │
└──────────────────────────┬──────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│                    SERVICE LAYER                          │
│  Business logic: Billing, Attendance, Notification, etc. │
│  Pydantic v2 schemas │ Dependency injection              │
└──────────────────────────┬──────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│                    DOMAIN LAYER                           │
│  SQLAlchemy async ORM models (11 entities)               │
│  User│Guardian│Student│Teacher│Room│Session│Invoice      │
│  Payment│Attendance│Notification│AuditLog                │
└──────────────────────────┬──────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│                  DATA ACCESS LAYER                        │
│  asyncpg connection pool │ Alembic migrations            │
│  AES-256-GCM encrypt/decrypt for PII columns             │
└──────────────────────────┬──────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│                    DATABASE LAYER                         │
│  PostgreSQL 16         │  Redis 7                        │
│  (primary datastore)   │  (sessions, rate-limit, cache)  │
└─────────────────────────────────────────────────────────┘
```

## Component Boundaries

| Component | Responsibility | Owns | Does NOT Own |
|-----------|---------------|------|--------------|
| PWA (Next.js) | UI rendering, face capture, GPS check, offline drafts | Components, state, IndexedDB | Business logic, auth tokens long-term |
| API Gateway (FastAPI) | Request routing, auth, rate limiting, audit logging | Middleware chain, OpenAPI docs | Business rules |
| Service Layer | Business logic, orchestration, cron jobs | Domain rules, invoice generation | HTTP concerns, DB queries |
| Domain Layer | Entity definitions, relationships | ORM models, enums | Transport, scheduling |
| Data Access | Connection management, migrations | Pool config, migration scripts | Query logic |
| PostgreSQL | Persistent storage, constraints, indexes | Data integrity, JSONB audit | Application logic |
| Redis | Ephemeral state | Rate limit counters, JWT blocklist | Persistent data |

## Tech Stack Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Backend framework | FastAPI 0.110+ | Async native, Pydantic v2 validation, auto OpenAPI |
| ORM | SQLAlchemy 2.0 (async) | Mature, async via asyncpg, declarative models |
| Frontend | Next.js 15 App Router | SSR/SSG, PWA support, React Server Components |
| Database | PostgreSQL 16 | JSONB for audit, partial indexes, partitioning |
| Cache | Redis 7 | Rate limiting (sliding window), session store |
| Auth | JWT HS256 | Stateless, role-based claims (admin/teacher/guardian) |
| Encryption | AES-256-GCM | PII fields at rest (NIK, phone, address) |
| Face detection | face-api.js | Client-side, no biometric data leaves device |
| Notifications | WhatsApp unofficial API + SMTP fallback | Guardian reach in Indonesia, email as backup |
| Deployment | Docker + docker-compose | Reproducible, single-VPS Jakarta region |

## Request Lifecycle

```
1. Client (PWA) → HTTPS request with Bearer JWT
2. Uvicorn ASGI → FastAPI router dispatch
3. CORS middleware → origin check
4. Auth middleware → JWT decode, role extraction, token blocklist check (Redis)
5. Rate limit middleware → sliding window counter (Redis INCR + EXPIRE)
6. Audit middleware → log request metadata to AuditLog queue
7. Router → Pydantic v2 input validation
8. Depends() injection → DB session, current user, service instances
9. Service layer → business logic, domain model mutation
10. SQLAlchemy async → SQL via asyncpg connection pool
11. PostgreSQL → query execution, constraint enforcement
12. Response → Pydantic v2 output schema serialization
13. Audit middleware → async write AuditLog (before/after JSONB)
14. Client ← JSON response
```

## Entity Count

11 domain entities: User, Guardian, Student, Teacher, Room, Session, Invoice, Payment, Attendance, Notification, AuditLog.

## Functional Scope

- 15 Functional Requirements (FR-01 through FR-15)
- 20 Use Cases (UC-01 through UC-20)
- GPS geofencing: 150m radius for attendance
- Billing cycle: invoices generated 5th, reminders 11th
- Roles: admin, teacher, guardian
