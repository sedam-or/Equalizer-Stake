//! Core-owned person and membership lookup for Identity (CA-30, ERD-02).

use crate::crypto::{Aad, Keyring, Keys, bidx_email, bidx_phone_e164};
use crate::db::Tx;
use crate::error::AppError;
use crate::fields::{DisplayName, Email, Phone};
use crate::ids::{PersonId, TenantId};
use crate::model::Role;
use crate::policy::types::Locale;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use secrecy::{ExposeSecret, SecretString};
use sqlx::FromRow;
use std::collections::HashMap;
use std::fmt;
use std::num::NonZeroI32;
use std::num::NonZeroU16;
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
    pub version: i32,
}

impl fmt::Debug for LoginProfile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LoginProfile")
            .field("display_name", &"[REDACTED]")
            .field("email", &"[REDACTED]")
            .field("phone", &"[REDACTED]")
            .field("locale", &self.locale)
            .field("version", &self.version)
            .finish()
    }
}

/// S1 summary of a Core-owned student linked to an authenticated guardian.
pub struct LinkedStudentProfile {
    pub person_id: PersonId,
    pub display_name: String,
}

/// IDN-10/ERD-02: the row scope Identity asks Core to enforce in SQL.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PeopleScope {
    /// Owner or admin may list all visible people in the resolved tenant.
    All,
    /// Guardians may see themselves and linked students.
    Guardian,
    /// Students and pre-scheduling instructors may see only themselves.
    SelfOnly,
}

/// SBD-10/AUTH-04: complete scoped input for a person-detail read across the Core port.
#[derive(Clone, Copy, Debug)]
pub struct ScopedPersonDetailQuery {
    tenant: TenantId,
    actor: PersonId,
    scope: PeopleScope,
    person: PersonId,
    include_pii: bool,
}

impl ScopedPersonDetailQuery {
    /// Creates the tenant and Actor-bound detail request.
    pub const fn new(
        tenant: TenantId,
        actor: PersonId,
        scope: PeopleScope,
        person: PersonId,
        include_pii: bool,
    ) -> Self {
        Self {
            tenant,
            actor,
            scope,
            person,
            include_pii,
        }
    }
}

/// API-05: supported stable people-list sort orders.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PeopleSort {
    /// Sort by display name, then person id, ascending.
    DisplayName,
    /// Sort by creation time descending, then person id ascending.
    CreatedAtDesc,
}

enum PeopleCursorKey {
    DisplayName(String),
    CreatedAt(DateTime<Utc>),
}

/// API-05/CQ-23: a cursor key whose kind is fixed by its constructor.
pub struct PeopleCursor {
    sort: PeopleSort,
    key: PeopleCursorKey,
    person_id: PersonId,
}

impl PeopleCursor {
    /// Creates a display-name cursor from a value read from the Core directory.
    pub fn display_name(name: String, person_id: PersonId) -> Self {
        Self {
            sort: PeopleSort::DisplayName,
            key: PeopleCursorKey::DisplayName(name),
            person_id,
        }
    }

    /// Creates a creation-time cursor from a value read from the Core directory.
    pub fn created_at(created_at: DateTime<Utc>, person_id: PersonId) -> Self {
        Self {
            sort: PeopleSort::CreatedAtDesc,
            key: PeopleCursorKey::CreatedAt(created_at),
            person_id,
        }
    }

    /// Returns the cursor's declared sort.
    pub const fn sort(&self) -> PeopleSort {
        self.sort
    }

    /// Returns its Core-owned id tie-breaker.
    pub const fn person_id(&self) -> PersonId {
        self.person_id
    }

    /// Returns its canonical wire key.
    pub fn key(&self) -> String {
        match &self.key {
            PeopleCursorKey::DisplayName(name) => name.clone(),
            PeopleCursorKey::CreatedAt(created_at) => created_at.to_rfc3339(),
        }
    }

    fn display_name_key(&self) -> Option<&str> {
        match &self.key {
            PeopleCursorKey::DisplayName(name) => Some(name),
            PeopleCursorKey::CreatedAt(_) => None,
        }
    }

    fn created_at_key(&self) -> Option<DateTime<Utc>> {
        match self.key {
            PeopleCursorKey::DisplayName(_) => None,
            PeopleCursorKey::CreatedAt(created_at) => Some(created_at),
        }
    }
}

impl fmt::Debug for PeopleCursor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PeopleCursor")
            .field("sort", &self.sort)
            .field("key", &"[REDACTED]")
            .field("person_id", &"[REDACTED]")
            .finish()
    }
}

/// API-05/CQ-23: a bounded pagination request; cursors cannot cross sort orders.
#[derive(Debug)]
pub struct PeoplePageQuery {
    role: Option<Role>,
    limit: NonZeroU16,
    sort: PeopleSort,
    cursor: Option<PeopleCursor>,
}

impl PeoplePageQuery {
    /// Validates the public page cap and cursor/sort pairing.
    pub fn new(
        role: Option<Role>,
        limit: u16,
        sort: PeopleSort,
        cursor: Option<PeopleCursor>,
    ) -> Option<Self> {
        if limit > 200 || cursor.as_ref().is_some_and(|value| value.sort() != sort) {
            return None;
        }
        Some(Self {
            role,
            limit: NonZeroU16::new(limit)?,
            sort,
            cursor,
        })
    }
}

/// API-05: Core page results after applying SQL scope and role filters.
#[derive(Debug)]
pub struct DirectoryPeoplePage {
    pub people: Vec<DirectoryPersonSummary>,
    pub has_more: bool,
}

