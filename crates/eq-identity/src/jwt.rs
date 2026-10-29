//! IDN-03 HS256 access-token issue and validation helpers.

use eq_core::clock::Clock;
use eq_core::crypto::Keys;
use eq_core::ids::{PersonId, TenantId};
use eq_core::model::Role;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

// technical: IDN-03 access tokens expire after ten minutes.
const ACCESS_TOKEN_TTL_SECONDS: u64 = 600;
// technical: IDN-03 allows the access-token verifier a thirty-second clock-skew leeway.
const ACCESS_TOKEN_LEEWAY_SECONDS: u64 = 30;

/// IDN-03/CQ-23: typed subject values used to issue one access token.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AccessTokenSubject {
    person_id: PersonId,
    tenant_id: TenantId,
    role: Role,
    session_id: Uuid,
    password_version: i64,
}

impl AccessTokenSubject {
    /// IDN-03/CQ-23: creates a subject from the identity module's typed values.
    pub const fn new(
        person_id: PersonId,
        tenant_id: TenantId,
        role: Role,
        session_id: Uuid,
        password_version: i64,
    ) -> Self {
        Self {
            person_id,
            tenant_id,
            role,
            session_id,
            password_version,
        }
    }

    /// IDN-03: returns the authenticated person identifier.
    pub const fn person_id(&self) -> PersonId {
        self.person_id
    }

    /// IDN-03: returns the selected tenant identifier.
    pub const fn tenant_id(&self) -> TenantId {
        self.tenant_id
    }

    /// IDN-03: returns the active membership role.
    pub const fn role(&self) -> Role {
        self.role
    }

    /// IDN-03: returns the refresh-session identifier.
    pub const fn session_id(&self) -> Uuid {
        self.session_id
    }

    /// IDN-03: returns the password-change epoch, or zero when absent.
    pub const fn password_version(&self) -> i64 {
        self.password_version
    }
}

/// IDN-03: private wire claims carried by an access token; unknown claims are rejected.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AccessClaimsWire {
    sub: PersonId,
    tid: TenantId,
    role: Role,
    sid: Uuid,
    iat: u64,
    exp: u64,
    ver: i64,
}

/// IDN-03/CQ-23: claims returned only after token and request-context validation.
pub struct AccessClaims {
    sub: PersonId,
    tid: TenantId,
    role: Role,
    sid: Uuid,
    iat: u64,
    exp: u64,
    ver: i64,
}

impl AccessClaims {
    /// IDN-03: returns the validated person identifier (`sub`).
    pub const fn sub(&self) -> PersonId {
        self.sub
    }

    /// IDN-03: returns the validated tenant identifier (`tid`).
    pub const fn tid(&self) -> TenantId {
        self.tid
    }

    /// IDN-03: returns the validated membership role.
    pub const fn role(&self) -> Role {
        self.role
    }

    /// IDN-03: returns the validated refresh-session identifier (`sid`).
    pub const fn sid(&self) -> Uuid {
        self.sid
    }

    /// IDN-03: returns the issued-at Unix timestamp (`iat`).
    pub const fn iat(&self) -> u64 {
        self.iat
    }

    /// IDN-03: returns the expiry Unix timestamp (`exp`).
    pub const fn exp(&self) -> u64 {
        self.exp
    }

    /// IDN-03: returns the credential password-change epoch (`ver`).
    pub const fn ver(&self) -> i64 {
        self.ver
    }
}

/// SBD-50: bearer access tokens must not be formatted or cloned as plain strings.
pub struct AccessToken(SecretString);

impl AccessToken {
    /// IDN-03/SBD-50: exposes the token only to the HTTP response edge.
    pub fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }
}

impl fmt::Debug for AccessToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("AccessToken([REDACTED])")
    }
}

/// IDN-03: safe token validation failures that do not contain token or key material.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum JwtError {
    /// Signature, format or claim shape is invalid.
    #[error("invalid access token")]
    Invalid,
    /// Expiry is older than the allowed leeway.
    #[error("access token has expired")]
    Expired,
    /// Token tenant differs from trusted request context.
    #[error("access token tenant does not match request context")]
    WrongTenant,
    /// Password-change epoch differs from the current credential version.
    #[error("access token credential version is stale")]
    VersionMismatch,
    /// The injected clock cannot be represented as a JWT Unix timestamp.
    #[error("clock timestamp is outside the supported Unix range")]
    InvalidTime,
}

