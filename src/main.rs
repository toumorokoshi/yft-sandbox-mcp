use std::{net::SocketAddr, path::Path};

use rmcp::ServiceExt;
use tokio_util::sync::CancellationToken;
use yft_sandbox_mcp::{
    SandboxServer,
    auth::{AUTH_TOKEN_FILE_ENV, auth_token_path, load_auth_token},
    default_server_config, run_http_server,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Tracing logs must be directed to stderr so stdout remains clean for MCP JSON-RPC protocol
    // when running in stdio mode, and Cloud Logging captures stderr cleanly on Cloud Run.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let transport_mode = std::env::var("MCP_TRANSPORT").unwrap_or_else(|_| "http".to_string());

    if transport_mode.eq_ignore_ascii_case("stdio") {
        tracing::info!("Starting yft-sandbox-mcp server on stdio transport");
        let server = SandboxServer::new();
        server
            .serve(rmcp::transport::stdio())
            .await?
            .waiting()
            .await?;
        return Ok(());
    }

    let token_path = auth_token_path(std::env::var(AUTH_TOKEN_FILE_ENV).ok());
    let auth_token = load_auth_token(Path::new(&token_path))
        .map_err(|err| format!("{err} (path: {token_path}, set {AUTH_TOKEN_FILE_ENV})"))?;
    tracing::info!(path = %token_path, "Loaded MCP auth token");

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr: SocketAddr = format!("{host}:{port}").parse()?;

    let cancellation_token = CancellationToken::new();
    let shutdown_token = cancellation_token.clone();

    tokio::spawn(async move {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{SignalKind, signal};
            let mut term =
                signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
            let mut int =
                signal(SignalKind::interrupt()).expect("failed to install SIGINT handler");
            tokio::select! {
                _ = term.recv() => {
                    tracing::info!("Received SIGTERM signal, shutting down");
                }
                _ = int.recv() => {
                    tracing::info!("Received SIGINT signal, shutting down");
                }
            }
        }
        #[cfg(not(unix))]
        {
            let _ = tokio::signal::ctrl_c().await;
            tracing::info!("Received Ctrl+C signal, shutting down");
        }
        shutdown_token.cancel();
    });

    let config = default_server_config(cancellation_token.clone());
    tracing::info!(%addr, "Starting yft-sandbox-mcp server on HTTP transport (Cloud Run ready)");

    run_http_server(addr, config, auth_token, cancellation_token).await?;

    Ok(())
}