/// SBD-50/IDN-10: validated person values crossing the Identity-to-Core ownership port.
pub struct NewDirectoryPerson {
    display_name: DisplayName,
    legal_name: Option<SecretString>,
    email: Option<SecretString>,
    phone: Option<SecretString>,
    birth_date: Option<SecretString>,
    roles: Vec<Role>,
    invite: bool,
}

impl NewDirectoryPerson {
    /// Captures validated, redacted values for a Core-owned person insert.
    pub fn new(
        display_name: DisplayName,
        legal_name: Option<SecretString>,
        email: Option<SecretString>,
        phone: Option<SecretString>,
        birth_date: Option<SecretString>,
        roles: Vec<Role>,
        invite: bool,
    ) -> Self {
        Self {
            display_name,
            legal_name,
            email,
            phone,
            birth_date,
            roles,
            invite,
        }
    }

    /// Returns requested roles for Identity permission checks without exposing person fields.
    pub fn roles(&self) -> &[Role] {
        &self.roles
    }
}

impl fmt::Debug for NewDirectoryPerson {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NewDirectoryPerson")
            .field("display_name", &"[REDACTED]")
            .field("legal_name", &self.legal_name.as_ref().map(|_| "[REDACTED]"))
            .field("email", &self.email.as_ref().map(|_| "[REDACTED]"))
            .field("phone", &self.phone.as_ref().map(|_| "[REDACTED]"))
            .field("birth_date", &self.birth_date.as_ref().map(|_| "[REDACTED]"))
            .field("roles", &self.roles)
            .field("invite", &self.invite)
            .finish()
    }
}

/// SBD-10: S2 person values returned only for an authorized, in-scope detail read.
pub struct DirectoryPersonDetail {
    pub summary: DirectoryPersonSummary,
    pub legal_name: Option<SecretString>,
    pub email: Option<SecretString>,
    pub phone: Option<SecretString>,
    pub birth_date: Option<SecretString>,
    pub locale: Option<String>,
    pub guardians: Vec<DirectoryPersonLink>,
    pub students: Vec<DirectoryPersonLink>,
}

impl DirectoryPersonDetail {
    /// Returns whether an authorized read actually decrypted any stored S2 value.
    pub fn has_decrypted_pii(&self) -> bool {
        self.legal_name.is_some() || self.email.is_some() || self.phone.is_some() || self.birth_date.is_some()
    }
}

impl fmt::Debug for DirectoryPersonDetail {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DirectoryPersonDetail")
            .field("summary", &self.summary)
            .field("legal_name", &self.legal_name.as_ref().map(|_| "[REDACTED]"))
            .field("email", &self.email.as_ref().map(|_| "[REDACTED]"))
            .field("phone", &self.phone.as_ref().map(|_| "[REDACTED]"))
            .field("birth_date", &self.birth_date.as_ref().map(|_| "[REDACTED]"))
            .field("locale", &self.locale)
            .field("guardians", &self.guardians)
            .field("students", &self.students)
            .finish()
    }
}

/// S1 relationship returned with an in-scope person detail.
pub struct DirectoryPersonLink {
    pub person_id: PersonId,
    pub display_name: String,
    pub relationship: Option<String>,
    pub is_primary: bool,
}

impl fmt::Debug for DirectoryPersonLink {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DirectoryPersonLink")
            .field("person_id", &"[REDACTED]")
            .field("display_name", &"[REDACTED]")
            .field("relationship", &"[REDACTED]")
            .field("is_primary", &self.is_primary)
            .finish()
    }
}

/// IDN-10/ERD-02: S1 people summary returned only after Core applies tenant and row scope.
pub struct DirectoryPersonSummary {
    /// Core-owned person identifier.
    pub person_id: PersonId,
    /// S1 display name, never formatted by Debug.
    pub display_name: String,
    /// Active roles returned for this tenant.
    pub roles: Vec<Role>,
    /// Core-owned optimistic version.
    pub version: i32,
    /// Core-owned creation instant used by the alternate stable list sort.
    pub created_at: DateTime<Utc>,
}

impl fmt::Debug for DirectoryPersonSummary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DirectoryPersonSummary")
            .field("person_id", &"[REDACTED]")
            .field("display_name", &"[REDACTED]")
            .field("roles", &self.roles)
            .field("version", &self.version)
            .field("created_at", &self.created_at)
            .finish()
    }
}

impl fmt::Debug for LinkedStudentProfile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LinkedStudentProfile")
            .field("person_id", &"[REDACTED]")
            .field("display_name", &"[REDACTED]")
            .finish()
    }
}

/// API-07: validated profile fields passed to Core's optimistic profile update.
pub struct LoginProfileUpdate {
    expected_version: NonZeroI32,
    display_name: Option<DisplayName>,
    phone: Option<Option<Phone>>,
    locale: Option<Option<Locale>>,
    updated_at: DateTime<Utc>,
}

impl LoginProfileUpdate {
    /// API-07/CQ-23: creates an update using only canonical, validated profile values.
    pub fn new(
        expected_version: NonZeroI32,
        display_name: Option<DisplayName>,
        phone: Option<Option<Phone>>,
        locale: Option<Option<Locale>>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            expected_version,
            display_name,
            phone,
            locale,
            updated_at,
        }
    }
}

