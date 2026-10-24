# System Architecture — Equalizer-Stake

Dokumen ini adalah index untuk seluruh arsitektur sistem Equalizer-Stake.

---

## Architecture Documents

| # | File | Deskripsi |
|---|------|-----------|
| 0 | [00-system-overview.md](00-system-overview.md) | High-level architecture, layered design, tech stack |
| 1 | [01-backend.md](01-backend.md) | FastAPI backend: structure, DI, middleware, services |
| 2 | [02-frontend.md](02-frontend.md) | Next.js PWA: App Router, components, offline, face-api.js |
| 3 | [03-database.md](03-database.md) | PostgreSQL: 11 tables, indexing, soft delete, migrations |
| 4 | [04-security.md](04-security.md) | Security: JWT, AES-256-GCM, OWASP, UU PDP |
| 5 | [05-integrations.md](05-integrations.md) | Integrations: WhatsApp, SMTP, face detection |
| 6 | [06-deployment.md](06-deployment.md) | Deployment: Docker, cloud, monitoring, backup |

## Architecture Decision Records (ADRs)

| # | Decision | Status |
|---|----------|--------|
| [001](adr/001-choice-fastapi-over-django.md) | FastAPI over Django | Accepted |
| [002](adr/002-choice-nextjs-pwa-over-native.md) | PWA over Native App | Accepted |
| [003](adr/003-soft-delete-over-cascade.md) | Soft Delete for PII | Accepted |
| [004](adr/004-client-side-face-detection.md) | Client-side Face Detection | Accepted |
| [005](adr/005-unofficial-whatsapp-with-fallback.md) | WhatsApp Unofficial + Email Fallback | Accepted |
| [006](adr/006-owner-superadmin.md) | Owner as Superadmin | Accepted |

## SRS Traceability

Architecture maps directly to SRS modules in `../srs/`:

| SRS Module | Architecture Doc |
|-----------|-----------------|
| srs/01-authentication.md | architecture/04-security.md, 01-backend.md |
| srs/02-scheduling.md | architecture/01-backend.md, 03-database.md |
| srs/03-billing.md | architecture/01-backend.md, 05-integrations.md |
| srs/04-attendance.md | architecture/02-frontend.md, 05-integrations.md |
| srs/05-notifications.md | architecture/05-integrations.md |
| srs/06-reporting.md | architecture/01-backend.md, 02-frontend.md |
| srs/07-audit-governance.md | architecture/03-database.md, 04-security.md |
| srs/08-studio-reservation.md | architecture/01-backend.md, 03-database.md |
| srs/99-nfr.md | architecture/06-deployment.md |
| srs/99-entities.md | architecture/03-database.md |

## Key Architecture Principles

1. **Layered Architecture**: Presentation → API → Service → Domain → Data Access
2. **Dependency Inversion**: Services depend on abstractions, not implementations
3. **Service Layer Pattern**: Business logic in services, not routers or models
4. **Immutable Audit**: Every critical change logged before confirmation
5. **Security by Default**: PII encrypted, JWT validated, rate limited
6. **Graceful Degradation**: Circuit breaker, fallback channels, offline PWA
