CREATE SCHEMA audit;

CREATE TABLE audit.entries (
  seq            bigserial PRIMARY KEY,
  id             uuid NOT NULL UNIQUE,
  tenant_id      uuid NOT NULL,
  at             timestamptz NOT NULL,
  actor_id       uuid,
  actor_role     text NOT NULL,
  action         text NOT NULL,
  entity_type    text NOT NULL,
  entity_id      uuid,
  category       text NOT NULL CHECK (category IN ('operational','security','compliance')),
  before         jsonb,
  after          jsonb,
  ip             inet,
  request_id     uuid,
  prev_hash      bytea,
  hash           bytea,
  sealed_at      timestamptz,
  anonymized_at  timestamptz,
  redaction_proof bytea
);

CREATE INDEX audit_entries_tenant_at ON audit.entries (tenant_id, at DESC);
CREATE INDEX audit_entries_entity ON audit.entries (tenant_id, entity_type, entity_id);
CREATE INDEX audit_entries_unsealed ON audit.entries (tenant_id, seq) WHERE hash IS NULL;

CREATE TABLE audit.checkpoints (
  tenant_id uuid NOT NULL,
  seq       bigint NOT NULL,
  hash      bytea NOT NULL,
  at        timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, seq)
);

ALTER TABLE audit.entries ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit.entries FORCE ROW LEVEL SECURITY;
CREATE POLICY audit_entries_tenant_isolation ON audit.entries
  USING (tenant_id = NULLIF(pg_catalog.current_setting('app.tenant_id', true), '')::uuid)
  WITH CHECK (tenant_id = NULLIF(pg_catalog.current_setting('app.tenant_id', true), '')::uuid);

CREATE FUNCTION audit.seal(p_tenant uuid, p_rows jsonb)
RETURNS void
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog, audit
AS $$
DECLARE
  item jsonb;
  row_seq bigint;
  previous_hash bytea;
  row_hash bytea;
  changed_rows bigint;
BEGIN
  IF p_tenant IS DISTINCT FROM NULLIF(pg_catalog.current_setting('app.tenant_id', true), '')::uuid THEN
    RAISE EXCEPTION 'audit tenant context mismatch' USING ERRCODE = '42501';
  END IF;
  IF pg_catalog.jsonb_typeof(p_rows) IS DISTINCT FROM 'array' THEN
    RAISE EXCEPTION 'audit seal rows must be an array' USING ERRCODE = '22023';
  END IF;

  PERFORM pg_catalog.pg_advisory_xact_lock(pg_catalog.hashtext('audit.seal:' || p_tenant::text));
  FOR item IN SELECT value FROM pg_catalog.jsonb_array_elements(p_rows) AS rows(value)
  LOOP
    row_seq := (item ->> 'seq')::bigint;
    previous_hash := pg_catalog.decode(item ->> 'prev_hash', 'hex');
    row_hash := pg_catalog.decode(item ->> 'hash', 'hex');
    IF pg_catalog.octet_length(previous_hash) <> 32 OR pg_catalog.octet_length(row_hash) <> 32 THEN
      RAISE EXCEPTION 'audit hashes must be SHA-256 values' USING ERRCODE = '22023';
    END IF;

    UPDATE audit.entries
       SET prev_hash = previous_hash,
           hash = row_hash,
           sealed_at = pg_catalog.statement_timestamp()
     WHERE tenant_id = p_tenant
       AND seq = row_seq
       AND hash IS NULL;
    GET DIAGNOSTICS changed_rows = ROW_COUNT;
    IF changed_rows <> 1 THEN
      RAISE EXCEPTION 'audit row is missing or already sealed' USING ERRCODE = '40001';
    END IF;
  END LOOP;
END
$$;

CREATE FUNCTION audit._redact_jsonb(p_value jsonb)
RETURNS jsonb
LANGUAGE plpgsql
IMMUTABLE
STRICT
SET search_path = pg_catalog, audit
AS $$
DECLARE
  item record;
  result jsonb;
  redacted text;
BEGIN
  CASE pg_catalog.jsonb_typeof(p_value)
    WHEN 'object' THEN
      result := '{}'::jsonb;
      FOR item IN SELECT key, value FROM pg_catalog.jsonb_each(p_value)
      LOOP
        IF item.key = ANY (ARRAY['display_name','email','phone','name','legal_name','address','note','comment','reason']) THEN
          redacted := 'REDACTED-' || pg_catalog.left(pg_catalog.encode(public.digest(item.value::text::bytea, 'sha256'), 'hex'), 16);
          result := result || pg_catalog.jsonb_build_object(item.key, redacted);
        ELSE
          result := result || pg_catalog.jsonb_build_object(item.key, audit._redact_jsonb(item.value));
        END IF;
      END LOOP;
      RETURN result;
    WHEN 'array' THEN
      SELECT COALESCE(
        pg_catalog.jsonb_agg(audit._redact_jsonb(elements.value) ORDER BY elements.ordinality),
        '[]'::jsonb
      )
        INTO result
        FROM pg_catalog.jsonb_array_elements(p_value) WITH ORDINALITY AS elements(value, ordinality);
      RETURN result;
    ELSE
      RETURN p_value;
  END CASE;
