# Equalizer-Stake — Backend Architecture

## Project Structure

```
backend/
├── alembic/ # Migration scripts
│ ├── versions/
│ └── env.py
├── alembic.ini
├── app/
│ ├── __init__.py
│ ├── main.py # FastAPI app factory, lifespan
│ ├── core/
│ │ ├── config.py # pydantic-settings: Settings class
│ │ ├── security.py # JWT encode/decode, AES-256-GCM encrypt/decrypt
│ │ ├── database.py # async engine, sessionmaker, Base
│ │ ├── redis.py # Redis connection pool
│ │ └── deps.py # Shared Depends: get_db, get_current_user
│ ├── middleware/
│ │ ├── cors.py # CORSMiddleware config
│ │ ├── auth.py # JWT verification middleware
│ │ ├── rate_limit.py # Sliding window rate limiter
│ │ └── audit.py # Request/response audit logging
│ ├── models/ # SQLAlchemy async ORM models
│ │ ├── base.py # Base, TimestampMixin, SoftDeleteMixin
│ │ ├── user.py
│ │ ├── guardian.py
│ │ ├── student.py
│ │ ├── teacher.py
│ │ ├── room.py
│ │ ├── session.py
│ │ ├── invoice.py
│ │ ├── payment.py
│ │ ├── attendance.py
│ │ ├── notification.py
│ │ └── audit_log.py
│ ├── schemas/ # Pydantic v2 request/response schemas
│ │ ├── user.py
│ │ ├── auth.py # LoginRequest, TokenResponse
│ │ ├── guardian.py
│ │ ├── student.py
│ │ ├── teacher.py
│ │ ├── room.py
│ │ ├── session.py
│ │ ├── invoice.py
│ │ ├── payment.py
│ │ ├── attendance.py
│ │ └── notification.py
│ ├── routers/ # FastAPI APIRouter per resource
│ │ ├── auth.py # POST /auth/login, /auth/refresh, /auth/logout
│ │ ├── users.py
│ │ ├── guardians.py
│ │ ├── students.py
│ │ ├── teachers.py
│ │ ├── rooms.py
│ │ ├── sessions.py
│ │ ├── invoices.py
│ │ ├── payments.py
│ │ ├── attendance.py
│ │ ├── notifications.py
│ │ └── admin.py # Dashboard stats, audit log viewer
│ ├── services/ # Business logic layer
│ │ ├── auth_service.py # Login, token refresh, password hashing
│ │ ├── billing_service.py # Invoice generation (5th), reminders (11th)
│ │ ├── attendance_service.py # GPS validation, face verification trigger
│ │ ├── notification_service.py # WhatsApp + email dispatch
│ │ ├── student_service.py
│ │ ├── teacher_service.py
│ │ └── audit_service.py
│ └── cron/
│ ├── scheduler.py # APScheduler AsyncIOScheduler
│ ├── billing_job.py # generate_monthly_invoices(), send_reminders()
│ └── cleanup_job.py # Soft-delete purge, token cleanup
├── tests/
├── Dockerfile
├── pyproject.toml
└── requirements.txt
```

## App Factory & Lifespan

```python
# app/main.py
from contextlib import asynccontextmanager
from fastapi import FastAPI
from app.core.database import engine
from app.cron.scheduler import start_scheduler, shutdown_scheduler

@asynccontextmanager
async def lifespan(app: FastAPI):
 # startup
 await start_scheduler() # APScheduler with billing_job, cleanup_job
 yield
 # shutdown
 await shutdown_scheduler()
 await engine.dispose()

def create_app() -> FastAPI:
 app = FastAPI(
 title="Equalizer-Stake API",
 version="1.0.0",
 lifespan=lifespan,
 )
 # middleware registration order (outermost first)
 app.add_middleware(AuditMiddleware)
 app.add_middleware(RateLimitMiddleware, redis=redis_client, limit=100, window=60)
 app.add_middleware(AuthMiddleware, exclude_paths=["/auth/login", "/health"])
 app.add_middleware(CORSMiddleware, allow_origins=settings.CORS_ORIGINS, ...)
 # router registration
 app.include_router(auth_router, prefix="/auth", tags=["auth"])
 app.include_router(users_router, prefix="/users", tags=["users"])
 # ... etc
 return app
```

## Dependency Injection Pattern

FastAPI `Depends()` for all shared resources:

```python
# app/core/deps.py
from sqlalchemy.ext.asyncio import AsyncSession
from app.core.database import async_session_maker

async def get_db() -> AsyncGenerator[AsyncSession, None]:
 async with async_session_maker() as session:
 try:
 yield session
 await session.commit()
 except Exception:
 await session.rollback()
 raise

async def get_current_user(
 token: str = Depends(oauth2_scheme),
 db: AsyncSession = Depends(get_db),
) -> User:
 payload = decode_jwt(token)
 if payload.get("jti") in await redis.smembers("jwt:blocklist"):
 raise HTTPException(401, "Token revoked")
 user = await db.get(User, payload["sub"])
 if not user or user.deleted_at is not None:
 raise HTTPException(401, "User not found")
 return user

def require_role(*roles: str):
 async def _check(user: User = Depends(get_current_user)):
 if user.role not in roles:
 raise HTTPException(403, "Insufficient permissions")
 return user
 return _check
```

Usage in routers:

