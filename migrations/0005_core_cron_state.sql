CREATE TABLE cron_state (
  tenant_id     uuid NOT NULL REFERENCES tenants(id),
  kind          text NOT NULL,
  last_fire_at  timestamptz NOT NULL,
  PRIMARY KEY (tenant_id, kind)
);
