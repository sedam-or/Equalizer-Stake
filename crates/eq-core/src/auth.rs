//! Core-owned identity lookup and cryptographic dependencies (CA-30, SEC-04).

pub mod directory;

use crate::crypto::{Keyring, Keys};
use std::fmt;
use std::sync::Arc;

pub use directory::{CoreIdentityDirectory, LoginIdentity, PgIdentityDirectory};

use crate::actor::ActorCtx;
use crate::error::{AppError, codes};
use crate::ids::{PersonId, TenantId};
use crate::model::Role;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use std::collections::BTreeSet;

/// SBD-50: key material shared by login services without exposing secrets in Debug.
pub struct AuthState {
    keys: Arc<Keys>,
    keyring: Keyring,
    directory: PgIdentityDirectory,
}

/// IDN-10/SBD-40: authenticated request identity with permissions from its active role.
#[derive(Clone, PartialEq, Eq)]
pub struct Actor {
    person_id: PersonId,
    tenant: TenantId,
    role: Role,
    session_id: uuid::Uuid,
    perms: BTreeSet<String>,
    ctx: ActorCtx,
}

impl Actor {
    /// IDN-03/10: constructs an Actor after the Identity boundary verifies token and membership.
    pub fn from_verified_token(
        person_id: PersonId,
        tenant: TenantId,
        role: Role,
        session_id: uuid::Uuid,
        perms: BTreeSet<String>,
        ctx: ActorCtx,
    ) -> Self {
        Self {
            person_id,
            tenant,
            role,
            session_id,
            perms,
            ctx,
        }
    }

    /// Returns the verified person identifier.
    pub const fn person_id(&self) -> PersonId {
        self.person_id
    }

    /// Returns the trusted tenant resolved from request context.
    pub const fn tenant(&self) -> TenantId {
        self.tenant
    }

    /// Returns the active role from the verified access token.
    pub const fn role(&self) -> Role {
        self.role
    }

    /// Returns the refresh-session identifier from the verified access token.
    pub const fn session_id(&self) -> uuid::Uuid {
        self.session_id
    }

    /// Returns the effective permissions for the active role.
    pub const fn permissions(&self) -> &BTreeSet<String> {
        &self.perms
    }

    /// Returns the request actor context used by tenant transaction helpers.
    pub const fn context(&self) -> &ActorCtx {
        &self.ctx
    }

    /// IDN-10: checks whether the active role has a permission.
    pub fn has(&self, permission: &str) -> bool {
        self.perms.contains(permission)
    }

    /// SBD-40: requires a permission before a handler accesses protected data.
    pub fn require(&self, permission: &'static str) -> Result<(), AppError> {
        if self.has(permission) {
            Ok(())
        } else {
            Err(AppError::Forbidden { perm: permission })
        }
    }
}

impl fmt::Debug for Actor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Actor")
            .field("person_id", &"[REDACTED]")
            .field("tenant", &"[REDACTED]")
            .field("role", &self.role)
            .field("session_id", &"[REDACTED]")
            .field("perms", &self.perms)
            .field("ctx", &"[REDACTED]")
            .finish()
    }
}

/// IDN-10: optional authenticated request identity for explicitly public handlers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaybeActor(pub Option<Actor>);

/// SBD-40: requires the verified actor inserted by Identity's bearer middleware.
impl<S> FromRequestParts<S> for Actor
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Actor>()
            .cloned()
            .ok_or(AppError::Unauthenticated {
                code: &codes::UNAUTHENTICATED,
            })
    }
}

impl<S> FromRequestParts<S> for MaybeActor
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(parts.extensions.get::<Actor>().cloned()))
    }
}

impl AuthState {
    /// SEC-01/SEC-02: derives auth and field-encryption keys from process configuration.
    pub fn new(master: &[u8], previous: Option<&[u8]>) -> Self {
        let keys = Arc::new(Keys::derive(master, previous));
        let keyring = Keyring::new(Keys::derive(master, previous));
        let directory = PgIdentityDirectory::new(Arc::clone(&keys));
        Self {
            keys,
            keyring,
            directory,
        }
    }

    /// IDN-03/SEC-04: returns process-derived signing and blind-index keys.
    pub fn keys(&self) -> &Keys {
        &self.keys
    }

    /// SEC-02..03: returns the tenant field-encryption keyring.
    pub fn keyring(&self) -> &Keyring {
        &self.keyring
    }

    /// CA-30: returns the core-owned tenant directory port used by Identity.
    pub fn directory(&self) -> &dyn CoreIdentityDirectory {
        &self.directory
    }
}

impl fmt::Debug for AuthState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AuthState")
            .field("keys", &"[REDACTED]")
            .field("keyring", &"[configured]")
            .field("directory", &self.directory)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::{Actor, MaybeActor};
    use crate::actor::{ActorCtx, ActorRole};
    use crate::ids::{PersonId, TenantId};
    use crate::model::Role;
    use axum::extract::FromRequestParts;
    use axum::http::Request;
    use std::collections::BTreeSet;
    use uuid::Uuid;

    fn actor() -> Actor {
        Actor::from_verified_token(
            PersonId(Uuid::from_u128(1)),
            TenantId(Uuid::from_u128(2)),
            Role::Admin,
            Uuid::from_u128(3),
            BTreeSet::from([String::from("identity.people.write")]),
            ActorCtx {
                person_id: Some(PersonId(Uuid::from_u128(1))),
                role: ActorRole::Admin,
                ip: None,
                request_id: None,
            },
        )
    }

    #[test]
    fn actor_requires_registered_permission() {
        let actor = actor();

        assert!(actor.require("identity.people.write").is_ok());
        assert!(matches!(
            actor.require("identity.people.read_pii"),
            Err(crate::error::AppError::Forbidden {
                perm: "identity.people.read_pii"
            })
        ));
    }

    #[tokio::test]
    async fn actor_requires_authenticated_extension() {
        let (mut parts, _) = Request::new(()).into_parts();
        let result = Actor::from_request_parts(&mut parts, &()).await;

        assert!(matches!(
            result,
            Err(crate::error::AppError::Unauthenticated { .. })
        ));
    }

    #[tokio::test]
    async fn maybe_actor_is_optional() {
        let (mut parts, _) = Request::new(()).into_parts();
        let MaybeActor(absent) = MaybeActor::from_request_parts(&mut parts, &())
            .await
            .expect("optional extraction succeeds");
        assert!(absent.is_none());

        parts.extensions.insert(actor());
        let MaybeActor(present) = MaybeActor::from_request_parts(&mut parts, &())
            .await
            .expect("optional extraction succeeds");
        assert!(present.is_some());
    }
}
