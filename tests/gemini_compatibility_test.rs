//! Gemini client compatibility tests.
//!
//! Validates the exact HTTP sequence and payload expectations observed from
//! Google's Gemini Developer Mode and automated MCP validators:
//! 1. Reachability `HEAD /mcp` returning `200 OK`.
//! 2. Unconfigured OAuth discovery `/.well-known/*` returning `404 Not Found`
//!    without an authentication challenge header per RFC 9728.
//! 3. Reachability `GET /mcp` returning `200 OK` JSON metadata.
//! 4. JSON-RPC `POST /mcp` returning `Content-Type: application/json` with pure
//!    JSON (rejecting any Server-Sent Events `data:` framing).

use std::net::SocketAddr;

use reqwest::Client;
use serde_json::json;
use tokio_util::sync::CancellationToken;
use yft_sandbox_mcp::{build_router, default_server_config, parse_auth_token};

const TEST_TOKEN: &str = "gemini-compat-secret-token";
const GEMINI_USER_AGENT: &str = "Google";
const GEMINI_PROTOCOL_VERSION: &str = "2025-11-25";
const JSON_CONTENT_TYPE: &str = "application/json";

/// Pure function verifying that a HEAD response meets Gemini reachability expectations.
pub fn verify_head_response(
    status: u16,
    content_type: Option<&str>,
    body_empty: bool,
) -> Result<(), String> {
    if status != 200 {
        return Err(format!("expected 200 OK for HEAD, got {status}"));
    }
    if !body_empty {
        return Err("expected empty body for HEAD response".to_string());
    }
    if let Some(ct) = content_type
        && !ct.starts_with(JSON_CONTENT_TYPE)
    {
        return Err(format!(
            "expected {JSON_CONTENT_TYPE} content type, got {ct}"
        ));
    }
    Ok(())
}

/// Pure function verifying that OAuth discovery returns 404 without an auth challenge.
pub fn verify_well_known_not_found(
    status: u16,
    has_www_authenticate: bool,
    body: &str,
) -> Result<(), String> {
    if status != 404 {
        return Err(format!("expected 404 Not Found, got {status}"));
    }
    if has_www_authenticate {
        return Err("404 response must not include WWW-Authenticate challenge".to_string());
    }
    if !body.contains("not_found") {
        return Err(format!("expected body containing not_found, got {body}"));
    }
    Ok(())
}

/// Pure function verifying that an MCP response is pure JSON and NOT SSE-framed.
pub fn verify_gemini_json_response(
    status: u16,
    content_type: Option<&str>,
    body: &str,
) -> Result<serde_json::Value, String> {
    if status != 200 {
        return Err(format!("expected status 200 OK, got {status}"));
    }
    let Some(ct) = content_type else {
        return Err("missing Content-Type header".to_string());
    };
    if !ct.starts_with(JSON_CONTENT_TYPE) {
        return Err(format!(
            "expected Content-Type starting with {JSON_CONTENT_TYPE}, got {ct}"
        ));
    }
    if body.trim_start().starts_with("data:") {
        return Err(
            "response is framed as Server-Sent Events (SSE); Gemini requires raw application/json"
                .to_string(),
        );
    }
    serde_json::from_str::<serde_json::Value>(body)
        .map_err(|e| format!("failed to parse response body as JSON: {e}"))
}

/// Build the Gemini MCP initialize request payload.
#[must_use]
pub fn make_gemini_initialize_request() -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": GEMINI_PROTOCOL_VERSION,
            "capabilities": {
                "roots": { "listChanged": true },
                "sampling": {}
            },
            "clientInfo": {
                "name": "mcp",
                "version": "0.1.0"
            }
        }
    })
}

/// Build the Gemini MCP tools/list request payload.
#[must_use]
pub fn make_gemini_tools_list_request(id: u64) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "tools/list",
        "params": {}
    })
}

/// Build the Gemini MCP tools/call request payload.
#[must_use]
pub fn make_gemini_tool_call_request(id: u64, name: &str, message: &str) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "tools/call",
        "params": {
            "name": name,
            "arguments": {
                "message": message
            }
        }
    })
}

