//! HTTP transport and routing for Cloud Run and web clients.

use std::{net::SocketAddr, sync::Arc};

use axum::{
    Router,
    http::{HeaderValue, Method, StatusCode, header},
    middleware,
    response::IntoResponse,
    routing::{any, get},
};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use tokio_util::sync::CancellationToken;

use crate::{
    auth::{AuthToken, require_auth_token},
    server::SandboxServer,
};

/// JSON MIME type constant.
pub const JSON_MIME_TYPE: &str = "application/json";

/// Server-Sent Events MIME type constant.
pub const EVENT_STREAM_MIME_TYPE: &str = "text/event-stream";

/// Header value required by rmcp StreamableHttpServer for POST requests.
pub const MCP_ACCEPT_REQUIRED: &str = "application/json, text/event-stream";

/// Response body for HTTP probe / info GET requests on MCP routes.
pub const SERVER_INFO_BODY: &str =
    r#"{"status":"ok","server":"yft-sandbox-mcp","protocol":"mcp","transport":"streamable-http"}"#;

/// Response body for unconfigured OAuth discovery.
pub const NOT_FOUND_BODY: &str = r#"{"error":"not_found"}"#;

/// Health check handler for Cloud Run liveness and startup probes.
pub async fn health_check() -> impl IntoResponse {
    "OK"
}

/// Handler for `/.well-known/*` paths returning 404 without authentication challenge.
pub async fn well_known_not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        [(header::CONTENT_TYPE, JSON_MIME_TYPE)],
        NOT_FOUND_BODY,
    )
}

/// Check if the request Accept header includes `text/event-stream` for SSE streaming.
#[must_use]
pub fn accepts_event_stream(accept_header: Option<&str>) -> bool {
    accept_header.is_some_and(|h| {
        h.split(',').any(|part| {
            part.trim()
                .split(';')
                .next()
                .is_some_and(|essence| essence.trim().eq_ignore_ascii_case(EVENT_STREAM_MIME_TYPE))
        })
    })
}

/// Determine whether an Accept header should be normalized for rmcp POST compatibility.
#[must_use]
pub fn needs_accept_normalization(accept_header: Option<&str>) -> bool {
    let Some(h) = accept_header else {
        return true;
    };
    let mut has_json = false;
    let mut has_sse = false;
    for part in h.split(',') {
        if let Some(essence) = part.trim().split(';').next() {
            let essence = essence.trim();
            if essence.eq_ignore_ascii_case(JSON_MIME_TYPE) {
                has_json = true;
            } else if essence.eq_ignore_ascii_case(EVENT_STREAM_MIME_TYPE) {
                has_sse = true;
            }
        }
    }
    !has_json || !has_sse
}

/// Axum middleware providing HTTP compatibility for MCP probes and client variations.
pub async fn mcp_compat_middleware(
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    if request.method() == Method::HEAD {
        return (StatusCode::OK, [(header::CONTENT_TYPE, JSON_MIME_TYPE)]).into_response();
    }

    if request.method() == Method::GET {
        let accept_header = request
            .headers()
            .get(header::ACCEPT)
            .and_then(|v| v.to_str().ok());
        if !accepts_event_stream(accept_header) {
            return (
                StatusCode::OK,
                [(header::CONTENT_TYPE, JSON_MIME_TYPE)],
                SERVER_INFO_BODY,
            )
                .into_response();
        }
    }

    if request.method() == Method::POST {
        let accept_header = request
            .headers()
            .get(header::ACCEPT)
            .and_then(|v| v.to_str().ok());
        if needs_accept_normalization(accept_header) {
            request.headers_mut().insert(
                header::ACCEPT,
                HeaderValue::from_static(MCP_ACCEPT_REQUIRED),
            );
        }
    }

    next.run(request).await
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
///
/// Health checks and OAuth discovery are unauthenticated; every MCP route
/// requires `auth_token` and HTTP compatibility layers.
pub fn build_router(config: StreamableHttpServerConfig, auth_token: AuthToken) -> Router {
    let session_manager = Arc::new(LocalSessionManager::default());
    let service: StreamableHttpService<SandboxServer, LocalSessionManager> =
        StreamableHttpService::new(|| Ok(SandboxServer::new()), session_manager, config);

    let mcp_routes = Router::new()
        .nest_service("/mcp", service.clone())
        .fallback_service(service)
        .layer(middleware::from_fn(mcp_compat_middleware))
        .layer(middleware::from_fn_with_state(
            auth_token,
            require_auth_token,
        ));

    Router::new()
        .route("/health", get(health_check))
        .route("/livez", get(health_check))
        .route("/.well-known", any(well_known_not_found))
        .route("/.well-known/{*path}", any(well_known_not_found))
        .merge(mcp_routes)
}

/// Run the HTTP server bound to the specified socket address until cancelled.
///
/// # Errors
/// Returns an error if binding to the address or serving traffic fails.
pub async fn run_http_server(
    addr: SocketAddr,
    config: StreamableHttpServerConfig,
    auth_token: AuthToken,
    cancellation_token: CancellationToken,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = build_router(config, auth_token);
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

    #[tokio::test]
    async fn test_well_known_not_found_endpoint() {
        let response = well_known_not_found().await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_accepts_event_stream() {
        assert!(accepts_event_stream(Some("text/event-stream")));
        assert!(accepts_event_stream(Some(
            "text/event-stream; charset=utf-8"
        )));
        assert!(accepts_event_stream(Some(
            "application/json, text/event-stream"
        )));
        assert!(!accepts_event_stream(Some("application/json")));
        assert!(!accepts_event_stream(Some("*/*")));
        assert!(!accepts_event_stream(None));
    }

    #[test]
    fn test_needs_accept_normalization() {
        assert!(needs_accept_normalization(None));
        assert!(needs_accept_normalization(Some("*/*")));
        assert!(needs_accept_normalization(Some("application/json")));
        assert!(needs_accept_normalization(Some("text/event-stream")));
        assert!(!needs_accept_normalization(Some(
            "application/json, text/event-stream"
        )));
        assert!(!needs_accept_normalization(Some(
            "text/event-stream, application/json"
        )));
    }
}
