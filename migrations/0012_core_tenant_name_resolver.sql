CREATE OR REPLACE FUNCTION public.resolve_tenant_name(p_tenant_id uuid)
RETURNS text
LANGUAGE sql
STABLE
SECURITY DEFINER
SET search_path = pg_catalog, public
AS $$
  SELECT t.name
  FROM public.tenants AS t
  WHERE t.id = p_tenant_id
    AND t.id = NULLIF(pg_catalog.current_setting('app.tenant_id', true), '')::uuid
$$;

REVOKE ALL ON FUNCTION public.resolve_tenant_name(uuid) FROM PUBLIC;

DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_catalog.pg_roles WHERE rolname = 'eq_app') THEN
    EXECUTE 'GRANT EXECUTE ON FUNCTION public.resolve_tenant_name(uuid) TO eq_app';
  END IF;
END
$$;
