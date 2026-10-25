//! Validated process configuration and secret-safe formatting (OPS-01, SBD-50).

use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE, URL_SAFE_NO_PAD};
use figment::providers::Env;
use secrecy::{ExposeSecret, SecretSlice, SecretString};
use std::collections::HashMap;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use zeroize::Zeroize;

const DEFAULT_LISTEN_PORT: u16 = 8080; // OPS-01
const DEFAULT_TILE_URL: &str = "https://tile.openstreetmap.org/{z}/{x}/{y}.png"; // technical: OPS-01 OSM tile URL
const DEFAULT_BACKUP_KEEP: u16 = 14; // OPS-01
const DEFAULT_DB_POOL: u32 = 10; // OPS-07

const KNOWN_ENV_VARS: &[&str] = &[
    "DATABASE_URL",
    "RITMA_DATABASE_OWNER_URL",
    "RITMA_SECRET_KEY",
    "RITMA_SECRET_KEY_PREVIOUS",
    "RITMA_PUBLIC_URL",
    "RITMA_MODE",
    "RITMA_BASE_DOMAIN",
    "RITMA_LISTEN",
    "RITMA_DATA_DIR",
    "RITMA_FILES_BACKEND",
    "RITMA_SETUP_TOKEN",
    "RITMA_TRUSTED_PROXIES",
    "RITMA_LOG",
    "RITMA_LOG_FORMAT",
    "RITMA_BACKUP_SCHEDULE",
    "RITMA_BACKUP_KEEP",
    "RITMA_SMTP_URL",
    "RITMA_CAPTCHA_PROVIDER",
    "RITMA_CAPTCHA_SITE_KEY",
    "RITMA_CAPTCHA_SECRET",
    "RITMA_TILE_URL",
    "RITMA_TELEMETRY",
    "RITMA_UPDATE_CHECK",
    "RITMA_METRICS_TOKEN",
    "RITMA_DB_POOL",
    "RITMA_DEMO",
];

/// OPS-01: single-instance or multi-tenant server mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Single,
    Multi,
}

/// OPS-01: file storage adapter selected at startup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesBackend {
    Local,
    S3,
}

/// OPS-01: configured tracing output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    Json,
    Pretty,
}

/// OPS-01: whether scheduled local backups are enabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupSchedule {
    Off,
    Daily,
}

/// OPS-01: supported optional CAPTCHA provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptchaProvider {
    Turnstile,
    Hcaptcha,
}

/// OPS-01, SBD-50: CAPTCHA credentials with redacted formatting.
pub struct CaptchaConfig {
    pub provider: CaptchaProvider,
    pub site_key: String,
    pub secret: SecretString,
}

impl fmt::Debug for CaptchaConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CaptchaConfig")
            .field("provider", &self.provider)
            .field("site_key", &self.site_key)
            .field("secret", &"[REDACTED]")
            .finish()
    }
}

/// OPS-01 and SBD-10: validated process configuration.
pub struct Config {
    pub database_url: SecretString,
    pub database_owner_url: SecretString,
    pub secret_key: SecretSlice<u8>,
    pub secret_key_previous: Option<SecretSlice<u8>>,
    pub public_url: url::Url,
    pub mode: Mode,
    pub base_domain: Option<String>,
    pub listen: SocketAddr,
    pub data_dir: PathBuf,
    pub files_backend: FilesBackend,
    pub setup_token: Option<SecretString>,
    pub trusted_proxies: Vec<ipnet::IpNet>,
    pub log: String,
    pub log_format: LogFormat,
    pub backup_schedule: BackupSchedule,
    pub backup_keep: u16,
    pub smtp_url: Option<SecretString>,
    pub captcha: Option<CaptchaConfig>,
    pub tile_url: String,
    pub telemetry: bool,
    pub update_check: bool,
    pub metrics_token: Option<SecretString>,
    pub db_pool: u32,
    pub demo: bool,
}

impl Config {
    /// OPS-01: reads environment values through Figment, then validates them in isolation.
    pub fn from_env() -> Result<Self, ConfigError> {
        let values = Env::raw()
            .lowercase(false)
            .iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect();
        Self::from_map(values)
    }

