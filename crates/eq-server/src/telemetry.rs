//! Structured tracing setup (OPS-06, SEC-21).

use anyhow::{Context, Result};
use eq_core::config::LogFormat;
use tracing_subscriber::EnvFilter;

/// OPS-06: initializes JSON or pretty tracing from the validated environment settings.
pub fn init(filter: &str, format: LogFormat) -> Result<()> {
    let filter = EnvFilter::try_new(filter).context("parse RITMA_LOG filter")?;

    match format {
        LogFormat::Json => tracing_subscriber::fmt()
            .with_env_filter(filter)
            .json()
            .try_init()
            .map_err(|error| anyhow::anyhow!("install JSON tracing subscriber: {error}")),
        LogFormat::Pretty => tracing_subscriber::fmt()
            .with_env_filter(filter)
            .pretty()
            .try_init()
            .map_err(|error| anyhow::anyhow!("install pretty tracing subscriber: {error}")),
    }
}
