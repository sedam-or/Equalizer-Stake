//! Identity-owned application error codes (ERR-21).

use eq_core::error::ErrorCodeDef;

/// Identity error codes included in the API registry.
pub mod codes {
    use super::ErrorCodeDef;

    /// IDN-02: new password is shorter than policy or in the common-password corpus.
    pub static PASSWORD_WEAK: ErrorCodeDef = ErrorCodeDef {
        code: "password_weak",
        status: 422,
        title: "Password does not meet the password policy",
    };

    /// Registered application codes owned by Identity.
    pub const CODES: &[&ErrorCodeDef] = &[&PASSWORD_WEAK];
}
