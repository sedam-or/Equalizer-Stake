//! IDN-07: invite issuance, resend revocation and link-channel delivery.

use crate::policy::IdentityPolicy;
use crate::repo;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::Duration;
use eq_audit::{AuditEvent, Category, record};
use eq_core::actor::ActorCtx;
use eq_core::auth::CoreIdentityDirectory;
use eq_core::clock::Clock;
use eq_core::config::Config;
use eq_core::crypto::Keyring;
use eq_core::db::Tx;
use eq_core::error::AppError;
use eq_core::ids::{PersonId, TenantId};
use eq_core::model::Role;
use eq_core::outbox::{DomainEvent, emit};
use eq_core::policy::types::Locale;
use rand::RngCore;
use rand::rngs::OsRng;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fmt;
use uuid::Uuid;
use zeroize::Zeroizing;

/// IDN-07: delivery choice accepted by the people-invite boundary.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum InviteChannel {
    /// Return the invite URL to the authorized caller.
    Link,
    /// Leave delivery to the Notify module.
    Email,
    /// Leave delivery to the Notify module.
    Whatsapp,
}

impl InviteChannel {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Link => "link",
            Self::Email => "email",
            Self::Whatsapp => "whatsapp",
        }
    }
}

/// IDN-07: values supplied by the tenant-aware invite boundary.
pub struct InviteRequest<'a> {
    tenant: TenantId,
    actor: &'a ActorCtx,
    person: PersonId,
    role: Role,
    channel: InviteChannel,
    tenant_locale: Locale,
    config: &'a Config,
    policy: &'a IdentityPolicy,
}

impl fmt::Debug for InviteRequest<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("InviteRequest")
            .field("tenant", &self.tenant)
            .field("actor", &"[REDACTED]")
            .field("person", &self.person)
            .field("role", &self.role)
            .field("channel", &self.channel)
            .field("tenant_locale", &self.tenant_locale)
            .field("public_url", &self.config.public_url)
            .field("policy", &self.policy)
            .finish()
    }
}

impl<'a> InviteRequest<'a> {
    /// IDN-07/CAN-80: captures the tenant-scoped invite inputs and configured public URL.
    #[expect(
        clippy::too_many_arguments,
        reason = "the request captures each value needed by the specified invite flow"
    )]
    pub fn new(
        tenant: TenantId,
        actor: &'a ActorCtx,
        person: PersonId,
        role: Role,
        channel: InviteChannel,
        tenant_locale: Locale,
        config: &'a Config,
        policy: &'a IdentityPolicy,
    ) -> Self {
        Self {
            tenant,
            actor,
            person,
            role,
            channel,
            tenant_locale,
            config,
            policy,
        }
    }
}

/// IDN-07: generated invite ID and optional single-use link for the link channel.
pub struct InviteResult {
    invite_id: Uuid,
    invite_url: Option<SecretString>,
}

impl InviteResult {
    /// IDN-07: returns the persisted invite identifier.
    pub const fn invite_id(&self) -> Uuid {
        self.invite_id
    }

    /// IDN-07/SBD-50: exposes the link-channel URL only to its authorized caller.
    pub fn invite_url(&self) -> Option<&SecretString> {
        self.invite_url.as_ref()
    }
}

impl fmt::Debug for InviteResult {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("InviteResult")
            .field("invite_id", &self.invite_id)
            .field("invite_url", &self.invite_url.as_ref().map(|_| "[REDACTED]"))
            .finish()
    }
}

/// IDN-07/CA-30: issues tenant-scoped invite tokens through Identity persistence and the Core port.
pub struct InviteService<'a> {
    directory: &'a dyn CoreIdentityDirectory,
    keyring: &'a Keyring,
    clock: &'a dyn Clock,
}

impl fmt::Debug for InviteService<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("InviteService")
            .field("directory", &"[core identity port]")
            .field("keyring", &"[REDACTED]")
            .field("clock", &self.clock)
            .finish()
    }
}

