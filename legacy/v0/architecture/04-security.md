# Equalizer-Stake — Security Architecture

## JWT Flow with Role-Based Claims

### Token Structure

```
Header: { "alg": "HS256", "typ": "JWT" }
Payload: {
 "sub": "uuid-of-user",
 "role": "admin" | "teacher" | "guardian",
 "email": "user@example.com",
 "jti": "unique-token-id",
 "iat": 1719475200,
 "exp": 1719478800 # iat + 60 minutes
}
Signature: HMACSHA256(base64(header) + "." + base64(payload), JWT_SECRET)
```

### Token Lifecycle

```
1. POST /auth/login → validate credentials → issue access_token + refresh_token
2. Client stores access_token in memory (Zustand), refresh_token in httpOnly cookie
3. Each API request: Authorization: Bearer <access_token>
4. Auth middleware: decode JWT → check jti not in Redis blocklist → attach user to request.state
5. POST /auth/refresh → validate refresh_token → issue new access_token pair
6. POST /auth/logout → add jti to Redis blocklist (TTL = remaining token expiry)
```

### Role-Based Access Control

```python
# Three roles with hierarchical permissions
ROLE_PERMISSIONS = {
 "admin": ["*"], # all CRUD + audit + settings
 "teacher": ["sessions:read", "attendance:*", "students:read"],
 "guardian": ["invoices:read", "payments:*", "students:read:own"],
}
```

Router-level enforcement via `Depends(require_role(...))`:

```python
@router.post("/invoices/{id}/pay")
async def pay_invoice(
 id: uuid.UUID,
 payload: PaymentCreate,
 user: User = Depends(require_role("guardian")),
 db: AsyncSession = Depends(get_db),
):
 # Additional check: guardian can only pay own invoices
 invoice = await db.get(Invoice, id)
 if invoice.guardian_id != user.guardian_profile.id:
 raise HTTPException(403, "Not your invoice")
 ...
```

## AES-256-GCM Encryption for PII Fields

### Encryption Service

```python
# app/core/security.py
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
import os

class EncryptionService:
 def __init__(self, key: str):
 # key: 256-bit hex string from settings.AES_KEY
 self.key = bytes.fromhex(key)
 self.aesgcm = AESGCM(self.key)

 def encrypt(self, plaintext: str) -> str:
 """Encrypt string, return base64(nonce + ciphertext + tag)."""
 nonce = os.urandom(12) # 96-bit nonce, unique per encryption
 ct = self.aesgcm.encrypt(nonce, plaintext.encode(), None)
 return base64.b64encode(nonce + ct).decode()

 def decrypt(self, ciphertext: str) -> str:
 """Decrypt base64(nonce + ciphertext + tag) back to string."""
 raw = base64.b64decode(ciphertext)
 nonce, ct = raw[:12], raw[12:]
 return self.aesgcm.decrypt(nonce, ct, None).decode()

enc = EncryptionService(settings.AES_KEY)
```

### SQLAlchemy TypeDecorator for Transparent Encrypt/Decrypt

```python
# app/models/types.py
from sqlalchemy.types import TypeDecorator, String

class EncryptedString(TypeDecorator):
 """Transparently encrypts on write, decrypts on read."""
 impl = String(500)
 cache_ok = True

 def process_bind_param(self, value, dialect):
 if value is not None:
 return enc.encrypt(value)
 return value

 def process_result_value(self, value, dialect):
 if value is not None:
 return enc.decrypt(value)
 return value

# Usage in models
class Guardian(Base, TimestampMixin, SoftDeleteMixin):
 __tablename__ = "guardians"
 phone: Mapped[str] = mapped_column(EncryptedString(500))
 nik: Mapped[str] = mapped_column(EncryptedString(500))
 address: Mapped[str] = mapped_column(EncryptedString(1000))
```

**Key management**: AES key stored in environment variable, rotated quarterly. Old ciphertext re-encrypted during low-traffic window via migration script.

## Rate Limiting Design

### Sliding Window (Redis)

```python
# app/middleware/rate_limit.py
async def check_rate_limit(redis: Redis, key: str, limit: int, window: int) -> bool:
 """Returns True if allowed, False if rate limited."""
 now = time.time()
 pipe = redis.pipeline()
 pipe.zremrangebyscore(key, 0, now - window) # remove expired entries
 pipe.zadd(key, {str(now): now}) # add current request
 pipe.zcard(key) # count requests in window
 pipe.expire(key, window) # auto-cleanup
 _, _, count, _ = await pipe.execute()
 return count <= limit
```

### Rate Limit Tiers

| Path Pattern | Limit | Window | Key |
|-------------|-------|--------|-----|
| `POST /auth/login` | 5 | 60s | `rl:login:{ip}` |
| `POST /auth/refresh` | 10 | 60s | `rl:refresh:{user_id}` |
| `POST /payments` | 10 | 60s | `rl:pay:{user_id}` |
| `GET /attendance` | 30 | 60s | `rl:att:{user_id}` |
| All other | 100 | 60s | `rl:default:{ip}` |

