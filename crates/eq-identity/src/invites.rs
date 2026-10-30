//! IDN-07: invite issuance, resend revocation and link-channel delivery.

use crate::policy::IdentityPolicy;
use crate::repo;
use crate::service::{LoginService, LoginSuccess};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Duration, Utc};
use eq_audit::{AuditEvent, Category, record};
use eq_core::actor::{ActorCtx, ActorRole};
use eq_core::auth::CoreIdentityDirectory;
use eq_core::clock::Clock;
use eq_core::config::Config;
use eq_core::crypto::Keyring;
use eq_core::db::Tx;
use eq_core::error::AppError;
use eq_core::fields::Password;
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
use std::net::IpAddr;
use subtle::ConstantTimeEq;
use utoipa::ToSchema;
use uuid::Uuid;
use zeroize::Zeroizing;

const INVITE_TOKEN_BYTES: usize = 32; // technical: matches the 32 random bytes generated for invite tokens.
const INVITE_TOKEN_ENCODED_BYTES: usize = 43; // technical: unpadded base64url length of 32 token bytes.

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

/// IDN-07: public, non-secret preview data for one pending invitation.
#[derive(Serialize, ToSchema)]
pub struct InvitePreview {
    pub tenant_name: String,
    pub display_name: String,
    #[schema(value_type = String)]
    pub role: Role,
    pub expires_at: DateTime<Utc>,
}

impl fmt::Debug for InvitePreview {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("InvitePreview")
            .field("tenant_name", &self.tenant_name)
            .field("display_name", &"[REDACTED]")
            .field("role", &self.role)
            .field("expires_at", &self.expires_at)
            .finish()
    }
}

/// IDN-07/SBD-50: values needed to accept one invite without exposing secrets in Debug.
pub struct InviteAcceptanceRequest<'a> {
    tenant: TenantId,
    actor: &'a ActorCtx,
    token: SecretString,
    password: &'a Password,
    policy: &'a IdentityPolicy,
    user_agent: Option<&'a str>,
    ip: Option<IpAddr>,
}

impl<'a> InviteAcceptanceRequest<'a> {
    /// IDN-07: captures the tenant-aware invite token, new password, policy, and request metadata.
    pub fn new(
        tenant: TenantId,
        actor: &'a ActorCtx,
        token: SecretString,
        password: &'a Password,
        policy: &'a IdentityPolicy,
        user_agent: Option<&'a str>,
        ip: Option<IpAddr>,
    ) -> Self {
        Self {
            tenant,
            actor,
            token,
            password,
            policy,
            user_agent,
            ip,
        }
    }
}

