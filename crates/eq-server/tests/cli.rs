use axum::Router;
use axum::http::StatusCode;
use axum::routing::get;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::process::Command;
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};
use tokio::net::TcpListener;

const TEST_POSTGRES_PASSWORD: &str = "test-secret-postgres";
const TEST_SECRET_KEY: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

#[test]
fn prints_version() -> anyhow::Result<()> {
    let output = Command::new(env!("CARGO_BIN_EXE_ritma"))
        .arg("--version")
        .output()?;

    assert!(output.status.success());
    assert_eq!(output.stdout, b"ritma 0.1.0\n");
    Ok(())
}

#[test]
fn version_subcommand_reports_build_metadata() -> anyhow::Result<()> {
    let output = Command::new(env!("CARGO_BIN_EXE_ritma"))
        .arg("version")
        .output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.starts_with("ritma 0.1.0 commit="));
    assert!(stdout.contains(" features="));
    Ok(())
}

#[tokio::test]
async fn migrate_applies() -> anyhow::Result<()> {
    let (_container, owner_url, pool) = start_postgres().await?;
    let output = configured_command("migrate", None, &owner_url).output()?;

    assert!(output.status.success(), "migration command failed");
    let applied: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM _sqlx_migrations WHERE version = 1 AND success = TRUE)",
    )
    .fetch_one(&pool)
    .await?;
    assert!(applied);
    pool.close().await;
    Ok(())
}

#[test]
fn unimplemented_subcommand_exits_2() -> anyhow::Result<()> {
    let output = Command::new(env!("CARGO_BIN_EXE_ritma")).arg("doctor").output()?;

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout, b"doctor is not implemented yet\n");
    Ok(())
}

#[tokio::test]
async fn healthcheck_exit_codes() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            Router::new().route("/healthz", get(|| async { StatusCode::OK })),
        )
        .await
    });
    let server_response = reqwest::get(format!("http://127.0.0.1:{}/healthz", address.port())).await?;
    assert_eq!(server_response.status(), StatusCode::OK);
    let mut command = configured_command(
        "healthcheck",
        Some(address),
        "postgres://eq_app:test-secret-app@127.0.0.1:5433/ritma",
    );
    let output = tokio::task::spawn_blocking(move || command.output()).await??;
    server.abort();
    assert!(
        output.status.success(),
        "healthcheck exited {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let unused_address = listener.local_addr()?;
    drop(listener);
    let mut command = configured_command(
        "healthcheck",
        Some(unused_address),
        "postgres://eq_app:test-secret-app@127.0.0.1:5433/ritma",
    );
    let output = tokio::task::spawn_blocking(move || command.output()).await??;
    assert_eq!(output.status.code(), Some(1));
    Ok(())
}

fn configured_command(command: &str, listen: Option<SocketAddr>, database_url: &str) -> Command {
    let mut process = Command::new(env!("CARGO_BIN_EXE_ritma"));
    process
        .arg(command)
        .env("DATABASE_URL", database_url)
        .env("RITMA_DATABASE_OWNER_URL", database_url)
        .env("RITMA_SECRET_KEY", TEST_SECRET_KEY)
        .env("RITMA_PUBLIC_URL", "http://localhost:8080");
    if let Some(listen) = listen {
        process.env("RITMA_LISTEN", listen.to_string());
    }
    process
}

async fn start_postgres() -> anyhow::Result<(ContainerAsync<GenericImage>, String, sqlx::PgPool)> {
    let image = GenericImage::new("postgres", "16-alpine")
        .with_exposed_port(5432.tcp())
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_wait_for(WaitFor::message_on_stdout(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_USER", "postgres")
        .with_env_var("POSTGRES_PASSWORD", TEST_POSTGRES_PASSWORD)
        .with_env_var("POSTGRES_DB", "postgres");
    let container = image.start().await?;
    let host = container.get_host().await?.to_string();
    let port = container.get_host_port_ipv4(5432).await?;
    let owner_url = format!("postgres://postgres:{TEST_POSTGRES_PASSWORD}@{host}:{port}/postgres");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&owner_url)
        .await?;
    Ok((container, owner_url, pool))
}
