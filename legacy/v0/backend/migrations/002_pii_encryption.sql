-- T-048: PII Encryption + GPS Precision Migration
-- Run this SQL to alter existing columns for AES-256-GCM encryption support.

-- Guardian: expand full_name and address for encrypted storage
ALTER TABLE guardians
ALTER COLUMN full_name TYPE VARCHAR(500);

ALTER TABLE guardians
ALTER COLUMN address TYPE VARCHAR(1000);

-- Student: expand full_name for encrypted storage
ALTER TABLE students
ALTER COLUMN full_name TYPE VARCHAR(500);

-- Attendance: expand photo_url for encrypted storage
ALTER TABLE attendance
ALTER COLUMN photo_url TYPE VARCHAR(1000);

-- Attendance: fix GPS precision from Float to Numeric(10,7)
ALTER TABLE attendance
ALTER COLUMN gps_lat TYPE NUMERIC(10, 7);

ALTER TABLE attendance
ALTER COLUMN gps_lng TYPE NUMERIC(10, 7);

-- NOTE: Existing plaintext data will still be readable (EncryptedString
-- falls back to returning raw value if decryption fails). New data will
-- be encrypted automatically. To encrypt existing data, run a migration
-- script that reads each row and re-saves it.