/// IDN-03: signs the specified claims with the shipped tenant-independent JWT key.
///
/// # Errors
/// Returns `JwtError::InvalidTime` if the injected timestamp or expiry cannot be
/// represented, or `JwtError::Invalid` if HS256 encoding fails.
pub fn issue_access_token(
    keys: &Keys,
    clock: &dyn Clock,
    subject: AccessTokenSubject,
) -> Result<AccessToken, JwtError> {
    let issued_at = unix_seconds(clock.now().timestamp())?;
    let claims = AccessClaimsWire {
        sub: subject.person_id,
        tid: subject.tenant_id,
        role: subject.role,
        sid: subject.session_id,
        iat: issued_at,
        exp: issued_at
            .checked_add(ACCESS_TOKEN_TTL_SECONDS)
            .ok_or(JwtError::InvalidTime)?,
        ver: subject.password_version,
    };
    let secret = keys.jwt.expose_secret();
    let encoded = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(&secret[..]),
    )
    .map_err(|_| JwtError::Invalid)?;
    Ok(AccessToken(SecretString::from(encoded)))
}

/// IDN-03: verifies signature, expiry, tenant and password-change version.
///
/// # Errors
/// Returns `JwtError::Invalid` for a malformed token, unsupported claims or invalid
/// signature; `JwtError::Expired`, `JwtError::WrongTenant`, `JwtError::VersionMismatch`
/// or `JwtError::InvalidTime` for the corresponding validation failure.
pub fn validate_access_token(
    token: &str,
    keys: &Keys,
    expected_tenant: TenantId,
    expected_password_version: i64,
    clock: &dyn Clock,
) -> Result<AccessClaims, JwtError> {
    let secret = keys.jwt.expose_secret();
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = false;
    validation.leeway = 0;
    let claims = decode::<AccessClaimsWire>(token, &DecodingKey::from_secret(&secret[..]), &validation)
        .map_err(|_| JwtError::Invalid)?
        .claims;

    let now = unix_seconds(clock.now().timestamp())?;
    if claims.exp <= claims.iat {
        return Err(JwtError::Invalid);
    }
    let expiry_with_leeway = claims
        .exp
        .checked_add(ACCESS_TOKEN_LEEWAY_SECONDS)
        .ok_or(JwtError::InvalidTime)?;
    if now >= expiry_with_leeway {
        return Err(JwtError::Expired);
    }
    if claims.tid != expected_tenant {
        return Err(JwtError::WrongTenant);
    }
    if claims.ver != expected_password_version {
        return Err(JwtError::VersionMismatch);
    }
    Ok(AccessClaims {
        sub: claims.sub,
        tid: claims.tid,
        role: claims.role,
        sid: claims.sid,
        iat: claims.iat,
        exp: claims.exp,
        ver: claims.ver,
    })
}

fn unix_seconds(timestamp: i64) -> Result<u64, JwtError> {
    u64::try_from(timestamp).map_err(|_| JwtError::InvalidTime)
}

#[cfg(test)]
mod tests {
    use super::{
        ACCESS_TOKEN_LEEWAY_SECONDS, ACCESS_TOKEN_TTL_SECONDS, AccessTokenSubject, JwtError,
        issue_access_token, validate_access_token,
    };
    use chrono::{TimeZone, Utc};
    use eq_core::clock::{Clock, Timestamp};
    use eq_core::crypto::Keys;
    use eq_core::ids::{PersonId, TenantId};
    use eq_core::model::Role;
    use secrecy::ExposeSecret;
    use uuid::Uuid;

    #[test]
    fn roundtrip() {
        let clock = FixedClock::at(2026, 1, 2, 3, 4, 5);
        let keys = test_keys();
        let subject = test_subject(10);
        let token = issue_access_token(&keys, &clock, subject).expect("token issues");

        let claims = validate_access_token(
            token.expose_secret(),
            &keys,
            subject.tenant_id(),
            subject.password_version(),
            &clock,
        )
        .expect("issued token validates");

        assert_eq!(claims.sub(), subject.person_id());
        assert_eq!(claims.tid(), subject.tenant_id());
        assert_eq!(claims.role(), Role::Admin);
        assert_eq!(claims.sid(), subject.session_id());
        assert_eq!(claims.ver(), subject.password_version());
        assert_eq!(claims.exp() - claims.iat(), ACCESS_TOKEN_TTL_SECONDS);
    }

