# Equalizer-Stake — Deployment Architecture

## Docker Multi-Stage Build

### Backend Dockerfile

```dockerfile
# backend/Dockerfile
FROM python:3.12-slim AS base
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
 gcc libpq-dev && rm -rf /var/lib/apt/lists/*

FROM base AS deps
COPY requirements.txt .
RUN pip install --no-cache-dir --prefix=/install -r requirements.txt

FROM base AS runtime
COPY --from=deps /install /usr/local
COPY alembic/ alembic/
COPY alembic.ini .
COPY app/ app/
RUN useradd -r -s /bin/false appuser
USER appuser
EXPOSE 8000
HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
 CMD python -c "import urllib.request; urllib.request.urlopen('http://localhost:8000/health')"
CMD ["uvicorn", "app.main:create_app", "--factory", "--host", "0.0.0.0", "--port", "8000", "--workers", "4"]
```

### Frontend Dockerfile

```dockerfile
# frontend/Dockerfile
FROM node:20-alpine AS deps
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci

FROM deps AS builder
COPY . .
RUN npm run build

FROM node:20-alpine AS runner
WORKDIR /app
ENV NODE_ENV=production
RUN addgroup --system --gid 1001 nodejs && \
 adduser --system --uid 1001 nextjs
COPY --from=builder /app/public ./public
COPY --from=builder --chown=nextjs:nodejs /app/.next/standalone ./
COPY --from=builder --chown=nextjs:nodejs /app/.next/static ./.next/static
USER nextjs
EXPOSE 3000
HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
 CMD wget --no-verbose --tries=1 --spider http://localhost:3000/ || exit 1
CMD ["node", "server.js"]
```

## docker-compose (Local Dev + Production)

```yaml
# docker-compose.yml
version: "3.9"

services:
 postgres:
 image: postgres:16-alpine
 environment:
 POSTGRES_DB: equalizer_stake
 POSTGRES_USER: ${DB_USER}
 POSTGRES_PASSWORD: ${DB_PASSWORD}
 volumes:
 - pgdata:/var/lib/postgresql/data
 - ./scripts/init.sql:/docker-entrypoint-initdb.d/init.sql
 ports:
 - "5432:5432"
 healthcheck:
 test: ["CMD-SHELL", "pg_isready -U ${DB_USER} -d equalizer_stake"]
 interval: 10s
 timeout: 5s
 retries: 5
 restart: unless-stopped

 redis:
 image: redis:7-alpine
 command: redis-server --requirepass ${REDIS_PASSWORD}
 volumes:
 - redisdata:/data
 ports:
 - "6379:6379"
 healthcheck:
 test: ["CMD", "redis-cli", "-a", "${REDIS_PASSWORD}", "ping"]
 interval: 10s
 timeout: 5s
 retries: 5
 restart: unless-stopped

 backend:
 build:
 context: ./backend
 dockerfile: Dockerfile
 environment:
 DATABASE_URL: postgresql+asyncpg://${DB_USER}:${DB_PASSWORD}@postgres:5432/equalizer_stake
 REDIS_URL: redis://:${REDIS_PASSWORD}@redis:6379/0
 JWT_SECRET: ${JWT_SECRET}
 AES_KEY: ${AES_KEY}
 WHATSAPP_GATEWAY_URL: ${WHATSAPP_GATEWAY_URL}
 WHATSAPP_API_KEY: ${WHATSAPP_API_KEY}
 depends_on:
 postgres:
 condition: service_healthy
 redis:
 condition: service_healthy
 ports:
 - "8000:8000"
 restart: unless-stopped

 frontend:
 build:
 context: ./frontend
 dockerfile: Dockerfile
 environment:
 NEXT_PUBLIC_API_URL: http://backend:8000
 depends_on:
 - backend
 ports:
 - "3000:3000"
 restart: unless-stopped

 nginx:
 image: nginx:alpine
 volumes:
 - ./nginx/nginx.conf:/etc/nginx/nginx.conf:ro
 - ./nginx/ssl:/etc/nginx/ssl:ro
 ports:
 - "80:80"
 - "443:443"
 depends_on:
 - frontend
 - backend
 restart: unless-stopped

volumes:
 pgdata:
 redisdata:
```

## Cloud Deployment — Jakarta Region

### Infrastructure (Single VPS / Docker Compose)

```
┌─────────────────────────────────────────────────┐
│ Cloud VPS (Jakarta) │
│ Provider: GCP asia-southeast2 / AWS ap-southeast│
│ Spec: 4 vCPU, 8GB RAM, 100GB SSD │
│ │
│ ┌─────────────┐ ┌─────────────┐ │
│ │ nginx │ │ postgres │ │
│ │ :443/:80 │ │ :5432 │ │
│ └──────┬──────┘ └──────┬──────┘ │
│ │ │ │
│ ┌──────┴──────┐ ┌──────┴──────┐ │
│ │ frontend │ │ redis │ │
│ │ :3000 │ │ :6379 │ │
│ └─────────────┘ └─────────────┘ │
│ │
│ ┌─────────────┐ ┌─────────────┐ │
│ │ backend │ │ certbot │ │
│ │ :8000 │ │ (cron) │ │
│ └─────────────┘ └─────────────┘ │
└─────────────────────────────────────────────────┘
```

