# Arsitektur Auth System — Analisis Logto & Desain Equalizer-Stake

> Referensi: [logto-io/logto](https://github.com/logto-io/logto)
> Backend: FastAPI + SQLAlchemy async + Pydantic v2
> Database: PostgreSQL 16 + JSONB
> Frontend: Next.js 15 App Router

---

## 1. Analisis Arsitektur Logto

### 1.1 Tiga API Layer (Management / Experience / Account)

Logto memisahkan API menjadi 3 layer dengan concerns berbeda:

```
┌─────────────────────────────────────────────────────────┐
│ Request Flow │
├─────────────────────────────────────────────────────────┤
│ │
│ HTTP Request │
│ │ │
│ ▼ │
│ ┌──────────────┐ tenant_id dari subdomain/domain │
│ │ Tenant Pool │ → resolve Tenant instance │
│ │ (LRU Cache) │ → tenant.run(ctx, next) │
│ └──────┬───────┘ │
│ │ │
│ ▼ │
│ ┌──────────────────────────────────────────────────┐ │
│ │ apisApp (Koa) │ │
│ │ ┌─────────────┐ ┌──────────────┐ ┌──────────┐ │ │
│ │ │ Experience │ │ Management │ │ Account │ │ │
│ │ │ Router │ │ Router │ │ Router │ │ │
│ │ │ (anonymous) │ │ (JWT+tenant) │ │ (OIDC) │ │ │
│ │ │ │ │ │ │ │ │ │
│ │ │ /sign-in │ │ /api/apps │ │ /my- │ │ │
│ │ │ /interaction│ │ /api/users │ │ account │ │ │
│ │ │ /consent │ │ /api/roles │ │ │ │ │
│ │ └─────────────┘ └──────────────┘ └──────────┘ │ │
│ └──────────────────────────────────────────────────┘ │
│ │
└─────────────────────────────────────────────────────────┘
```

**Experience API** (`/sign-in/*`, `/interaction/*`)
- Public-facing, end-user authentication flows
- Handles sign-in, sign-up, forgot password, consent, MFA
- AnonymousRouter — no JWT auth required (user IS the auth subject)
- Uses session cookies to track interaction state
- Routes: `packages/core/src/routes/experience/`, `routes/interaction/`

**Management API** (`/api/*`)
- Admin-only, for managing tenants, users, apps, roles
- Protected by Management API JWT (resource indicator scoped)
- `koaAuth(tenant.envSet, getManagementApiResourceIndicator(tenant.id))`
- `koaTenantGuard` ensures tenant isolation
- Routes: `packages/core/src/routes/` (all non-experience routes)

**Account API** (`/my-account/*`)
- End-user self-service (change profile, manage sessions, MFA)
- Protected by OIDC auth (`koaOidcAuth`)
- Only available in admin tenant (`initMeApis` checks `tenant.id !== adminTenantId`)
- Routes: `packages/core/src/routes/account/`, `routes-me/init.ts`

### 1.2 Database Schema Design

**Core Pattern: `tenant_id` + JSONB + Single Source of Truth**

```sql
-- Every table has tenant_id as first column
CREATE TABLE users (
 tenant_id VARCHAR(21) NOT NULL REFERENCES tenants(id),
 id VARCHAR(21) NOT NULL,
 username VARCHAR(256) NOT NULL,
 primary_identity JSONB NOT NULL DEFAULT '{}'::jsonb,
 -- ^^^ JSONB for flexible user data (email, phone, social)
 custom_data JSONB NOT NULL DEFAULT '{}'::jsonb,
 -- ^^^ extension point for tenant-specific fields
 created_at TIMESTAMPTZ NOT NULL DEFAULT(now()),
 PRIMARY KEY (tenant_id, id)
);

CREATE INDEX users__id ON users (tenant_id, id);
CREATE UNIQUE INDEX users__username ON users (tenant_id, username);
```

**Key Design Decisions:**

1. **tenant_id di semua tabel** — Setiap query WHERE harus include `tenant_id`. Ini physical isolation per tenant di shared database.

2. **JSONB untuk flexible data** — `custom_data`, `custom_client_metadata`, `oidc_client_metadata` memungkinkan tenant menambah field tanpa schema migration. Struktur di-validasi di application layer (Zod guards).

3. **Single source of truth** — Satu `users` table untuk semua tenant. Tenant isolation via `tenant_id` column, bukan separate databases.

4. **Composite primary keys** — `(tenant_id, id)` ensures ID unik per tenant, global ID unik via tenant prefix.

### 1.3 Multi-Tenancy Architecture

```typescript
// packages/core/src/tenants/TenantPool.ts
class TenantPool {
 private cache = new LRUCache<string, Promise<Tenant>>({
 max: EnvSet.values.tenantPoolSize,
 dispose: (entry) => { /* cleanup: dispose DB pool */ },
 });

 async get(tenantId: string, customDomain?: string): Promise<Tenant> {
 // 1. Check LRU cache
 // 2. If miss → Tenant.create({ id, redisCache, customDomain })
 // 3. Health check before serving
 // 4. Request slot reservation (requestStart/requestEnd)
 // 5. Max 10 retries with fallback
 }
}

// packages/core/src/tenants/Tenant.ts
class Tenant implements TenantContext {
 static async create({ id, redisCache, customDomain }): Promise<Tenant> {
 const tenantDatabaseDsn = await getTenantDatabaseDsn(id);
 const envSet = new EnvSet(id, tenantDatabaseDsn);
 await envSet.load(customDomain);
 return new Tenant(envSet, id, new WellKnownCache(id, redisCache));
 }

 // Per-tenant: provider (OIDC), queries, libraries, sentinel
 readonly queries: Queries; // DB query layer
 readonly libraries: Libraries; // Business logic layer
 readonly wellKnownCache: WellKnownCache; // Redis cache
}
```

**Multi-Tenancy Flow:**

```
Request → app/init.ts → getTenantId(URL) // from subdomain or path
 → tenantPool.get(tenantId) // LRU cache lookup
 → tenant.run(ctx, next) // Koa middleware chain
 → tenant.requestEnd() // Release DB pool slot
```

**Tenant Class Composition:**

```
Tenant
├── EnvSet — env config, DB pool, OIDC config per tenant
├── WellKnownCache — Redis-backed cache for tenant config
├── Queries — DB query methods (slonik SQL tagged templates)
├── Libraries — Business logic (users, apps, sessions, etc.)
├── Provider — OIDC provider instance
└── Sentinel — Rate limiting / abuse detection
```

### 1.4 Monorepo Structure

```
logto/
├── packages/
│ ├── core/ — Backend server (Koa + OIDC)
│ ├── schemas/ — DB tables + TypeScript types (single source of truth)
│ ├── console/ — Admin dashboard (React SPA)
│ ├── experience/ — Sign-in UI (React)
│ ├── account/ — Account center UI
│ ├── connectors/ — Email, SMS, social connectors (15+)
│ ├── phrases/ — i18n strings
│ ├── shared/ — Shared utilities
│ ├── toolkit/ — CLI tools, SDK
│ └── cli/ — CLI interface
├── pnpm-workspace.yaml
└── package.json — workspace scripts
```

**Dependency Hierarchy:**

```
core ← schemas ← shared
 ↓
connectors ← core
 ↓
console ← core (API)
experience ← core (API)
account ← core (API)
```

---

## 2. Desain Equalizer-Stake Auth System

### 2.1 Mapping Logto → Equalizer-Stake

| Logto Concept | Equalizer-Stake Equivalent |
|---|---|
| `tenant_id` (VARCHAR(21)) | `org_id` (UUID, nullable untuk superadmin) |
| `Tenant` class | `OrgConfig` dataclass + Redis cache |
| `TenantPool` (LRU) | `org_cache` dict + TTL (≤50 users, simple) |
| Management API (`/api/*`) | Management API (`/api/*`) — same |
| Account API (`/my-account/*`) | Profile API (`/api/auth/me/*`) |
| Experience API (`/sign-in/*`) | Auth Pages (`/login`, `/register`, `/forgot-password`) |
| `Libraries` class | `ServiceLayer` — business logic |
| `Queries` class | Repository layer — SQLAlchemy queries |
| JSONB `custom_data` | JSONB `metadata` column per table |

### 2.2 API Layer Design

```
┌─────────────────────────────────────────────────────┐
│ FastAPI App │
├─────────────────────────────────────────────────────┤
│ │
│ /api/auth/* → Authentication (public + JWT) │
│ │ /login POST (public) │
│ │ /register POST (public) │
│ │ /forgot-password POST (public) │
│ │ /reset-password POST (public) │
│ │ /me GET (JWT) │
│ │ /me/profile GET/PATCH (JWT) │
│ │ /change-password POST (JWT) │
│ │ │
│ /api/admin/* → Management (admin+owner only) │
│ │ /users CRUD │
│ │ /teachers CRUD │
│ │ /students CRUD │
│ │ /rooms CRUD │
│ │ /settings GET/PATCH │
│ │ │
│ /api/org/* → Organization (org-scoped) │
│ │ /schedule GET │
│ │ /billing GET │
│ │ /attendance GET/POST │
│ │ /notifications GET │
│ │
│ Middleware: org_id injection via JWT sub→org_id │
│ │
└─────────────────────────────────────────────────────┘
```

### 2.3 Database Schema with org_id

```sql
-- Users table: org_id nullable (superadmin has NULL)
CREATE TABLE users (
 id_user UUID PRIMARY KEY DEFAULT gen_random_uuid(),
 org_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
 username VARCHAR(50) UNIQUE NOT NULL,
 email VARCHAR(255) UNIQUE NOT NULL,
 password_hash VARCHAR(255) NOT NULL,
 role user_role NOT NULL, -- 'owner','admin','teacher','guardian'
 is_active BOOLEAN NOT NULL DEFAULT true,
 metadata JSONB NOT NULL DEFAULT '{}', -- extensible profile data
 failed_login_attempts INT NOT NULL DEFAULT 0,
 locked_until TIMESTAMPTZ,
 last_login_at TIMESTAMPTZ,
 created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
 updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
 deleted_at TIMESTAMPTZ
);

-- Composite index for tenant isolation
CREATE INDEX idx_users_org ON users (org_id) WHERE org_id IS NOT NULL;
CREATE UNIQUE INDEX idx_users_username_org ON users (org_id, username)
 WHERE org_id IS NOT NULL;

-- Organizations (multi-tenancy)
CREATE TABLE organizations (
 id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
 name VARCHAR(100) NOT NULL,
 slug VARCHAR(50) UNIQUE NOT NULL,
 settings JSONB NOT NULL DEFAULT '{}',
 -- settings: { studio_gps: {lat, lng, radius}, billing_day: 5, late_fee: 100000 }
 created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- All business tables get org_id
CREATE TABLE sessions (
 id_session UUID PRIMARY KEY DEFAULT gen_random_uuid(),
 org_id UUID NOT NULL REFERENCES organizations(id),
 id_student UUID NOT NULL REFERENCES students(id_student),
 id_teacher UUID NOT NULL REFERENCES teachers(id_teacher),
 id_room UUID NOT NULL REFERENCES rooms(id_room),
 ...
);
```

### 2.4 Service Layer (equivalent to Logto's Libraries + Queries)

```python
# app/services/base.py — Query layer (like Logto's Queries)
class BaseRepository:
 def __init__(self, session: AsyncSession, org_id: UUID | None = None):
 self.session = session
 self.org_id = org_id

 def _org_filter(self, model):
 """Apply org_id filter if set."""
 if self.org_id:
 return model.org_id == self.org_id
 return True # no filter for superadmin


# app/services/user_service.py — Business logic (like Logto's Libraries)
class UserService(BaseRepository):
 async def authenticate(self, username: str, password: str) -> User | None:
 ...

 async def create(self, data: UserCreate) -> User:
 ...

 async def get_profile(self, user_id: UUID) -> ProfileResponse:
 ...


# app/services/org_service.py — Org cache (simplified Logto TenantPool)
class OrgCache:
 """Simple org config cache. ≤50 concurrent users, no LRU needed."""
 def __init__(self, redis: Redis):
 self.redis = redis
 self.ttl = 300 # 5 minutes

 async def get(self, org_id: UUID) -> dict | None:
 cached = await self.redis.get(f"org:{org_id}")
 if cached:
 return json.loads(cached)
 org = await self._load_from_db(org_id)
 if org:
 await self.redis.setex(f"org:{org_id}", self.ttl, json.dumps(org))
 return org
```

### 2.5 Auth Middleware (JWT + org_id injection)

```python
# app/middleware/auth.py
async def get_current_user(
 credentials: HTTPAuthorizationCredentials = Depends(security),
 db: AsyncSession = Depends(get_db),
) -> User:
 payload = decode_access_token(credentials.credentials)
 if not payload:
 raise HTTPException(401, "Token tidak valid")

 user = await get_user_by_id(db, UUID(payload["sub"]))
 if not user or user.deleted_at:
 raise HTTPException(401, "User tidak ditemukan")
 if not user.is_active:
 raise HTTPException(403, "Akun tidak aktif")

 return user

# org_id derived from JWT → user.org_id
async def get_org_id(user: User = Depends(get_current_user)) -> UUID | None:
 return user.org_id # NULL for superadmin
```

### 2.6 Frontend Auth Flow

```
Landing Page
 ├── Hero → /register (GlowButton)
 ├── CTA → /register (GlowButton)
 ├── Pricing → /register (GlowButton per card)
 └── Footer → /login, /register, /forgot-password

Login Page (/login)
 ├── Tab: Masuk → POST /api/auth/login → JWT → /dashboard
 ├── Tab: Daftar → redirect /register
 └── Tab: Lupa Password → redirect /forgot-password

Register Page (/register)
 ├── Form → POST /api/auth/register → /login?success=1
 └── Footer: "Sudah punya akun? Masuk"

Forgot Password (/forgot-password)
 ├── Form → POST /api/auth/forgot-password → success message
 └── Footer: "Kembali ke Login" | "Daftar akun baru"

Dashboard (/dashboard)
 ├── GET /api/auth/me → validate JWT → render role-based dashboard
 └── Redirect to /login if no valid token
```

---

## 3. Perbandingan Arsitektur

| Aspek | Logto | Equalizer-Stake |
|---|---|---|
| **Framework** | Koa + Slonik (raw SQL) | FastAPI + SQLAlchemy async |
| **Schema** | TypeScript types + SQL files | Pydantic v2 + SQLAlchemy models |
| **Multi-tenancy** | `tenant_id` per table, LRU TenantPool | `org_id` per table, Redis cache |
| **API Split** | 3 routers (Management/Experience/Account) | 2 routers (Management/Auth+Profile) |
| **Caching** | WellKnownCache (Redis) per tenant | Redis org config cache |
| **OIDC** | Full OIDC provider (oidc-provider) | JWT HS256 (no OIDC) |
| **Connectors** | 15+ (email, SMS, social, SSO) | WhatsApp + email (SMTP) |
| **Session** | Server-side OIDC session | JWT stateless |
| **User Mgmt** | Management API + Console | Admin pages (Next.js) |
| **DB Pool** | Per-tenant pool (EnvSet) | Shared pool (20 connections) |

---

## 4. Implementation Roadmap

### Phase 1: Add org_id to all tables (current)
- [x] Create `organizations` table
- [x] Add `org_id` column to `users`
- [x] Update all business tables with `org_id`
- [x] Add composite indexes

### Phase 2: Org-scoped queries
- [ ] Create `BaseRepository` with `org_filter()`
- [ ] Update all services to use `org_id` in queries
- [ ] Add org_id injection middleware

### Phase 3: Org config cache
- [ ] Create `OrgCache` class (Redis-backed)
- [ ] Cache org settings (GPS, billing, etc.)
- [ ] Invalidate on settings update

### Phase 4: Auth page improvements
- [x] Forgot password page redesign
- [x] Landing page → auth page links
- [ ] Email verification flow
- [ ] Password reset token flow

### Phase 5: Admin org management
- [ ] Create org (superadmin)
- [ ] Assign users to org
- [ ] Org settings page

---

## 5. Referensi

- **Logto Tenant**: `packages/core/src/tenants/Tenant.ts`
- **Logto TenantPool**: `packages/core/src/tenants/index.ts`
- **Logto DB Schema**: `packages/schemas/tables/*.sql`
- **Logto API Init**: `packages/core/src/routes/init.ts`
- **Logto Account API**: `packages/core/src/routes/account/`
- **Logto EnvSet**: `packages/core/src/env-set/index.ts`