    #[test]
    fn expired_rejected() {
        let issued_clock = FixedClock::at(2026, 1, 2, 3, 4, 5);
        let seconds_after_issue = i64::try_from(ACCESS_TOKEN_TTL_SECONDS + ACCESS_TOKEN_LEEWAY_SECONDS)
            .expect("small technical token interval fits i64");
        let validation_clock =
            FixedClock(issued_clock.0.clone() + chrono::Duration::seconds(seconds_after_issue));
        let keys = test_keys();
        let subject = test_subject(10);
        let token = issue_access_token(&keys, &issued_clock, subject).expect("token issues");

        assert!(matches!(
            validate_access_token(
                token.expose_secret(),
                &keys,
                subject.tenant_id(),
                subject.password_version(),
                &validation_clock,
            ),
            Err(JwtError::Expired)
        ));
    }

    #[test]
    fn wrong_tenant_rejected() {
        let clock = FixedClock::at(2026, 1, 2, 3, 4, 5);
        let keys = test_keys();
        let subject = test_subject(10);
        let token = issue_access_token(&keys, &clock, subject).expect("token issues");
        let other_tenant = TenantId(Uuid::from_u128(3));

        assert!(matches!(
            validate_access_token(
                token.expose_secret(),
                &keys,
                other_tenant,
                subject.password_version(),
                &clock,
            ),
            Err(JwtError::WrongTenant)
        ));
    }

    #[test]
    fn password_version_mismatch_rejected() {
        let clock = FixedClock::at(2026, 1, 2, 3, 4, 5);
        let keys = test_keys();
        let subject = test_subject(10);
        let token = issue_access_token(&keys, &clock, subject).expect("token issues");

        assert!(matches!(
            validate_access_token(
                token.expose_secret(),
                &keys,
                subject.tenant_id(),
                subject.password_version() + 1,
                &clock,
            ),
            Err(JwtError::VersionMismatch)
        ));
    }

    #[test]
    fn invalid_signature_rejected() {
        let clock = FixedClock::at(2026, 1, 2, 3, 4, 5);
        let signing_keys = test_keys();
        let other_keys = Keys::derive(b"test-secret-different-jwt-master-key", None);
        let subject = test_subject(10);
        let token = issue_access_token(&signing_keys, &clock, subject).expect("token issues");

        assert!(matches!(
            validate_access_token(
                token.expose_secret(),
                &other_keys,
                subject.tenant_id(),
                subject.password_version(),
                &clock,
            ),
            Err(JwtError::Invalid)
        ));
    }

    #[test]
    fn key_and_token_debug_are_redacted() {
        let clock = FixedClock::at(2026, 1, 2, 3, 4, 5);
        let keys = test_keys();
        let token = issue_access_token(&keys, &clock, test_subject(0)).expect("token issues");
        let jwt_key_debug = format!("{:?}", keys.jwt.expose_secret());

        assert!(!format!("{keys:?}").contains(&jwt_key_debug));
        assert!(!format!("{token:?}").contains(token.expose_secret()));
    }

    fn test_keys() -> Keys {
        Keys::derive(b"test-secret-jwt-master-key", None)
    }

    fn test_subject(password_version: i64) -> AccessTokenSubject {
        AccessTokenSubject::new(
            PersonId(Uuid::from_u128(1)),
            TenantId(Uuid::from_u128(2)),
            Role::Admin,
            Uuid::from_u128(4),
            password_version,
        )
    }

    #[derive(Debug)]
    struct FixedClock(Timestamp);

    impl FixedClock {
        fn at(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> Self {
            Self(
                Utc.with_ymd_and_hms(year, month, day, hour, minute, second)
                    .single()
                    .expect("fixed test timestamp is valid"),
            )
        }
    }

    impl Clock for FixedClock {
        fn now(&self) -> Timestamp {
            self.0
        }
    }
}
