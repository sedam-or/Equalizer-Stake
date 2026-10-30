//! Core-owned person and membership lookup for Identity (CA-30, ERD-02).

use crate::crypto::{Aad, Keyring, Keys, bidx_email};
use crate::db::Tx;
use crate::error::AppError;
use crate::fields::Email;
use crate::ids::PersonId;
use crate::model::Role;
use crate::policy::types::Locale;
use async_trait::async_trait;
use secrecy::SecretString;
use sqlx::FromRow;
use std::fmt;
use std::sync::Arc;
use uuid::Uuid;

/// IDN-01/CA-30: only the core-owned identity facts needed by authentication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginIdentity {
    person_id: PersonId,
    roles: Vec<Role>,
}

/// SBD-10/ERD-01: core-owned profile facts returned only to the authenticated person.
pub struct LoginProfile {
    pub display_name: String,
    pub email: Option<SecretString>,
    pub phone: Option<SecretString>,
    pub locale: Option<Locale>,
}

impl fmt::Debug for LoginProfile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LoginProfile")
            .field("display_name", &"[REDACTED]")
            .field("email", &"[REDACTED]")
            .field("phone", &"[REDACTED]")
            .field("locale", &self.locale)
            .finish()
    }
}

impl LoginIdentity {
    /// Returns the core-owned person identifier.
    pub const fn person_id(&self) -> PersonId {
        self.person_id
    }

    /// Returns active tenant roles in the IDN-09 privilege order.
    pub fn roles(&self) -> &[Role] {
        &self.roles
    }
}

/// CA-30: Identity queries core-owned membership facts through this port.
#[async_trait]
pub trait CoreIdentityDirectory: Send + Sync {
    /// IDN-01/SEC-04: resolves an active person and roles by canonical email in the caller tenant.
    async fn by_email(&self, tx: &mut Tx, email: &Email) -> Result<Option<LoginIdentity>, AppError>;

    /// IDN-10/11: checks whether the tenant has any active admin membership.
    async fn has_active_admin(&self, tx: &mut Tx, tenant: crate::ids::TenantId) -> Result<bool, AppError>;

    /// IDN-03/09: checks whether this tenant role is still an active membership.
    async fn has_active_role(
        &self,
        tx: &mut Tx,
        tenant: crate::ids::TenantId,
        person: PersonId,
        role: Role,
    ) -> Result<bool, AppError>;

    /// STM-20/CA-30: inserts the invited membership through Core ownership without changing an existing state.
    async fn ensure_invited_membership(
        &self,
        tx: &mut Tx,
        tenant: crate::ids::TenantId,
        person: PersonId,
        role: Role,
    ) -> Result<(), AppError>;

    /// STM-20/21: activates one invited membership and returns the person's active roles.
    async fn activate_invited_membership(
        &self,
        tx: &mut Tx,
        tenant: crate::ids::TenantId,
        person: PersonId,
        role: Role,
    ) -> Result<Option<LoginIdentity>, AppError>;

    /// TD-138/ERD-01: resolves the caller tenant's display name through the tenant-scoped function.
    async fn tenant_name(
        &self,
        tx: &mut Tx,
        tenant: crate::ids::TenantId,
    ) -> Result<Option<String>, AppError>;

    /// ERD-02/SBD-10: reads the caller's core-owned profile and decrypts PII only when authorized.
    async fn login_profile(
        &self,
        tx: &mut Tx,
        tenant: crate::ids::TenantId,
        person: PersonId,
        include_pii: bool,
        keyring: &Keyring,
    ) -> Result<Option<LoginProfile>, AppError>;
}

/// SEC-04: PostgreSQL adapter for the core-owned identity directory port.
pub struct PgIdentityDirectory {
    keys: Arc<Keys>,
}

impl PgIdentityDirectory {
    /// SEC-04: creates the directory adapter with process-derived blind-index keys.
    pub fn new(keys: Arc<Keys>) -> Self {
        Self { keys }
    }
}

impl fmt::Debug for PgIdentityDirectory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PgIdentityDirectory")
            .field("keys", &"[REDACTED]")
            .finish()
    }
}

#[derive(FromRow)]
struct LoginIdentityRow {
    person_id: Uuid,
    role: Role,
}