    fn from_map(values: HashMap<String, String>) -> Result<Self, ConfigError> {
        let unknown = unknown_ritma_variables(&values);
        if !unknown.is_empty() {
            tracing::warn!(variables = ?unknown, "unknown Ritma environment variables");
        }

        let mut values = SensitiveMap(
            values
                .into_iter()
                .map(|(name, value)| (name, SecretString::from(value)))
                .collect(),
        );
        let database_url = take_required_secret(&mut values, "DATABASE_URL")?;
        let database_owner_url = take_owner_url(&mut values, &database_url)?;
        let secret_key = take_required_key(&mut values, "RITMA_SECRET_KEY")?;
        let secret_key_previous = take_optional_key(&mut values, "RITMA_SECRET_KEY_PREVIOUS")?;
        let public_url = take_public_url(&mut values)?;
        let mode = parse_optional(&mut values, "RITMA_MODE", Mode::Single, parse_mode)?;
        let base_domain = take_base_domain(&mut values, mode)?;
        let listen = parse_optional(&mut values, "RITMA_LISTEN", default_listen(), |value| {
            SocketAddr::from_str(value).ok()
        })?;
        let data_dir = take_data_dir(&mut values)?;
        let files_backend = parse_optional(
            &mut values,
            "RITMA_FILES_BACKEND",
            FilesBackend::Local,
            parse_files_backend,
        )?;
        let setup_token = take_optional_secret(&mut values, "RITMA_SETUP_TOKEN")?;
        let trusted_proxies = take_trusted_proxies(&mut values)?;
        let log = take_optional_string(&mut values, "RITMA_LOG", "info")?;
        let log_format = take_log_format(&mut values)?;
        let backup_schedule = parse_optional(
            &mut values,
            "RITMA_BACKUP_SCHEDULE",
            BackupSchedule::Daily,
            parse_backup_schedule,
        )?;
        let backup_keep = parse_optional(&mut values, "RITMA_BACKUP_KEEP", DEFAULT_BACKUP_KEEP, |value| {
            value.parse().ok()
        })?;
        let smtp_url = take_optional_secret(&mut values, "RITMA_SMTP_URL")?;
        let captcha = take_captcha(&mut values)?;
        let tile_url = take_optional_string(&mut values, "RITMA_TILE_URL", DEFAULT_TILE_URL)?;
        let telemetry = parse_optional(&mut values, "RITMA_TELEMETRY", false, parse_on_off)?;
        let update_check = parse_optional(&mut values, "RITMA_UPDATE_CHECK", true, parse_on_off)?;
        let metrics_token = take_optional_secret(&mut values, "RITMA_METRICS_TOKEN")?;
        let db_pool = parse_optional(&mut values, "RITMA_DB_POOL", DEFAULT_DB_POOL, |value| {
            value.parse::<u32>().ok().filter(|pool_size| *pool_size > 0)
        })?;
        let demo = parse_optional(&mut values, "RITMA_DEMO", false, parse_bool)?;

        Ok(Self {
            database_url,
            database_owner_url,
            secret_key,
            secret_key_previous,
            public_url,
            mode,
            base_domain,
            listen,
            data_dir,
            files_backend,
            setup_token,
            trusted_proxies,
            log,
            log_format,
            backup_schedule,
            backup_keep,
            smtp_url,
            captcha,
            tile_url,
            telemetry,
            update_check,
            metrics_token,
            db_pool,
            demo,
        })
    }

