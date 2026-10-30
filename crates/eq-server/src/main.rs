use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use eq_core::clock::SystemClock;
use eq_core::config::Config;
use eq_core::db::{connect, run_migrations};
use secrecy::ExposeSecret;
use std::future::IntoFuture;
use std::io::Write;
use std::net::SocketAddr;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::timeout;

mod telemetry;

const HEALTHCHECK_TIMEOUT: Duration = Duration::from_secs(2); // technical: CTR-05 bound
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(25); // technical: OPS-03 drain bound

#[derive(Parser)]
#[command(name = "ritma", version, about = "Ritma course school engine")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Serve,
    Migrate,
    Openapi,
    Backup {
        #[arg(long)]
        out: String,
    },
    Restore {
        #[arg(long = "in")]
        input: String,
        #[arg(long)]
        force: bool,
    },
    Doctor,
    CreateOwner {
        #[arg(long)]
        email: String,
        #[arg(long)]
        name: String,
    },
    RotateKeys {
        #[arg(long)]
        master: bool,
        #[arg(long)]
        tenant_dek: bool,
    },
    SeedDemo,
    Version,
    Healthcheck,
}

#[tokio::main]
async fn main() -> ExitCode {
    match run(Cli::parse()).await {
        Ok(code) => code,
        Err(error) => {
            if writeln!(std::io::stderr().lock(), "{error:#}").is_err() {
                return ExitCode::FAILURE;
            }
            ExitCode::FAILURE
        }
    }
}

async fn run(cli: Cli) -> Result<ExitCode> {
    match cli.command.unwrap_or(Command::Serve) {
        Command::Serve => serve().await,
        Command::Migrate => {
            let config = Config::from_env().context("load Ritma configuration")?;
            run_migrations(config.database_owner_url.expose_secret())
                .await
                .context("run migrations")?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Version => {
            print_stdout(&version_output())?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Healthcheck => {
            let config = Config::from_env().context("load Ritma configuration")?;
            Ok(healthcheck(config.listen).await)
        }
        Command::Openapi => {
            let document = eq_api::openapi::build();
            print_stdout(&eq_api::openapi::pretty_json(&document)?)?;
            Ok(ExitCode::SUCCESS)
        }
        command => {
            let name = command.name();
            print_stdout(&format!("{name} is not implemented yet"))?;
            Ok(ExitCode::from(2))
        }
    }
}

fn version_output() -> String {
    let commit = option_env!("GIT_COMMIT").unwrap_or("unknown");
    let features = [
        ("enrollment", cfg!(feature = "enrollment")),
        ("billing", cfg!(feature = "billing")),
        ("attendance", cfg!(feature = "attendance")),
        ("rental", cfg!(feature = "rental")),
        ("notify", cfg!(feature = "notify")),
        ("reports", cfg!(feature = "reports")),
        ("feedback", cfg!(feature = "feedback")),
    ]
    .into_iter()
    .filter_map(|(name, enabled)| enabled.then_some(name))
    .collect::<Vec<_>>()
    .join(",");
    format!(
        "ritma {} commit={commit} features={features}",
        env!("CARGO_PKG_VERSION")
    )
}

impl Command {
    fn name(&self) -> &'static str {
        match self {
            Self::Serve => "serve",
            Self::Migrate => "migrate",
            Self::Openapi => "openapi",
            Self::Backup { .. } => "backup",
            Self::Restore { .. } => "restore",
            Self::Doctor => "doctor",
            Self::CreateOwner { .. } => "create-owner",
            Self::RotateKeys { .. } => "rotate-keys",
            Self::SeedDemo => "seed-demo",
            Self::Version => "version",
            Self::Healthcheck => "healthcheck",
        }
    }
}

async fn serve() -> Result<ExitCode> {
    let config = Config::from_env().context("load Ritma configuration")?;
    telemetry::init(&config.log, config.log_format).context("initialize tracing")?;
    let config = Arc::new(config);
    run_migrations(config.database_owner_url.expose_secret())
        .await
        .context("run migrations")?;
    let pool = connect(&config)
        .await
        .context("connect to the application database")?;
    let listener = TcpListener::bind(config.listen)
        .await
        .with_context(|| format!("bind HTTP listener at {}", config.listen))?;
    let local_addr = listener.local_addr()?;
    let state = eq_api::AppState::new(pool, config, Arc::new(SystemClock));
    let (shutdown_sender, shutdown_receiver) = tokio::sync::oneshot::channel();
    let make_service = eq_api::router(state).into_make_service_with_connect_info::<SocketAddr>();
    let server = axum::serve(listener, make_service)
        .with_graceful_shutdown(async move {
            let _ = shutdown_receiver.await;
        })
        .into_future();
    tokio::pin!(server);

    tracing::info!(listen = %local_addr, "Ritma HTTP server started");
    tokio::select! {
        result = &mut server => result.context("serve HTTP requests")?,
        _ = shutdown_signal() => {
            tracing::info!("graceful shutdown started");
            let _ = shutdown_sender.send(());
            match timeout(SHUTDOWN_TIMEOUT, &mut server).await {
                Ok(result) => result.context("drain HTTP requests")?,
                Err(_) => tracing::warn!("graceful shutdown timed out"),
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

async fn healthcheck(listen: SocketAddr) -> ExitCode {
    let url = format!("http://127.0.0.1:{}/healthz", listen.port());
    let client = reqwest::Client::builder()
        .no_proxy()
        .timeout(HEALTHCHECK_TIMEOUT)
        .build();
    match client {
        Ok(client) => match client.get(url).send().await {
            Ok(response) if response.status().is_success() => ExitCode::SUCCESS,
            Ok(_) | Err(_) => ExitCode::FAILURE,
        },
        Err(_) => ExitCode::FAILURE,
    }
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut terminate) => {
                tokio::select! {
                    result = tokio::signal::ctrl_c() => {
                        if let Err(error) = result {
                            tracing::error!(%error, "failed to receive interrupt signal");
                        }
                    }
                    _ = terminate.recv() => {}
                }
            }
            Err(error) => {
                tracing::error!(%error, "failed to register terminate signal");
                if let Err(error) = tokio::signal::ctrl_c().await {
                    tracing::error!(%error, "failed to receive interrupt signal");
                }
            }
        }
    }
    #[cfg(not(unix))]
    if let Err(error) = tokio::signal::ctrl_c().await {
        tracing::error!(%error, "failed to receive interrupt signal");
    }
}

fn print_stdout(message: &str) -> Result<()> {
    writeln!(std::io::stdout().lock(), "{message}").context("write command output")
}