#[async_trait]
impl CoreIdentityDirectory for PgIdentityDirectory {
    async fn by_email(&self, tx: &mut Tx, email: &Email) -> Result<Option<LoginIdentity>, AppError> {
        let tenant = tx.tenant();
        let email_bidx = bidx_email(&self.keys, tenant, email.as_ref())
            .map_err(|error| AppError::Internal(error.into()))?;
        let rows = sqlx::query_as!(
            LoginIdentityRow,
            r#"SELECT p.id AS "person_id!: Uuid", m.role AS "role!: Role"
               FROM public.persons AS p
               JOIN public.memberships AS m
                 ON m.tenant_id = p.tenant_id AND m.person_id = p.id
               WHERE p.tenant_id = $1
                 AND p.email_bidx = $2
                 AND p.deleted_at IS NULL
                 AND m.status = 'active'
               ORDER BY CASE m.role
                 WHEN 'owner' THEN 0
                 WHEN 'admin' THEN 1
                 WHEN 'instructor' THEN 2
                 WHEN 'guardian' THEN 3
                 WHEN 'student' THEN 4
               END"#,
            tenant.0,
            email_bidx.as_slice()
        )
        .fetch_all(tx.conn())
        .await?;

        let Some(person_id) = rows.first().map(|row| PersonId(row.person_id)) else {
            return Ok(None);
        };
        let roles = rows.into_iter().map(|row| row.role).collect();
        Ok(Some(LoginIdentity { person_id, roles }))
    }

    async fn has_active_admin(&self, tx: &mut Tx, tenant: crate::ids::TenantId) -> Result<bool, AppError> {
        if tx.tenant() != tenant {
            return Err(AppError::Internal(anyhow::anyhow!(
                "identity tenant does not match transaction context"
            )));
        }
        let row = sqlx::query!(
            "SELECT EXISTS (SELECT 1 FROM public.memberships \
             WHERE tenant_id = $1 AND role = 'admin' AND status = 'active') AS \"has_admin!\"",
            tenant.0
        )
        .fetch_one(tx.conn())
        .await?;
        Ok(row.has_admin)
    }

    async fn has_active_role(
        &self,
        tx: &mut Tx,
        tenant: crate::ids::TenantId,
        person: PersonId,
        role: Role,
    ) -> Result<bool, AppError> {
        if tx.tenant() != tenant {
            return Err(AppError::Internal(anyhow::anyhow!(
                "identity tenant does not match transaction context"
            )));
        }
        let role = match role {
            Role::Owner => "owner",
            Role::Admin => "admin",
            Role::Instructor => "instructor",
            Role::Guardian => "guardian",
            Role::Student => "student",
        };
        let row = sqlx::query!(
            "SELECT EXISTS (SELECT 1 FROM public.memberships \
             WHERE tenant_id = $1 AND person_id = $2 AND role = $3 AND status = 'active') AS \"is_active!\"",
            tenant.0,
            person.0,
            role
        )
        .fetch_one(tx.conn())
        .await?;
        Ok(row.is_active)
    }

    async fn ensure_invited_membership(
        &self,
        tx: &mut Tx,
        tenant: crate::ids::TenantId,
        person: PersonId,
        role: Role,
    ) -> Result<(), AppError> {
        if tx.tenant() != tenant {
            return Err(AppError::Internal(anyhow::anyhow!(
                "identity tenant does not match transaction context"
            )));
        }
        sqlx::query!(
            "INSERT INTO public.memberships (tenant_id, person_id, role, status) \
             VALUES ($1, $2, $3, 'invited') \
             ON CONFLICT (tenant_id, person_id, role) DO NOTHING",
            tenant.0,
            person.0,
            role as Role
        )
        .execute(tx.conn())
        .await?;
        Ok(())
    }

    async fn activate_invited_membership(
        &self,
        tx: &mut Tx,
        tenant: crate::ids::TenantId,
        person: PersonId,
        role: Role,
    ) -> Result<Option<LoginIdentity>, AppError> {
        if tx.tenant() != tenant {
            return Err(AppError::Internal(anyhow::anyhow!(
                "identity tenant does not match transaction context"
            )));
        }
        let changed = sqlx::query!(
            "UPDATE public.memberships SET status = 'active' \
             WHERE tenant_id = $1 AND person_id = $2 AND role = $3 AND status = 'invited'",
            tenant.0,
            person.0,
            role as Role
        )
        .execute(tx.conn())
        .await?;
        if changed.rows_affected() != 1 {
            return Ok(None);
        }

        let rows = sqlx::query_as!(
            LoginIdentityRow,
            r#"SELECT person_id AS "person_id!: Uuid", role AS "role!: Role"
               FROM public.memberships
               WHERE tenant_id = $1 AND person_id = $2 AND status = 'active'
               ORDER BY CASE role
                 WHEN 'owner' THEN 0
                 WHEN 'admin' THEN 1
                 WHEN 'instructor' THEN 2
                 WHEN 'guardian' THEN 3
                 WHEN 'student' THEN 4
               END"#,
            tenant.0,
            person.0
        )
        .fetch_all(tx.conn())
        .await?;
        if rows.is_empty() {
            return Err(AppError::Internal(anyhow::anyhow!(
                "activated identity has no active membership"
            )));
        }
        Ok(Some(LoginIdentity {
            person_id: person,
            roles: rows.into_iter().map(|row| row.role).collect(),
        }))
    }

