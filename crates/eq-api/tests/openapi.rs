use std::collections::BTreeSet;

use eq_api::openapi;
use serde_json::Value;

#[test]
fn deterministic() {
    let first = openapi::pretty_json(&openapi::build()).expect("OpenAPI serializes");
    let second = openapi::pretty_json(&openapi::build()).expect("OpenAPI serializes");
    assert_eq!(first, second);
}

#[test]
fn routes_match_openapi() {
    let document = openapi::build();
    let documented = operations(&serde_json::to_value(document).expect("OpenAPI serializes"));
    assert_eq!(documented, openapi::registered_route_operations());
}

#[test]
fn problem_schema_present() {
    let document = serde_json::to_value(openapi::build()).expect("OpenAPI serializes");
    let schemas = &document["components"]["schemas"];
    let problem = &schemas["Problem"]["properties"];

    for field in [
        "type",
        "title",
        "status",
        "code",
        "detail",
        "request_id",
        "errors",
        "conflicts",
        "alternatives",
        "perm",
        "module",
        "rule",
        "until",
        "missing",
    ] {
        assert!(problem.get(field).is_some(), "Problem is missing {field}");
    }

    assert_eq!(document["openapi"], "3.1.0");
    assert_eq!(document["info"]["title"], "Ritma API");
    assert_eq!(document["info"]["license"]["name"], "AGPL-3.0-only");
    assert_eq!(document["servers"][0]["url"], "/api/v1");
    assert_eq!(
        document["components"]["securitySchemes"]["bearer"]["scheme"],
        "bearer"
    );
    assert_eq!(
        document["components"]["securitySchemes"]["bearer"]["bearerFormat"],
        "JWT"
    );
    assert_eq!(
        document["components"]["securitySchemes"]["refreshCookie"]["in"],
        "cookie"
    );
    assert_eq!(
        document["components"]["securitySchemes"]["refreshCookie"]["name"],
        "eq_rt"
    );
    assert_eq!(schemas["DisplayName"]["x-field-kind"], "DisplayName");
    assert_eq!(schemas["DisplayName"]["x-sanitize"], "line");

    let paths = document["paths"].as_object().expect("OpenAPI paths object");
    for (path, item) in paths {
        for method in [
            "get", "put", "post", "delete", "options", "head", "patch", "trace",
        ] {
            if let Some(operation) = item.get(method) {
                assert_eq!(
                    operation["responses"]["default"]["content"]["application/problem+json"]["schema"]["$ref"],
                    "#/components/schemas/Problem",
                    "{method} {path} has a Problem response"
                );
            }
        }
    }
}

#[test]
fn error_codes_unique_and_valid() {
    let document = serde_json::to_value(openapi::build()).expect("OpenAPI serializes");
    let codes = document["x-error-codes"]
        .as_array()
        .expect("ERR-21 registry extension");
    let mut unique = BTreeSet::new();

    for definition in codes {
        let code = definition["code"].as_str().expect("registered code is a string");
        assert!(eq_core::error::validate_code(code), "invalid error code {code}");
        assert!(unique.insert(code), "duplicate error code {code}");
        assert!((400..=599).contains(&definition["status"].as_u64().unwrap_or_default()));
        assert!(!definition["title"].as_str().unwrap_or_default().is_empty());
    }
    assert!(!codes.is_empty(), "Core and Identity define shared codes");
}

fn operations(document: &Value) -> BTreeSet<(String, String)> {
    let methods = [
        "get", "put", "post", "delete", "options", "head", "patch", "trace",
    ];
    document["paths"]
        .as_object()
        .into_iter()
        .flat_map(|paths| paths.iter())
        .flat_map(|(path, item)| {
            methods
                .into_iter()
                .filter_map(move |method| item.get(method).map(|_| (path.clone(), method.to_owned())))
        })
        .collect()
}
