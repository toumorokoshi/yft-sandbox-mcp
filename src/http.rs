//! HTTP transport and routing for Cloud Run and web clients.

use std::{net::SocketAddr, sync::Arc};

use axum::{Router, response::IntoResponse, routing::get};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use tokio_util::sync::CancellationToken;

use crate::server::SandboxServer;

/// Health check handler for Cloud Run liveness and startup probes.
pub async fn health_check() -> impl IntoResponse {
    "OK"
}

/// Parse a comma-separated list of allowed hostnames or disable if wildcard.
#[must_use]
pub fn parse_allowed_hosts(hosts_str: &str) -> Option<Vec<String>> {
    let trimmed = hosts_str.trim();
    if trimmed == "*" || trimmed.eq_ignore_ascii_case("all") {
        return None;
    }
    let hosts: Vec<String> = hosts_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    Some(hosts)
}

/// Construct the default `StreamableHttpServerConfig` honoring environment settings.
#[must_use]
pub fn default_server_config(cancellation_token: CancellationToken) -> StreamableHttpServerConfig {
    let mut config = StreamableHttpServerConfig::default()
        .with_cancellation_token(cancellation_token)
        .with_json_response(true);

    let allowed_hosts_env = std::env::var("ALLOWED_HOSTS").unwrap_or_else(|_| {
        "nachochips.tsutsumi.io,nachochips.yusuketsutsumi.com,localhost,127.0.0.1,0.0.0.0"
            .to_string()
    });

    if let Some(hosts) = parse_allowed_hosts(&allowed_hosts_env) {
        config = config.with_allowed_hosts(hosts);
    } else {
        config = config.disable_allowed_hosts();
    }

    config
}

/// Build the Axum router with health checks and MCP streamable HTTP service.
pub fn build_router(config: StreamableHttpServerConfig) -> Router {
    let session_manager = Arc::new(LocalSessionManager::default());
    let service: StreamableHttpService<SandboxServer, LocalSessionManager> =
        StreamableHttpService::new(|| Ok(SandboxServer::new()), session_manager, config);

    Router::new()
        .route("/health", get(health_check))
        .route("/livez", get(health_check))
        .nest_service("/mcp", service.clone())
        .fallback_service(service)
}

/// Run the HTTP server bound to the specified socket address until cancelled.
///
/// # Errors
/// Returns an error if binding to the address or serving traffic fails.
pub async fn run_http_server(
    addr: SocketAddr,
    config: StreamableHttpServerConfig,
    cancellation_token: CancellationToken,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = build_router(config);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "HTTP server listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            cancellation_token.cancelled_owned().await;
            tracing::info!("HTTP server received shutdown signal");
        })
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_allowed_hosts() {
        assert_eq!(parse_allowed_hosts("*"), None);
        assert_eq!(parse_allowed_hosts("all"), None);
        assert_eq!(
            parse_allowed_hosts("example.com, localhost"),
            Some(vec!["example.com".to_string(), "localhost".to_string()])
        );
    }

    #[tokio::test]
    async fn test_health_check_endpoint() {
        let response = health_check().await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }
}