impl fmt::Debug for LoginProfileUpdate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LoginProfileUpdate")
            .field("display_name", &self.display_name.as_ref().map(|_| "[REDACTED]"))
            .field(
                "phone",
                &self
                    .phone
                    .as_ref()
                    .map(|value| value.as_ref().map(|_| "[REDACTED]")),
            )
            .field("locale", &self.locale)
            .field("expected_version", &self.expected_version)
            .field("updated_at", &self.updated_at)
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
    async fn has_active_admin(&self, tx: &mut Tx, tenant: TenantId) -> Result<bool, AppError>;

    /// IDN-03/09: checks whether this tenant role is still an active membership.
    async fn has_active_role(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        person: PersonId,
        role: Role,
    ) -> Result<bool, AppError>;

    /// STM-20/CA-30: inserts the invited membership through Core ownership without changing an existing state.
    async fn ensure_invited_membership(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        person: PersonId,
        role: Role,
    ) -> Result<(), AppError>;

    /// STM-20/21: activates one invited membership and returns the person's active roles.
    async fn activate_invited_membership(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        person: PersonId,
        role: Role,
    ) -> Result<Option<LoginIdentity>, AppError>;

    /// TD-138/ERD-01: resolves the caller tenant's display name through the tenant-scoped function.
    async fn tenant_name(&self, tx: &mut Tx, tenant: TenantId) -> Result<Option<String>, AppError>;

    /// ERD-02/SBD-10: reads the caller's core-owned profile and decrypts PII only when authorized.
    async fn login_profile(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        person: PersonId,
        include_pii: bool,
        keyring: &Keyring,
    ) -> Result<Option<LoginProfile>, AppError>;

    /// IDN-01/CA-30: returns active role memberships for one tenant-scoped person.
    async fn identity_by_person(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        person: PersonId,
    ) -> Result<Option<LoginIdentity>, AppError>;

    /// IDN-01/ERD-02: returns only the active guardian's linked student names through Core.
    async fn linked_students(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        guardian: PersonId,
    ) -> Result<Vec<LinkedStudentProfile>, AppError>;

    /// API-07/ERD-02: optimistically updates Core-owned self-profile fields; nested options distinguish clear from unchanged.
    async fn update_login_profile(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        person: PersonId,
        update: LoginProfileUpdate,
        keyring: &Keyring,
    ) -> Result<Option<i32>, AppError>;

    /// IDN-10/AUTH-04: returns only active people rows within the supplied SQL scope.
    async fn people_in_scope(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        actor: PersonId,
        scope: PeopleScope,
        role: Option<Role>,
        only_person: Option<PersonId>,
    ) -> Result<Vec<DirectoryPersonSummary>, AppError>;

    /// IDN-10/API-05: applies scope, role, stable ordering and keyset pagination in Core SQL.
    async fn people_page_in_scope(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        actor: PersonId,
        scope: PeopleScope,
        query: &PeoplePageQuery,
    ) -> Result<DirectoryPeoplePage, AppError>;

    /// SBD-10/AUD-01: returns a scoped Core profile and decrypts S2 fields only when permitted.
    async fn person_detail_in_scope(
        &self,
        tx: &mut Tx,
        query: &ScopedPersonDetailQuery,
        keyring: &Keyring,
    ) -> Result<Option<DirectoryPersonDetail>, AppError>;

    /// IDN-10/ERD-02: creates one Core-owned person and its memberships; None means email index conflict.
    async fn create_person(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        input: NewDirectoryPerson,
        keyring: &Keyring,
    ) -> Result<Option<DirectoryPersonSummary>, AppError>;

    /// STM-20/IDN-13: deactivates every active or invited Core-owned membership for one person.
    async fn deactivate_person_memberships(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        person: PersonId,
    ) -> Result<bool, AppError>;

    /// STM-20: locks active owner rows before the last-owner invariant is evaluated.
    async fn lock_active_owners(&self, tx: &mut Tx, tenant: TenantId) -> Result<Vec<PersonId>, AppError>;

    /// STM-20: creates or reactivates a role membership for a person in the same tenant.
    async fn activate_membership(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        person: PersonId,
        role: Role,
    ) -> Result<bool, AppError>;

    /// STM-20: deactivates an active or invited membership without deleting history.
    async fn deactivate_membership(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        person: PersonId,
        role: Role,
    ) -> Result<bool, AppError>;

    /// STM-27: creates or updates a guardian link; false means the primary-guardian index rejected it.
    async fn create_guardian_link(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        guardian: PersonId,
        student: PersonId,
        relationship: Option<String>,
        is_primary: bool,
    ) -> Result<bool, AppError>;
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

    async fn has_active_admin(&self, tx: &mut Tx, tenant: TenantId) -> Result<bool, AppError> {
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
        tenant: TenantId,
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
        tenant: TenantId,
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
        tenant: TenantId,
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

    async fn tenant_name(&self, tx: &mut Tx, tenant: TenantId) -> Result<Option<String>, AppError> {
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
        tenant: TenantId,
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
            "SELECT display_name, email_enc, phone_enc, locale, version \
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
            version: row.version,
        }))
    }

    async fn identity_by_person(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        person: PersonId,
    ) -> Result<Option<LoginIdentity>, AppError> {
        if tx.tenant() != tenant {
            return Err(AppError::Internal(anyhow::anyhow!(
                "identity tenant does not match transaction context"
            )));
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
            return Ok(None);
        }
        Ok(Some(LoginIdentity {
            person_id: person,
            roles: rows.into_iter().map(|row| row.role).collect(),
        }))
    }

    async fn linked_students(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        guardian: PersonId,
    ) -> Result<Vec<LinkedStudentProfile>, AppError> {
        if tx.tenant() != tenant {
            return Err(AppError::Internal(anyhow::anyhow!(
                "identity tenant does not match transaction context"
            )));
        }
        let rows = sqlx::query!(
            "SELECT p.id, p.display_name FROM public.person_links AS l \
             JOIN public.persons AS p ON p.tenant_id = l.tenant_id AND p.id = l.student_id \
             WHERE l.tenant_id = $1 AND l.guardian_id = $2 AND p.deleted_at IS NULL \
             ORDER BY p.display_name, p.id",
            tenant.0,
            guardian.0
        )
        .fetch_all(tx.conn())
        .await?;
        Ok(rows
            .into_iter()
            .map(|row| LinkedStudentProfile {
                person_id: PersonId(row.id),
                display_name: row.display_name,
            })
            .collect())
    }

    async fn update_login_profile(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        person: PersonId,
        update: LoginProfileUpdate,
        keyring: &Keyring,
    ) -> Result<Option<i32>, AppError> {
        if tx.tenant() != tenant {
            return Err(AppError::Internal(anyhow::anyhow!(
                "identity tenant does not match transaction context"
            )));
        }
        let (phone_enc, phone_bidx) = match update.phone.as_ref().and_then(Option::as_ref) {
            Some(value) => {
                let phone = value.as_str();
                let bidx = bidx_phone_e164(&self.keys, tenant, phone).map_err(|_| {
                    AppError::Internal(anyhow::anyhow!("validated profile phone could not be indexed"))
                })?;
                let ciphertext = keyring
                    .encrypt_field(
                        tx,
                        &Aad {
                            tenant,
                            table: "persons",
                            column: "phone_enc",
                            row_id: person.0,
                        },
                        phone.as_bytes(),
                    )
                    .await
                    .map_err(|error| {
                        AppError::Internal(anyhow::anyhow!("profile phone encryption failed: {error}"))
                    })?;
                (Some(ciphertext), Some(bidx.to_vec()))
            }
            _ => (None, None),
        };
        let change_phone = update.phone.is_some();
        let change_locale = update.locale.is_some();
        let display_name = update.display_name.map(|value| value.as_ref().to_owned());
        let locale_value = update.locale.flatten().map(|value| match value {
            Locale::En => "en",
            Locale::Id => "id",
        });
        let row = sqlx::query!(
            "UPDATE public.persons SET \
               display_name = COALESCE($3, display_name), \
               phone_enc = CASE WHEN $4 THEN $5 ELSE phone_enc END, \
               phone_bidx = CASE WHEN $4 THEN $6 ELSE phone_bidx END, \
               locale = CASE WHEN $7 THEN $8 ELSE locale END, \
               updated_at = $9, version = version + 1 \
             WHERE tenant_id = $1 AND id = $2 AND version = $10 AND deleted_at IS NULL \
             RETURNING version",
            tenant.0,
            person.0,
            display_name.as_deref(),
            change_phone,
            phone_enc.as_deref(),
            phone_bidx.as_deref(),
            change_locale,
            locale_value,
            update.updated_at,
            update.expected_version.get()
        )
        .fetch_optional(tx.conn())
        .await?;
        Ok(row.map(|value| value.version))
    }

    async fn people_in_scope(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        actor: PersonId,
        scope: PeopleScope,
        role: Option<Role>,
        only_person: Option<PersonId>,
    ) -> Result<Vec<DirectoryPersonSummary>, AppError> {
        if tx.tenant() != tenant {
            return Err(AppError::Internal(anyhow::anyhow!(
                "identity tenant does not match transaction context"
            )));
        }
        let scope = match scope {
            PeopleScope::All => "all",
            PeopleScope::Guardian => "guardian",
            PeopleScope::SelfOnly => "self",
        };
        let role = role.map(role_name);
        let only_person = only_person.map(|person| person.0);
        let rows = sqlx::query_as!(
            DirectoryPersonRow,
            r#"SELECT DISTINCT p.id AS "id!: Uuid", p.display_name, p.version, p.created_at, m.role AS "role!: Role"
               FROM public.persons AS p
               JOIN public.memberships AS m
                 ON m.tenant_id = p.tenant_id AND m.person_id = p.id
               WHERE p.tenant_id = $1
                 AND p.deleted_at IS NULL
                 AND m.status = 'active'
                 AND ($2::text = 'all' OR p.id = $3 OR
                      ($2::text = 'guardian' AND EXISTS (
                        SELECT 1 FROM public.person_links AS l
                        WHERE l.tenant_id = $1 AND l.guardian_id = $3 AND l.student_id = p.id
                      )))
                 AND ($4::text IS NULL OR m.role = $4)
                 AND ($5::uuid IS NULL OR p.id = $5)
               ORDER BY p.display_name, p.id, m.role"#,
            tenant.0,
            scope,
            actor.0,
            role,
            only_person
        )
        .fetch_all(tx.conn())
        .await?;

        let mut people = Vec::<DirectoryPersonSummary>::new();
        let mut positions = HashMap::<Uuid, usize>::new();
        for row in rows {
            if let Some(position) = positions.get(&row.id).copied() {
                people[position].roles.push(row.role);
            } else {
                positions.insert(row.id, people.len());
                people.push(DirectoryPersonSummary {
                    person_id: PersonId(row.id),
                    display_name: row.display_name,
                    roles: vec![row.role],
                    version: row.version,
                    created_at: row.created_at,
                });
            }
        }
        Ok(people)
    }

    async fn people_page_in_scope(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        actor: PersonId,
        scope: PeopleScope,
        query: &PeoplePageQuery,
    ) -> Result<DirectoryPeoplePage, AppError> {
        if tx.tenant() != tenant {
            return Err(AppError::Internal(anyhow::anyhow!(
                "identity tenant does not match transaction context"
            )));
        }
        let mut rows = fetch_people_page_rows(tx, tenant, actor, scope, query).await?;
        let has_more = rows.len() > usize::from(query.limit.get());
        rows.truncate(usize::from(query.limit.get()));
        let person_ids = rows.iter().map(|row| row.id).collect::<Vec<_>>();
        let mut roles_by_person = active_roles_for_people(tx, tenant, &person_ids).await?;
        let people = rows
            .into_iter()
            .map(|row| DirectoryPersonSummary {
                person_id: PersonId(row.id),
                display_name: row.display_name,
                roles: roles_by_person.remove(&row.id).unwrap_or_default(),
                version: row.version,
                created_at: row.created_at,
            })
            .collect();
        Ok(DirectoryPeoplePage { people, has_more })
    }

    async fn person_detail_in_scope(
        &self,
        tx: &mut Tx,
        query: &ScopedPersonDetailQuery,
        keyring: &Keyring,
    ) -> Result<Option<DirectoryPersonDetail>, AppError> {
        if tx.tenant() != query.tenant {
            return Err(AppError::Internal(anyhow::anyhow!(
                "identity tenant does not match transaction context"
            )));
        }
        let scope = match query.scope {
            PeopleScope::All => "all",
            PeopleScope::Guardian => "guardian",
            PeopleScope::SelfOnly => "self",
        };
        let Some(row) = scoped_person_detail_row(tx, query, scope).await? else {
            return Ok(None);
        };
        let DirectoryPersonDetailRow {
            id,
            display_name,
            version,
            created_at,
            locale,
            legal_name_enc,
            email_enc,
            phone_enc,
            birth_date_enc,
        } = row;
        let roles = active_roles_for_person(tx, query.tenant, query.person).await?;
        let pii = decrypt_person_pii(
            keyring,
            tx,
            query.tenant,
            query.person,
            EncryptedPersonPii {
                legal_name: legal_name_enc,
                email: email_enc,
                phone: phone_enc,
                birth_date: birth_date_enc,
            },
        )
        .await?;
        let (guardians, students) = person_detail_links(tx, query.tenant, query.person).await?;
        Ok(Some(DirectoryPersonDetail {
            summary: DirectoryPersonSummary {
                person_id: PersonId(id),
                display_name,
                roles,
                version,
                created_at,
            },
            legal_name: pii.legal_name,
            email: pii.email,
            phone: pii.phone,
            birth_date: pii.birth_date,
            locale,
            guardians,
            students,
        }))
    }

    async fn create_person(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        input: NewDirectoryPerson,
        keyring: &Keyring,
    ) -> Result<Option<DirectoryPersonSummary>, AppError> {
        if tx.tenant() != tenant {
            return Err(AppError::Internal(anyhow::anyhow!(
                "identity tenant does not match transaction context"
            )));
        }
        let NewDirectoryPerson {
            display_name,
            legal_name,
            email,
            phone,
            birth_date,
            roles,
            invite,
        } = input;
        let person_id = PersonId(Uuid::now_v7());
        let encrypted = encrypt_new_person_fields(
            &self.keys,
            tx,
            tenant,
            person_id,
            NewPersonPii {
                legal_name: legal_name.as_ref(),
                email: email.as_ref(),
                phone: phone.as_ref(),
                birth_date: birth_date.as_ref(),
            },
            keyring,
        )
        .await?;
        let row = match sqlx::query!(
            "INSERT INTO public.persons (id, tenant_id, display_name, legal_name_enc, email_enc, email_bidx, phone_enc, phone_bidx, birth_date_enc) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING version, created_at",
            person_id.0,
            tenant.0,
            display_name.as_ref(),
            encrypted.legal_name_enc,
            encrypted.email_enc,
            encrypted.email_bidx,
            encrypted.phone_enc,
            encrypted.phone_bidx,
            encrypted.birth_date_enc
        )
        .fetch_one(tx.conn())
        .await
        {
            Ok(row) => row,
            Err(sqlx::Error::Database(error)) if error.constraint() == Some("persons_email_unique") => {
                return Ok(None);
            }
            Err(error) => return Err(error.into()),
        };
        let status = if invite { "invited" } else { "active" };
        for role in &roles {
            sqlx::query!(
                "INSERT INTO public.memberships (tenant_id, person_id, role, status) VALUES ($1, $2, $3, $4)",
                tenant.0,
                person_id.0,
                *role as Role,
                status
            )
            .execute(tx.conn())
            .await?;
        }
        Ok(Some(DirectoryPersonSummary {
            person_id,
            display_name: display_name.as_ref().to_owned(),
            roles,
            version: row.version,
            created_at: row.created_at,
        }))
    }

    async fn deactivate_person_memberships(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        person: PersonId,
    ) -> Result<bool, AppError> {
        if tx.tenant() != tenant {
            return Err(AppError::Internal(anyhow::anyhow!(
                "identity tenant does not match transaction context"
            )));
        }
        let changed = sqlx::query!(
            "UPDATE public.memberships SET status = 'deactivated' \
             WHERE tenant_id = $1 AND person_id = $2 AND status IN ('active', 'invited') \
             RETURNING role",
            tenant.0,
            person.0
        )
        .fetch_all(tx.conn())
        .await?;
        Ok(!changed.is_empty())
    }

    async fn lock_active_owners(&self, tx: &mut Tx, tenant: TenantId) -> Result<Vec<PersonId>, AppError> {
        if tx.tenant() != tenant {
            return Err(AppError::Internal(anyhow::anyhow!(
                "identity tenant does not match transaction context"
            )));
        }
        let rows = sqlx::query!(
            "SELECT person_id AS \"person_id!: Uuid\" FROM public.memberships \
             WHERE tenant_id = $1 AND role = 'owner' AND status = 'active' \
             ORDER BY person_id FOR UPDATE",
            tenant.0
        )
        .fetch_all(tx.conn())
        .await?;
        Ok(rows.into_iter().map(|row| PersonId(row.person_id)).collect())
    }

    async fn activate_membership(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        person: PersonId,
        role: Role,
    ) -> Result<bool, AppError> {
        if tx.tenant() != tenant {
            return Err(AppError::Internal(anyhow::anyhow!(
                "identity tenant does not match transaction context"
            )));
        }
        let row = sqlx::query!(
            "INSERT INTO public.memberships (tenant_id, person_id, role, status) \
             SELECT $1, p.id, $3, 'active' FROM public.persons AS p \
             WHERE p.tenant_id = $1 AND p.id = $2 AND p.deleted_at IS NULL \
             ON CONFLICT (tenant_id, person_id, role) DO UPDATE SET status = 'active' \
             RETURNING person_id",
            tenant.0,
            person.0,
            role as Role
        )
        .fetch_optional(tx.conn())
        .await?;
        Ok(row.is_some())
    }

    async fn deactivate_membership(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        person: PersonId,
        role: Role,
    ) -> Result<bool, AppError> {
        if tx.tenant() != tenant {
            return Err(AppError::Internal(anyhow::anyhow!(
                "identity tenant does not match transaction context"
            )));
        }
        let changed = sqlx::query!(
            "UPDATE public.memberships SET status = 'deactivated' \
             WHERE tenant_id = $1 AND person_id = $2 AND role = $3 \
               AND status IN ('active', 'invited')",
            tenant.0,
            person.0,
            role as Role
        )
        .execute(tx.conn())
        .await?;
        Ok(changed.rows_affected() == 1)
    }

    async fn create_guardian_link(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        guardian: PersonId,
        student: PersonId,
        relationship: Option<String>,
        is_primary: bool,
    ) -> Result<bool, AppError> {
        if tx.tenant() != tenant {
            return Err(AppError::Internal(anyhow::anyhow!(
                "identity tenant does not match transaction context"
            )));
        }
        let result = sqlx::query!(
            "INSERT INTO public.person_links (tenant_id, guardian_id, student_id, relationship, is_primary) \
             VALUES ($1, $2, $3, $4, $5) \
             ON CONFLICT (tenant_id, guardian_id, student_id) DO UPDATE \
               SET relationship = EXCLUDED.relationship, is_primary = EXCLUDED.is_primary",
            tenant.0,
            guardian.0,
            student.0,
            relationship,
            is_primary
        )
        .execute(tx.conn())
        .await;
        match result {
            Ok(_) => Ok(true),
            Err(sqlx::Error::Database(error)) if error.constraint() == Some("person_links_one_primary") => {
                Ok(false)
            }
            Err(error) => Err(error.into()),
        }
    }
}