// -----------------------------------------------------------------------------
// Pure Unit Tests (Testing Data Structures Directly Without IO)
// -----------------------------------------------------------------------------

#[test]
fn test_verify_head_response() {
    assert!(verify_head_response(200, Some("application/json"), true).is_ok());
    assert!(verify_head_response(200, None, true).is_ok());
    assert!(verify_head_response(405, Some("application/json"), true).is_err());
    assert!(verify_head_response(200, Some("application/json"), false).is_err());
    assert!(verify_head_response(200, Some("text/html"), true).is_err());
}

#[test]
fn test_verify_well_known_not_found() {
    assert!(verify_well_known_not_found(404, false, r#"{"error":"not_found"}"#).is_ok());
    assert!(verify_well_known_not_found(401, true, "Unauthorized").is_err());
    assert!(verify_well_known_not_found(404, true, r#"{"error":"not_found"}"#).is_err());
    assert!(verify_well_known_not_found(200, false, "{}").is_err());
}

#[test]
fn test_verify_gemini_json_response() {
    let valid_json = r#"{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-11-25"}}"#;
    assert!(verify_gemini_json_response(200, Some("application/json"), valid_json).is_ok());

    // SSE data framing must be rejected
    let sse_framed = "data: {\"jsonrpc\":\"2.0\",\"id\":1}\n\n";
    assert!(verify_gemini_json_response(200, Some("application/json"), sse_framed).is_err());

    // text/event-stream content type must be rejected
    assert!(verify_gemini_json_response(200, Some("text/event-stream"), valid_json).is_err());

    // non-200 status must be rejected
    assert!(verify_gemini_json_response(406, Some("application/json"), valid_json).is_err());
}

#[test]
fn test_gemini_request_builders() {
    let init = make_gemini_initialize_request();
    assert_eq!(init["method"], "initialize");
    assert_eq!(init["params"]["protocolVersion"], GEMINI_PROTOCOL_VERSION);

    let tools_list = make_gemini_tools_list_request(2);
    assert_eq!(tools_list["method"], "tools/list");
    assert_eq!(tools_list["id"], 2);

    let call = make_gemini_tool_call_request(3, "echo", "test message");
    assert_eq!(call["method"], "tools/call");
    assert_eq!(call["params"]["name"], "echo");
    assert_eq!(call["params"]["arguments"]["message"], "test message");
}

// -----------------------------------------------------------------------------
// Integration Test (Single IO Test Exercising Live Server)
// -----------------------------------------------------------------------------

async fn spawn_test_server() -> (Client, String, CancellationToken) {
    let cancellation_token = CancellationToken::new();
    let config = default_server_config(cancellation_token.clone()).with_allowed_hosts(vec![
        "localhost",
        "127.0.0.1",
        "testserver",
    ]);

    let auth_token = parse_auth_token(TEST_TOKEN).expect("valid test token");
    let app = build_router(config, auth_token);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind listener");
    let addr: SocketAddr = listener.local_addr().expect("listener should have addr");

    let ct = cancellation_token.clone();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                ct.cancelled_owned().await;
            })
            .await;
    });

    (Client::new(), format!("http://{addr}"), cancellation_token)
}

