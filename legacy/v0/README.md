# Equalizer-Stake

[![CI](https://github.com/sedam-or/Equalizer-Stake/actions/workflows/ci.yml/badge.svg)](https://github.com/sedam-or/Equalizer-Stake/actions)
[![Go Report](https://goreportcard.com/badge/github.com/sedam-or/Equalizer-Stake)](https://goreportcard.com/report/github.com/sedam-or/Equalizer-Stake)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

An open-source, deterministic course reservation and studio allocation engine built for zero double-bookings and absolute concurrency control.

---

## Why Equalizer-Stake

Most booking tools break under pressure. Equalizer-Stake locks tight.

- **Deterministic Core** — Time slots computed using exact spatial-temporal interval logic. Same inputs produce the same schedule, every time.
- **Atomic Race Safety** — PostgreSQL exclusion constraints + Redis distributed locks prevent race conditions. 50 concurrent requests to the same slot? Exactly 1 succeeds, 49 fail gracefully with 409 Conflict.
- **Contract-First Design** — Strict OpenAPI definitions ensure seamless integrations across web, mobile, or headless kiosk interfaces.

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Backend | Go 1.23 / Chi router / Clean Architecture |
| Database | PostgreSQL 16 with btree_gist + GIST exclusion constraints |
| Cache & Locks | Redis 7 (distributed locks, rate limiting) |
| Auth | JWT HS256, RBAC (Super Admin, Resource Manager, Instructor, Member) |
| Container | Docker Compose (App + PostgreSQL + Redis + Worker) |
| Frontend | Minimalist dashboard (TailwindCSS) |

---

## Core Modules

1. **Resource Management** — Studio rooms, equipment, dynamic operating hours, buffer time enforcement
2. **Course & Instructor Scheduling** — Multi-week modules, availability grid, anti-overlap logic
3. **Reservation Engine** — DRAFT → PENDING_PAYMENT → CONFIRMED → COMPLETED → CANCELLED → EXPIRED
4. **Identity & Access Control** — Stateless JWT with fine-grained RBAC policies
5. **Analytics & Reconciliation** — Utilization rates, peak hours heatmaps, booking pressure indicators

---

## Quick Start

```bash
# Clone
git clone https://github.com/sedam-or/Equalizer-Stake.git
cd Equalizer-Stake

# Setup
make setup
make up

# Migrate
make migrate

# Test
make test

# Open
open http://localhost:8080
```

Requires Docker and Make.

---

## Project Structure

```
Equalizer-Stake/
├── cmd/
│   ├── server/main.go          # HTTP server entry
│   └── worker/main.go          # Background worker entry
├── internal/
│   ├── config/                  # App configuration
│   ├── domain/                  # Business logic (resource, reservation, identity, analytics)
│   ├── infrastructure/          # PostgreSQL, Redis, JWT implementations
│   └── api/                     # HTTP handlers, middleware, router, DTOs
├── migrations/                  # SQL schema with GIST exclusion constraints
├── tests/                       # Unit, integration, and concurrency tests
├── Dockerfile                   # Multi-stage Go build
├── docker-compose.yml           # App + PostgreSQL + Redis + Worker
├── Makefile                     # setup, up, migrate, test, build
└── LICENSE                      # MIT
```

---

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/availability` | Get open/booked/buffered slots for a resource |
| POST | `/api/v1/reservations` | Atomic slot reservation with distributed lock |
| POST | `/api/v1/reservations/{id}/confirm` | Transition to CONFIRMED |
| POST | `/api/v1/reservations/{id}/cancel` | Release slot, compute refund |
| GET | `/api/v1/analytics/utilization` | Usage density metrics |

---

## Testing

```bash
make test              # All tests (unit + integration + concurrency)
make test-unit         # Slot math, state machine, buffer time
make test-integration  # Full reservation lifecycle via HTTP
make test-concurrency  # 50 concurrent requests → exactly 1 succeeds
```

---

## License

[MIT](LICENSE) — © 2026 Equalizer-Stake Contributors
