CREATE TABLE policies (
  tenant_id       uuid NOT NULL REFERENCES tenants(id),
  module_id       text NOT NULL,
  version         int NOT NULL CHECK (version >= 1),
  schema_version  int NOT NULL,
  effective_from  timestamptz NOT NULL,
  document        jsonb NOT NULL,
  note            text,
  created_by      uuid REFERENCES persons(id),
  created_at      timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, module_id, version)
);
CREATE INDEX policies_lookup ON policies (tenant_id, module_id, effective_from DESC);
SELECT eq_enable_rls('public.policies'::regclass);

DO $$
BEGIN
  REVOKE UPDATE, DELETE ON TABLE public.policies FROM eq_app;
EXCEPTION
  WHEN undefined_object THEN
    RAISE NOTICE 'skipping append-only privileges (eq_app role is unavailable)';
END
$$;

CREATE TABLE files (
  id            uuid PRIMARY KEY,
  tenant_id     uuid NOT NULL REFERENCES tenants(id),
  purpose       text NOT NULL CHECK (purpose IN ('avatar','checkin_photo','payment_proof','registration_doc','export','import','logo')),
  content_type  text NOT NULL,
  size_bytes    bigint NOT NULL,
  sha256        bytea NOT NULL,
  storage_key   text NOT NULL,
  encrypted     boolean NOT NULL,
  created_by    uuid REFERENCES persons(id),
  created_at    timestamptz NOT NULL DEFAULT now(),
  deleted_at    timestamptz
);
SELECT eq_enable_rls('public.files'::regclass);

CREATE TABLE drafts (
  id          uuid PRIMARY KEY,
  tenant_id   uuid NOT NULL REFERENCES tenants(id),
  person_id   uuid NOT NULL REFERENCES persons(id),
  form_key    text NOT NULL,
  payload     jsonb NOT NULL,
  updated_at  timestamptz NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, person_id, form_key)
);
SELECT eq_enable_rls('public.drafts'::regclass);

CREATE TABLE attention_items (
  id           uuid PRIMARY KEY,
  tenant_id    uuid NOT NULL REFERENCES tenants(id),
  module_id    text NOT NULL,
  kind         text NOT NULL,
  ref_type     text,
  ref_id       uuid,
  payload      jsonb NOT NULL DEFAULT '{}',
  severity     text NOT NULL DEFAULT 'normal' CHECK (severity IN ('low','normal','high')),
  dedupe_key   text,
  created_at   timestamptz NOT NULL DEFAULT now(),
  resolved_at  timestamptz,
  resolved_by  uuid REFERENCES persons(id),
  UNIQUE (tenant_id, dedupe_key)
);
CREATE INDEX attention_open ON attention_items (tenant_id, created_at DESC) WHERE resolved_at IS NULL;
SELECT eq_enable_rls('public.attention_items'::regclass);

CREATE TABLE consents (
  tenant_id  uuid NOT NULL REFERENCES tenants(id),
  person_id  uuid NOT NULL REFERENCES persons(id),
  kind       text NOT NULL CHECK (kind IN ('registration','terms','privacy')),
  text_hash  bytea NOT NULL,
  at         timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, person_id, kind, text_hash)
);
SELECT eq_enable_rls('public.consents'::regclass);

CREATE TABLE tenant_keys (
  tenant_id   uuid NOT NULL REFERENCES tenants(id),
  key_version int NOT NULL,
  dek_enc     bytea NOT NULL,
  created_at  timestamptz NOT NULL DEFAULT now(),
  retired_at  timestamptz,
  PRIMARY KEY (tenant_id, key_version)
);

CREATE TABLE idempotency_keys (
  tenant_id    uuid NOT NULL REFERENCES tenants(id),
  key          text NOT NULL,
  route        text NOT NULL,
  request_hash bytea NOT NULL,
  status       int NOT NULL,
  body         jsonb NOT NULL,
  created_at   timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, key)
);
SELECT eq_enable_rls('public.idempotency_keys'::regclass);
