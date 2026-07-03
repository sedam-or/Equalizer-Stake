-- Migration 004: Studio reservation — UniqueConstraint + status enum + indexes
-- T-055: Guardian Self-Booking + DB Constraints

-- 1. Add new status values to session_status enum
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_enum WHERE enumlabel = 'pending_approval' AND enumtypid = 'session_status'::regtype) THEN
        ALTER TYPE session_status ADD VALUE 'pending_approval';
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_enum WHERE enumlabel = 'approved' AND enumtypid = 'session_status'::regtype) THEN
        ALTER TYPE session_status ADD VALUE 'approved';
    END IF;
END$$;

-- 2. Add UniqueConstraint on (id_room, session_date, start_time) to prevent double-booking
-- First remove any existing duplicates (safety check)
DO $$
DECLARE
    dup_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO dup_count
    FROM (
        SELECT id_room, session_date, start_time, COUNT(*)
        FROM sessions
        WHERE status NOT IN ('cancelled')
        GROUP BY id_room, session_date, start_time
        HAVING COUNT(*) > 1
    ) d;

    IF dup_count > 0 THEN
        RAISE EXCEPTION 'Ditemukan % slot duplikat. Harap selesaikan konflik sebelum migrasi.', dup_count;
    END IF;
END$$;

-- Add the unique constraint
ALTER TABLE sessions
    ADD CONSTRAINT uq_session_room_date_start
    UNIQUE (id_room, session_date, start_time);

-- 3. Indexes for performance on common queries
CREATE INDEX IF NOT EXISTS idx_sessions_room_date ON sessions(id_room, session_date);
CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
CREATE INDEX IF NOT EXISTS idx_sessions_type_status ON sessions(session_type, status);