    /// Returns an isolated configuration for tests and testkit consumers.
    #[cfg(any(test, feature = "testkit"))]
    pub fn for_tests() -> Self {
        #[expect(clippy::expect_used, reason = "this fixed test URL is a known-valid fixture")]
        let public_url = url::Url::parse("https://example.invalid").expect("static test URL is valid");
        Self {
            database_url: SecretString::from("postgres://eq:test-secret-db@localhost/ritma"),
            database_owner_url: SecretString::from("postgres://eq:test-secret-owner@localhost/ritma"),
            secret_key: SecretSlice::from(vec![0; 32]),
            secret_key_previous: None,
            public_url,
            mode: Mode::Single,
            base_domain: None,
            listen: SocketAddr::from(([127, 0, 0, 1], 8080)),
            data_dir: PathBuf::from("./data"),
            files_backend: FilesBackend::Local,
            setup_token: None,
            trusted_proxies: loopback_proxies(),
            log: String::from("info"),
            log_format: LogFormat::Pretty,
            backup_schedule: BackupSchedule::Daily,
            backup_keep: DEFAULT_BACKUP_KEEP,
            smtp_url: None,
            captcha: None,
            tile_url: String::from(DEFAULT_TILE_URL),
            telemetry: false,
            update_check: true,
            metrics_token: None,
            db_pool: DEFAULT_DB_POOL,
            demo: false,
        }
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Config")
            .field("database_url", &"[REDACTED]")
            .field("database_owner_url", &"[REDACTED]")
            .field("secret_key", &"[REDACTED]")
            .field(
                "secret_key_previous",
                &self.secret_key_previous.as_ref().map(|_| "[REDACTED]"),
            )
            .field("public_url", &self.public_url)
            .field("mode", &self.mode)
            .field("base_domain", &self.base_domain)
            .field("listen", &self.listen)
            .field("data_dir", &self.data_dir)
            .field("files_backend", &self.files_backend)
            .field("setup_token", &redacted_option(&self.setup_token))
            .field("trusted_proxies", &self.trusted_proxies)
            .field("log", &self.log)
            .field("log_format", &self.log_format)
            .field("backup_schedule", &self.backup_schedule)
            .field("backup_keep", &self.backup_keep)
            .field("smtp_url", &redacted_option(&self.smtp_url))
            .field("captcha", &self.captcha)
            .field("tile_url", &self.tile_url)
            .field("telemetry", &self.telemetry)
            .field("update_check", &self.update_check)
            .field("metrics_token", &redacted_option(&self.metrics_token))
            .field("db_pool", &self.db_pool)
            .field("demo", &self.demo)
            .finish()
    }
}

fn redacted_option(secret: &Option<SecretString>) -> Option<&'static str> {
    secret.as_ref().map(|_| "[REDACTED]")
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing required env var {0}")]
    Missing(&'static str),
    #[error("invalid {var}: {reason}")]
    Invalid { var: &'static str, reason: String },
}

struct SensitiveMap(HashMap<String, SecretString>);

impl fmt::Debug for SensitiveMap {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_map()
            .entries(self.0.keys().map(|name| (name, "[REDACTED]")))
            .finish()
    }
}

impl SensitiveMap {
    fn take(&mut self, name: &'static str) -> Option<SecretString> {
        self.0.remove(name)
    }
}

