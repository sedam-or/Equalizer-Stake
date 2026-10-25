use super::*;
use axum::body::to_bytes;
use axum::http::HeaderMap;

static SLOT_TAKEN: ErrorCodeDef = ErrorCodeDef {
    code: "slot_taken",
    status: 409,
    title: "Slot already taken",
};
static GONE_CODE: ErrorCodeDef = ErrorCodeDef {
    code: "invite_expired",
    status: 410,
    title: "Invite expired",
};
static FORBIDDEN_RULE_CODE: ErrorCodeDef = ErrorCodeDef {
    code: "marks_locked",
    status: 403,
    title: "Marks locked",
};
static UNAUTHENTICATED_CODE: ErrorCodeDef = ErrorCodeDef {
    code: "invalid_credentials",
    status: 401,
    title: "Invalid credentials",
};
static UNPROCESSABLE_CODE: ErrorCodeDef = ErrorCodeDef {
    code: "unknown_reference",
    status: 422,
    title: "Unknown reference",
};
static BAD_REQUEST_CODE: ErrorCodeDef = ErrorCodeDef {
    code: "token_invalid",
    status: 400,
    title: "Invalid token",
};

async fn response_parts(response: Response) -> (StatusCode, HeaderMap, Value) {
    let (parts, body) = response.into_parts();
    let body = to_bytes(body, usize::MAX).await.unwrap();
    let value = serde_json::from_slice(&body).unwrap();
    (parts.status, parts.headers, value)
}

#[tokio::test]
async fn conflict_problem_json() {
    let extra = Map::from_iter([(String::from("conflicts"), json!(["session-1"]))]);
    let response = AppError::Conflict {
        code: &SLOT_TAKEN,
        detail: String::from("The selected slot is taken"),
        extra,
    }
    .into_response();
    let (status, headers, problem) = response_parts(response).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(headers[CONTENT_TYPE], "application/problem+json");
    assert_eq!(problem["code"], "slot_taken");
    assert_eq!(problem["title"], "Slot already taken");
    assert!(problem["type"].as_str().unwrap().ends_with("/slot-taken"));
    assert_eq!(problem["conflicts"][0], "session-1");
}

#[tokio::test]
async fn internal_hides_detail() {
    let response = AppError::Internal(AnyError::msg("test-secret-internal-message")).into_response();
    let (status, _, problem) = response_parts(response).await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(problem["detail"], "Internal error");
    assert!(!problem.to_string().contains("test-secret-internal-message"));
}

#[tokio::test]
async fn rate_limited_sets_retry_after() {
    let response = AppError::RateLimited { retry_after_s: 17 }.into_response();
    let (status, headers, problem) = response_parts(response).await;

    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(headers[RETRY_AFTER], "17");
    assert_eq!(problem["code"], "rate_limited");
}

#[tokio::test]
async fn validation_lists_errors() {
    let response = AppError::Validation {
        errors: vec![FieldError {
            pointer: String::from("/name"),
            code: String::from("required"),
            params: Map::new(),
        }],
    }
    .into_response();
    let (status, _, problem) = response_parts(response).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(problem["errors"][0]["pointer"], "/name");
    assert_eq!(problem["errors"][0]["code"], "required");
    assert!(problem["errors"][0].get("params").is_none());
}

#[tokio::test]
async fn unavailable_sets_retry_after() {
    let response = AppError::Unavailable { retry_after_s: 5 }.into_response();
    let (status, headers, problem) = response_parts(response).await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(headers[RETRY_AFTER], "5");
    assert_eq!(problem["code"], "unavailable");
}

#[tokio::test]
async fn request_id_is_included_when_scoped() {
    let request_id = Uuid::from_u128(1);
    let (_, _, problem) = REQUEST_ID
        .scope(request_id, async {
            response_parts(AppError::NotFound { entity: "person" }.into_response()).await
        })
        .await;

    assert_eq!(problem["request_id"], request_id.to_string());
}

