//! Shared-secret authentication for the HTTP transport.
//!
//! Clients authenticate by sending the secret either as a standard
//! `Authorization: Bearer <token>` header (preferred, per the MCP authorization
//! spec) or as a `?token=<token>` query parameter for clients that can only be
//! configured with a URL.

use std::{path::Path, sync::Arc};

use axum::{
    extract::{Request, State},
    http::{HeaderMap, Method, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use subtle::ConstantTimeEq;

/// Environment variable that overrides the path of the auth token file.
pub const AUTH_TOKEN_FILE_ENV: &str = "MCP_AUTH_TOKEN_FILE";

/// Default auth token file path, relative to the working directory.
pub const DEFAULT_AUTH_TOKEN_FILE: &str = "secrets/mcp-auth-token";

/// Query parameter name accepted as a fallback to the `Authorization` header.
pub const TOKEN_QUERY_PARAM: &str = "token";

const BEARER_PREFIX: &str = "Bearer ";

/// The shared secret clients must present to use the MCP endpoint.
#[derive(Clone)]
pub struct AuthToken(Arc<str>);

impl std::fmt::Debug for AuthToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AuthToken(<redacted>)")
    }
}

/// Errors produced while loading the auth token.
#[derive(Debug)]
pub enum AuthTokenError {
    /// The token file could not be read.
    Io(std::io::Error),
    /// The token file was empty or contained only whitespace.
    Empty,
}

impl std::fmt::Display for AuthTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "failed to read auth token file: {err}"),
            Self::Empty => f.write_str("auth token file is empty"),
        }
    }
}

impl std::error::Error for AuthTokenError {}

/// Parse the contents of a token file into an `AuthToken`.
///
/// # Errors
/// Returns `AuthTokenError::Empty` if the contents are blank.
pub fn parse_auth_token(contents: &str) -> Result<AuthToken, AuthTokenError> {
    let trimmed = contents.trim();
    if trimmed.is_empty() {
        return Err(AuthTokenError::Empty);
    }
    Ok(AuthToken(Arc::from(trimmed)))
}

/// Read and parse the auth token from a file.
///
/// # Errors
/// Returns an error if the file cannot be read or is empty.
pub fn load_auth_token(path: &Path) -> Result<AuthToken, AuthTokenError> {
    let contents = std::fs::read_to_string(path).map_err(AuthTokenError::Io)?;
    parse_auth_token(&contents)
}

/// Resolve the auth token file path from an optional environment override.
#[must_use]
pub fn auth_token_path(env_override: Option<String>) -> String {
    env_override.unwrap_or_else(|| DEFAULT_AUTH_TOKEN_FILE.to_string())
}

/// Extract the token from an `Authorization: Bearer <token>` header value.
#[must_use]
pub fn bearer_token(authorization: Option<&str>) -> Option<&str> {
    authorization?.strip_prefix(BEARER_PREFIX).map(str::trim)
}

/// Extract the token from the `token` query parameter of a raw query string.
#[must_use]
pub fn query_token(query: Option<&str>) -> Option<String> {
    form_urlencoded::parse(query?.as_bytes())
        .find(|(key, _)| key == TOKEN_QUERY_PARAM)
        .map(|(_, value)| value.into_owned())
}

/// Compare a presented token against the expected token in constant time.
#[must_use]
pub fn token_matches(expected: &AuthToken, presented: &str) -> bool {
    expected.0.as_bytes().ct_eq(presented.as_bytes()).into()
}

/// Decide whether a request is authorized from its header and query string.
#[must_use]
pub fn is_authorized(expected: &AuthToken, headers: &HeaderMap, query: Option<&str>) -> bool {
    let header_value = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());
    let presented = bearer_token(header_value)
        .map(str::to_string)
        .or_else(|| query_token(query));
    presented.is_some_and(|token| token_matches(expected, &token))
}

fn unauthorized_response() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        [(header::WWW_AUTHENTICATE, "Bearer")],
        "Unauthorized",
    )
        .into_response()
}

/// Axum middleware that rejects requests without a valid auth token.
pub async fn require_auth_token(
    State(expected): State<AuthToken>,
    request: Request,
    next: Next,
) -> Response {
    if request.method() == Method::HEAD {
        return (StatusCode::OK, [(header::CONTENT_TYPE, "application/json")]).into_response();
    }
    if is_authorized(&expected, request.headers(), request.uri().query()) {
        return next.run(request).await;
    }
    tracing::warn!(uri.path = %request.uri().path(), "Rejected unauthorized MCP request");
    unauthorized_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn token() -> AuthToken {
        parse_auth_token("s3cret\n").expect("valid token")
    }

    #[test]
    fn test_parse_auth_token_trims_and_rejects_empty() {
        assert!(token_matches(&token(), "s3cret"));
        assert!(matches!(
            parse_auth_token("  \n"),
            Err(AuthTokenError::Empty)
        ));
    }

    #[test]
    fn test_auth_token_debug_is_redacted() {
        assert_eq!(format!("{:?}", token()), "AuthToken(<redacted>)");
    }

    #[test]
    fn test_auth_token_path() {
        assert_eq!(auth_token_path(None), DEFAULT_AUTH_TOKEN_FILE);
        assert_eq!(auth_token_path(Some("/x".to_string())), "/x");
    }

    #[test]
    fn test_bearer_token() {
        assert_eq!(bearer_token(Some("Bearer abc")), Some("abc"));
        assert_eq!(bearer_token(Some("Basic abc")), None);
        assert_eq!(bearer_token(None), None);
    }

    #[test]
    fn test_query_token() {
        assert_eq!(
            query_token(Some("a=1&token=a%2Bb")),
            Some("a+b".to_string())
        );
        assert_eq!(query_token(Some("a=1")), None);
        assert_eq!(query_token(None), None);
    }

    #[test]
    fn test_is_authorized() {
        let expected = token();
        let mut headers = HeaderMap::new();
        assert!(!is_authorized(&expected, &headers, None));
        assert!(is_authorized(&expected, &headers, Some("token=s3cret")));
        assert!(!is_authorized(&expected, &headers, Some("token=wrong")));

        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer s3cret"),
        );
        assert!(is_authorized(&expected, &headers, None));

        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer wrong"),
        );
        assert!(!is_authorized(&expected, &headers, None));
    }
}
