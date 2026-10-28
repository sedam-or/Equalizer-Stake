CREATE TABLE outbox (
  id             uuid PRIMARY KEY,
  tenant_id      uuid NOT NULL,
  event_type     text NOT NULL,
  aggregate_id   uuid,
  payload        jsonb NOT NULL,
  occurred_at    timestamptz NOT NULL,
  dispatched_at  timestamptz,
  attempts       int NOT NULL DEFAULT 0,
  last_error     text
);
CREATE INDEX outbox_pending ON outbox (id) WHERE dispatched_at IS NULL;

CREATE TABLE event_receipts (
  handler_id   text NOT NULL,
  event_id     uuid NOT NULL,
  processed_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (handler_id, event_id)
);

CREATE TABLE jobs (
  id             uuid PRIMARY KEY,
  tenant_id      uuid,
  kind           text NOT NULL,
  payload        jsonb NOT NULL DEFAULT '{}',
  run_at         timestamptz NOT NULL,
  status         text NOT NULL DEFAULT 'queued' CHECK (status IN ('queued','running','done','failed','dead')),
  attempts       int NOT NULL DEFAULT 0,
  max_attempts   int NOT NULL DEFAULT 5,
  locked_until   timestamptz,
  last_error     text,
  dedupe_key     text UNIQUE,
  created_at     timestamptz NOT NULL DEFAULT now(),
  finished_at    timestamptz
);
CREATE INDEX jobs_ready ON jobs (run_at) WHERE status = 'queued';

CREATE OR REPLACE FUNCTION public.notify_outbox_insert()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  PERFORM pg_catalog.pg_notify('eq_outbox', '');
  RETURN NULL;
END
$$;

CREATE TRIGGER outbox_notify_after_insert
AFTER INSERT ON public.outbox
FOR EACH STATEMENT
EXECUTE FUNCTION public.notify_outbox_insert();