```python
@router.get("/students", response_model=list[StudentOut])
async def list_students(
 db: AsyncSession = Depends(get_db),
 user: User = Depends(require_role("admin", "teacher")),
):
 return await StudentService(db).list_students()
```

## Middleware Chain

Order (outermost → innermost):

1. **AuditMiddleware** — captures request/response metadata, writes to AuditLog asynchronously
2. **RateLimitMiddleware** — sliding window via Redis `INCR` + `EXPIRE` per IP+user, 100 req/min default
3. **AuthMiddleware** — JWT decode, attaches `request.state.user`, excludes public paths
4. **CORSMiddleware** — configured origins from `settings.CORS_ORIGINS`

```python
# app/middleware/rate_limit.py
class RateLimitMiddleware(BaseHTTPMiddleware):
 async def dispatch(self, request, call_next):
 key = f"rate:{client_ip(request)}:{request.url.path}"
 count = await self.redis.incr(key)
 if count == 1:
 await self.redis.expire(key, self.window)
 if count > self.limit:
 return JSONResponse(status_code=429, content={"detail": "Rate limit exceeded"})
 return await call_next(request)
```

## Service Layer Pattern

Services encapsulate business logic. Routers are thin — only validation + delegation.

```python
# app/services/billing_service.py
class BillingService:
 def __init__(self, db: AsyncSession):
 self.db = db

 async def generate_monthly_invoices(self, month: date) -> list[Invoice]:
 """Called by cron on 5th of each month."""
 students = await self.db.execute(
 select(Student).where(Student.is_active == True, Student.deleted_at.is_(None))
 )
 invoices = []
 for student in students.scalars():
 session_count = await self._count_sessions(student.id, month)
 if session_count == 0:
 continue
 invoice = Invoice(
 student_id=student.id,
 guardian_id=student.guardian_id,
 amount=session_count * student.rate_per_session,
 period=month,
 status="pending",
 due_date=month.replace(day=11),
 )
 self.db.add(invoice)
 invoices.append(invoice)
 await self.db.flush()
 return invoices

 async def send_payment_reminders(self) -> int:
 """Called by cron on 11th of each month. Returns count sent."""
 overdue = await self.db.execute(
 select(Invoice).where(
 Invoice.status == "pending",
 Invoice.due_date <= date.today(),
 Invoice.deleted_at.is_(None),
 )
 )
 sent = 0
 for inv in overdue.scalars():
 guardian = await self.db.get(Guardian, inv.guardian_id)
 await NotificationService(self.db).send_payment_reminder(guardian, inv)
 sent += 1
 return sent
```

## Async Patterns

All DB operations use SQLAlchemy async with `asyncpg`:

```python
# app/core/database.py
from sqlalchemy.ext.asyncio import create_async_engine, async_sessionmaker, AsyncSession
from sqlalchemy.orm import DeclarativeBase

engine = create_async_engine(
 settings.DATABASE_URL, # postgresql+asyncpg://user:pass@host:5432/db
 pool_size=20,
 max_overflow=10,
 pool_pre_ping=True,
)
async_session_maker = async_sessionmaker(engine, expire_on_commit=False)

class Base(DeclarativeBase):
 pass
```

External HTTP calls (WhatsApp gateway) use `httpx.AsyncClient` with connection pooling.

## Cron Job Design

APScheduler `AsyncIOScheduler` running inside FastAPI lifespan:

```python
# app/cron/scheduler.py
from apscheduler.schedulers.asyncio import AsyncIOScheduler
from apscheduler.triggers.cron import CronTrigger

scheduler = AsyncIOScheduler()

async def start_scheduler():
 scheduler.add_job(
 billing_job_generate_invoices,
 CronTrigger(day=5, hour=1, minute=0), # 5th, 01:00 WIB
 id="billing_generate",
 replace_existing=True,
 )
 scheduler.add_job(
 billing_job_send_reminders,
 CronTrigger(day=11, hour=8, minute=0), # 11th, 08:00 WIB
 id="billing_reminders",
 replace_existing=True,
 )
 scheduler.start()

# app/cron/billing_job.py
async def billing_job_generate_invoices():
 async with async_session_maker() as db:
 service = BillingService(db)
 invoices = await service.generate_monthly_invoices(date.today().replace(day=1))
 await db.commit()
 logger.info(f"Generated {len(invoices)} invoices")

async def billing_job_send_reminders():
 async with async_session_maker() as db:
 service = BillingService(db)
 sent = await service.send_payment_reminders()
 await db.commit()
 logger.info(f"Sent {sent} payment reminders")
```

## Config (pydantic-settings)

```python
# app/core/config.py
from pydantic_settings import BaseSettings

class Settings(BaseSettings):
 DATABASE_URL: str
 REDIS_URL: str = "redis://localhost:6379/0"
 JWT_SECRET: str
 JWT_ALGORITHM: str = "HS256"
 JWT_EXPIRE_MINUTES: int = 60
 AES_KEY: str # 256-bit key, hex-encoded
 CORS_ORIGINS: list[str] = ["http://localhost:3000"]
 WHATSAPP_GATEWAY_URL: str
 SMTP_HOST: str = "smtp.gmail.com"
 SMTP_PORT: int = 587
 SMTP_USER: str = ""
 SMTP_PASS: str = ""
 RATE_LIMIT_PER_MINUTE: int = 100
 GPS_RADIUS_METERS: int = 150

 model_config = {"env_file": ".env"}
```
