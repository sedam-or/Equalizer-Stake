-- Migration 003: Add category field to audit_logs table
-- SRS T-052: Audit Log Completeness

-- Create enum type for audit category
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'audit_category') THEN
        CREATE TYPE audit_category AS ENUM ('operasional', 'keamanan', 'kepatuhan');
    END IF;
END$$;

-- Add category column (nullable, defaults to 'operasional')
ALTER TABLE audit_logs
    ADD COLUMN IF NOT EXISTS category audit_category DEFAULT 'operasional';

-- Create index for category-based queries (retention, filtering)
CREATE INDEX IF NOT EXISTS idx_audit_logs_category ON audit_logs(category);
CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at ON audit_logs(created_at);

-- Create archive table (same schema, for 7-year retention)
CREATE TABLE IF NOT EXISTS audit_logs_archive (
    LIKE audit_logs INCLUDING ALL
);
