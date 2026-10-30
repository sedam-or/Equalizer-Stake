//! IDN-02 password policy, Argon2id hashing and verification primitives.

use crate::errors::codes::PASSWORD_WEAK;
use crate::policy::IdentityPolicy;
use argon2::{Algorithm, Argon2, Params, Version};
use eq_core::error::ErrorCodeDef;
use eq_core::fields::Password;
use password_hash::{
    Error as PasswordHashFormatError, PasswordHash as ParsedPasswordHash, PasswordHasher, PasswordVerifier,
    SaltString,
};
use rand::RngCore;
use rand::rngs::OsRng;
use secrecy::{ExposeSecret, SecretString};
use std::collections::HashSet;
use std::fmt;
use std::sync::OnceLock;
use thiserror::Error;
use zeroize::Zeroizing;

// technical: IDN-02 fixes the Argon2id memory cost at 19 MiB.
const ARGON2_MEMORY_KIB: u32 = 19_456;
// technical: IDN-02 fixes the Argon2id iteration count.
const ARGON2_ITERATIONS: u32 = 2;
// technical: IDN-02 fixes the Argon2id parallelism.
const ARGON2_PARALLELISM: u32 = 1;
// technical: A random dummy input is process-local and never represents a user password.
const DUMMY_PASSWORD_BYTES: usize = 32;

static COMMON_PASSWORDS: OnceLock<HashSet<&'static str>> = OnceLock::new();

fn common_passwords() -> &'static HashSet<&'static str> {
    COMMON_PASSWORDS.get_or_init(|| {
        include_str!("../data/common-passwords.txt")
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect()
    })
}

/// IDN-02: new passwords use the policy minimum and bundled common-password list.
///
/// # Errors
/// Returns `PasswordError::Weak` when the password is shorter than the policy minimum
/// or occurs in the bundled common-password corpus.
pub fn validate_new_password(password: &Password, policy: &IdentityPolicy) -> Result<(), PasswordError> {
    let value = password.expose_secret();
    if value.chars().count() < usize::from(policy.password_min_length()) || common_passwords().contains(value)
    {
        return Err(PasswordError::Weak);
    }
    Ok(())
}

/// IDN-02: typed password validation, hashing and verification failures.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PasswordError {
    /// The password is below the policy minimum or appears in the common-password corpus.
    #[error("password does not meet the identity password policy")]
    Weak,
    /// The fixed Argon2id parameters failed validation.
    #[error("invalid Argon2id parameters")]
    Parameters,
    /// Argon2 could not hash or verify an encoded credential.
    #[error("Argon2id operation failed")]
    Operation,
    /// A persisted PHC value could not be parsed.
    #[error("stored password hash is invalid")]
    InvalidStoredHash,
    /// The system random source could not supply dummy password bytes.
    #[error("system random source failed")]
    Random,
}

impl PasswordError {
    /// ERR-21: returns the registered Identity application code for a policy rejection.
    pub const fn app_code(&self) -> Option<&'static ErrorCodeDef> {
        match self {
            Self::Weak => Some(&PASSWORD_WEAK),
            Self::Parameters | Self::Operation | Self::InvalidStoredHash | Self::Random => None,
        }
    }
}

/// SBD-50: stored Argon2 output is secret-bearing credential material.
pub struct PasswordHash(SecretString);

impl PasswordHash {
    /// IDN-02/SBD-50: exposes the encoded Argon2 hash to the credential repository.
    pub fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }
}

impl TryFrom<String> for PasswordHash {
    type Error = PasswordError;

    fn try_from(encoded: String) -> Result<Self, Self::Error> {
        let encoded = SecretString::from(encoded);
        ParsedPasswordHash::new(encoded.expose_secret()).map_err(|_| PasswordError::InvalidStoredHash)?;
        Ok(Self(encoded))
    }
}

impl fmt::Debug for PasswordHash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PasswordHash([REDACTED])")
    }
}

fn configured_argon2() -> Result<Argon2<'static>, PasswordError> {
    let params = Params::new(ARGON2_MEMORY_KIB, ARGON2_ITERATIONS, ARGON2_PARALLELISM, None)
        .map_err(|_| PasswordError::Parameters)?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

/// IDN-02: hashes a validated password with a fresh random salt.
///
/// # Errors
/// Returns `PasswordError::Parameters` if the fixed Argon2 parameters are invalid, or
/// `PasswordError::Operation` if Argon2 cannot create the encoded hash.
pub fn hash_password(password: &Password) -> Result<PasswordHash, PasswordError> {
    hash_password_bytes(password.expose_secret().as_bytes())
}

/// IDN-12/SBD-50: hashes one recovery code with the existing Argon2id credential parameters.
pub(crate) fn hash_recovery_code(code: &SecretString) -> Result<PasswordHash, PasswordError> {
    hash_password_bytes(code.expose_secret().as_bytes())
}

fn hash_password_bytes(password: &[u8]) -> Result<PasswordHash, PasswordError> {
    let salt = SaltString::generate(&mut OsRng);
    let encoded = configured_argon2()?
        .hash_password(password, &salt)
        .map_err(|_| PasswordError::Operation)?
        .to_string();
    PasswordHash::try_from(encoded)
}

/// IDN-02: verifies a password against an encoded Argon2id credential.
///
/// # Errors
/// Returns `PasswordError::InvalidStoredHash` for malformed stored data,
/// `PasswordError::Parameters` for invalid fixed parameters, or
/// `PasswordError::Operation` for another Argon2 verification failure.
pub fn verify_password(password: &Password, stored_hash: &PasswordHash) -> Result<bool, PasswordError> {
    verify_secret_bytes(password.expose_secret().as_bytes(), stored_hash)
}

