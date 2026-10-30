use anyhow::{Context, Result, ensure};
use eq_core::actor::{ActorCtx, ActorRole};
use eq_core::auth::PgIdentityDirectory;
use eq_core::clock::Clock;
use eq_core::config::Config;
use eq_core::crypto::{Keyring, Keys};
use eq_core::db;
use eq_core::ids::{PersonId, TenantId};
use eq_core::model::Role;
use eq_core::policy::types::Locale;
use eq_identity::invites::{InviteChannel, InviteRequest, InviteService};
use eq_identity::policy::IdentityPolicy;
use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;

struct Fixture {
    owner_pool: PgPool,
    app_pool: PgPool,
    directory: PgIdentityDirectory,
    keyring: Keyring,
    tenant: TenantId,
    person_id: PersonId,
    actor: ActorCtx,
    clock: eq_testkit::clock::ManualClock,
    policy: IdentityPolicy,
    config: Config,
}

impl Fixture {
    async fn new() -> Result<Self> {
        let (config, owner_pool, app_pool) = connect_task_pools().await?;

        let tenant = TenantId(Uuid::now_v7());
        let actor_id = PersonId(Uuid::now_v7());
        let person_id = PersonId(Uuid::now_v7());
        seed_people(&owner_pool, tenant, actor_id, person_id).await?;

        let master = [0x42; 32];
        let keys = Arc::new(Keys::derive(&master, None));
        let keyring = Keyring::new(Keys::derive(&master, None));
        let directory = PgIdentityDirectory::new(keys);
        let actor = ActorCtx {
            person_id: Some(actor_id),
            role: ActorRole::Admin,
            ip: None,
            request_id: Some(Uuid::now_v7()),
        };

        Ok(Self {
            owner_pool,
            app_pool,
            directory,
            keyring,
            tenant,
            person_id,
            actor,
            clock: eq_testkit::clock::test_clock(),
            policy: IdentityPolicy::default(),
            config,
        })
    }

    fn service(&self) -> InviteService<'_> {
        InviteService::new(&self.directory, &self.keyring, &self.clock)
    }

    fn request(&self, channel: InviteChannel) -> InviteRequest<'_> {
        InviteRequest::new(
            self.tenant,
            &self.actor,
            self.person_id,
            Role::Instructor,
            channel,
            Locale::En,
            &self.config,
            &self.policy,
        )
    }
}

async fn connect_task_pools() -> Result<(Config, PgPool, PgPool)> {
    let config = Config::from_env().context("load task database configuration")?;
    ensure!(task_database(config.database_url.expose_secret()) == "ritma_test_T-110c");
    ensure!(task_database(config.database_owner_url.expose_secret()) == "ritma_test_T-110c");
    let owner_pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(config.database_owner_url.expose_secret())
        .await
        .context("connect to the task database owner")?;
    let app_pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(config.database_url.expose_secret())
        .await
        .context("connect to the task database application role")?;
    verify_app_pool(&app_pool).await?;
    Ok((config, owner_pool, app_pool))
}

fn task_database(url: &str) -> &str {
    url.rsplit('/').next().unwrap_or_default()
}

async fn verify_app_pool(pool: &PgPool) -> Result<()> {
    let role = sqlx::query(
        "SELECT current_database() AS database, current_user AS username, rolsuper, rolbypassrls \
         FROM pg_roles WHERE rolname = current_user",
    )
    .fetch_one(pool)
    .await?;
    ensure!(role.try_get::<String, _>("database")? == "ritma_test_T-110c");
    ensure!(role.try_get::<String, _>("username")? == "eq_app");
    ensure!(!role.try_get::<bool, _>("rolsuper")? && !role.try_get::<bool, _>("rolbypassrls")?);
    Ok(())
}

