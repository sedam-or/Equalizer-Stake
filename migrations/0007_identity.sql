CREATE TABLE credentials (
  person_id uuid PRIMARY KEY REFERENCES persons(id),
  tenant_id uuid NOT NULL REFERENCES tenants(id),
  password_hash text,
  password_changed_at timestamptz,
  totp_secret_enc bytea,
  totp_enabled boolean NOT NULL DEFAULT false,
  failed_count int NOT NULL DEFAULT 0,
  failed_window_start timestamptz,
  locked_until timestamptz
);
SELECT eq_enable_rls('public.credentials'::regclass);

CREATE TABLE refresh_sessions (
  id uuid PRIMARY KEY,
  tenant_id uuid NOT NULL REFERENCES tenants(id),
  person_id uuid NOT NULL REFERENCES persons(id),
  family_id uuid NOT NULL,
  token_hash bytea NOT NULL UNIQUE,
  active_role text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  last_used_at timestamptz NOT NULL DEFAULT now(),
  idle_expires_at timestamptz NOT NULL,
  absolute_expires_at timestamptz NOT NULL,
  revoked_at timestamptz,
  user_agent text,
  ip inet
);
SELECT eq_enable_rls('public.refresh_sessions'::regclass);

CREATE TABLE invites (
  id uuid PRIMARY KEY,
  tenant_id uuid NOT NULL REFERENCES tenants(id),
  person_id uuid NOT NULL REFERENCES persons(id),
  role text NOT NULL,
  token_hash bytea NOT NULL UNIQUE,
  expires_at timestamptz NOT NULL,
  accepted_at timestamptz,
  created_by uuid REFERENCES persons(id),
  created_at timestamptz NOT NULL DEFAULT now()
);
SELECT eq_enable_rls('public.invites'::regclass);

CREATE TABLE password_resets (
  id uuid PRIMARY KEY,
  tenant_id uuid NOT NULL REFERENCES tenants(id),
  person_id uuid NOT NULL REFERENCES persons(id),
  token_hash bytea NOT NULL UNIQUE,
  expires_at timestamptz NOT NULL,
  used_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now()
);
SELECT eq_enable_rls('public.password_resets'::regclass);