struct DirectoryPeoplePageRow {
    id: Uuid,
    display_name: String,
    version: i32,
    created_at: DateTime<Utc>,
}

async fn fetch_people_page_rows(
    tx: &mut Tx,
    tenant: TenantId,
    actor: PersonId,
    scope: PeopleScope,
    query: &PeoplePageQuery,
) -> Result<Vec<DirectoryPeoplePageRow>, AppError> {
    let scope = match scope {
        PeopleScope::All => "all",
        PeopleScope::Guardian => "guardian",
        PeopleScope::SelfOnly => "self",
    };
    let role = query.role.map(role_name);
    let cursor_name = query
        .cursor
        .as_ref()
        .and_then(PeopleCursor::display_name_key)
        .map(str::to_owned);
    let cursor_created_at = query.cursor.as_ref().and_then(PeopleCursor::created_at_key);
    let cursor_person_id = query.cursor.as_ref().map(|cursor| cursor.person_id().0);
    let sort = match query.sort {
        PeopleSort::DisplayName => "display_name",
        PeopleSort::CreatedAtDesc => "-created_at",
    };
    let row_limit = i64::from(query.limit.get()) + 1;
    Ok(sqlx::query_as!(
        DirectoryPeoplePageRow,
        r#"SELECT p.id AS "id!: Uuid", p.display_name, p.version, p.created_at
           FROM public.persons AS p
           WHERE p.tenant_id = $1
             AND p.deleted_at IS NULL
             AND ($2::text = 'all' OR p.id = $3 OR
                  ($2::text = 'guardian' AND EXISTS (
                    SELECT 1 FROM public.person_links AS l
                    WHERE l.tenant_id = $1 AND l.guardian_id = $3 AND l.student_id = p.id
                  )))
             AND EXISTS (
               SELECT 1 FROM public.memberships AS m
               WHERE m.tenant_id = $1 AND m.person_id = p.id AND m.status = 'active'
                 AND ($4::text IS NULL OR m.role = $4)
             )
             AND (
               $7::uuid IS NULL
               OR ($8::text = 'display_name' AND
                   (p.display_name > $5 OR (p.display_name = $5 AND p.id > $7)))
               OR ($8::text = '-created_at' AND
                   (p.created_at < $6 OR (p.created_at = $6 AND p.id > $7)))
             )
           ORDER BY
             CASE WHEN $8::text = 'display_name' THEN p.display_name END ASC NULLS LAST,
             CASE WHEN $8::text = '-created_at' THEN p.created_at END DESC NULLS LAST,
             p.id ASC
           LIMIT $9"#,
        tenant.0,
        scope,
        actor.0,
        role,
        cursor_name,
        cursor_created_at,
        cursor_person_id,
        sort,
        row_limit
    )
    .fetch_all(tx.conn())
    .await?)
}

