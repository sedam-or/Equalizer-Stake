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

    /// IDN-07/STM-21/ERR-21: an invite is expired or has already been consumed.
    pub static INVITE_EXPIRED: ErrorCodeDef = ErrorCodeDef {
        code: "invite_expired",
        status: 410,
        title: "Invite expired",
    };

    /// Registered application codes owned by Identity.
    pub const CODES: &[&ErrorCodeDef] = &[
        &INVALID_CREDENTIALS,
        &SESSION_EXPIRED,
        &TOTP_REQUIRED,
        &PASSWORD_WEAK,
        &INVITE_EXPIRED,
    ];
}
