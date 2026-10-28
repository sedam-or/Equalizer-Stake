CREATE OR REPLACE FUNCTION public.resolve_tenant_preset(p_tenant_id uuid)
RETURNS TABLE(
  preset_id text,
  preset_version integer
)
LANGUAGE sql
STABLE
SECURITY DEFINER
SET search_path = pg_catalog, public
AS $$
  SELECT t.preset_id, t.preset_version
  FROM public.tenants AS t
  WHERE t.id = p_tenant_id
$$;

REVOKE ALL ON FUNCTION public.resolve_tenant_preset(uuid) FROM PUBLIC;

GRANT EXECUTE ON FUNCTION public.resolve_tenant_preset(uuid) TO eq_app;
