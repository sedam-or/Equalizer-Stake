# TASKS.md — Equalizer-Stake: Generic Reservation Engine

> **Audit-Based Task Breakdown**: All tasks derived from comprehensive audit of existing
> codebase against technical requirements. Score: 1.3/10 — full rebuild required.
> **Architecture Decision**: Pending (Go Clean Architecture or Laravel Modular Monolith)
> **Landing Page**: `index.html` (GitHub Pages, Kinevo-style)

---

## Phase 0: Foundation Cleanup & Institutional Detox

> **Goal**: Strip ALL references to Equalizer-Stake, and any
> commercial/institutional entity. Achieve zero institution branding.

### T-001: Strip Backend Institution References
- **Object**: `backend/` directory
- **Action**: Global find-replace across all `.py` files:
 - `"Equalizer-Stake"` → `"Equalizer-Stake"` or generic placeholder
 - `"Equalizer-Stake"` → `"Equalizer-Stake"`
 - `"EQS"` → `"EQS"`
 - `"equalizer-stake.id"` → `"noreply@equalizer-stake.dev"`
 - `"equalizer-stake.local"` → `"noreply@equalizer-stake.dev"`
 - `""` → remove or genericize
 - `""` → remove
 - `""` → remove
 - GPS coordinates `(-7.737428, 110.429669)` → remove or make configurable
 - `"EQS"` → `"EQS"`
 - `"guardian"` → `"guardian"` (or keep Indonesian but make generic)
 - `"instructor"` → `"instructor"` (or keep Indonesian but make generic)
 - `"student"` → `"student"` (or keep Indonesian but make generic)
 - `"course"` → `"course"` (or keep Indonesian but make generic)
- **Files affected**: `backend/app/main.py`, `backend/app/services/email.py`, `backend/app/services/whatsapp.py`, `backend/app/services/scheduling.py`, `backend/app/services/reports.py`, `backend/app/services/pricing.py`, `backend/app/templates/emails.py`, `backend/pyproject.toml`, `backend/.env.example`
- **Verify**: `grep -rn "EQS\|bmd\|\|\|guardian\|instructor\|student\|course" backend/` returns 0 matches

### T-002: Strip Frontend Institution References
- **Object**: `frontend/` directory
- **Action**: Global find-replace across all `.tsx`, `.ts`, `.json` files:
 - `"Equalizer-Stake"` → `"Equalizer-Stake"` or generic
 - `"EQS"` → `"EQS"`
 - `"Equalizer-Stake"` → `"Equalizer-Stake"`
 - `""` → remove or genericize
 - `""` → remove
 - `"Jl. No. XX"` → remove or genericize
 - `"XXXXXXXXXXXX"` → placeholder or remove
 - `"6281234567890"` → placeholder or remove
 - `"BCA 1234567890"` → placeholder or remove
 - `"a.n. Equalizer-Stake"` → remove
 - `"EQS coordinates"` → remove
 - `"Kelas Privat music Premium"` → generic
 - `"150+ pengguna "` → generic
 - `"Equalizer-Stake -- Sistem Penjadwalan music course"` → `"Equalizer-Stake -- Deterministic Reservation Engine"`
 - `"Sistem Penjadwalan Course & Reservasi Studio Equalizer-Stake "` → `"Deterministic Course & Studio Reservation Engine"`
- **Files affected**: All files in `src/app/`, `src/components/landing/`, `src/lib/`, `public/manifest.json`
- **Verify**: `grep -rn "EQS\|bmd\|\|\|XXXXXXXXXXXX\|BCA\|guardian" frontend/` returns 0 matches

### T-003: Strip Documentation References
- **Object**: Root-level `.md` files, `docs/`, `srs/`, `architecture/`, `design/`, `tasks/`
- **Action**: Update or remove institution references in:
 - `README.md` — rewrite as generic open-source project
 - `PRODUCT.md` — rewrite for generic reservation engine
 - `PROJECT_RULES.md` — keep rules, remove EQS-specific context
 - `SOUL.md` — rewrite agent identity
 - `ROLES.md` — rewrite roles for generic system
 - `AGENTS.md` — rewrite for generic system
 - `CONTRIBUTING.md` — keep, remove EQS workflow refs
 - `DESIGN.md` — rewrite for generic system
 - `srs/` — rewrite all SRS docs for generic reservation engine
 - `architecture/` — rewrite for chosen tech stack
 - `design/` — rewrite or remove EQS-specific design
 - `docs/` — rewrite or remove EQS-specific docs
- **Verify**: `grep -rn "EQS\|bmd\|\|" *.md srs/ architecture/ design/ docs/` returns 0 matches

### T-004: Remove EQS-Specific Assets
- **Object**: Root-level files, `public/images/`, `backups/`
- **Action**: Delete files that are EQS-specific:
 - `logo.jpg` (EQS logo)
 - `screen.png` (EQS screenshot)
 - `backups/` directory (EQS database backups)
 - `frontend/public/images/studio-*.jpg` (EQS studio photos)
 - `frontend/public/logo.jpg` (EQS logo)
 - `frontend/src/image/*` (EQS screenshots/generated images)
 - `hermes-verify-sidebar.sh` (empty)
 - `tmp-verify.sh.bak` (empty)
 - All root-level React Bits `.md` files (BlurText, BorderGlow, etc.) — move to `docs/design/` or delete
- **Verify**: No EQS-specific images or empty files remain