struct DirectoryPersonDetailRow {
    id: Uuid,
    display_name: String,
    version: i32,
    created_at: DateTime<Utc>,
    locale: Option<String>,
    legal_name_enc: Option<Vec<u8>>,
    email_enc: Option<Vec<u8>>,
    phone_enc: Option<Vec<u8>>,
    birth_date_enc: Option<Vec<u8>>,
}

async fn scoped_person_detail_row(
    tx: &mut Tx,
    query: &ScopedPersonDetailQuery,
    scope: &str,
) -> Result<Option<DirectoryPersonDetailRow>, AppError> {
    Ok(sqlx::query_as!(
        DirectoryPersonDetailRow,
        r#"SELECT p.id AS "id!: Uuid", p.display_name, p.version, p.created_at, p.locale,
                  CASE WHEN $4 THEN p.legal_name_enc END AS legal_name_enc,
                  CASE WHEN $4 THEN p.email_enc END AS email_enc,
                  CASE WHEN $4 THEN p.phone_enc END AS phone_enc,
                  CASE WHEN $4 THEN p.birth_date_enc END AS birth_date_enc
           FROM public.persons AS p
           WHERE p.tenant_id = $1 AND p.id = $2 AND p.deleted_at IS NULL
             AND EXISTS (
               SELECT 1 FROM public.memberships AS m
               WHERE m.tenant_id = $1 AND m.person_id = p.id AND m.status = 'active'
             )
             AND ($3::text = 'all' OR p.id = $5 OR
                  ($3::text = 'guardian' AND EXISTS (
                    SELECT 1 FROM public.person_links AS l
                    WHERE l.tenant_id = $1 AND l.guardian_id = $5 AND l.student_id = p.id
                  )))"#,
        query.tenant.0,
        query.person.0,
        scope,
        query.include_pii,
        query.actor.0
    )
    .fetch_optional(tx.conn())
    .await?)
}