impl fmt::Debug for InviteAcceptanceRequest<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("InviteAcceptanceRequest")
            .field("tenant", &self.tenant)
            .field("actor", &"[REDACTED]")
            .field("token", &"[REDACTED]")
            .field("password", &"[REDACTED]")
            .field("policy", &"[identity policy]")
            .field("user_agent", &"[REDACTED]")
            .field("ip", &"[REDACTED]")
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

    /// IDN-07/STM-21: returns a tenant-scoped pending invite preview without exposing profile PII.
    pub async fn preview_invite(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        token: &SecretString,
    ) -> Result<InvitePreview, AppError> {
        let now = self.clock.now();
        let invite = self.pending_invite(tx, tenant, token, now).await?;
        let profile = self
            .directory
            .login_profile(tx, tenant, invite.person, false, self.keyring)
            .await?
            .ok_or(AppError::NotFound { entity: "invite" })?;
        let tenant_name = self
            .directory
            .tenant_name(tx, tenant)
            .await?
            .ok_or(AppError::NotFound { entity: "invite" })?;
        Ok(InvitePreview {
            tenant_name,
            display_name: profile.display_name,
            role: invite.role,
            expires_at: invite.expires_at,
        })
    }

    /// IDN-07/STM-20: resolves the bearer invite to its invitee for the acceptance transaction actor.
    pub(crate) async fn acceptance_actor(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        token: &SecretString,
        request_actor: &ActorCtx,
    ) -> Result<ActorCtx, AppError> {
        let invite = self.pending_invite(tx, tenant, token, self.clock.now()).await?;
        Ok(ActorCtx {
            person_id: Some(invite.person),
            role: actor_role(invite.role),
            ip: request_actor.ip,
            request_id: request_actor.request_id,
        })
    }

    /// IDN-07/STM-20/21: consumes a pending invite, activates its membership, sets credentials, and starts a session.
    pub async fn accept_invite(
        &self,
        tx: &mut Tx,
        request: InviteAcceptanceRequest<'_>,
        login: &LoginService<'_>,
    ) -> Result<LoginSuccess, AppError> {
        let now = self.clock.now();
        let invite = self
            .pending_invite(tx, request.tenant, &request.token, now)
            .await?;
        crate::password::validate_new_password(request.password, request.policy).map_err(|error| {
            error.app_code().map_or_else(
                || AppError::Internal(anyhow::anyhow!("invite password validation failed: {error}")),
                |code| AppError::Unprocessable {
                    code,
                    detail: String::from("password does not meet the current identity policy"),
                    extra: serde_json::Map::new(),
                },
            )
        })?;
        let identity = self
            .directory
            .activate_invited_membership(tx, request.tenant, invite.person, invite.role)
            .await?
            .ok_or_else(invite_expired)?;
        if !repo::mark_invite_accepted(tx, request.tenant, invite.id, now).await? {
            return Err(invite_expired());
        }
        let success = login
            .accept_invited_identity(
                tx,
                request.tenant,
                request.actor,
                identity,
                request.password,
                request.policy,
                request.user_agent,
                request.ip,
                now,
            )
            .await?;
        record(
            tx,
            AuditEvent {
                action: "identity.invite.accepted",
                category: Category::Security,
                entity_type: "invite",
                entity_id: Some(invite.id),
                before: Some(json!({"status": "pending"})),
                after: Some(json!({
                    "tenant_id": request.tenant.0,
                    "person_id": invite.person.0,
                    "role": role_name(invite.role),
                })),
                actor: request.actor,
            },
        )
        .await?;
        Ok(success)
    }

    async fn pending_invite(
        &self,
        tx: &mut Tx,
        tenant: TenantId,
        token: &SecretString,
        now: DateTime<Utc>,
    ) -> Result<repo::InviteRecord, AppError> {
        let parsed_hash = invite_token_hash(token);
        let digest = parsed_hash.unwrap_or([0; INVITE_TOKEN_BYTES]);
        let candidate = repo::invite_for_token(tx, tenant, &digest).await?;
        let dummy_hash = [0; INVITE_TOKEN_BYTES];
        let stored_hash = candidate
            .as_ref()
            .map(repo::InviteRecord::token_hash)
            .unwrap_or(&dummy_hash);
        let matches = bool::from(stored_hash.ct_eq(&digest));
        let invite = candidate
            .filter(|_| parsed_hash.is_some() && matches)
            .ok_or(AppError::NotFound { entity: "invite" })?;
        if invite.accepted_at.is_some() || invite.expires_at <= now {
            return Err(invite_expired());
        }
        Ok(invite)
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

fn actor_role(role: Role) -> ActorRole {
    match role {
        Role::Owner => ActorRole::Owner,
        Role::Admin => ActorRole::Admin,
        Role::Instructor => ActorRole::Instructor,
        Role::Guardian => ActorRole::Guardian,
        Role::Student => ActorRole::Student,
    }
}

fn invite_expired() -> AppError {
    AppError::Gone {
        code: &crate::errors::codes::INVITE_EXPIRED,
    }
}

fn invite_token_hash(token: &SecretString) -> Option<[u8; 32]> {
    if token.expose_secret().len() != INVITE_TOKEN_ENCODED_BYTES {
        return None;
    }
    let decoded = Zeroizing::new(URL_SAFE_NO_PAD.decode(token.expose_secret()).ok()?);
    if decoded.len() != INVITE_TOKEN_BYTES {
        return None;
    }
    Some(Sha256::digest(decoded.as_slice()).into())
}

#[cfg(test)]
mod tests {
    use super::{InviteAcceptanceRequest, InviteResult};
    use crate::policy::IdentityPolicy;
    use eq_core::actor::{ActorCtx, ActorRole};
    use eq_core::fields::{FieldCtx, FieldKind, Password, PasswordInput};
    use eq_core::ids::TenantId;
    use eq_core::money::Currency;
    use eq_core::policy::types::Locale;
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

    #[test]
    fn acceptance_request_debug_redacts_fake_token_and_password() {
        let actor = ActorCtx {
            person_id: None,
            role: ActorRole::Anonymous,
            ip: None,
            request_id: None,
        };
        let context = FieldCtx {
            country: None,
            tenant_currency: Currency::USD,
            tenant_locale: Locale::En,
            today: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).expect("test date is valid"),
            password_min_length: None,
        };
        let raw: PasswordInput = serde_json::from_str("\"test-secret-invite-password\"")
            .expect("fake password input deserializes");
        let password = Password::parse(raw, &context).expect("fake password is valid");
        let policy = IdentityPolicy::default();
        let request = InviteAcceptanceRequest::new(
            TenantId(Uuid::now_v7()),
            &actor,
            SecretString::from("test-secret-invite-token"),
            &password,
            &policy,
            None,
            None,
        );
        let debug = format!("{request:?}");
        assert!(!debug.contains("test-secret-invite-token"));
        assert!(!debug.contains("test-secret-invite-password"));
        assert!(debug.contains("[REDACTED]"));
    }
}