### Nginx Config

```nginx
# nginx/nginx.conf
upstream backend {
 server backend:8000;
}
upstream frontend {
 server frontend:3000;
}

server {
 listen 443 ssl http2;
 server_name equalizer-stake.com;

 ssl_certificate /etc/nginx/ssl/fullchain.pem;
 ssl_certificate_key /etc/nginx/ssl/privkey.pem;
 ssl_protocols TLSv1.2 TLSv1.3;

 # API proxy
 location /api/ {
 proxy_pass http://backend/;
 proxy_set_header Host $host;
 proxy_set_header X-Real-IP $remote_addr;
 proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
 proxy_set_header X-Forwarded-Proto $scheme;
 }

 # Frontend
 location / {
 proxy_pass http://frontend;
 proxy_set_header Host $host;
 proxy_set_header X-Real-IP $remote_addr;
 }
}

server {
 listen 80;
 server_name equalizer-stake.com;
 return 301 https://$host$request_uri;
}
```

## Health Check Endpoints

```python
# Backend: GET /health
@router.get("/health")
async def health_check(db: AsyncSession = Depends(get_db)):
 checks = {}
 try:
 await db.execute(text("SELECT 1"))
 checks["postgres"] = "ok"
 except Exception:
 checks["postgres"] = "error"
 raise HTTPException(503, checks)
 try:
 await redis.ping()
 checks["redis"] = "ok"
 except Exception:
 checks["redis"] = "error"
 raise HTTPException(503, checks)
 return {"status": "healthy", "checks": checks, "version": settings.APP_VERSION}

# Frontend: GET / (200 OK = healthy)
```

## Monitoring & Alerting Thresholds

| Metric | Warning | Critical | Action |
|--------|---------|----------|--------|
| CPU usage | > 70% | > 90% | Scale or optimize |
| Memory usage | > 75% | > 90% | Investigate leak |
| Disk usage | > 70% | > 85% | Archive logs, expand |
| API response time (p95) | > 500ms | > 2s | Profile slow endpoints |
| API error rate (5xx) | > 1% | > 5% | Check logs, rollback |
| PostgreSQL connections | > 50 | > 80 | Pool tuning |
| Redis memory | > 512MB | > 1GB | Key eviction review |
| WhatsApp delivery rate | < 80% | < 50% | Check gateway status |

### Monitoring Stack

- **Prometheus** — metrics scraping (backend `/metrics` endpoint via `prometheus-fastapi-instrumentator`)
- **Grafana** — dashboards for API latency, error rates, DB connections
- **Loki** — log aggregation (structured JSON logs from Uvicorn)
- **Alertmanager** — webhook to WhatsApp/Slack for critical alerts

## Backup Strategy

```bash
# Daily PostgreSQL backup (cron job, 02:00 WIB)
# scripts/backup.sh
#!/bin/bash
BACKUP_DIR="/backups/postgres"
DATE=$(date +%Y%m%d_%H%M%S)
docker exec postgres pg_dump -U ${DB_USER} -Fc equalizer_stake > \
 ${BACKUP_DIR}/eq_stake_${DATE}.dump

# Retention: keep 7 daily, 4 weekly, 3 monthly
find ${BACKUP_DIR} -name "*.dump" -mtime +7 -delete

# Weekly backup upload to S3/GCS
aws s3 sync ${BACKUP_DIR} s3://eq-stake-backups/postgres/ --storage-class GLACIER
```

### Backup Schedule

| Type | Frequency | Retention | Location |
|------|-----------|-----------|----------|
| Full DB dump | Daily 02:00 | 7 days local, 90 days cloud | Local + S3/GCS |
| Redis RDB snapshot | Every 6h | 3 days | Local volume |
| Application logs | Rotated daily | 30 days | Local + Loki |
| Docker volumes | Weekly | 4 weeks | Local tarball |

## Rollback Procedure

```bash
# 1. Identify last known good image tag
git log --oneline -5 # find commit hash
export PREVIOUS_TAG="eq-stake:abc1234"

# 2. Rollback backend
docker compose pull backend:${PREVIOUS_TAG}
docker compose up -d --no-deps backend

# 3. Rollback database migration (if schema changed)
docker exec backend alembic downgrade -1

# 4. Verify health
curl -s https://equalizer-stake.com/api/health | jq .

# 5. If frontend also needs rollback
docker compose pull frontend:${PREVIOUS_TAG}
docker compose up -d --no-deps frontend

# Emergency: rollback everything to previous compose version
git checkout HEAD~1 -- docker-compose.yml
docker compose up -d --force-recreate
```

### Deployment Checklist

1. Run `alembic upgrade head` (backend entrypoint does this)
2. Health check passes on `/health`
3. Smoke test: login → dashboard → list students
4. Monitor error rate for 15 minutes
5. If error rate > 1% → automatic rollback trigger