#[tokio::test]
async fn test_gemini_validation_sequence_and_tool_execution() {
    let (client, base_url, ct) = spawn_test_server().await;

    // 1. HEAD /mcp (anonymous preflight reachability check)
    let res_head_anon = client
        .head(format!("{base_url}/mcp"))
        .header("User-Agent", GEMINI_USER_AGENT)
        .send()
        .await
        .expect("head anon failed");
    let ct_header = res_head_anon
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok());
    assert!(verify_head_response(res_head_anon.status().as_u16(), ct_header, true).is_ok());

    // 2. HEAD /mcp?token=... (authenticated preflight check)
    let res_head_auth = client
        .head(format!("{base_url}/mcp?token={TEST_TOKEN}"))
        .header("User-Agent", GEMINI_USER_AGENT)
        .send()
        .await
        .expect("head auth failed");
    let ct_header = res_head_auth
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok());
    assert!(verify_head_response(res_head_auth.status().as_u16(), ct_header, true).is_ok());

    // 3. GET /.well-known/oauth-protected-resource (OAuth discovery probe)
    let res_well_known = client
        .get(format!("{base_url}/.well-known/oauth-protected-resource"))
        .header("User-Agent", GEMINI_USER_AGENT)
        .send()
        .await
        .expect("well-known probe failed");
    let has_auth_challenge = res_well_known.headers().contains_key("www-authenticate");
    let well_known_status = res_well_known.status().as_u16();
    let well_known_body = res_well_known.text().await.expect("read well-known body");
    assert!(
        verify_well_known_not_found(well_known_status, has_auth_challenge, &well_known_body)
            .is_ok()
    );

    // 4. GET /mcp?token=... (probe without Accept: text/event-stream)
    let res_get_probe = client
        .get(format!("{base_url}/mcp?token={TEST_TOKEN}"))
        .header("User-Agent", GEMINI_USER_AGENT)
        .send()
        .await
        .expect("get probe failed");
    let probe_body: serde_json::Value = res_get_probe.json().await.expect("json parse probe");
    assert_eq!(probe_body["status"], "ok");
    assert_eq!(probe_body["server"], "yft-sandbox-mcp");

    // 5. POST /mcp?token=... (Gemini initialize request)
    let res_init = client
        .post(format!("{base_url}/mcp?token={TEST_TOKEN}"))
        .header("User-Agent", GEMINI_USER_AGENT)
        .header("Content-Type", JSON_CONTENT_TYPE)
        .header("Accept", JSON_CONTENT_TYPE)
        .json(&make_gemini_initialize_request())
        .send()
        .await
        .expect("init request failed");

    let init_ct = res_init
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let init_body = res_init.text().await.expect("read init body");
    let parsed_init = verify_gemini_json_response(200, init_ct.as_deref(), &init_body)
        .expect("verify gemini initialize response");
    assert_eq!(parsed_init["jsonrpc"], "2.0");
    assert_eq!(parsed_init["id"], 1);
    assert_eq!(
        parsed_init["result"]["protocolVersion"],
        GEMINI_PROTOCOL_VERSION
    );

    // 6. POST /mcp?token=... (Gemini tools/list request)
    let res_tools = client
        .post(format!("{base_url}/mcp?token={TEST_TOKEN}"))
        .header("User-Agent", GEMINI_USER_AGENT)
        .header("Content-Type", JSON_CONTENT_TYPE)
        .header("Accept", JSON_CONTENT_TYPE)
        .json(&make_gemini_tools_list_request(2))
        .send()
        .await
        .expect("tools/list request failed");

    let tools_ct = res_tools
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let tools_body = res_tools.text().await.expect("read tools body");
    let parsed_tools = verify_gemini_json_response(200, tools_ct.as_deref(), &tools_body)
        .expect("verify gemini tools/list response");
    assert_eq!(parsed_tools["jsonrpc"], "2.0");
    assert_eq!(parsed_tools["id"], 2);
    let tools = parsed_tools["result"]["tools"]
        .as_array()
        .expect("tools array");
    assert!(tools.iter().any(|t| t["name"] == "echo"));

    // 7. POST /mcp?token=... (Gemini tools/call echo request)
    let res_call = client
        .post(format!("{base_url}/mcp?token={TEST_TOKEN}"))
        .header("User-Agent", GEMINI_USER_AGENT)
        .header("Content-Type", JSON_CONTENT_TYPE)
        .header("Accept", JSON_CONTENT_TYPE)
        .json(&make_gemini_tool_call_request(
            3,
            "echo",
            "Gemini test message",
        ))
        .send()
        .await
        .expect("tools/call request failed");

    let call_ct = res_call
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let call_body = res_call.text().await.expect("read call body");
    let parsed_call = verify_gemini_json_response(200, call_ct.as_deref(), &call_body)
        .expect("verify gemini tools/call response");
    assert_eq!(parsed_call["jsonrpc"], "2.0");
    assert_eq!(parsed_call["id"], 3);
    assert_eq!(
        parsed_call["result"]["content"][0]["text"],
        "Echo: Gemini test message"
    );

    ct.cancel();
}