impl<'a> InviteService<'a> {
    /// IDN-07: creates an invite service with Core, keyring and injected clock dependencies.
    pub const fn new(
        directory: &'a dyn CoreIdentityDirectory,
        keyring: &'a Keyring,
        clock: &'a dyn Clock,
    ) -> Self {
        Self {
            directory,
            keyring,
            clock,
        }
    }

    /// IDN-07/STM-21: expires any prior invite and records the new invite in the caller's transaction.
    pub async fn create_invite(
        &self,
        tx: &mut Tx,
        request: InviteRequest<'_>,
    ) -> Result<InviteResult, AppError> {
        let profile = self
            .directory
            .login_profile(tx, request.tenant, request.person, false, self.keyring)
            .await?
            .ok_or(AppError::NotFound { entity: "person" })?;
        let locale = profile.locale.unwrap_or(request.tenant_locale);
        repo::lock_invite_series(tx, request.tenant, request.person, request.role).await?;
        let now = self.clock.now();
        let expires_at = now
            .checked_add_signed(Duration::days(i64::from(request.policy.invite_expires_days())))
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("invite expiry is outside timestamp range")))?;

        repo::expire_pending_invites(tx, request.tenant, request.person, request.role, now).await?;
        self.directory
            .ensure_invited_membership(tx, request.tenant, request.person, request.role)
            .await?;

        let mut token_bytes = Zeroizing::new([0_u8; 32]);
        OsRng.fill_bytes(&mut *token_bytes);
        let token_hash = Sha256::digest(token_bytes.as_ref());
        let token = SecretString::from(URL_SAFE_NO_PAD.encode(token_bytes.as_ref()));
        let invite_id = Uuid::now_v7();
        repo::insert_invite(
            tx,
            invite_id,
            request.tenant,
            request.person,
            request.role,
            &token_hash[..],
            expires_at,
            request.actor.person_id,
        )
        .await?;

        record(
            tx,
            AuditEvent {
                action: "identity.invite.created",
                category: Category::Security,
                entity_type: "invite",
                entity_id: Some(invite_id),
                before: None,
                after: Some(json!({
                    "tenant_id": request.tenant.0,
                    "person_id": request.person.0,
                    "role": role_name(request.role),
                    "channel": request.channel.as_str(),
                })),
                actor: request.actor,
            },
        )
        .await?;

        emit(
            tx,
            request.tenant,
            &UserInvitedEvent {
                person_id: request.person,
                role: request.role,
                invite_id,
            },
        )
        .await?;

        let invite_url = if request.channel == InviteChannel::Link {
            Some(build_invite_url(request.config, locale, &token)?)
        } else {
            None
        };

        Ok(InviteResult {
            invite_id,
            invite_url,
        })
    }
}

#[derive(Serialize)]
struct UserInvitedEvent {
    person_id: PersonId,
    role: Role,
    invite_id: Uuid,
}

impl DomainEvent for UserInvitedEvent {
    const TYPE: &'static str = "identity.user_invited.v1";

    fn aggregate_id(&self) -> Option<Uuid> {
        Some(self.person_id.0)
    }
}

fn build_invite_url(config: &Config, locale: Locale, token: &SecretString) -> Result<SecretString, AppError> {
    let mut invite_url = config.public_url.clone();
    let language = match locale {
        Locale::En => "en",
        Locale::Id => "id",
    };
    invite_url
        .path_segments_mut()
        .map_err(|_| AppError::Internal(anyhow::anyhow!("public URL cannot contain invite path segments")))?
        .pop_if_empty()
        .push(language)
        .push("invite")
        .push(token.expose_secret());
    Ok(SecretString::from(invite_url.to_string()))
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

#[cfg(test)]
mod tests {
    use super::InviteResult;
    use secrecy::SecretString;
    use uuid::Uuid;

    #[test]
    fn debug_redacts_fake_invite_url() {
        let result = InviteResult {
            invite_id: Uuid::now_v7(),
            invite_url: Some(SecretString::from(
                "https://ritma.example.test/en/invite/test-secret-invite-token",
            )),
        };

        let debug = format!("{result:?}");
        assert!(!debug.contains("test-secret-invite-token"));
        assert!(debug.contains("[REDACTED]"));
    }
}
