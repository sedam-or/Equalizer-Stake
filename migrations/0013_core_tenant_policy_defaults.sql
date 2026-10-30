CREATE OR REPLACE FUNCTION public.update_tenant_policy_defaults(
  p_tenant_id uuid,
  p_locale text,
  p_currency char(3),
  p_timezone text
)
RETURNS void
LANGUAGE plpgsql
VOLATILE
SECURITY DEFINER
SET search_path = pg_catalog
AS $$
DECLARE
  transaction_tenant text;
BEGIN
  transaction_tenant := NULLIF(pg_catalog.current_setting('app.tenant_id', true), '');
  IF transaction_tenant IS NULL OR transaction_tenant::uuid <> p_tenant_id THEN
    RAISE EXCEPTION 'tenant context mismatch' USING ERRCODE = '42501';
  END IF;

  UPDATE public.tenants
  SET locale = p_locale,
      currency = p_currency,
      timezone = p_timezone
  WHERE id = p_tenant_id;

  IF NOT FOUND THEN
    RAISE EXCEPTION 'tenant not found' USING ERRCODE = 'P0002';
  END IF;
END
$$;

REVOKE ALL ON FUNCTION public.update_tenant_policy_defaults(uuid, text, char(3), text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.update_tenant_policy_defaults(uuid, text, char(3), text) TO eq_app;