fn unknown_ritma_variables(values: &HashMap<String, String>) -> Vec<String> {
    let mut names = values
        .keys()
        .filter(|name| name.starts_with("RITMA_") && !KNOWN_ENV_VARS.contains(&name.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    names.sort();
    names
}

fn take_required_text(values: &mut SensitiveMap, name: &'static str) -> Result<SecretString, ConfigError> {
    match values.take(name) {
        None => Err(ConfigError::Missing(name)),
        Some(value) if value.expose_secret().is_empty() => Err(invalid(name, "must not be empty")),
        Some(value) => Ok(value),
    }
}

fn take_optional_text(
    values: &mut SensitiveMap,
    name: &'static str,
) -> Result<Option<SecretString>, ConfigError> {
    match values.take(name) {
        None => Ok(None),
        Some(value) if value.expose_secret().is_empty() => Err(invalid(name, "must not be empty")),
        Some(value) => Ok(Some(value)),
    }
}

fn take_required_secret(values: &mut SensitiveMap, name: &'static str) -> Result<SecretString, ConfigError> {
    take_required_text(values, name)
}

fn take_optional_secret(
    values: &mut SensitiveMap,
    name: &'static str,
) -> Result<Option<SecretString>, ConfigError> {
    take_optional_text(values, name)
}

fn take_owner_url(
    values: &mut SensitiveMap,
    database_url: &SecretString,
) -> Result<SecretString, ConfigError> {
    match take_optional_secret(values, "RITMA_DATABASE_OWNER_URL")? {
        Some(url) => Ok(url),
        None => Ok(SecretString::from(database_url.expose_secret().to_owned())),
    }
}

fn take_required_key(values: &mut SensitiveMap, name: &'static str) -> Result<SecretSlice<u8>, ConfigError> {
    decode_key(take_required_text(values, name)?, name)
}

fn take_optional_key(
    values: &mut SensitiveMap,
    name: &'static str,
) -> Result<Option<SecretSlice<u8>>, ConfigError> {
    take_optional_text(values, name)?
        .map(|value| decode_key(value, name))
        .transpose()
}

fn decode_key(value: SecretString, name: &'static str) -> Result<SecretSlice<u8>, ConfigError> {
    let encoded = value.expose_secret().as_bytes();
    let mut decoded = STANDARD
        .decode(encoded)
        .or_else(|_| URL_SAFE.decode(encoded))
        .or_else(|_| URL_SAFE_NO_PAD.decode(encoded))
        .map_err(|_| invalid(name, "must be valid base64"))?;
    if decoded.len() < 32 {
        decoded.zeroize();
        return Err(invalid(name, "must decode to at least 32 bytes"));
    }
    Ok(SecretSlice::from(decoded))
}

fn take_public_url(values: &mut SensitiveMap) -> Result<url::Url, ConfigError> {
    let raw = take_required_text(values, "RITMA_PUBLIC_URL")?;
    let mut public_url = url::Url::parse(raw.expose_secret())
        .map_err(|_| invalid("RITMA_PUBLIC_URL", "must be an absolute HTTP or HTTPS URL"))?;
    if !matches!(public_url.scheme(), "http" | "https") {
        return Err(invalid("RITMA_PUBLIC_URL", "must use HTTP or HTTPS"));
    }
    if !public_url.username().is_empty() || public_url.password().is_some() {
        return Err(invalid("RITMA_PUBLIC_URL", "must not contain credentials"));
    }
    let path = public_url.path().trim_end_matches('/').to_owned();
    if path.is_empty() {
        public_url.set_path("/");
    } else {
        public_url.set_path(&path);
    }
    Ok(public_url)
}

fn take_base_domain(values: &mut SensitiveMap, mode: Mode) -> Result<Option<String>, ConfigError> {
    let base_domain = take_optional_text(values, "RITMA_BASE_DOMAIN")?
        .map(|value| value.expose_secret().to_owned())
        .map(|value| {
            parse_base_domain(&value).ok_or_else(|| {
                invalid(
                    "RITMA_BASE_DOMAIN",
                    "must be a host name without a scheme, port or path",
                )
            })
        })
        .transpose()?;

    if mode == Mode::Multi && base_domain.is_none() {
        return Err(ConfigError::Missing("RITMA_BASE_DOMAIN"));
    }

    Ok(base_domain)
}

fn parse_base_domain(value: &str) -> Option<String> {
    let authority = axum::http::uri::Authority::from_str(value).ok()?;
    if authority.port().is_some() || value.contains('@') {
        return None;
    }

    let parsed = url::Url::parse(&format!("http://{value}")).ok()?;
    let host = match parsed.host()? {
        url::Host::Domain(host) => host.trim_end_matches('.'),
        url::Host::Ipv4(_) | url::Host::Ipv6(_) => return None,
    };

    Some(host.to_ascii_lowercase())
}

fn parse_optional<T>(
    values: &mut SensitiveMap,
    name: &'static str,
    default: T,
    parse: impl FnOnce(&str) -> Option<T>,
) -> Result<T, ConfigError> {
    match take_optional_text(values, name)? {
        None => Ok(default),
        Some(value) => parse(value.expose_secret()).ok_or_else(|| invalid(name, "has an unsupported value")),
    }
}

fn take_optional_string(
    values: &mut SensitiveMap,
    name: &'static str,
    default: &str,
) -> Result<String, ConfigError> {
    Ok(take_optional_text(values, name)?
        .map_or_else(|| String::from(default), |value| value.expose_secret().to_owned()))
}

fn parse_mode(value: &str) -> Option<Mode> {
    match value {
        "single" => Some(Mode::Single),
        "multi" => Some(Mode::Multi),
        _ => None,
    }
}

fn parse_files_backend(value: &str) -> Option<FilesBackend> {
    match value {
        "local" => Some(FilesBackend::Local),
        "s3" => Some(FilesBackend::S3),
        _ => None,
    }
}

fn parse_backup_schedule(value: &str) -> Option<BackupSchedule> {
    match value {
        "off" => Some(BackupSchedule::Off),
        "daily" => Some(BackupSchedule::Daily),
        _ => None,
    }
}

fn parse_on_off(value: &str) -> Option<bool> {
    match value {
        "off" => Some(false),
        "on" => Some(true),
        _ => None,
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value {
        "false" | "0" => Some(false),
        "true" | "1" => Some(true),
        _ => None,
    }
}

fn take_data_dir(values: &mut SensitiveMap) -> Result<PathBuf, ConfigError> {
    let default = if cfg!(debug_assertions) {
        "./data"
    } else {
        "/var/lib/ritma"
    };
    take_optional_text(values, "RITMA_DATA_DIR").map(|value| {
        value.map_or_else(
            || PathBuf::from(default),
            |path| PathBuf::from(path.expose_secret()),
        )
    })
}

fn take_log_format(values: &mut SensitiveMap) -> Result<LogFormat, ConfigError> {
    let default = if cfg!(debug_assertions) {
        LogFormat::Pretty
    } else {
        LogFormat::Json
    };
    parse_optional(values, "RITMA_LOG_FORMAT", default, |value| match value {
        "json" => Some(LogFormat::Json),
        "pretty" => Some(LogFormat::Pretty),
        _ => None,
    })
}

fn take_trusted_proxies(values: &mut SensitiveMap) -> Result<Vec<ipnet::IpNet>, ConfigError> {
    let Some(raw) = take_optional_text(values, "RITMA_TRUSTED_PROXIES")? else {
        return Ok(loopback_proxies());
    };
    let mut proxies = loopback_proxies();
    for entry in raw
        .expose_secret()
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
    {
        let proxy = entry
            .parse()
            .map_err(|_| invalid("RITMA_TRUSTED_PROXIES", "must be a CSV list of CIDRs"))?;
        if !proxies.contains(&proxy) {
            proxies.push(proxy);
        }
    }
    Ok(proxies)
}

fn loopback_proxies() -> Vec<ipnet::IpNet> {
    ["127.0.0.1/32", "::1/128"]
        .into_iter()
        .filter_map(|value| value.parse().ok())
        .collect()
}

fn default_listen() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), DEFAULT_LISTEN_PORT)
}

