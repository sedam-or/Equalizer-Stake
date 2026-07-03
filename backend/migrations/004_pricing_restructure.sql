-- T-067: Pricing model restructure
-- Drop old pricing table and recreate with new schema

DROP TABLE IF EXISTS pricings CASCADE;

CREATE TABLE pricings (
    id_pricing UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    instrument VARCHAR(50) NOT NULL,
    class_type VARCHAR(50) NOT NULL,
    level VARCHAR(50) NOT NULL,
    monthly_fee NUMERIC(12,2) NOT NULL,
    duration_minutes INTEGER NOT NULL DEFAULT 45,
    registration_fee NUMERIC(12,2) NOT NULL DEFAULT 0,
    effective_date DATE NOT NULL,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_pricings_instrument ON pricings(instrument);
CREATE INDEX idx_pricings_class_type ON pricings(class_type);
CREATE INDEX idx_pricings_active ON pricings(is_active);