Response headers: `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`.

## OWASP Mitigations

### A02: Cryptographic Failures

- PII fields (NIK, phone, address) encrypted at rest with AES-256-GCM
- Passwords hashed with bcrypt (12 rounds) via `passlib`
- JWT signed with HS256, secret ≥ 256 bits
- HTTPS enforced (HSTS header in production)
- No sensitive data in URLs or logs (audit middleware redacts password_hash)

### A03: Injection

- SQLAlchemy ORM parameterized queries — no raw SQL for user input
- Pydantic v2 strict type validation on all inputs
- `email` field validated with `EmailStr`
- UUID fields validated as proper UUID type

### A04: Insecure Design

- Rate limiting per endpoint type
- Business logic in service layer, not routers (enforces consistent validation)
- Guardian can only access own children's data (checked in service layer)
- Teacher can only mark attendance for assigned sessions

### A05: Security Misconfiguration

- CORS restricted to configured origins (no `*`)
- Debug mode disabled in production
- Database credentials rotated quarterly
- Redis AUTH enabled
- Docker containers run as non-root user

### A06: Vulnerable Components

- `pip-audit` in CI pipeline
- `npm audit` in frontend CI
- Dependabot alerts enabled
- Pinned versions in requirements.txt / package-lock.json

### A07: Authentication Failures

- Account lockout after 5 failed login attempts (Redis counter, 15min window)
- JWT blocklist on logout (Redis SET with TTL)
- Refresh tokens in httpOnly secure cookies (XSS-proof)
- Token expiry: access=60min, refresh=7 days

## UU PDP Compliance Pipeline

Indonesia's Personal Data Protection Law (UU No. 27/2022):

### Data Classification

| Classification | Fields | Protection |
|---------------|--------|------------|
| Personal Data | name, email, role | Encrypted at rest |
| Sensitive Personal Data | NIK, phone, address, photo | AES-256-GCM + access logging |
| Special Category | face biometric | Never stored server-side (client-side only) |

### Compliance Requirements

```python
# 1. Data minimization — only collect necessary fields
# 2. Purpose limitation — data used only for stated purpose
# 3. Storage limitation — retention period enforced
# 4. Right to access — user can request data export
# 5. Right to erasure — data anonymization workflow (see below)
# 6. Data breach notification — audit log monitors unauthorized access
# 7. Consent management — guardian consent recorded on registration
```

### Data Subject Rights Implementation

```python
# Right to access (data export)
async def export_user_data(db: AsyncSession, user_id: uuid.UUID) -> dict:
 user = await db.get(User, user_id)
 guardian = await db.execute(select(Guardian).where(Guardian.user_id == user_id))
 students = await db.execute(
 select(Student).where(Student.guardian_id == guardian.scalar_one_or_none().id)
 )
 return {
 "user": model_to_dict(user),
 "guardian": model_to_dict(guardian.scalar_one()),
 "students": [model_to_dict(s) for s in students.scalars()],
 "invoices": [...],
 "exported_at": datetime.utcnow().isoformat(),
 }

# Right to erasure (anonymization)
async def anonymize_user(db: AsyncSession, user_id: uuid.UUID):
 user = await db.get(User, user_id)
 user.email = f"anonymized_{user_id}@deleted.local"
 user.full_name = "ANONYMIZED"
 user.is_active = False

 guardian = await get_guardian_by_user(db, user_id)
 if guardian:
 guardian.phone = enc.encrypt("ANONYMIZED")
 guardian.nik = enc.encrypt("ANONYMIZED")
 guardian.address = enc.encrypt("ANONYMIZED")
 guardian.full_name = "ANONYMIZED"

 # Preserve financial records with anonymized references
 # Invoice/payment amounts retained for accounting, personal data replaced
 await db.commit()
```

## Data Anonymization Workflow

```
1. Admin requests anonymization via POST /admin/anonymize/{user_id}
2. System checks: all invoices must be paid or cancelled
3. Soft-delete user, guardian, student records
4. Replace PII fields with "ANONYMIZED" (encrypted)
5. Generate audit log entry with action=ANONYMIZE
6. Remove from Redis caches
7. Anonymized records remain in DB for FK integrity (invoices, attendance)
```

## Security Headers

```python
# app/middleware/security_headers.py
@app.middleware("http")
async def add_security_headers(request, call_next):
 response = await call_next(request)
 response.headers["X-Content-Type-Options"] = "nosniff"
 response.headers["X-Frame-Options"] = "DENY"
 response.headers["X-XSS-Protection"] = "1; mode=block"
 response.headers["Strict-Transport-Security"] = "max-age=31536000; includeSubDomains"
 response.headers["Content-Security-Policy"] = "default-src 'self'; script-src 'self'"
 response.headers["Referrer-Policy"] = "strict-origin-when-cross-origin"
 return response
```