async fn active_roles_for_people(
    tx: &mut Tx,
    tenant: TenantId,
    people: &[Uuid],
) -> Result<HashMap<Uuid, Vec<Role>>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT person_id AS "person_id!: Uuid", role AS "role!: Role"
           FROM public.memberships
           WHERE tenant_id = $1 AND person_id = ANY($2) AND status = 'active'
           ORDER BY person_id, role"#,
        tenant.0,
        people
    )
    .fetch_all(tx.conn())
    .await?;
    let mut roles = HashMap::<Uuid, Vec<Role>>::new();
    for row in rows {
        roles.entry(row.person_id).or_default().push(row.role);
    }
    Ok(roles)
}

async fn active_roles_for_person(
    tx: &mut Tx,
    tenant: TenantId,
    person: PersonId,
) -> Result<Vec<Role>, AppError> {
    Ok(active_roles_for_people(tx, tenant, &[person.0])
        .await?
        .remove(&person.0)
        .unwrap_or_default())
}

struct EncryptedPersonPii {
    legal_name: Option<Vec<u8>>,
    email: Option<Vec<u8>>,
    phone: Option<Vec<u8>>,
    birth_date: Option<Vec<u8>>,
}

struct DecryptedPersonPii {
    legal_name: Option<SecretString>,
    email: Option<SecretString>,
    phone: Option<SecretString>,
    birth_date: Option<SecretString>,
}

