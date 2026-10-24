-- Migration 005: Registrations table for visitor registration flow
-- T-066: Visitor submits form → admin approves → auto-create guardian+student+user

CREATE TYPE registration_status AS ENUM ('pending', 'approved', 'rejected');

CREATE TABLE registrations (
    id_registration UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    full_name       VARCHAR(255) NOT NULL,
    phone_whatsapp  VARCHAR(20)  NOT NULL,
    email           VARCHAR(255),
    address         TEXT,
    instrument      VARCHAR(50)  NOT NULL,
    level           VARCHAR(50)  NOT NULL,
    class_type      VARCHAR(50)  NOT NULL,
    status          registration_status NOT NULL DEFAULT 'pending',
    reviewed_by     UUID REFERENCES users(id_user),
    reviewed_at     TIMESTAMPTZ,
    notes           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_registrations_status ON registrations(status);
CREATE INDEX idx_registrations_phone  ON registrations(phone_whatsapp);
