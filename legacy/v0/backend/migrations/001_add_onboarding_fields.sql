-- T-045 FINDING 1: Onboarding fields for User model
-- Run this SQL against the database to add onboarding columns

ALTER TABLE users
ADD COLUMN IF NOT EXISTS first_login BOOLEAN NOT NULL DEFAULT FALSE,
ADD COLUMN IF NOT EXISTS is_temp_password BOOLEAN NOT NULL DEFAULT FALSE,
ADD COLUMN IF NOT EXISTS password_expires_at TIMESTAMPTZ NULL;
