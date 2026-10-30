//! Shared application errors and RFC 9457 responses (ERR-10, ERR-30).

use anyhow::Error as AnyError;
use axum::Json;
use axum::http::header::{CONTENT_TYPE, RETRY_AFTER};
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::{Map, Value, json};
use uuid::Uuid;

pub use crate::errors::{
    FIELD_ERROR_CODES, INVALID_CHARACTERS, INVALID_FORMAT, INVALID_TYPE, MUST_BE_GTE, MUST_BE_LTE,
    NOT_ALLOWED, NOT_UNIQUE, REQUIRED, TOO_FEW_ITEMS, TOO_LONG, TOO_MANY_ITEMS, TOO_SHORT, UNKNOWN_REFERENCE,
};

/// Technical client back-off hint when the SQL pool times out or loses a connection.
pub const DB_UNAVAILABLE_RETRY_AFTER_S: u64 = 5; // technical: ERR-10 retry hint, not a business deadline

/// Base URI for application problem types (ERR-20).
pub const PROBLEM_BASE: &str = "https://ritma.dev/problems/";

/// ERR-21: stable metadata for an application error code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorCodeDef {
    pub code: &'static str,
    pub status: u16,
    pub title: &'static str,
}

/// Core codes shared by all modules (ERR-21).
pub mod codes {
    use super::ErrorCodeDef;

    pub static BAD_REQUEST: ErrorCodeDef = ErrorCodeDef {
        code: "bad_request",
        status: 400,
        title: "Bad request",
    };
    pub static UNAUTHENTICATED: ErrorCodeDef = ErrorCodeDef {
        code: "unauthenticated",
        status: 401,
        title: "Authentication required",
    };
    pub static FORBIDDEN: ErrorCodeDef = ErrorCodeDef {
        code: "forbidden",
        status: 403,
        title: "Forbidden",
    };
    pub static NOT_FOUND: ErrorCodeDef = ErrorCodeDef {
        code: "not_found",
        status: 404,
        title: "Not found",
    };
    pub static MODULE_DISABLED: ErrorCodeDef = ErrorCodeDef {
        code: "module_disabled",
        status: 404,
        title: "Module disabled",
    };
    pub static MODULE_DEPENDENCY: ErrorCodeDef = ErrorCodeDef {
        code: "module_dependency",
        status: 422,
        title: "Module dependency not satisfied",
    };
    pub static MODULE_REQUIRED_BY: ErrorCodeDef = ErrorCodeDef {
        code: "module_required_by",
        status: 422,
        title: "Module is required by an enabled module",
    };
    pub static VALIDATION: ErrorCodeDef = ErrorCodeDef {
        code: "validation",
        status: 422,
        title: "Invalid request",
    };
    pub static POLICY_VIOLATION: ErrorCodeDef = ErrorCodeDef {
        code: "policy_violation",
        status: 422,
        title: "Policy violation",
    };
    pub static PRECONDITION_REQUIRED: ErrorCodeDef = ErrorCodeDef {
        code: "precondition_required",
        status: 428,
        title: "Precondition required",
    };
    pub static VERSION_CONFLICT: ErrorCodeDef = ErrorCodeDef {
        code: "version_conflict",
        status: 409,
        title: "Version conflict",
    };
    pub static PAYLOAD_TOO_LARGE: ErrorCodeDef = ErrorCodeDef {
        code: "payload_too_large",
        status: 413,
        title: "Payload too large",
    };
    pub static LOCKED: ErrorCodeDef = ErrorCodeDef {
        code: "locked",
        status: 423,
        title: "Locked",
    };
    pub static RATE_LIMITED: ErrorCodeDef = ErrorCodeDef {
        code: "rate_limited",
        status: 429,
        title: "Too many requests",
    };
    pub static UNAVAILABLE: ErrorCodeDef = ErrorCodeDef {
        code: "unavailable",
        status: 503,
        title: "Service unavailable",
    };
    pub static SETUP_REQUIRED: ErrorCodeDef = ErrorCodeDef {
        code: "setup_required",
        status: 503,
        title: "Setup required",
    };
    pub static INTERNAL: ErrorCodeDef = ErrorCodeDef {
        code: "internal",
        status: 500,
        title: "Internal error",
    };

