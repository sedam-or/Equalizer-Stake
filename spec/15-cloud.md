# 15 — Managed Cloud

The cloud is **the same AGPL code** running with `RITMA_MODE=multi`, plus a small **platform** layer (signup, plans, metering, subscription billing) that lives in this repo under `crates/eq-platform` (feature `platform`, off by default). The source stays open (AGPL §13 compliance), and the moat is operations, not code.

## 1. Tenancy and routing

- **CLD-01.** Each school is one tenant reached at `https://<slug>.app.<domain>`. Custom domains are a paid plan: CNAME to `edge.<domain>`, with on-demand TLS through Caddy's `on_demand_tls` and an `ask` endpoint `GET /platform/domain-check?domain=`.
- **CLD-02.** Tenant resolution comes from `Host` (ARC-04 step 3). Unknown hosts get the platform 404 page.
- **CLD-03.** Isolation is Postgres RLS (SEC-05) in a shared database for the free and standard plans. **Dedicated DB** is an enterprise option: a separate PG database and the same image with `RITMA_MODE=single`.

## 2. Signup and onboarding

- **CLD-10.** Flow: `app.<domain>/signup` → email (magic link) or Google OIDC → **school slug and region** → the 3-step setup wizard (05), unchanged → dashboard.
  - "Import from file" in step 1 accepts a desktop or self-host `.eqbak` plus its key, or the spreadsheet import (DST-41).
- **CLD-11.** The platform tables (`platform` schema, no RLS, role `eq_platform`):
  - `accounts(id, email, created_at)` (the signup identity)
  - `account_tenants(account_id, tenant_id, role)`
  - `plans(id, name, limits jsonb, price jsonb)`
  - `subscriptions(tenant_id, plan_id, status, provider, provider_ref, current_period_end, trial_ends_at)`
  - `usage_snapshots(tenant_id, day, active_students, sessions, notifications_sent, storage_bytes)`
- **CLD-12.** The trial is 30 days on the Standard plan, then it downgrades to Free if no payment method is added. A downgrade never deletes data. Above the free limits the tenant becomes **read-only for new enrollments**; existing operations keep working for 30 days with banners.

## 3. Plans (draft; final pricing in `brand/GTM.md`)

| Plan | Active students | Included | Price (draft) |
|---|---|---|---|
| Free | ≤ 25 | All modules, in-app + email (platform sender, 500/mo), 1 GB files | 0 |
| Standard | ≤ 300 | Everything + own WhatsApp channel + custom sender email + daily off-site backups + priority email support | IDR 149k/mo or USD 12/mo |
| Pro | ≤ 1000 | + custom domain, 3 locations, audit export API, SSO (Google Workspace) | IDR 399k/mo or USD 29/mo |
| Enterprise | custom | Dedicated DB, region pinning, DPA, SLA 99.9% | quote |

"Active student" = a person with an `active` or `on_leave` enrollment, or any session participation in the last 30 days. It is computed daily into `usage_snapshots`.

## 4. Subscription billing

- **CLD-20.** A `SubscriptionProvider` trait has implementations **Xendit** (IDR: VA, QRIS, e-wallet) and **Stripe** (cards, global). These are platform billing, separate from the school's own billing module.
- **CLD-21.** Webhooks update `subscriptions`. Failed payments get dunning emails (3, 7, 14 days), then a downgrade to Free (limits apply). Nothing is ever deleted.

## 5. Operations

- **CLD-30. Regions.** `id-jkt` (ap-southeast-3, default for Indonesia) and `sg`. Data stays in the chosen region, including backups.
- **CLD-31. Infra (initial):**
  - 2 app instances behind a load balancer (stateless; files in S3-compatible storage; one cron planner per instance is safe through dedupe keys).
  - Managed Postgres 16 with PITR (RPO ≤ 5 min).
  - Nightly logical backups per tenant (CLD-33) for tenant-level restores.
- **CLD-32.** Platform email via a transactional provider (SES/Postmark) through `RITMA_SMTP_URL`, with per-tenant rate limits.
- **CLD-33.** Per-tenant export and restore for support: `ritma backup --tenant <id>` produces a tenant-scoped dump (COPY of tenant rows by table order plus files). Restoring into a new tenant id remaps ids.
- **CLD-34. SLA targets:** 99.5% (Standard), 99.9% (Enterprise). Status page at `status.<domain>`.
- **CLD-35. Abuse and safety.** Signup rate limits, disposable-email blocking, suspend switch (`tenants.status = suspended` → read-only plus a banner), and a takedown process in `docs/legal/`.

## 6. AGPL compliance in hosted mode

- **CLD-40.** Every page footer links "Source code" to the exact git tag running (from `meta/app.commit`). Any platform modifications are published in the same repo (feature `platform`). There is no closed fork.
