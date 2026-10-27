CREATE OR REPLACE FUNCTION public.resolve_tenant_info_by_slug(p_slug text)
RETURNS TABLE(
  id uuid,
  slug text,
  status text,
  timezone text,
  locale text,
  currency text
)
LANGUAGE sql
STABLE
SECURITY DEFINER
SET search_path = pg_catalog
AS $$
  SELECT t.id, t.slug, t.status, t.timezone, t.locale, t.currency::text
  FROM public.tenants AS t
  WHERE t.slug = p_slug
$$;

REVOKE ALL ON FUNCTION public.resolve_tenant_info_by_slug(text) FROM PUBLIC;

DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_catalog.pg_roles WHERE rolname = 'eq_app') THEN
    EXECUTE 'GRANT EXECUTE ON FUNCTION public.resolve_tenant_info_by_slug(text) TO eq_app';
  END IF;
END
$$;