    async fn tenant_name(
        &self,
        tx: &mut Tx,
        tenant: crate::ids::TenantId,
    ) -> Result<Option<String>, AppError> {
        if tx.tenant() != tenant {
            return Err(AppError::Internal(anyhow::anyhow!(
                "identity tenant does not match transaction context"
            )));
        }
        Ok(
            sqlx::query_scalar!("SELECT public.resolve_tenant_name($1)", tenant.0)
                .fetch_one(tx.conn())
                .await?,
        )
    }

    async fn login_profile(
        &self,
        tx: &mut Tx,
        tenant: crate::ids::TenantId,
        person: PersonId,
        include_pii: bool,
        keyring: &Keyring,
    ) -> Result<Option<LoginProfile>, AppError> {
        if tx.tenant() != tenant {
            return Err(AppError::Internal(anyhow::anyhow!(
                "identity tenant does not match transaction context"
            )));
        }
        let row = sqlx::query!(
            "SELECT display_name, email_enc, phone_enc, locale \
             FROM public.persons \
             WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL",
            tenant.0,
            person.0
        )
        .fetch_optional(tx.conn())
        .await?;
        let Some(row) = row else {
            return Ok(None);
        };

        let email = if include_pii {
            match row.email_enc {
                Some(ciphertext) => {
                    Some(decrypt_person_field(keyring, tx, tenant, person, "email_enc", &ciphertext).await?)
                }
                None => None,
            }
        } else {
            None
        };
        let phone = if include_pii {
            match row.phone_enc {
                Some(ciphertext) => {
                    Some(decrypt_person_field(keyring, tx, tenant, person, "phone_enc", &ciphertext).await?)
                }
                None => None,
            }
        } else {
            None
        };
        let locale = row
            .locale
            .map(|value| match value.as_str() {
                "en" => Ok(Locale::En),
                "id" => Ok(Locale::Id),
                _ => Err(AppError::Internal(anyhow::anyhow!("person locale is invalid"))),
            })
            .transpose()?;

        Ok(Some(LoginProfile {
            display_name: row.display_name,
            email,
            phone,
            locale,
        }))
    }
}

async fn decrypt_person_field(
    keyring: &Keyring,
    tx: &mut Tx,
    tenant: crate::ids::TenantId,
    person: PersonId,
    column: &'static str,
    ciphertext: &[u8],
) -> Result<SecretString, AppError> {
    let plaintext = keyring
        .decrypt_field(
            tx,
            &Aad {
                tenant,
                table: "persons",
                column,
                row_id: person.0,
            },
            ciphertext,
        )
        .await
        .map_err(|error| AppError::Internal(anyhow::anyhow!("person PII decryption failed: {error}")))?;
    let value = std::str::from_utf8(&plaintext)
        .map_err(|_| AppError::Internal(anyhow::anyhow!("person PII is not valid UTF-8")))?;
    Ok(SecretString::from(value.to_owned()))
}

#[cfg(test)]
#[path = "../../../eq-testkit/src/db.rs"]
mod shared_test_db;

#[cfg(test)]
mod tests {
    use super::{CoreIdentityDirectory, PgIdentityDirectory};
    use crate::actor::ActorCtx;
    use crate::crypto::{Keys, bidx_email};
    use crate::db::begin;
    use crate::fields::{Email, FieldCtx, FieldKind};
    use crate::ids::{PersonId, TenantId};
    use crate::model::Role;
    use crate::money::Currency;
    use crate::policy::types::Locale;
    use std::sync::Arc;
    use uuid::Uuid;

    use super::shared_test_db::TestDb;

    const MASTER: &[u8] = b"test-secret-core-directory-master-key";