### T-005: Clean Up Component Documentation Files
- **Object**: Root-level `.md` files (BlurText_component.md, BorderGlow_compone.md, etc.)
- **Action**: Either:
 - Move to `docs/design/components/` for reference, OR
 - Delete entirely (they're React Bits integration docs, not project code)
- **Verify**: Root directory contains only project-critical files

---

## Phase 1: Project Infrastructure & Skeleton

> **Goal**: Set up the project foundation — Go module, Docker, Makefile, LICENSE,
> .env, .gitignore, project structure.

### T-010: Initialize Go Module
- **Object**: `go.mod`, `go.sum`
- **Action**:
 - `go mod init github.com/sedam-or/Equalizer-Stake`
 - Add dependencies:
 - `github.com/go-chi/chi/v5` (HTTP router)
 - `github.com/jackc/pgx/v5` (PostgreSQL driver)
 - `github.com/redis/go-redis/v9` (Redis client)
 - `github.com/golang-jwt/jwt/v5` (JWT)
 - `github.com/google/uuid` (UUID)
 - `github.com/joho/godotenv` (.env loading)
 - `github.com/kelseyhightower/envconfig` (config)
 - `go.uber.org/zap` (structured logging)
 - `github.com/rs/cors` (CORS middleware)
 - `github.com/swaggo/swag` + `github.com/swaggo/http-swagger` (OpenAPI)
 - `github.com/stretchr/testify` (test assertions)
 - Run `go mod tidy`
- **Verify**: `go build ./...` compiles successfully

### T-011: Create Project Directory Structure
- **Object**: Project root
- **Action**: Create the following structure:
 ```
 Equalizer-Stake/
 ├── cmd/
 │ ├── server/main.go # HTTP server entry
 │ └── worker/main.go # Background worker entry
 ├── internal/
 │ ├── config/config.go # App configuration
 │ ├── domain/
 │ │ ├── resource/ # Resource module
 │ │ │ ├── model.go # Resource, Schedule models
 │ │ │ ├── repository.go # Repository interface
 │ │ │ └── service.go # Business logic
 │ │ ├── reservation/ # Reservation module
 │ │ │ ├── model.go # Reservation, Status models
 │ │ │ ├── repository.go # Repository interface
 │ │ │ ├── service.go # Business logic
 │ │ │ └── state.go # State machine
 │ │ ├── identity/ # Identity module
 │ │ │ ├── model.go # User, Role models
 │ │ │ ├── repository.go # Repository interface
 │ │ │ └── service.go # Auth + RBAC logic
 │ │ └── analytics/ # Analytics module
 │ │ ├── repository.go # Repository interface
 │ │ └── service.go # Business logic
 │ ├── infrastructure/
 │ │ ├── postgres/
 │ │ │ ├── db.go # Connection pool
 │ │ │ ├── resource_repo.go
 │ │ │ ├── reservation_repo.go
 │ │ │ ├── identity_repo.go
 │ │ │ └── analytics_repo.go
 │ │ ├── redis/
 │ │ │ └── lock.go # Distributed lock
 │ │ └── jwt/
 │ │ └── jwt.go # JWT service
 │ └── api/
 │ ├── router.go # Route registration
 │ ├── middleware/
 │ │ ├── auth.go # JWT middleware
 │ │ └── ratelimit.go # Rate limiting
 │ ├── handler/
 │ │ ├── availability.go # GET /availability
 │ │ ├── reservation.go # POST /reservations
 │ │ ├── resource.go # Resource CRUD
 │ │ ├── auth.go # Login/Register
 │ │ └── analytics.go # Analytics endpoints
 │ └── dto/
 │ └── requests.go # Request/response DTOs
 ├── migrations/
 │ ├── 001_create_extensions.sql
 │ ├── 002_create_resources.sql
 │ ├── 003_create_schedules.sql
 │ ├── 004_create_reservations.sql
 │ ├── 005_create_users.sql
 │ └── 006_create_analytics.sql
 ├── tests/
 │ ├── unit/
 │ ├── integration/
 │ └── concurrency/
 ├── Dockerfile
 ├── docker-compose.yml
 ├── Makefile
 ├── .env.example
 ├── .gitignore
 ├── LICENSE
 ├── README.md
 └── index.html
 ```
- **Verify**: Directory structure exists with placeholder files

### T-012: Create Makefile
- **Object**: `Makefile`
- **Action**: Implement targets:
 ```makefile
 setup: # Install dependencies, create .env from .env.example
 up: # docker-compose up -d
 down: # docker-compose down
 migrate: # Run SQL migrations against PostgreSQL
 test: # Run all tests (unit + integration + concurrency)
 seed: # Seed demo data (resources, schedules, test users)
 lint: # Run golangci-lint
 build: # go build -o bin/server ./cmd/server
 clean: # Remove build artifacts
 logs: # docker-compose logs -f
 ```
- **Verify**: `make setup && make up && make migrate && make test` runs without errors

### T-013: Create Dockerfile
- **Object**: `Dockerfile`
- **Action**: Multi-stage build:
 - Stage 1 (builder): `golang:1.23-alpine`, build binary
 - Stage 2 (runtime): `alpine:3.19`, copy binary, expose port 8080
 - Include health check endpoint
- **Verify**: `docker build -t equalizer-stake .` succeeds

### T-014: Create docker-compose.yml
- **Object**: `docker-compose.yml`
- **Action**: Services:
 - `app`: Build from Dockerfile, port 8080, depends on postgres + redis
 - `postgres`: `postgres:16-alpine`, port 5432, volume for data persistence, init script with `btree_gist` extension
 - `redis`: `redis:7-alpine`, port 6379
 - `worker`: Same image as app, runs `cmd/worker/main.go`
 - Volumes: `postgres_data`, `redis_data`
 - Network: `equalizer-net`
- **Verify**: `docker-compose up -d` starts all services

### T-015: Create .env.example
- **Object**: `.env.example`
- **Action**: Comprehensive defaults:
 ```
 APP_PORT=8080
 APP_ENV=development
 APP_DEBUG=true

 POSTGRES_HOST=postgres
 POSTGRES_PORT=5432
 POSTGRES_DB=equalizer_stake
 POSTGRES_USER=eqs
 POSTGRES_PASSWORD=secret

 REDIS_HOST=redis
 REDIS_PORT=6379
 REDIS_PASSWORD=

 JWT_SECRET=change-me-in-production
 JWT_ALGORITHM=HS256
 JWT_EXPIRY=15m
 JWT_REFRESH_EXPIRY=168h

 RESERVATION_TTL_MINUTES=15
 BUFFER_TIME_MINUTES=15
 ```
- **Verify**: `cp .env.example .env && go run ./cmd/server` starts successfully

### T-016: Create LICENSE (MIT)
- **Object**: `LICENSE`
- **Action**: Standard MIT License text with year 2026 and "Equalizer-Stake Contributors"
- **Verify**: File exists with correct text

### T-017: Rewrite .gitignore
- **Object**: `.gitignore`
- **Action**: Go-specific ignores:
 ```
 /bin/
 /vendor/
 *.exe
 *.test
 *.out
 .env
 *.sql.gz
 ```
- **Verify**: `go build` artifacts are ignored

### T-018: Rewrite README.md
- **Object**: `README.md`
- **Action**: Generic open-source README:
 - Project name, badges (CI, coverage, Go version, license)
 - One-liner description
 - Features list
 - Quick Start (make setup → make up → make test)
 - Architecture overview
 - API documentation link
 - Contributing guide link
 - License
- **Verify**: No institution references, clean markdown

---

## Phase 2: Domain Models & Database Schema

> **Goal**: Create the database schema with proper types, constraints, indexes,
> and GIST exclusion constraints for overlap prevention.

### T-020: Create Migration 001 — Extensions
- **Object**: `migrations/001_create_extensions.sql`
- **Action**: SQL:
 ```sql
 CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
 CREATE EXTENSION IF NOT EXISTS "btree_gist";
 ```
- **Verify**: Extensions created successfully

### T-021: Create Migration 002 — Resources Table
- **Object**: `migrations/002_create_resources.sql`
- **Action**: SQL:
 ```sql
 CREATE TYPE resource_type AS ENUM ('STUDIO', 'EQUIPMENT');

 CREATE TABLE resources (
 id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
 name VARCHAR(255) NOT NULL,
 slug VARCHAR(255) NOT NULL UNIQUE,
 type resource_type NOT NULL,
 base_price_per_hour NUMERIC(12,2) NOT NULL CHECK (base_price_per_hour >= 0),
 is_active BOOLEAN NOT NULL DEFAULT true,
 metadata JSONB DEFAULT '{}',
 created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
 updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
 );

 CREATE INDEX idx_resources_slug ON resources(slug);
 CREATE INDEX idx_resources_type ON resources(type);
 CREATE INDEX idx_resources_active ON resources(is_active) WHERE is_active = true;
 ```
- **Verify**: Table created with correct columns and constraints

### T-022: Create Migration 003 — Schedules Table
- **Object**: `migrations/003_create_schedules.sql`
- **Action**: SQL:
 ```sql
 CREATE TABLE schedules (
 id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
 resource_id UUID NOT NULL REFERENCES resources(id) ON DELETE RESTRICT,
 instructor_id UUID, -- nullable, FK to users added after users table
 day_of_week SMALLINT NOT NULL CHECK (day_of_week BETWEEN 0 AND 6),
 start_time TIME NOT NULL,
 end_time TIME NOT NULL CHECK (end_time > start_time),
 is_recurring BOOLEAN NOT NULL DEFAULT true,
 created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
 updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
 );

 CREATE INDEX idx_schedules_resource ON schedules(resource_id);
 CREATE INDEX idx_schedules_day ON schedules(day_of_week);
 CREATE INDEX idx_schedules_instructor ON schedules(instructor_id) WHERE instructor_id IS NOT NULL;
 CREATE UNIQUE INDEX idx_schedules_unique_slot ON schedules(resource_id, day_of_week, start_time, end_time);
 ```
- **Verify**: Table created with unique slot constraint

### T-023: Create Migration 004 — Reservations Table (CRITICAL)
- **Object**: `migrations/004_create_reservations.sql`
- **Action**: SQL:
 ```sql
 CREATE TYPE reservation_status AS ENUM (
 'DRAFT',
 'PENDING_PAYMENT',
 'CONFIRMED',
 'COMPLETED',
 'CANCELLED',
 'EXPIRED'
 );

 CREATE TABLE reservations (
 id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
 reference_code VARCHAR(20) NOT NULL UNIQUE,
 resource_id UUID NOT NULL REFERENCES resources(id) ON DELETE RESTRICT,
 user_id UUID NOT NULL, -- FK to users added after users table
 start_time TIMESTAMPTZ NOT NULL,
 end_time TIMESTAMPTZ NOT NULL,
 status reservation_status NOT NULL DEFAULT 'DRAFT',
 total_amount NUMERIC(12,2) NOT NULL CHECK (total_amount >= 0),
 created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
 updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
 CONSTRAINT chk_time_order CHECK (end_time > start_time)
 );

 -- GIST index for sub-millisecond overlap validation
 CREATE INDEX idx_reservations_overlap ON reservations
 USING GiST (resource_id, tstzrange(start_time, end_time));

 -- CRITICAL: Exclusion constraint prevents double-bookings at DB level
 -- Only active reservations (not CANCELLED/EXPIRED) are checked
 ALTER TABLE reservations ADD CONSTRAINT excl_no_overlap
 EXCLUDE USING GiST (
 resource_id WITH =,
 tstzrange(start_time, end_time) WITH &&
 ) WHERE (status NOT IN ('CANCELLED', 'EXPIRED'));

 CREATE INDEX idx_reservations_user ON reservations(user_id);
 CREATE INDEX idx_reservations_status ON reservations(status);
 CREATE INDEX idx_reservations_reference ON reservations(reference_code);
 ```
- **Verify**:
 - `EXPLAIN ANALYZE` on overlap query uses GiST index
 - Inserting overlapping reservation for same resource throws exclusion violation
 - CANCELLED/EXPIRED reservations don't trigger exclusion

### T-024: Create Migration 005 — Users Table
- **Object**: `migrations/005_create_users.sql`
- **Action**: SQL:
 ```sql
 CREATE TYPE user_role AS ENUM (
 'SUPER_ADMIN',
 'RESOURCE_MANAGER',
 'INSTRUCTOR',
 'MEMBER'
 );

 CREATE TABLE users (
 id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
 email VARCHAR(255) NOT NULL UNIQUE,
 password_hash VARCHAR(255) NOT NULL,
 name VARCHAR(255) NOT NULL,
 role user_role NOT NULL DEFAULT 'MEMBER',
 is_active BOOLEAN NOT NULL DEFAULT true,
 created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
 updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
 );

 CREATE INDEX idx_users_email ON users(email);
 CREATE INDEX idx_users_role ON users(role);

 -- Add FK constraints to schedules and reservations
 ALTER TABLE schedules ADD CONSTRAINT fk_schedules_instructor
 FOREIGN KEY (instructor_id) REFERENCES users(id) ON DELETE SET NULL;

 ALTER TABLE reservations ADD CONSTRAINT fk_reservations_user
 FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE RESTRICT;
 ```
- **Verify**: Users table created, FK constraints valid

### T-025: Create Migration 006 — Analytics Views
- **Object**: `migrations/006_create_analytics.sql`
- **Action**: SQL:
 ```sql
 -- Utilization rate per resource per day
 CREATE VIEW v_resource_utilization AS
 SELECT
 r.id AS resource_id,
 r.name AS resource_name,
 r.type AS resource_type,
 DATE(res.start_time) AS booking_date,
 COUNT(res.id) AS total_bookings,
 SUM(EXTRACT(EPOCH FROM (res.end_time - res.start_time)) / 3600) AS hours_booked,
 r.base_price_per_hour,
 SUM(res.total_amount) AS revenue
 FROM resources r
 LEFT JOIN reservations res ON r.id = res.resource_id
 AND res.status IN ('CONFIRMED', 'COMPLETED')
 GROUP BY r.id, r.name, r.type, DATE(res.start_time), r.base_price_per_hour;

 -- Peak hours heatmap data
 CREATE VIEW v_peak_hours AS
 SELECT
 EXTRACT(HOUR FROM res.start_time) AS hour_of_day,
 EXTRACT(DOW FROM res.start_time) AS day_of_week,
 COUNT(*) AS booking_count,
 COUNT(DISTINCT res.resource_id) AS resources_used
 FROM reservations res
 WHERE res.status IN ('CONFIRMED', 'COMPLETED')
 GROUP BY EXTRACT(HOUR FROM res.start_time), EXTRACT(DOW FROM res.start_time);

 -- Booking pressure (requests vs confirmations per slot)
 CREATE VIEW v_booking_pressure AS
 SELECT
 res.resource_id,
 r.name AS resource_name,
 DATE(res.start_time) AS booking_date,
 EXTRACT(HOUR FROM res.start_time) AS hour_of_day,
 COUNT(*) FILTER (WHERE res.status = 'CONFIRMED') AS confirmed,
 COUNT(*) FILTER (WHERE res.status = 'PENDING_PAYMENT') AS pending,
 COUNT(*) FILTER (WHERE res.status = 'CANCELLED') AS cancelled,
 COUNT(*) FILTER (WHERE res.status = 'EXPIRED') AS expired,
 COUNT(*) AS total_attempts
 FROM reservations res
 JOIN resources r ON res.resource_id = r.id
 GROUP BY res.resource_id, r.name, DATE(res.start_time), EXTRACT(HOUR FROM res.start_time);
 ```
- **Verify**: Views return correct aggregation

### T-026: Implement Go Domain Models
- **Object**: `internal/domain/resource/model.go`, `internal/domain/reservation/model.go`, `internal/domain/identity/model.go`
- **Action**: Go structs matching DB schema:
 - `Resource` struct with JSON tags, validation
 - `Schedule` struct
 - `Reservation` struct with Status enum
 - `User` struct with Role enum
 - `ReservationStatus` type with constants (DRAFT, PENDING_PAYMENT, CONFIRMED, COMPLETED, CANCELLED, EXPIRED)
 - `UserRole` type with constants (SUPER_ADMIN, RESOURCE_MANAGER, INSTRUCTOR, MEMBER)
 - `ResourceType` type with constants (STUDIO, EQUIPMENT)
- **Verify**: `go build ./internal/domain/...` compiles

---

## Phase 3: Reservation Engine (The Deterministic Core)

> **Goal**: Implement the core reservation logic — slot math, overlap validation,
> state machine, buffer time, distributed locking, and automated cleanup.

### T-030: Implement Slot Math Logic
- **Object**: `internal/domain/reservation/service.go`
- **Action**: Functions:
 - `CalculateSlotDuration(start, end time.Time) time.Duration` — returns duration
 - `IsSlotAvailable(ctx, resourceID, start, end) (bool, error)` — checks DB for overlaps using exclusion constraint
 - `GetAvailableSlots(ctx, resourceID, date, duration) ([]TimeSlot, error)` — returns all open slots for a date
 - `HasOverlap(startA, endA, startB, endB time.Time) bool` — pure function: `(startA < endB) && (endA > startB)`
 - `AddBuffer(slot TimeSlot, bufferMinutes int) TimeSlot` — adds buffer before/after
- **Verify**: Unit tests pass for all slot math functions

### T-031: Implement Reservation State Machine
- **Object**: `internal/domain/reservation/state.go`
- **Action**: State machine with strict transitions:
 ```
 DRAFT → PENDING_PAYMENT
 PENDING_PAYMENT → CONFIRMED
 PENDING_PAYMENT → EXPIRED (TTL timeout)
 PENDING_PAYMENT → CANCELLED
 CONFIRMED → COMPLETED
 CONFIRMED → CANCELLED
 DRAFT → CANCELLED
 ```
 - `CanTransition(from, to ReservationStatus) bool` — checks valid transitions
 - `Transition(reservation, newStatus) (Reservation, error)` — validates and applies transition
 - Invalid transitions return `ErrInvalidTransition`
- **Verify**: Unit tests for all valid and invalid transitions

### T-032: Implement Buffer Time Enforcement
- **Object**: `internal/domain/reservation/service.go`
- **Action**:
 - Configurable buffer time (default 15 minutes)
 - When checking availability, expand requested slot by buffer on both sides
 - `ValidateWithBuffer(resourceID, start, end, bufferMinutes) (bool, error)` — checks expanded slot
 - Ensure buffer doesn't conflict with existing bookings
- **Verify**: Unit tests: booking A ends 14:00, booking B starts 14:10 with 15min buffer → conflict

### T-033: Implement Distributed Lock (Redis)
- **Object**: `internal/infrastructure/redis/lock.go`
- **Action**:
 - `AcquireLock(ctx, resourceID string, ttl time.Duration) (Lock, error)` — Redis SET NX EX
 - `ReleaseLock(ctx, lock Lock) error` — Redis DELETE with Lua script (compare-and-delete)
 - `WithLock(ctx, resourceID, fn) error` — acquire → execute → release pattern
 - Lock key format: `lock:resource:{resourceID}`
 - TTL: configurable (default 30 seconds)
 - Error handling: `ErrLockNotAcquired` if concurrent request
- **Verify**: Unit test: two goroutines try to acquire same lock → only one succeeds

### T-034: Implement Reservation Creation with Atomic Slot Reservation
- **Object**: `internal/domain/reservation/service.go`
- **Action**: `CreateReservation(ctx, req CreateReservationRequest) (Reservation, error)`:
 1. Validate input (resource exists, times valid, duration within limits)
 2. Acquire Redis lock for resource
 3. Check overlap via DB (INSERT will fail with exclusion constraint if overlap)
 4. Generate reference code (e.g., `EQS-20260828-A1B2C`)
 5. Calculate total amount (duration × base_price_per_hour)
 6. Insert reservation with status DRAFT
 7. Release lock
 8. Return reservation
- **Verify**: Integration test: create reservation, verify in DB

### T-035: Implement Reservation Confirm
- **Object**: `internal/domain/reservation/service.go`
- **Action**: `ConfirmReservation(ctx, reservationID, userID) (Reservation, error)`:
 1. Fetch reservation (must be PENDING_PAYMENT status)
 2. Verify user owns reservation
 3. Transition to CONFIRMED
 4. Update DB
 5. Return updated reservation
- **Verify**: Integration test: create → confirm flow

### T-036: Implement Reservation Cancel
- **Object**: `internal/domain/reservation/service.go`
- **Action**: `CancelReservation(ctx, reservationID, userID) (Reservation, error)`:
 1. Fetch reservation (must be DRAFT or PENDING_PAYMENT or CONFIRMED)
 2. Verify user owns reservation or is admin
 3. Transition to CANCELLED
 4. Update DB (exclusion constraint no longer applies to CANCELLED)
 5. Compute refund policy if applicable (e.g., full refund if >24h before start, partial if >2h, none otherwise)
 6. Return updated reservation with refund info
- **Verify**: Integration test: create → cancel → verify slot available again

### T-037: Implement Automated Cleanup Worker
- **Object**: `cmd/worker/main.go`
- **Action**: Background worker that:
 1. Runs every minute
 2. Finds reservations with status = PENDING_PAYMENT AND created_at < NOW() - INTERVAL '15 minutes'
 3. Transitions them to EXPIRED
 4. Logs each expired reservation
 5. Handles errors gracefully (doesn't crash on single failure)
- **Verify**: Integration test: create PENDING_PAYMENT reservation, wait (or mock time), verify expires

### T-038: Implement Availability Endpoint Logic
- **Object**: `internal/domain/resource/service.go`
- **Action**: `GetAvailability(ctx, resourceID, date) (AvailabilityResponse, error)`:
 1. Fetch resource details
 2. Fetch schedules for that day of week
 3. Fetch existing reservations for that date
 4. Fetch buffer time config
 5. Compute available slots by subtracting booked + buffered slots from schedule
 6. Return structured response with open, booked, and buffered slots
- **Verify**: Unit test: schedule 9:00-17:00, booking 10:00-12:00 with 15min buffer → available: 9:00-9:45, 12:15-17:00

---

## Phase 4: API Layer

> **Goal**: Implement RESTful API endpoints matching the required contracts
> with proper DTOs, validation, error handling, and OpenAPI docs.

### T-040: Create Request/Response DTOs
- **Object**: `internal/api/dto/requests.go`
- **Action**: Structs:
 - `CreateReservationRequest` (resource_id, start_time, end_time)
 - `ConfirmReservationRequest` (reservation_id)
 - `CancelReservationRequest` (reservation_id, reason)
 - `AvailabilityRequest` (resource_id, date)
 - `LoginRequest` (email, password)
 - `RegisterRequest` (email, password, name, role)
 - `UtilizationRequest` (start_date, end_date, resource_id)
 - Response structs for each
 - Validation tags (`binding:"required"`)
- **Verify**: Compilation succeeds

### T-041: Implement Auth Handler
- **Object**: `internal/api/handler/auth.go`
- **Action**: Endpoints:
 - `POST /api/v1/auth/register` — create user account
 - `POST /api/v1/auth/login` — return JWT access + refresh tokens
 - `POST /api/v1/auth/refresh` — refresh access token
 - `GET /api/v1/auth/me` — get current user profile
- **Verify**: Integration test: register → login → access protected route

### T-042: Implement Resource Handler
- **Object**: `internal/api/handler/resource.go`
- **Action**: Endpoints:
 - `GET /api/v1/resources` — list all active resources
 - `GET /api/v1/resources/{id}` — get resource details
 - `POST /api/v1/resources` — create resource (RESOURCE_MANAGER+)
 - `PUT /api/v1/resources/{id}` — update resource
 - `DELETE /api/v1/resources/{id}` — soft delete
- **Verify**: Integration test: CRUD lifecycle

### T-043: Implement Availability Handler
- **Object**: `internal/api/handler/availability.go`
- **Action**: Endpoint:
 - `GET /api/v1/availability?resource_id={uuid}&date={YYYY-MM-DD}`
 - Query params: resource_id (required), date (required)
 - Response: `{ resource: {...}, date: "2026-08-28", slots: [{ start, end, status: "OPEN"|"BOOKED"|"BUFFERED" }] }`
 - Error: 400 (invalid params), 404 (resource not found)
- **Verify**: Integration test: setup schedule + booking → query availability → verify slots

### T-044: Implement Reservation Handler
- **Object**: `internal/api/handler/reservation.go`
- **Action**: Endpoints:
 - `POST /api/v1/reservations` — create reservation (atomic, with distributed lock)
 - Request: `{ resource_id, start_time, end_time }`
 - Response: 201 with reservation object
 - Error: 409 Conflict if slot unavailable
 - `POST /api/v1/reservations/{id}/confirm` — confirm reservation
 - Request: `{}` (empty body)
 - Response: 200 with updated reservation
 - Error: 400 (invalid transition), 404 (not found)
 - `POST /api/v1/reservations/{id}/cancel` — cancel reservation
 - Request: `{ reason? }`
 - Response: 200 with updated reservation + refund info
 - Error: 400 (invalid transition), 404 (not found)
 - `GET /api/v1/reservations` — list user's reservations (filtered by role)
 - `GET /api/v1/reservations/{id}` — get reservation details
- **Verify**: Integration test: full lifecycle (create → confirm → complete)

### T-045: Implement Analytics Handler
- **Object**: `internal/api/handler/analytics.go`
- **Action**: Endpoint:
 - `GET /api/v1/analytics/utilization`
 - Query params: start_date, end_date, resource_id (optional)
 - Response: `{ data: [{ resource_id, name, date, utilization_rate, hours_booked, hours_available, revenue }] }`
 - `GET /api/v1/analytics/peak-hours` — heatmap data
 - `GET /api/v1/analytics/booking-pressure` — booking pressure indicators
- **Verify**: Integration test: create bookings → query analytics → verify metrics

### T-046: Implement Router
- **Object**: `internal/api/router.go`
- **Action**: Route registration with middleware:
 ```
 /api/v1/auth/* — public
 /api/v1/resources/* — authenticated
 /api/v1/availability/* — public
 /api/v1/reservations/* — authenticated
 /api/v1/analytics/* — authenticated (RESOURCE_MANAGER+)
 ```
 - CORS middleware
 - JWT middleware (except public routes)
 - Rate limiting middleware
 - Request logging middleware
 - Panic recovery middleware
- **Verify**: All routes registered, middleware chain correct

### T-047: Implement Middleware
- **Object**: `internal/api/middleware/auth.go`, `internal/api/middleware/ratelimit.go`
- **Action**:
 - `AuthMiddleware` — validates JWT, injects user into context
 - `RequireRole(roles ...string)` — checks user role against allowed roles
 - `RateLimitMiddleware` — Redis-backed sliding window (100 req/min per IP)
 - `CORSMiddleware` — configurable origins
 - `LoggingMiddleware` — structured request logging
- **Verify**: Unit test: protected route without token → 401, with valid token → 200

### T-048: Implement OpenAPI/Swagger
- **Object**: `docs/swagger.json`, `docs/swagger.yaml`
- **Action**:
 - Use `swag init` to generate from code annotations
 - OR manually write OpenAPI 3.0 spec
 - Document all endpoints with request/response schemas
 - Include error response schemas (400, 401, 403, 404, 409, 429, 500)
 - Serve at `GET /docs/` via `http-swagger`
- **Verify**: Swagger UI loads at localhost:8080/docs/

---

## Phase 5: Authentication & RBAC

> **Goal**: Implement JWT-based authentication with role-based access control
> matching the required 4 roles.

### T-050: Implement JWT Service
- **Object**: `internal/infrastructure/jwt/jwt.go`
- **Action**:
 - `GenerateAccessToken(user) (string, error)` — HS256, 15min expiry
 - `GenerateRefreshToken(user) (string, error)` — HS256, 7-day expiry
 - `ValidateToken(tokenString) (*Claims, error)` — validate and extract claims
 - Claims: `user_id`, `email`, `role`, `exp`, `iat`, `iss`
 - Secret from config
- **Verify**: Unit test: generate → validate → extract claims

### T-051: Implement Password Hashing
- **Object**: `internal/domain/identity/service.go`
- **Action**:
 - `HashPassword(password) (string, error)` — bcrypt cost 12
 - `CheckPassword(hash, password) bool` — bcrypt compare
- **Verify**: Unit test: hash → check → verify mismatch fails

### T-052: Implement Role-Based Access Control
- **Object**: `internal/api/middleware/auth.go`
- **Action**: Role hierarchy:
 ```
 SUPER_ADMIN → full access
 RESOURCE_MANAGER → manage resources, reservations, analytics
 INSTRUCTOR → view own schedule, manage attendance
 MEMBER → create/view own reservations
 ```
 - `RequireRole(SUPER_ADMIN)` — only super admin
 - `RequireRole(SUPER_ADMIN, RESOURCE_MANAGER)` — admin or resource manager
 - `RequireRole(any authenticated)` — any logged-in user
- **Verify**: Unit test: MEMBER tries to access /admin → 403, SUPER_ADMIN → 200

---

## Phase 6: Resource & Course Management

> **Goal**: Implement CRUD for resources and schedules with proper validation.

### T-060: Implement Resource CRUD Service
- **Object**: `internal/domain/resource/service.go`
- **Action**:
 - `CreateResource(ctx, req) (Resource, error)` — validate input, generate slug
 - `GetResource(ctx, id) (Resource, error)` — fetch by ID
 - `ListResources(ctx, filter) ([]Resource, int, error)` — paginated list
 - `UpdateResource(ctx, id, req) (Resource, error)` — update fields
 - `DeleteResource(ctx, id) error` — soft delete (set is_active = false)
 - Validation: name required, slug unique, base_price > 0
- **Verify**: Integration test: full CRUD lifecycle

### T-061: Implement Schedule CRUD Service
- **Object**: `internal/domain/resource/service.go`
- **Action**:
 - `CreateSchedule(ctx, req) (Schedule, error)` — validate day_of_week, times
 - `GetSchedulesForResource(ctx, resourceID) ([]Schedule, error)` — list schedules
 - `UpdateSchedule(ctx, id, req) (Schedule, error)` — update
 - `DeleteSchedule(ctx, id) error` — delete
 - Validation: end_time > start_time, no overlapping schedules for same resource+day
- **Verify**: Integration test: create schedule → verify availability reflects it

### T-062: Implement Dynamic Operating Hours
- **Object**: `internal/domain/resource/service.go`
- **Action**:
 - Per-resource recurring time blocks (already in schedules table)
 - Blackout dates: add `blackout_dates` JSONB field to resources or separate table
 - Maintenance windows: add `maintenance_windows` table
 - `IsResourceOpen(ctx, resourceID, dateTime) (bool, error)` — checks schedule + blackouts + maintenance
- **Verify**: Unit test: resource with schedule 9-17, blackout on 2026-08-28 → closed

---

## Phase 7: Testing

> **Goal**: Comprehensive test suites — unit, integration, and concurrency —
> achieving high coverage.

### T-070: Unit Tests — Slot Math
- **Object**: `tests/unit/slot_math_test.go`
- **Action**: Test cases:
 - `TestHasOverlap` — various overlap scenarios (partial, full, adjacent, none)
 - `TestCalculateSlotDuration` — 1h, 2h, 30min, 0min
 - `TestAddBuffer` — buffer before, after, both sides
 - `TestIsSlotAvailable` — mock DB, various scenarios
 - `TestGetAvailableSlots` — mock DB, schedule + bookings → available slots
 - Target: 100% coverage of slot math functions
- **Verify**: `go test ./tests/unit/ -v` passes

### T-071: Unit Tests — State Machine
- **Object**: `tests/unit/state_machine_test.go`
- **Action**: Test cases:
 - `TestValidTransitions` — all valid transitions (DRAFT→PENDING, PENDING→CONFIRMED, etc.)
 - `TestInvalidTransitions` — all invalid transitions (CONFIRMED→DRAFT, COMPLETED→CONFIRMED, etc.)
 - `TestTransition` — full transition lifecycle
 - Target: 100% coverage of state machine
- **Verify**: `go test ./tests/unit/ -v` passes

### T-072: Unit Tests — Buffer Time
- **Object**: `tests/unit/buffer_test.go`
- **Action**: Test cases:
 - `TestBufferPreventsAdjacentBookings` — A ends 14:00, B starts 14:10, buffer 15min → conflict
 - `TestBufferAllowsSpaciousBookings` — A ends 14:00, B starts 14:30, buffer 15min → ok
 - `TestBufferEdgeCases` — exact boundary conditions
- **Verify**: `go test ./tests/unit/ -v` passes

### T-073: Unit Tests — Distributed Lock
- **Object**: `tests/unit/lock_test.go`
- **Action**: Test cases:
 - `TestAcquireAndRelease` — acquire lock → release → re-acquire
 - `TestConcurrentAcquire` — 2 goroutines → only 1 gets lock
 - `TestLockExpiry` — lock expires after TTL
 - `TestReleaseNonOwnedLock` — can't release someone else's lock
- **Verify**: `go test ./tests/unit/ -v` passes

### T-074: Integration Tests — Reservation Lifecycle
- **Object**: `tests/integration/reservation_lifecycle_test.go`
- **Action**: Full HTTP lifecycle:
 1. `POST /auth/register` → create user
 2. `POST /auth/login` → get token
 3. `POST /resources` → create studio resource
 4. `POST /schedules` → set operating hours (9:00-17:00)
 5. `GET /availability?resource_id=...&date=2026-08-28` → get open slots
 6. `POST /reservations` → create reservation (DRAFT)
 7. `POST /reservations/{id}/confirm` → confirm (CONFIRMED)
 8. `POST /reservations/{id}/cancel` → cancel (CANCELLED)
 9. `GET /availability` → verify slot is open again
 - Each step: verify HTTP status, response body, DB state
- **Verify**: `go test ./tests/integration/ -v` passes

### T-075: Integration Tests — Availability
- **Object**: `tests/integration/availability_test.go`
- **Action**: Test cases:
 - `TestAvailabilityReturnsOpenSlots` — schedule 9-17, no bookings → 8h available
 - `TestAvailabilityShowsBookedSlots` — schedule 9-17, booking 10-12 → shows booked
 - `TestAvailabilityAppliesBuffer` — booking 10-12, buffer 15min → shows buffer 9:45-12:15
 - `TestAvailabilityRespectsBlackouts` — blackout date → no slots
 - `TestAvailabilityInvalidResource` — 404 response
 - `TestAvailabilityInvalidDate` — 400 response
- **Verify**: `go test ./tests/integration/ -v` passes

### T-076: Concurrency Tests — Double-Booking Prevention
- **Object**: `tests/concurrency/race_test.go`
- **Action**: THE CRITICAL TEST:
 ```
 TestConcurrentReservationCreation:
 1. Create resource "Studio A" with schedule 9:00-17:00
 2. Launch 50 goroutines simultaneously
 3. Each goroutine tries to reserve Studio A for 10:00-11:00
 4. Wait for all goroutines to complete
 5. Count successes (HTTP 201) and failures (HTTP 409 Conflict)
 6. ASSERT: exactly 1 success, exactly 49 failures
 7. Verify DB has exactly 1 reservation for that slot
 ```
 - Use `sync.WaitGroup` + `errgroup` for parallel execution
 - Use real PostgreSQL (not mock) to test exclusion constraint
 - Use Redis for distributed lock testing
 - Measure total execution time (target: <2s for all 50)
- **Verify**: `go test ./tests/concurrency/ -v -race` passes with exactly 1 success

### T-077: Integration Tests — State Machine via HTTP
- **Object**: `tests/integration/state_machine_test.go`
- **Action**: Test all valid and invalid transitions via HTTP:
 - Confirm a DRAFT reservation → 400 (must be PENDING_PAYMENT)
 - Confirm a PENDING_PAYMENT reservation → 200
 - Confirm an already CONFIRMED reservation → 400
 - Cancel a CONFIRMED reservation → 200
 - Cancel a COMPLETED reservation → 400
- **Verify**: All transition tests pass

---

## Phase 8: Repository & Deployment Artifacts

> **Goal**: Complete repository setup for open-source publication.

### T-080: Create CONTRIBUTING.md
- **Object**: `CONTRIBUTING.md`
- **Action**: Generic contributing guide:
 - How to set up dev environment
 - Branch strategy (main/develop/feature/hotfix)
 - Commit convention
 - PR template
 - Code style (gofmt, golangci-lint)
 - Testing requirements
- **Verify**: No institution references

### T-081: Create Architecture Decision Records
- **Object**: `docs/adr/`
- **Action**: ADRs:
 - `0001-modular-monolith.md` — Why modular monolith over microservices
 - `0002-interval-validation.md` — Why PostgreSQL exclusion constraints over application-level checks
 - `0003-postgres-vs-redis-lock.md` — Why PostgreSQL exclusion + Redis lock (defense in depth)
 - `0004-uuid-primary-keys.md` — Why UUIDs over serial IDs
 - `0005-clean-architecture.md` — Why clean architecture with domain layer
- **Verify**: Each ADR has Status, Context, Decision, Consequences

### T-082: Create .env.example (Updated)
- **Object**: `.env.example`
- **Action**: Ensure comprehensive defaults (may already exist from T-015)
- **Verify**: All env vars documented

---

## Phase 9: CI/CD & Landing Page

> **Goal**: GitHub Actions CI pipeline and GitHub Pages landing page.

### T-090: Create GitHub Actions CI
- **Object**: `.github/workflows/ci.yml`
- **Action**: Pipeline:
 ```yaml
 name: CI
 on: [push, pull_request]
 jobs:
 lint:
 runs-on: ubuntu-latest
 steps:
 - uses: actions/checkout@v4
 - uses: actions/setup-go@v5
 with: { go-version: '1.23' }
 - run: golangci-lint run
 test:
 runs-on: ubuntu-latest
 services:
 postgres:
 image: postgres:16-alpine
 env:
 POSTGRES_DB: test
 POSTGRES_USER: test
 POSTGRES_PASSWORD: test
 ports: ['5432:5432']
 redis:
 image: redis:7-alpine
 ports: ['6379:6379']
 steps:
 - uses: actions/checkout@v4
 - uses: actions/setup-go@v5
 with: { go-version: '1.23' }
 - run: go test ./... -v -race -coverprofile=coverage.out
 - run: go tool cover -func=coverage.out
 build:
 runs-on: ubuntu-latest
 needs: [lint, test]
 steps:
 - uses: actions/checkout@v4
 - uses: actions/setup-go@v5
 with: { go-version: '1.23' }
 - run: go build -o bin/server ./cmd/server
 ```
- **Verify**: Push to GitHub → CI passes

### T-091: Create Landing Page
- **Object**: `index.html` (root, for GitHub Pages)
- **Action**: Kinevo-style landing page (from user's provided HTML):
 - Hero section with stats banner
 - Philosophy section (Deterministic Core, Atomic Race Safety, Contract-First Design)
 - Features grid
 - Quickstart section with terminal commands
 - Footer with MIT license link
 - No institution references — pure technical portfolio
- **Verify**: Opens correctly in browser, no broken links

### T-092: Update README.md (Final)
- **Object**: `README.md`
- **Action**: Final version:
 - Project name + badges
 - One-liner: "An open-source, deterministic course reservation and studio allocation engine"
 - Architecture diagram
 - Quick Start
 - API Reference
 - Testing
 - Contributing
 - License (MIT)
- **Verify**: Renders correctly on GitHub

---

## Phase 10: Final Verification & Polish

> **Goal**: Ensure everything works end-to-end and meets quality standards.

### T-100: Run Full Test Suite
- **Object**: All tests
- **Action**:
 - `make test` runs unit + integration + concurrency tests
 - Verify all tests pass
 - Check coverage ≥80%
 - Run with `-race` flag for data race detection
- **Verify**: 100% test pass rate

### T-101: Run Linter
- **Object**: All Go code
- **Action**:
 - `golangci-lint run` — no warnings
 - `gofmt -l .` — no unformatted files
- **Verify**: Clean lint output

### T-102: Verify Docker Build
- **Object**: Docker
- **Action**:
 - `docker-compose build` — succeeds
 - `docker-compose up -d` — all services start
 - `curl localhost:8080/health` — returns 200
 - `docker-compose down` — clean shutdown
- **Verify**: All services healthy

### T-103: Verify Zero Institution References (Final)
- **Object**: Entire repository
- **Action**:
 - `grep -rn "EQS\|bmd\|\|\|guardian\|instructor\|student\|course" --include="*.go" --include="*.html" --include="*.md" --include="*.yml" --include="*.json" --include="*.sql" .` returns 0 matches
- **Verify**: Zero matches across entire codebase

### T-104: Verify API Endpoints
- **Object**: All API endpoints
- **Action**: Manual verification:
 - `GET /api/v1/availability?resource_id=...&date=2026-08-28` → 200 with slots
 - `POST /api/v1/reservations` → 201 with reservation
 - `POST /api/v1/reservations/{id}/confirm` → 200
 - `POST /api/v1/reservations/{id}/cancel` → 200
 - `GET /api/v1/analytics/utilization` → 200 with metrics
 - `GET /docs/` → Swagger UI loads
- **Verify**: All endpoints respond correctly

### T-105: Verify Concurrency Protection
- **Object**: Concurrency test
- **Action**: Run concurrency test one final time:
 - 50 concurrent requests to same slot → exactly 1 succeeds
 - Total time <2s
 - No data races detected
- **Verify**: Test passes consistently (run 3 times)

---

## Summary

| Phase | Tasks | Description |
|-------|-------|-------------|
| **0** | T-001 to T-005 | Strip institution references |
| **1** | T-010 to T-018 | Project infrastructure & skeleton |
| **2** | T-020 to T-026 | Domain models & database schema |
| **3** | T-030 to T-038 | Reservation engine (core) |
| **4** | T-040 to T-048 | API layer |
| **5** | T-050 to T-052 | Authentication & RBAC |
| **6** | T-060 to T-062 | Resource & course management |
| **7** | T-070 to T-077 | Testing |
| **8** | T-080 to T-082 | Repository artifacts |
| **9** | T-090 to T-092 | CI/CD & landing page |
| **10** | T-100 to T-105 | Final verification |
| **TOTAL** | **48 tasks** | **Full rebuild** |

### Dependencies
```
Phase 0 → Phase 1 → Phase 2 → Phase 3 → Phase 4 → Phase 7 → Phase 10
 ↘ Phase 5 → Phase 4
 ↘ Phase 6 → Phase 4
Phase 8 → Phase 9
Phase 0 → Phase 9 (landing page must be institution-free)
```

### Estimated Effort
- Phase 0: 2-3 hours (find-replace across ~200 files)
- Phase 1: 1-2 hours (project setup)
- Phase 2: 2-3 hours (schema + models)
- Phase 3: 4-6 hours (core engine — most complex)
- Phase 4: 3-4 hours (API handlers)
- Phase 5: 1-2 hours (auth)
- Phase 6: 2-3 hours (resource management)
- Phase 7: 4-6 hours (testing — second most complex)
- Phase 8: 1 hour (docs)
- Phase 9: 2 hours (CI + landing page)
- Phase 10: 1-2 hours (verification)
- **Total: ~25-35 hours of focused development**