async fn seed_people(
    owner_pool: &PgPool,
    tenant: TenantId,
    actor_id: PersonId,
    person_id: PersonId,
) -> Result<()> {
    let slug = format!("invite-{}", tenant.0.simple());
    sqlx::query!(
        "INSERT INTO public.tenants (id, slug, name, country, preset_id, preset_version) \
         VALUES ($1, $2, 'Test School', 'ID', 'test', 1)",
        tenant.0,
        slug
    )
    .execute(owner_pool)
    .await?;
    sqlx::query!(
        "INSERT INTO public.persons (id, tenant_id, display_name, locale) VALUES ($1, $2, $3, $4)",
        actor_id.0,
        tenant.0,
        "Test Admin",
        Some("en")
    )
    .execute(owner_pool)
    .await?;
    sqlx::query!(
        "INSERT INTO public.persons (id, tenant_id, display_name, locale) VALUES ($1, $2, $3, $4)",
        person_id.0,
        tenant.0,
        "Test Invitee",
        Some("id")
    )
    .execute(owner_pool)
    .await?;
    sqlx::query!(
        "INSERT INTO public.memberships (tenant_id, person_id, role, status) \
         VALUES ($1, $2, 'admin', 'active')",
        tenant.0,
        actor_id.0
    )
    .execute(owner_pool)
    .await?;
    Ok(())
}

#[tokio::test]
async fn resend_revokes_previous() -> Result<()> {
    let fixture = Fixture::new().await?;
    let service = fixture.service();

    let mut first_tx = db::begin(&fixture.app_pool, fixture.tenant, &fixture.actor).await?;
    let first = service
        .create_invite(&mut first_tx, fixture.request(InviteChannel::Link))
        .await?;
    first_tx.commit().await?;
    let first_url = first
        .invite_url()
        .context("link channel must return the first invite URL")?
        .expose_secret()
        .to_owned();
    ensure!(first_url.contains("/id/invite/"));
    let first_debug = format!("{first:?}");
    ensure!(!first_debug.contains(&first_url));

    fixture.clock.advance(chrono::Duration::seconds(1));
    let mut second_tx = db::begin(&fixture.app_pool, fixture.tenant, &fixture.actor).await?;
    let second = service
        .create_invite(&mut second_tx, fixture.request(InviteChannel::Link))
        .await?;
    second_tx.commit().await?;
    let second_url = second
        .invite_url()
        .context("link channel must return the resent invite URL")?
        .expose_secret()
        .to_owned();
    ensure!(first_url != second_url);

    let invite_rows = sqlx::query!(
        "SELECT expires_at, accepted_at FROM public.invites \
         WHERE tenant_id = $1 AND person_id = $2 AND role = 'instructor' ORDER BY created_at, id",
        fixture.tenant.0,
        fixture.person_id.0
    )
    .fetch_all(&fixture.owner_pool)
    .await?;
    ensure!(invite_rows.len() == 2);
    ensure!(invite_rows[0].expires_at == fixture.clock.now());
    ensure!(invite_rows[0].accepted_at.is_none());
    ensure!(invite_rows[1].expires_at > fixture.clock.now());

    let membership = sqlx::query!(
        "SELECT status FROM public.memberships WHERE tenant_id = $1 AND person_id = $2 AND role = 'instructor'",
        fixture.tenant.0,
        fixture.person_id.0
    )
    .fetch_one(&fixture.owner_pool)
    .await?;
    ensure!(membership.status == "invited");
    Ok(())
}

#[tokio::test]
async fn invited_event_excludes_token() -> Result<()> {
    let fixture = Fixture::new().await?;
    let service = fixture.service();
    let mut tx = db::begin(&fixture.app_pool, fixture.tenant, &fixture.actor).await?;
    let invite = service
        .create_invite(&mut tx, fixture.request(InviteChannel::Email))
        .await?;
    ensure!(invite.invite_url().is_none());
    let whatsapp_invite = service
        .create_invite(&mut tx, fixture.request(InviteChannel::Whatsapp))
        .await?;
    ensure!(whatsapp_invite.invite_url().is_none());
    tx.commit().await?;

    let row = sqlx::query!(
        "SELECT event_type, payload FROM public.outbox \
         WHERE tenant_id = $1 AND aggregate_id = $2 ORDER BY occurred_at DESC LIMIT 1",
        fixture.tenant.0,
        fixture.person_id.0
    )
    .fetch_one(&fixture.owner_pool)
    .await?;
    ensure!(row.event_type == "identity.user_invited.v1");
    let payload = row.payload["payload"]
        .as_object()
        .context("event payload must be an object")?;
    ensure!(payload.len() == 3);
    ensure!(payload.contains_key("person_id"));
    ensure!(payload.contains_key("role"));
    ensure!(payload.contains_key("invite_id"));
    ensure!(!row.payload.to_string().contains("token"));
    Ok(())
}