#[tokio::test]
async fn request_id_is_omitted_without_scope() {
    let (_, _, problem) = response_parts(AppError::NotFound { entity: "person" }.into_response()).await;

    assert!(problem.get("request_id").is_none());
}

#[test]
fn core_codes_valid_and_unique() {
    let codes = codes::CODES
        .iter()
        .map(|code| code.code)
        .collect::<std::collections::HashSet<_>>();

    assert_eq!(codes.len(), codes::CODES.len());
    assert!(codes::CODES.iter().all(|code| validate_code(code.code)));
}

#[test]
fn invalid_code_syntax_is_rejected() {
    for code in ["ab", "Upper", "bad-code", "_invalid", "bad code"] {
        assert!(!validate_code(code));
    }
    assert!(validate_code("slot_taken"));
}

#[tokio::test]
async fn core_app_errors_render_expected_status_and_code() {
    let cases = [
        (AppError::NotFound { entity: "person" }, 404, "not_found"),
        (AppError::Forbidden { perm: "person.read" }, 403, "forbidden"),
        (
            AppError::ModuleDisabled {
                module: String::from("billing"),
            },
            404,
            "module_disabled",
        ),
        (AppError::PreconditionRequired, 428, "precondition_required"),
        (AppError::Locked { until: None }, 423, "locked"),
        (AppError::SetupRequired, 503, "setup_required"),
        (AppError::PayloadTooLarge, 413, "payload_too_large"),
    ];
    for (error, status, code) in cases {
        let (actual_status, _, problem) = response_parts(error.into_response()).await;
        assert_eq!(actual_status.as_u16(), status);
        assert_eq!(problem["code"], code);
    }
}

#[tokio::test]
async fn remaining_app_errors_render_expected_status_and_code() {
    let cases = [
        (AppError::Gone { code: &GONE_CODE }, 410, "invite_expired"),
        (
            AppError::ForbiddenRule {
                code: &FORBIDDEN_RULE_CODE,
                detail: String::from("Marks locked"),
            },
            403,
            "marks_locked",
        ),
        (
            AppError::Unauthenticated {
                code: &UNAUTHENTICATED_CODE,
            },
            401,
            "invalid_credentials",
        ),
        (
            AppError::Unprocessable {
                code: &UNPROCESSABLE_CODE,
                detail: String::from("Unknown reference"),
                extra: Map::new(),
            },
            422,
            "unknown_reference",
        ),
        (
            AppError::PolicyViolation {
                rule: String::from("policy_rule"),
                detail: String::from("Policy violation"),
                extra: Map::new(),
            },
            422,
            "policy_violation",
        ),
        (
            AppError::BadRequest {
                code: &BAD_REQUEST_CODE,
                detail: String::from("Invalid token"),
            },
            400,
            "token_invalid",
        ),
    ];
    for (error, status, code) in cases {
        let (actual_status, _, problem) = response_parts(error.into_response()).await;
        assert_eq!(actual_status.as_u16(), status);
        assert_eq!(problem["code"], code);
    }
}

#[test]
fn sqlx_errors_map_to_core_errors() {
    assert!(matches!(
        AppError::from(sqlx::Error::RowNotFound),
        AppError::NotFound { entity: "record" }
    ));
    for error in [
        sqlx::Error::PoolTimedOut,
        sqlx::Error::PoolClosed,
        sqlx::Error::WorkerCrashed,
        sqlx::Error::Io(std::io::Error::other("connection lost")),
    ] {
        assert!(matches!(
            AppError::from(error),
            AppError::Unavailable {
                retry_after_s: DB_UNAVAILABLE_RETRY_AFTER_S
            }
        ));
    }
    assert!(matches!(
        AppError::from(sqlx::Error::ColumnNotFound(String::from("name"))),
        AppError::Internal(_)
    ));
}
