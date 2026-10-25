use super::*;
use std::collections::HashMap;

fn valid_values() -> HashMap<String, String> {
    HashMap::from([
        (
            String::from("DATABASE_URL"),
            String::from("postgres://eq:test-secret-db-password@localhost/ritma"),
        ),
        (String::from("RITMA_SECRET_KEY"), STANDARD.encode([0_u8; 32])),
        (
            String::from("RITMA_PUBLIC_URL"),
            String::from("https://school.example.test/"),
        ),
    ])
}

#[test]
fn missing_database_url() {
    let mut values = valid_values();
    values.remove("DATABASE_URL");

    assert!(matches!(
        Config::from_map(values),
        Err(ConfigError::Missing("DATABASE_URL"))
    ));
}

#[test]
fn short_secret_rejected() {
    let mut values = valid_values();
    values.insert(String::from("RITMA_SECRET_KEY"), STANDARD.encode([1_u8; 16]));

    let error = Config::from_map(values).unwrap_err();

    assert!(matches!(
        error,
        ConfigError::Invalid {
            var: "RITMA_SECRET_KEY",
            ..
        }
    ));
}

#[test]
fn defaults() {
    let config = Config::from_map(valid_values()).unwrap();

    assert_eq!(config.listen, default_listen());
    assert_eq!(config.mode, Mode::Single);
    assert_eq!(config.backup_keep, 14);
    assert_eq!(config.backup_schedule, BackupSchedule::Daily);
    assert_eq!(config.db_pool, 10);
    assert_eq!(config.trusted_proxies.len(), 2);
    assert_eq!(config.public_url.as_str(), "https://school.example.test/");
    assert_eq!(
        config.database_url.expose_secret(),
        config.database_owner_url.expose_secret()
    );
}

#[test]
fn unknown_vars_reported() {
    let mut values = valid_values();
    values.insert(
        String::from("RITMA_FUTURE_SETTING"),
        String::from("test-secret-unknown-value"),
    );

    assert_eq!(
        unknown_ritma_variables(&values),
        vec![String::from("RITMA_FUTURE_SETTING")]
    );
    assert!(Config::from_map(values).is_ok());
}

#[test]
fn debug_redacts_secrets() {
    let mut values = valid_values();
    values.insert(
        String::from("RITMA_SECRET_KEY"),
        STANDARD.encode(b"test-secret-key-material-123456789"),
    );
    values.insert(
        String::from("RITMA_SECRET_KEY_PREVIOUS"),
        STANDARD.encode(b"test-secret-old-key-material-12345"),
    );
    values.insert(
        String::from("RITMA_DATABASE_OWNER_URL"),
        String::from("postgres://eq:test-secret-owner-password@localhost/ritma"),
    );
    values.insert(
        String::from("RITMA_SETUP_TOKEN"),
        String::from("test-secret-setup-token"),
    );
    values.insert(
        String::from("RITMA_SMTP_URL"),
        String::from("smtp://user:test-secret-smtp-password@example.test"),
    );
    values.insert(String::from("RITMA_CAPTCHA_PROVIDER"), String::from("turnstile"));
    values.insert(
        String::from("RITMA_CAPTCHA_SITE_KEY"),
        String::from("test-site-key"),
    );
    values.insert(
        String::from("RITMA_CAPTCHA_SECRET"),
        String::from("test-secret-captcha"),
    );
    values.insert(
        String::from("RITMA_METRICS_TOKEN"),
        String::from("test-secret-metrics-token"),
    );
    let config = Config::from_map(values).unwrap();
    let formatted = format!("{config:?}\n{config:#?}");

    for secret in [
        "test-secret-db-password",
        "test-secret-key-material-123456789",
        "test-secret-old-key-material-12345",
        "test-secret-owner-password",
        "test-secret-setup-token",
        "test-secret-smtp-password",
        "test-secret-captcha",
        "test-secret-metrics-token",
    ] {
        assert!(!formatted.contains(secret));
    }
    assert!(formatted.contains("[REDACTED]"));
}

#[test]
fn error_does_not_echo_secret() {
    let mut values = valid_values();
    values.insert(
        String::from("RITMA_SECRET_KEY"),
        String::from("test-secret-invalid-key-value"),
    );

    let error = Config::from_map(values).unwrap_err();

    assert!(!error.to_string().contains("test-secret-invalid-key-value"));
}

#[test]
fn public_url_rejects_credentials() {
    let mut values = valid_values();
    values.insert(
        String::from("RITMA_PUBLIC_URL"),
        String::from("https://user:test-secret-url-password@example.test/"),
    );

    let error = Config::from_map(values).unwrap_err();

    assert!(matches!(
        error,
        ConfigError::Invalid {
            var: "RITMA_PUBLIC_URL",
            ..
        }
    ));
    assert!(!error.to_string().contains("test-secret-url-password"));
}

#[test]
fn public_url_trailing_path_slash_is_normalized() {
    let mut values = valid_values();
    values.insert(
        String::from("RITMA_PUBLIC_URL"),
        String::from("https://school.example.test/portal/"),
    );

    let config = Config::from_map(values).unwrap();

    assert_eq!(config.public_url.as_str(), "https://school.example.test/portal");
}

#[test]
fn explicit_trusted_proxies_extend_loopback_defaults() {
    let mut values = valid_values();
    values.insert(String::from("RITMA_TRUSTED_PROXIES"), String::from("10.1.0.0/16"));

    let config = Config::from_map(values).unwrap();

    assert_eq!(config.trusted_proxies.len(), 3);
    let configured_proxy = "10.1.0.0/16".parse().unwrap();
    assert!(config.trusted_proxies.contains(&configured_proxy));
}

#[test]
fn captcha_configuration_requires_all_fields() {
    let mut values = valid_values();
    values.insert(String::from("RITMA_CAPTCHA_PROVIDER"), String::from("turnstile"));

    assert!(matches!(
        Config::from_map(values),
        Err(ConfigError::Invalid {
            var: "RITMA_CAPTCHA_PROVIDER",
            ..
        })
    ));
}

#[test]
fn standard_and_url_safe_secret_keys_are_accepted() {
    for encoded in [
        STANDARD.encode([253_u8; 32]),
        URL_SAFE.encode([253_u8; 32]),
        URL_SAFE_NO_PAD.encode([253_u8; 32]),
    ] {
        let mut values = valid_values();
        values.insert(String::from("RITMA_SECRET_KEY"), encoded);
        assert!(Config::from_map(values).is_ok());
    }
}

#[test]
fn for_tests_has_safe_defaults() {
    let config = Config::for_tests();
    assert_eq!(config.mode, Mode::Single);
    assert_eq!(config.listen.port(), 8080);
}