async fn decrypt_person_pii(
    keyring: &Keyring,
    tx: &mut Tx,
    tenant: TenantId,
    person: PersonId,
    encrypted: EncryptedPersonPii,
) -> Result<DecryptedPersonPii, AppError> {
    let legal_name = match encrypted.legal_name {
        Some(value) => {
            Some(decrypt_person_field(keyring, tx, tenant, person, "legal_name_enc", &value).await?)
        }
        None => None,
    };
    let email = match encrypted.email {
        Some(value) => Some(decrypt_person_field(keyring, tx, tenant, person, "email_enc", &value).await?),
        None => None,
    };
    let phone = match encrypted.phone {
        Some(value) => Some(decrypt_person_field(keyring, tx, tenant, person, "phone_enc", &value).await?),
        None => None,
    };
    let birth_date = match encrypted.birth_date {
        Some(value) => {
            Some(decrypt_person_field(keyring, tx, tenant, person, "birth_date_enc", &value).await?)
        }
        None => None,
    };
    Ok(DecryptedPersonPii {
        legal_name,
        email,
        phone,
        birth_date,
    })
}

struct DirectoryPersonLinkRow {
    person_id: Uuid,
    display_name: String,
    relationship: Option<String>,
    is_primary: bool,
}

async fn person_detail_links(
    tx: &mut Tx,
    tenant: TenantId,
    person: PersonId,
) -> Result<(Vec<DirectoryPersonLink>, Vec<DirectoryPersonLink>), AppError> {
    let guardians = sqlx::query_as!(
        DirectoryPersonLinkRow,
        r#"SELECT guardian.id AS "person_id!: Uuid", guardian.display_name,
                  link.relationship, link.is_primary
           FROM public.person_links AS link
           JOIN public.persons AS guardian
             ON guardian.tenant_id = link.tenant_id AND guardian.id = link.guardian_id
           WHERE link.tenant_id = $1 AND link.student_id = $2 AND guardian.deleted_at IS NULL
             AND EXISTS (
               SELECT 1 FROM public.memberships AS member
               WHERE member.tenant_id = $1 AND member.person_id = guardian.id
                 AND member.role = 'guardian' AND member.status = 'active'
             )
           ORDER BY guardian.id"#,
        tenant.0,
        person.0
    )
    .fetch_all(tx.conn())
    .await?
    .into_iter()
    .map(directory_person_link)
    .collect::<Vec<_>>();
    let students = sqlx::query_as!(
        DirectoryPersonLinkRow,
        r#"SELECT student.id AS "person_id!: Uuid", student.display_name,
                  link.relationship, link.is_primary
           FROM public.person_links AS link
           JOIN public.persons AS student
             ON student.tenant_id = link.tenant_id AND student.id = link.student_id
           WHERE link.tenant_id = $1 AND link.guardian_id = $2 AND student.deleted_at IS NULL
             AND EXISTS (
               SELECT 1 FROM public.memberships AS member
               WHERE member.tenant_id = $1 AND member.person_id = student.id
                 AND member.role = 'student' AND member.status = 'active'
             )
           ORDER BY student.id"#,
        tenant.0,
        person.0
    )
    .fetch_all(tx.conn())
    .await?
    .into_iter()
    .map(directory_person_link)
    .collect::<Vec<_>>();
    Ok((guardians, students))
}

