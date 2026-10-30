CREATE TABLE totp_recovery_codes (
  tenant_id uuid NOT NULL REFERENCES tenants(id),
  person_id uuid NOT NULL REFERENCES persons(id),
  code_hash text NOT NULL,
  used_at timestamptz,
  PRIMARY KEY (person_id, code_hash)
);
SELECT eq_enable_rls('public.totp_recovery_codes'::regclass);