    fn test_field_ctx() -> FieldCtx {
        FieldCtx {
            country: Some(celes::Country::from_alpha2("ID").expect("known test country")),
            tenant_currency: Currency::IDR,
            tenant_locale: Locale::Id,
            today: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid test date"),
            password_min_length: None,
        }
    }

    async fn seed_tenant(db: &TestDb, tenant: TenantId) -> Result<(), sqlx::Error> {
        let slug = format!("dir-{}", tenant.0.simple());
        sqlx::query!(
            "INSERT INTO public.tenants (id, slug, name, country, preset_id, preset_version) \
             VALUES ($1, $2, 'Test School', 'ID', 'test', 1)",
            tenant.0,
            slug
        )
        .execute(&db.pool)
        .await?;
        Ok(())
    }

    async fn seed_person(
        db: &TestDb,
        tenant: TenantId,
        person: PersonId,
        email_bidx: [u8; 16],
        role: Role,
        status: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO public.persons (id, tenant_id, display_name, email_bidx) VALUES ($1, $2, 'Test Person', $3)",
            person.0,
            tenant.0,
            email_bidx.as_slice()
        )
        .execute(&db.pool)
        .await?;
        sqlx::query!(
            "INSERT INTO public.memberships (tenant_id, person_id, role, status) VALUES ($1, $2, $3, $4)",
            tenant.0,
            person.0,
            role as _,
            status
        )
        .execute(&db.pool)
        .await?;
        Ok(())
    }

    fn email() -> Result<Email, Vec<crate::fields::FieldIssue>> {
        Email::parse(String::from("teacher@example.test"), &test_field_ctx())
    }

    #[tokio::test]
    async fn login_directory_is_tenant_scoped() -> anyhow::Result<()> {
        let db = TestDb::new().await;
        let _test_urls = (&db.url, &db.app_url);
        let first_tenant = TenantId(Uuid::now_v7());
        let second_tenant = TenantId(Uuid::now_v7());
        let first_person = PersonId(Uuid::now_v7());
        let second_person = PersonId(Uuid::now_v7());
        seed_tenant(&db, first_tenant).await?;
        seed_tenant(&db, second_tenant).await?;
        let keys = Arc::new(Keys::derive(MASTER, None));
        let parsed_email = email().map_err(|_| anyhow::anyhow!("synthetic email is valid"))?;
        let first_bidx = bidx_email(&keys, first_tenant, parsed_email.as_ref())?;
        let second_bidx = bidx_email(&keys, second_tenant, parsed_email.as_ref())?;
        seed_person(&db, first_tenant, first_person, first_bidx, Role::Owner, "active").await?;
        seed_person(
            &db,
            second_tenant,
            second_person,
            second_bidx,
            Role::Admin,
            "active",
        )
        .await?;
        let directory = PgIdentityDirectory::new(keys);

        let mut first_tx = begin(&db.app_pool, first_tenant, &ActorCtx::anonymous(None, None)).await?;
        let first = directory.by_email(&mut first_tx, &parsed_email).await?;
        assert_eq!(first.map(|identity| identity.person_id()), Some(first_person));
        first_tx.rollback().await?;

        let mut second_tx = begin(&db.app_pool, second_tenant, &ActorCtx::anonymous(None, None)).await?;
        let second = directory.by_email(&mut second_tx, &parsed_email).await?;
        assert_eq!(second.map(|identity| identity.person_id()), Some(second_person));
        second_tx.rollback().await?;
        Ok(())
    }

    #[tokio::test]
    async fn deactivated_membership_is_not_active() -> anyhow::Result<()> {
        let db = TestDb::new().await;
        let _test_urls = (&db.url, &db.app_url);
        let tenant = TenantId(Uuid::now_v7());
        let person = PersonId(Uuid::now_v7());
        seed_tenant(&db, tenant).await?;
        let keys = Arc::new(Keys::derive(MASTER, None));
        let parsed_email = email().map_err(|_| anyhow::anyhow!("synthetic email is valid"))?;
        let index = bidx_email(&keys, tenant, parsed_email.as_ref())?;
        seed_person(&db, tenant, person, index, Role::Instructor, "deactivated").await?;
        let directory = PgIdentityDirectory::new(keys);

        let mut tx = begin(&db.app_pool, tenant, &ActorCtx::anonymous(None, None)).await?;
        assert!(directory.by_email(&mut tx, &parsed_email).await?.is_none());
        tx.rollback().await?;
        Ok(())
    }
}
