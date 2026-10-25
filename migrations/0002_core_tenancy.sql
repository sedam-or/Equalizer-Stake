CREATE TABLE tenants (
  id                  uuid PRIMARY KEY,
  slug                text NOT NULL UNIQUE CHECK (slug ~ '^[a-z0-9](-?[a-z0-9])*$' AND length(slug) BETWEEN 3 AND 40),
  name                text NOT NULL CHECK (length(name) BETWEEN 2 AND 80),
  country             char(2) NOT NULL,
  preset_id           text NOT NULL,
  preset_version      int  NOT NULL,
  setup_step          int  NOT NULL DEFAULT 1 CHECK (setup_step BETWEEN 1 AND 4),
  setup_completed_at  timestamptz,
  status              text NOT NULL DEFAULT 'active' CHECK (status IN ('active','suspended','closed')),
  locale              text NOT NULL DEFAULT 'en',
  currency            char(3) NOT NULL DEFAULT 'USD',
  timezone            text NOT NULL DEFAULT 'UTC',
  created_at          timestamptz NOT NULL DEFAULT now()
);

CREATE OR REPLACE FUNCTION public.resolve_tenant_by_slug(p_slug text)
RETURNS TABLE(id uuid, slug text, status text)
LANGUAGE sql
STABLE
SECURITY DEFINER
SET search_path = pg_catalog
AS $$
  SELECT t.id, t.slug, t.status
  FROM public.tenants AS t
  WHERE t.slug = p_slug
$$;

CREATE OR REPLACE FUNCTION public.resolve_single_tenant()
RETURNS TABLE(id uuid, slug text, status text)
LANGUAGE plpgsql
STABLE
SECURITY DEFINER
SET search_path = pg_catalog
AS $$
DECLARE
  tenant_count bigint;
BEGIN
  SELECT pg_catalog.count(*)
  INTO tenant_count
  FROM public.tenants;

  IF tenant_count > 1 THEN
    RAISE EXCEPTION 'resolve_single_tenant requires at most one tenant'
      USING ERRCODE = '21000';
  END IF;

  RETURN QUERY
  SELECT t.id, t.slug, t.status
  FROM public.tenants AS t;
END
$$;

REVOKE ALL ON TABLE public.tenants FROM PUBLIC;
REVOKE ALL ON FUNCTION public.resolve_tenant_by_slug(text) FROM PUBLIC;
REVOKE ALL ON FUNCTION public.resolve_single_tenant() FROM PUBLIC;

DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_catalog.pg_roles WHERE rolname = 'eq_app') THEN
    EXECUTE 'REVOKE ALL ON TABLE public.tenants FROM eq_app';
    EXECUTE 'GRANT EXECUTE ON FUNCTION public.resolve_tenant_by_slug(text) TO eq_app';
    EXECUTE 'GRANT EXECUTE ON FUNCTION public.resolve_single_tenant() TO eq_app';
  END IF;
END
$$;

CREATE TABLE locations (
  id          uuid PRIMARY KEY,
  tenant_id   uuid NOT NULL REFERENCES tenants(id),
  name        text NOT NULL,
  address     text,
  timezone    text,
  geo_lat     double precision CHECK (geo_lat BETWEEN -90 AND 90),
  geo_lng     double precision CHECK (geo_lng BETWEEN -180 AND 180),
  archived_at timestamptz,
  created_at  timestamptz NOT NULL DEFAULT now(),
  CHECK ((geo_lat IS NULL) = (geo_lng IS NULL))
);
SELECT eq_enable_rls('public.locations'::regclass);

CREATE TABLE resources (
  id                  uuid PRIMARY KEY,
  tenant_id           uuid NOT NULL REFERENCES tenants(id),
  location_id         uuid NOT NULL REFERENCES locations(id),
  name                text NOT NULL,
  kind                text NOT NULL DEFAULT 'room' CHECK (kind IN ('room','equipment','other')),
  capacity            int NOT NULL DEFAULT 1 CHECK (capacity BETWEEN 1 AND 500),
  rentable            boolean NOT NULL DEFAULT false,
  color               text,
  sort                int NOT NULL DEFAULT 0,
  archived_at         timestamptz,
  created_at          timestamptz NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, name)
);
SELECT eq_enable_rls('public.resources'::regclass);

CREATE TABLE persons (
  id              uuid PRIMARY KEY,
  tenant_id       uuid NOT NULL REFERENCES tenants(id),
  display_name    text NOT NULL CHECK (length(display_name) BETWEEN 1 AND 80),
  legal_name_enc  bytea,
  email_enc       bytea,
  email_bidx      bytea,
  phone_enc       bytea,
  phone_bidx      bytea,
  birth_date_enc  bytea,
  address_enc     bytea,
  locale          text,
  avatar_file_id  uuid,
  anonymized_at   timestamptz,
  deleted_at      timestamptz,
  created_at      timestamptz NOT NULL DEFAULT now(),
  updated_at      timestamptz NOT NULL DEFAULT now(),
  version         int NOT NULL DEFAULT 1
);
CREATE UNIQUE INDEX persons_email_unique ON persons (tenant_id, email_bidx) WHERE email_bidx IS NOT NULL AND deleted_at IS NULL;
CREATE INDEX persons_phone ON persons (tenant_id, phone_bidx);
CREATE INDEX persons_name_trgm ON persons USING gin (display_name gin_trgm_ops);
SELECT eq_enable_rls('public.persons'::regclass);

CREATE TABLE memberships (
  tenant_id   uuid NOT NULL REFERENCES tenants(id),
  person_id   uuid NOT NULL REFERENCES persons(id),
  role        text NOT NULL CHECK (role IN ('owner','admin','instructor','guardian','student')),
  status      text NOT NULL DEFAULT 'active' CHECK (status IN ('invited','active','deactivated')),
  created_at  timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, person_id, role)
);
SELECT eq_enable_rls('public.memberships'::regclass);

CREATE TABLE person_links (
  tenant_id     uuid NOT NULL REFERENCES tenants(id),
  guardian_id   uuid NOT NULL REFERENCES persons(id),
  student_id    uuid NOT NULL REFERENCES persons(id),
  relationship  text,
  is_primary    boolean NOT NULL DEFAULT true,
  created_at    timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, guardian_id, student_id),
  CHECK (guardian_id <> student_id)
);
CREATE UNIQUE INDEX person_links_one_primary ON person_links (tenant_id, student_id) WHERE is_primary;
SELECT eq_enable_rls('public.person_links'::regclass);

CREATE TABLE tenant_modules (
  tenant_id   uuid NOT NULL REFERENCES tenants(id),
  module_id   text NOT NULL,
  enabled     boolean NOT NULL,
  changed_at  timestamptz NOT NULL DEFAULT now(),
  changed_by  uuid REFERENCES persons(id),
  PRIMARY KEY (tenant_id, module_id)
);
SELECT eq_enable_rls('public.tenant_modules'::regclass);

CREATE TABLE closures (
  id              uuid PRIMARY KEY,
  tenant_id       uuid NOT NULL REFERENCES tenants(id),
  location_id     uuid REFERENCES locations(id),
  dates           daterange NOT NULL CHECK (NOT isempty(dates)),
  name            jsonb NOT NULL,
  kind            text NOT NULL DEFAULT 'holiday' CHECK (kind IN ('holiday','closure')),
  created_at      timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX closures_dates ON closures USING gist (tenant_id, dates);
SELECT eq_enable_rls('public.closures'::regclass);