/// IDN-12/SBD-50: verifies one recovery code without converting its secret into a password field.
pub(crate) fn verify_recovery_code(
    code: &SecretString,
    stored_hash: &PasswordHash,
) -> Result<bool, PasswordError> {
    verify_secret_bytes(code.expose_secret().as_bytes(), stored_hash)
}

fn verify_secret_bytes(secret: &[u8], stored_hash: &PasswordHash) -> Result<bool, PasswordError> {
    let parsed =
        ParsedPasswordHash::new(stored_hash.expose_secret()).map_err(|_| PasswordError::InvalidStoredHash)?;
    match configured_argon2()?.verify_password(secret, &parsed) {
        Ok(()) => Ok(true),
        Err(PasswordHashFormatError::Password) => Ok(false),
        Err(_) => Err(PasswordError::Operation),
    }
}

/// IDN-06: creates one random process-local hash for unknown-account dummy work.
///
/// # Errors
/// Returns `PasswordError::Random`, `PasswordError::Parameters`, or
/// `PasswordError::Operation` if initialization fails.
pub fn create_dummy_password_hash() -> Result<PasswordHash, PasswordError> {
    let mut random_password = Zeroizing::new([0_u8; DUMMY_PASSWORD_BYTES]);
    let mut rng = OsRng;
    rng.try_fill_bytes(&mut random_password[..])
        .map_err(|_| PasswordError::Random)?;
    hash_password_bytes(&random_password[..])
}

/// IDN-06: performs the same Argon2id verification work without returning a match result.
///
/// # Errors
/// Returns a stored-hash, parameter or Argon2 operation error from verification.
pub fn verify_dummy_password(password: &Password, dummy_hash: &PasswordHash) -> Result<(), PasswordError> {
    let _matched = verify_password(password, dummy_hash)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        ARGON2_ITERATIONS, ARGON2_MEMORY_KIB, ARGON2_PARALLELISM, PasswordHash, create_dummy_password_hash,
        hash_password, hash_recovery_code, validate_new_password, verify_dummy_password, verify_password,
        verify_recovery_code,
    };
    use crate::policy::IdentityPolicy;
    use argon2::Params;
    use eq_core::fields::{FieldCtx, FieldKind, Password, PasswordInput};
    use eq_core::money::Currency;
    use eq_core::policy::types::Locale;
    use secrecy::SecretString;

    #[test]
    fn common_rejected() {
        let password = parse_password("password123").expect("test password input parses");
        let error = validate_new_password(&password, &IdentityPolicy::default())
            .expect_err("known common password is rejected");
        let code = error.app_code().expect("weak-password code is registered");

        assert!(crate::errors::codes::CODES.contains(&code));
        assert_eq!(code.code, "password_weak");
        assert_eq!(code.status, 422);
    }

    #[test]
    fn max_len_128() {
        let accepted = "🦀".repeat(128);
        let rejected = "🦀".repeat(129);

        assert!(parse_password(&accepted).is_ok());
        assert!(parse_password(&rejected).is_err());
    }

    #[test]
    fn argon_params() {
        let params = Params::new(ARGON2_MEMORY_KIB, ARGON2_ITERATIONS, ARGON2_PARALLELISM, None)
            .expect("IDN-02 parameters are valid");

        assert_eq!(params.m_cost(), 19_456);
        assert_eq!(params.t_cost(), 2);
        assert_eq!(params.p_cost(), 1);
    }

    #[test]
    fn hash_and_dummy_verification() {
        let correct = parse_password("test-secret-correct-password").expect("fake test password parses");
        let incorrect = parse_password("test-secret-incorrect-password").expect("fake test password parses");
        let stored = hash_password(&correct).expect("password hash succeeds");
        let dummy = create_dummy_password_hash().expect("dummy hash succeeds");

        assert!(verify_password(&correct, &stored).expect("hash verifies"));
        assert!(!verify_password(&incorrect, &stored).expect("mismatch verifies in constant work"));
        verify_dummy_password(&incorrect, &dummy).expect("dummy verification completes");
    }

    #[test]
    fn recovery_code_hash_verifies_only_the_matching_secret() {
        let code = SecretString::from("test-secret-recovery-code");
        let incorrect = SecretString::from("test-secret-other-recovery");
        let stored = hash_recovery_code(&code).expect("recovery code hash succeeds");

        assert!(verify_recovery_code(&code, &stored).expect("recovery code verifies"));
        assert!(!verify_recovery_code(&incorrect, &stored).expect("different recovery code mismatches"));
        assert!(!format!("{code:?}").contains("test-secret-recovery-code"));
    }

    #[test]
    fn secret_debug_is_redacted() {
        let password = parse_password("test-secret-password").expect("test password parses");
        let encoded = hash_password(&password).expect("password hash succeeds");

        assert!(!format!("{password:?}").contains("test-secret-password"));
        assert!(!format!("{encoded:?}").contains(encoded.expose_secret()));
        assert!(!format!("{encoded:?}").contains("test-secret-password"));
        let parsed = PasswordHash::try_from(encoded.expose_secret().to_owned())
            .expect("the generated PHC value parses");
        assert!(!format!("{parsed:?}").contains(parsed.expose_secret()));
    }

    fn parse_password(value: &str) -> Result<Password, Vec<eq_core::fields::FieldIssue>> {
        let encoded = serde_json::to_string(value).expect("test password serializes");
        let input = serde_json::from_str::<PasswordInput>(&encoded)
            .expect("test password deserializes into a secrecy wrapper");
        Password::parse(input, &test_context())
    }

    fn test_context() -> FieldCtx {
        FieldCtx {
            country: Some(celes::Country::from_alpha2("ID").expect("known test country")),
            tenant_currency: Currency::IDR,
            tenant_locale: Locale::Id,
            today: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid test date"),
            password_min_length: None,
        }
    }
}