    pub const CODES: &[&ErrorCodeDef] = &[
        &BAD_REQUEST,
        &UNAUTHENTICATED,
        &FORBIDDEN,
        &NOT_FOUND,
        &MODULE_DISABLED,
        &MODULE_DEPENDENCY,
        &MODULE_REQUIRED_BY,
        &VALIDATION,
        &POLICY_VIOLATION,
        &PRECONDITION_REQUIRED,
        &VERSION_CONFLICT,
        &PAYLOAD_TOO_LARGE,
        &LOCKED,
        &RATE_LIMITED,
        &UNAVAILABLE,
        &SETUP_REQUIRED,
        &INTERNAL,
    ];
}

/// ERR-20: checks the canonical lowercase code format.
pub fn validate_code(code: &str) -> bool {
    let bytes = code.as_bytes();
    (3..=48).contains(&bytes.len())
        && bytes[0].is_ascii_lowercase()
        && bytes[1..]
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_')
}

/// VAL-03 field-level validation error.
#[derive(Debug, Clone, Serialize)]
pub struct FieldError {
    pub pointer: String,
    pub code: String,
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub params: Map<String, Value>,
}

/// RFC 9457 problem response with flattened extensions (ERR-30/31).
#[derive(Debug, Serialize)]
pub struct Problem {
    #[serde(rename = "type")]
    pub type_uri: String,
    pub title: String,
    pub status: u16,
    pub code: String,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<Uuid>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Problem {
    fn new(
        code: &ErrorCodeDef,
        status: StatusCode,
        detail: String,
        request_id: Option<Uuid>,
        mut extra: Map<String, Value>,
    ) -> Self {
        for reserved in ["type", "title", "status", "code", "detail", "request_id"] {
            extra.remove(reserved);
        }
        Self {
            type_uri: format!("{PROBLEM_BASE}{}", code.code.replace('_', "-")),
            title: String::from(code.title),
            status: status.as_u16(),
            code: String::from(code.code),
            detail,
            request_id,
            extra,
        }
    }
}

// Task-local request identifier supplied by the HTTP middleware (ARC-04).
tokio::task_local! {
    pub static REQUEST_ID: Uuid;
}

/// ERR-10: closed set of failures understood by the HTTP edge.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found")]
    NotFound { entity: &'static str },
    #[error("conflict")]
    Conflict {
        code: &'static ErrorCodeDef,
        detail: String,
        extra: Map<String, Value>,
    },
    #[error("validation failed")]
    Validation { errors: Vec<FieldError> },
    #[error("forbidden")]
    Forbidden { perm: &'static str },
    #[error("forbidden rule")]
    ForbiddenRule {
        code: &'static ErrorCodeDef,
        detail: String,
    },
    #[error("unauthenticated")]
    Unauthenticated { code: &'static ErrorCodeDef },
    #[error("module disabled")]
    ModuleDisabled { module: String },
    #[error("unprocessable request")]
    Unprocessable {
        code: &'static ErrorCodeDef,
        detail: String,
        extra: Map<String, Value>,
    },
    #[error("policy violation")]
    PolicyViolation {
        rule: String,
        detail: String,
        extra: Map<String, Value>,
    },
    #[error("precondition required")]
    PreconditionRequired,
    #[error("gone")]
    Gone { code: &'static ErrorCodeDef },
    #[error("locked")]
    Locked {
        until: Option<chrono::DateTime<chrono::Utc>>,
    },
    #[error("setup required")]
    SetupRequired,
    #[error("rate limited")]
    RateLimited { retry_after_s: u64 },
    #[error("payload too large")]
    PayloadTooLarge,
    #[error("bad request")]
    BadRequest {
        code: &'static ErrorCodeDef,
        detail: String,
    },
    #[error("unavailable")]
    Unavailable { retry_after_s: u64 },
    #[error(transparent)]
    Internal(#[from] AnyError),
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::RowNotFound => Self::NotFound { entity: "record" },
            sqlx::Error::PoolTimedOut
            | sqlx::Error::PoolClosed
            | sqlx::Error::WorkerCrashed
            | sqlx::Error::Io(_)
            | sqlx::Error::Tls(_) => Self::Unavailable {
                retry_after_s: DB_UNAVAILABLE_RETRY_AFTER_S,
            },
            sqlx::Error::Database(database_error)
                if database_error.code().is_some_and(|code| code.starts_with("08")) =>
            {
                Self::Unavailable {
                    retry_after_s: DB_UNAVAILABLE_RETRY_AFTER_S,
                }
            }
            other => Self::Internal(AnyError::new(other)),
        }
    }
}

impl IntoResponse for AppError {
    #[expect(
        clippy::too_many_lines,
        reason = "ERR-10 defines one exhaustive mapping at the HTTP error edge"
    )]
    fn into_response(self) -> Response {
        let request_id = REQUEST_ID.try_with(|id| *id).ok();
        let (status, code, detail, extra, retry_after_s) = match self {
            Self::NotFound { entity } => (
                StatusCode::NOT_FOUND,
                &codes::NOT_FOUND,
                format!("{entity} not found"),
                Map::new(),
                None,
            ),
            Self::Conflict { code, detail, extra } => (StatusCode::CONFLICT, code, detail, extra, None),
            Self::Validation { errors } => {
                let mut extra = Map::new();
                extra.insert("errors".to_owned(), json!(errors));
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    &codes::VALIDATION,
                    String::from("Validation failed"),
                    extra,
                    None,
                )
            }
            Self::Forbidden { perm } => {
                let mut extra = Map::new();
                extra.insert("perm".to_owned(), json!(perm));
                (
                    StatusCode::FORBIDDEN,
                    &codes::FORBIDDEN,
                    String::from("Forbidden"),
                    extra,
                    None,
                )
            }
            Self::ForbiddenRule { code, detail } => (StatusCode::FORBIDDEN, code, detail, Map::new(), None),
            Self::Unauthenticated { code } => (
                StatusCode::UNAUTHORIZED,
                code,
                String::from("Authentication required"),
                Map::new(),
                None,
            ),
            Self::ModuleDisabled { module } => {
                let detail = format!("{module} module is disabled");
                let mut extra = Map::new();
                extra.insert("module".to_owned(), Value::String(module));
                (
                    StatusCode::NOT_FOUND,
                    &codes::MODULE_DISABLED,
                    detail,
                    extra,
                    None,
                )
            }
            Self::Unprocessable { code, detail, extra } => {
                (StatusCode::UNPROCESSABLE_ENTITY, code, detail, extra, None)
            }
            Self::PolicyViolation {
                rule,
                detail,
                mut extra,
            } => {
                extra.insert("rule".to_owned(), Value::String(rule));
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    &codes::POLICY_VIOLATION,
                    detail,
                    extra,
                    None,
                )
            }
            Self::PreconditionRequired => (
                StatusCode::PRECONDITION_REQUIRED,
                &codes::PRECONDITION_REQUIRED,
                String::from("A matching If-Match header is required"),
                Map::new(),
                None,
            ),
            Self::Gone { code } => (StatusCode::GONE, code, String::from("Gone"), Map::new(), None),
            Self::Locked { until } => {
                let mut extra = Map::new();
                if let Some(until) = until {
                    extra.insert("until".to_owned(), json!(until));
                }
                (
                    StatusCode::LOCKED,
                    &codes::LOCKED,
                    String::from("Locked"),
                    extra,
                    None,
                )
            }
            Self::SetupRequired => (
                StatusCode::SERVICE_UNAVAILABLE,
                &codes::SETUP_REQUIRED,
                String::from("Setup is required"),
                Map::new(),
                None,
            ),
            Self::RateLimited { retry_after_s } => (
                StatusCode::TOO_MANY_REQUESTS,
                &codes::RATE_LIMITED,
                String::from("Too many requests"),
                Map::new(),
                Some(retry_after_s),
            ),
            Self::PayloadTooLarge => (
                StatusCode::PAYLOAD_TOO_LARGE,
                &codes::PAYLOAD_TOO_LARGE,
                String::from("Payload too large"),
                Map::new(),
                None,
            ),
            Self::BadRequest { code, detail } => (StatusCode::BAD_REQUEST, code, detail, Map::new(), None),
            Self::Unavailable { retry_after_s } => (
                StatusCode::SERVICE_UNAVAILABLE,
                &codes::UNAVAILABLE,
                String::from("The service is temporarily unavailable"),
                Map::new(),
                Some(retry_after_s),
            ),
            Self::Internal(error) => {
                tracing::error!(
                    error = %error,
                    chain = ?error,
                    request_id = ?request_id,
                    code = codes::INTERNAL.code,
                    "unhandled application error"
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    &codes::INTERNAL,
                    String::from("Internal error"),
                    Map::new(),
                    None,
                )
            }
        };

        let problem = Problem::new(code, status, detail, request_id, extra);
        let mut response = (status, Json(problem)).into_response();
        response
            .headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static("application/problem+json"));
        if let Some(retry_after_s) = retry_after_s
            && let Ok(value) = HeaderValue::from_bytes(retry_after_s.to_string().as_bytes())
        {
            response.headers_mut().insert(RETRY_AFTER, value);
        }
        response
    }
}

#[cfg(test)]
#[path = "error/tests.rs"]
mod tests;