fn directory_person_link(row: DirectoryPersonLinkRow) -> DirectoryPersonLink {
    DirectoryPersonLink {
        person_id: PersonId(row.person_id),
        display_name: row.display_name,
        relationship: row.relationship,
        is_primary: row.is_primary,
    }
}

struct NewPersonPii<'a> {
    legal_name: Option<&'a SecretString>,
    email: Option<&'a SecretString>,
    phone: Option<&'a SecretString>,
    birth_date: Option<&'a SecretString>,
}

struct EncryptedNewPersonFields {
    legal_name_enc: Option<Vec<u8>>,
    email_enc: Option<Vec<u8>>,
    email_bidx: Option<Vec<u8>>,
    phone_enc: Option<Vec<u8>>,
    phone_bidx: Option<Vec<u8>>,
    birth_date_enc: Option<Vec<u8>>,
}

async fn encrypt_new_person_fields(
    keys: &Keys,
    tx: &mut Tx,
    tenant: TenantId,
    person: PersonId,
    pii: NewPersonPii<'_>,
    keyring: &Keyring,
) -> Result<EncryptedNewPersonFields, AppError> {
    let legal_name_enc =
        encrypt_person_field(keyring, tx, tenant, person, "legal_name_enc", pii.legal_name).await?;
    let email_enc = encrypt_person_field(keyring, tx, tenant, person, "email_enc", pii.email).await?;
    let phone_enc = encrypt_person_field(keyring, tx, tenant, person, "phone_enc", pii.phone).await?;
    let birth_date_enc =
        encrypt_person_field(keyring, tx, tenant, person, "birth_date_enc", pii.birth_date).await?;
    let email_bidx = pii
        .email
        .map(|value| bidx_email(keys, tenant, value.expose_secret()))
        .transpose()
        .map_err(|error| AppError::Internal(error.into()))?
        .map(|index| index.to_vec());
    let phone_bidx = pii
        .phone
        .map(|value| bidx_phone_e164(keys, tenant, value.expose_secret()))
        .transpose()
        .map_err(|error| AppError::Internal(error.into()))?
        .map(|index| index.to_vec());
    Ok(EncryptedNewPersonFields {
        legal_name_enc,
        email_enc,
        email_bidx,
        phone_enc,
        phone_bidx,
        birth_date_enc,
    })
}

struct DirectoryPersonRow {
    id: Uuid,
    display_name: String,
    version: i32,
    created_at: DateTime<Utc>,
    role: Role,
}

fn role_name(role: Role) -> &'static str {
    match role {
        Role::Owner => "owner",
        Role::Admin => "admin",
        Role::Instructor => "instructor",
        Role::Guardian => "guardian",
        Role::Student => "student",
    }
}

async fn decrypt_person_field(
    keyring: &Keyring,
    tx: &mut Tx,
    tenant: TenantId,
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

async fn encrypt_person_field(
    keyring: &Keyring,
    tx: &mut Tx,
    tenant: TenantId,
    person: PersonId,
    column: &'static str,
    value: Option<&SecretString>,
) -> Result<Option<Vec<u8>>, AppError> {
    let Some(value) = value else {
        return Ok(None);
    };
    keyring
        .encrypt_field(
            tx,
            &Aad {
                tenant,
                table: "persons",
                column,
                row_id: person.0,
            },
            value.expose_secret().as_bytes(),
        )
        .await
        .map(Some)
        .map_err(|error| AppError::Internal(anyhow::anyhow!("person PII encryption failed: {error}")))
}

#[cfg(test)]
#[path = "../../../eq-testkit/src/db.rs"]
mod shared_test_db;

#[cfg(test)]
mod tests {
    use super::{CoreIdentityDirectory, NewDirectoryPerson, PgIdentityDirectory};
    use crate::actor::ActorCtx;
    use crate::crypto::{Keys, bidx_email};
    use crate::db::begin;
    use crate::fields::{Email, FieldCtx, FieldKind};
    use crate::ids::{PersonId, TenantId};
    use crate::model::Role;
    use crate::money::Currency;
    use crate::policy::types::Locale;
    use secrecy::SecretString;
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

    #[test]
    fn new_directory_person_debug_redacts_secret_fields() {
        let display_name =
            crate::fields::DisplayName::parse(String::from("test-secret-display-name"), &test_field_ctx())
                .expect("valid test display name");
        let person = NewDirectoryPerson::new(
            display_name,
            Some(SecretString::from("test-secret-legal-name".to_owned())),
            Some(SecretString::from("test-secret-email@example.test".to_owned())),
            Some(SecretString::from("test-secret-phone".to_owned())),
            Some(SecretString::from("test-secret-birth-date".to_owned())),
            vec![Role::Student],
            false,
        );
        let debug = format!("{person:?}");
        for secret in [
            "test-secret-display-name",
            "test-secret-legal-name",
            "test-secret-email@example.test",
            "test-secret-phone",
            "test-secret-birth-date",
        ] {
            assert!(!debug.contains(secret));
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
