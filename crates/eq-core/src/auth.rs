//! Core-owned identity lookup and cryptographic dependencies (CA-30, SEC-04).

pub mod directory;

use crate::crypto::{Keyring, Keys};
use std::fmt;
use std::sync::Arc;

pub use directory::{CoreIdentityDirectory, LoginIdentity, PgIdentityDirectory};

/// SBD-50: key material shared by login services without exposing secrets in Debug.
pub struct AuthState {
    keys: Arc<Keys>,
    keyring: Keyring,
    directory: PgIdentityDirectory,
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
