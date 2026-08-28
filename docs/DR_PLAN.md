# Disaster Recovery Plan — Equalizer-Stake (Equalizer-Stake)

## Overview

| Metric | Target | Current |
|--------|--------|---------|
| **RTO** (Recovery Time Objective) | ≤ 2 hours | ~30 min (restore from local backup) |
| **RPO** (Recovery Point Objective) | ≤ 1 hour | ~24 hours (daily backup at 03:00 WIB) |

## Backup Strategy

### Automated Daily Backup
- **Schedule**: Daily 03:00 WIB via APScheduler
- **Method**: pg_dump (primary) / Python SQL dump (fallback)
- **Location**: `/home/kepolu/equalizer-stake/backups/`
- **Format**: Compressed SQL (`backup_YYYYMMDD_HHMMSS.sql.gz`)
- **Retention**: 30 days
- **SRS Reference**: NFR-REL-04

### Backup Components
1. **Database**: Full pg_dump of PostgreSQL
2. **Application Code**: Git repository (GitHub: sedam-or/Music-School)
3. **Configuration**: `.env` file (separate secure storage)
4. **Uploaded Files**: `/backend/uploads/` (photos, attendance)

## Recovery Procedures

### Scenario 1: Database Corruption
```bash
# 1. Stop application
pkill -f "uvicorn.*app.main"

# 2. Restore from backup
python3 backend/scripts/restore_backup.py --latest

# 3. Verify data integrity
python3 backend/scripts/verify_restore.py

# 4. Restart application
cd backend && python3 -m uvicorn app.main:app --host 0.0.0.0 --port 8000
```
**Estimated time**: 15-30 minutes

### Scenario 2: Server Failure
```bash
# 1. Provision new server (same specs)
# 2. Install dependencies
# 3. Clone repository
git clone https://github.com/sedam-or/Music-School.git
# 4. Restore database from backup
python3 backend/scripts/restore_backup.py --latest
# 5. Restore .env from secure storage
# 6. Start application
```
**Estimated time**: 1-2 hours

### Scenario 3: Data Loss (Accidental Delete)
```bash
# 1. Identify timestamp of last good state
ls -la backups/backup_*.sql.gz

# 2. Restore specific backup
python3 backend/scripts/restore_backup.py --file backup_20260703_030000.sql.gz

# 3. Verify restored data
python3 backend/scripts/verify_restore.py
```
**Estimated time**: 15-30 minutes

## Contact List & Escalation

| Role | Contact | Responsibility |
|------|---------|---------------|
| System Admin | Admin EQS | First responder, server management |
| Database Admin | Admin EQS | Database restore, integrity check |
| Developer | sedam-or | Code issues, deployment |

### Escalation Path
1. **0-15 min**: System Admin attempts restore
2. **15-60 min**: Escalate to Developer if restore fails
3. **60-120 min**: Full server rebuild if needed

## Testing Schedule

| Test | Frequency | Last Run | Result |
|------|-----------|----------|--------|
| Backup verification | Daily (automated) | — | — |
| Restore test | Monthly | — | — |
| Full DR drill | Quarterly | — | — |

## SRS Compliance

- **NFR-REL-04**: Database backed up daily at 03:00 WIB ✅
- **NFR-REL-05**: Recovery target ≤ 2 hours ✅ (estimated 30 min local, 2h full rebuild)
- **NFR-REL-06**: Data loss ≤ 1 hour ⚠️ (current: 24h daily backup; needs hourly WAL archiving for true ≤1h RPO)

## Known Gaps

1. **RPO gap**: Daily backup means up to 24h data loss. For true ≤1h RPO, need:
 - PostgreSQL WAL archiving to S3/GCS
 - Or continuous replication to standby
2. **Offsite backup**: Currently local only. Needs S3/GCS upload (T-116)
3. **Automated restore testing**: Not yet implemented