fn take_captcha(values: &mut SensitiveMap) -> Result<Option<CaptchaConfig>, ConfigError> {
    let provider = take_optional_text(values, "RITMA_CAPTCHA_PROVIDER")?;
    let site_key = take_optional_text(values, "RITMA_CAPTCHA_SITE_KEY")?;
    let secret = take_optional_secret(values, "RITMA_CAPTCHA_SECRET")?;
    if provider.is_none() && site_key.is_none() && secret.is_none() {
        return Ok(None);
    }
    let (Some(provider), Some(site_key), Some(secret)) = (provider, site_key, secret) else {
        return Err(invalid(
            "RITMA_CAPTCHA_PROVIDER",
            "provider, site key and secret must be configured together",
        ));
    };
    let provider = parse_captcha_provider(provider.expose_secret())
        .ok_or_else(|| invalid("RITMA_CAPTCHA_PROVIDER", "must be turnstile or hcaptcha"))?;
    Ok(Some(CaptchaConfig {
        provider,
        site_key: site_key.expose_secret().to_owned(),
        secret,
    }))
}

fn parse_captcha_provider(value: &str) -> Option<CaptchaProvider> {
    match value {
        "turnstile" => Some(CaptchaProvider::Turnstile),
        "hcaptcha" => Some(CaptchaProvider::Hcaptcha),
        _ => None,
    }
}

fn invalid(var: &'static str, reason: &str) -> ConfigError {
    ConfigError::Invalid {
        var,
        reason: String::from(reason),
    }
}

#[cfg(test)]
#[path = "config/tests.rs"]
mod tests;
