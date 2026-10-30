//! Identity-owned application error codes (ERR-21).

use eq_core::error::ErrorCodeDef;

/// Identity error codes included in the API registry.
pub mod codes {
    use super::ErrorCodeDef;

    /// IDN-06/ERR-21: generic login failure that does not disclose whether an account exists.
    pub static INVALID_CREDENTIALS: ErrorCodeDef = ErrorCodeDef {
        code: "invalid_credentials",
        status: 401,
        title: "Invalid credentials",
    };

    /// IDN-04/STM-22: an unavailable, expired, revoked or replayed refresh session needs a new login.
    pub static SESSION_EXPIRED: ErrorCodeDef = ErrorCodeDef {
        code: "session_expired",
        status: 401,
        title: "Session expired",
    };

    /// IDN-12/ERR-21: an enrolled account needs a valid TOTP code to finish authentication.
    pub static TOTP_REQUIRED: ErrorCodeDef = ErrorCodeDef {
        code: "totp_required",
        status: 401,
        title: "TOTP required",
    };

    /// IDN-02: new password is shorter than policy or in the common-password corpus.
    pub static PASSWORD_WEAK: ErrorCodeDef = ErrorCodeDef {
        code: "password_weak",
        status: 422,
        title: "Password does not meet the password policy",
    };

    /// IDN-08/VAL-21: an unknown, malformed, expired or consumed reset token is invalid.
    pub static TOKEN_INVALID: ErrorCodeDef = ErrorCodeDef {
        code: "token_invalid",
        status: 400,
        title: "Invalid token",
    };

    /// IDN-07/STM-21/ERR-21: an invite is expired or has already been consumed.
    pub static INVITE_EXPIRED: ErrorCodeDef = ErrorCodeDef {
        code: "invite_expired",
        status: 410,
        title: "Invite expired",
    };

    /// STM-20: removing the final active tenant owner is rejected.
    pub static LAST_OWNER: ErrorCodeDef = ErrorCodeDef {
        code: "last_owner",
        status: 409,
        title: "The last owner cannot be removed",
    };

    /// STM-27: a student may have only one primary guardian.
    pub static PRIMARY_EXISTS: ErrorCodeDef = ErrorCodeDef {
        code: "primary_exists",
        status: 409,
        title: "A primary guardian already exists",
    };

    /// IDN-10: normalized tenant email blind index already belongs to another person.
    pub static EMAIL_TAKEN: ErrorCodeDef = ErrorCodeDef {
        code: "email_taken",
        status: 409,
        title: "Email is already used in this tenant",
    };

    /// Registered application codes owned by Identity.
    pub const CODES: &[&ErrorCodeDef] = &[
        &INVALID_CREDENTIALS,
        &SESSION_EXPIRED,
        &TOTP_REQUIRED,
        &PASSWORD_WEAK,
        &TOKEN_INVALID,
        &INVITE_EXPIRED,
        &LAST_OWNER,
        &PRIMARY_EXISTS,
        &EMAIL_TAKEN,
    ];
}

/// ERR-21: application error codes owned by Identity.
pub use codes::CODES;