END
$$;

CREATE FUNCTION audit.anonymize(p_tenant uuid, p_before timestamptz, p_subject uuid DEFAULT NULL)
RETURNS void
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog, audit
AS $$
BEGIN
  IF p_tenant IS DISTINCT FROM NULLIF(pg_catalog.current_setting('app.tenant_id', true), '')::uuid THEN
    RAISE EXCEPTION 'audit tenant context mismatch' USING ERRCODE = '42501';
  END IF;
  PERFORM pg_catalog.pg_advisory_xact_lock(pg_catalog.hashtext('audit.seal:' || p_tenant::text));

  UPDATE audit.entries
     SET redaction_proof = COALESCE(
           redaction_proof,
           public.digest(COALESCE("before"::text, '') || COALESCE("after"::text, ''), 'sha256')
         ),
         "before" = audit._redact_jsonb("before"),
         "after" = audit._redact_jsonb("after"),
         ip = NULL,
         anonymized_at = pg_catalog.statement_timestamp()
   WHERE tenant_id = p_tenant
     AND anonymized_at IS NULL
     AND (at < p_before OR (p_subject IS NOT NULL AND (actor_id = p_subject OR entity_id = p_subject)));
END
$$;

CREATE FUNCTION audit.purge_expired(p_tenant uuid, p_before timestamptz)
RETURNS void
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog, audit
AS $$
DECLARE
  last_seq bigint;
  last_hash bytea;
BEGIN
  IF p_tenant IS DISTINCT FROM NULLIF(pg_catalog.current_setting('app.tenant_id', true), '')::uuid THEN
    RAISE EXCEPTION 'audit tenant context mismatch' USING ERRCODE = '42501';
  END IF;
  PERFORM pg_catalog.pg_advisory_xact_lock(pg_catalog.hashtext('audit.seal:' || p_tenant::text));

  IF EXISTS (
    SELECT 1 FROM audit.entries
     WHERE tenant_id = p_tenant AND at < p_before AND hash IS NULL
  ) THEN
    RAISE EXCEPTION 'cannot purge unsealed audit entries' USING ERRCODE = '55000';
  END IF;

  SELECT seq, hash INTO last_seq, last_hash
    FROM audit.entries
   WHERE tenant_id = p_tenant AND at < p_before
   ORDER BY seq DESC
   LIMIT 1;

  IF last_seq IS NOT NULL THEN
    INSERT INTO audit.checkpoints (tenant_id, seq, hash)
    VALUES (p_tenant, last_seq, last_hash)
    ON CONFLICT (tenant_id, seq) DO UPDATE SET hash = EXCLUDED.hash;
  END IF;

  DELETE FROM audit.entries WHERE tenant_id = p_tenant AND at < p_before;
END
$$;

REVOKE ALL ON TABLE audit.entries FROM PUBLIC;
REVOKE ALL ON TABLE audit.checkpoints FROM PUBLIC;
REVOKE ALL ON FUNCTION audit.seal(uuid, jsonb) FROM PUBLIC;
REVOKE ALL ON FUNCTION audit._redact_jsonb(jsonb) FROM PUBLIC;
REVOKE ALL ON FUNCTION audit.anonymize(uuid, timestamptz, uuid) FROM PUBLIC;
REVOKE ALL ON FUNCTION audit.purge_expired(uuid, timestamptz) FROM PUBLIC;

DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_catalog.pg_roles WHERE rolname = 'eq_app') THEN
    GRANT USAGE ON SCHEMA audit TO eq_app;
    GRANT SELECT, INSERT ON TABLE audit.entries TO eq_app;
    GRANT SELECT ON TABLE audit.checkpoints TO eq_app;
    GRANT USAGE ON SEQUENCE audit.entries_seq_seq TO eq_app;
    REVOKE UPDATE, DELETE ON TABLE audit.entries FROM eq_app;
    GRANT EXECUTE ON FUNCTION audit.seal(uuid, jsonb) TO eq_app;
    GRANT EXECUTE ON FUNCTION audit.anonymize(uuid, timestamptz, uuid) TO eq_app;
    GRANT EXECUTE ON FUNCTION audit.purge_expired(uuid, timestamptz) TO eq_app;
  END IF;
END
$$;
