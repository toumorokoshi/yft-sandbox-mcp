use std::net::SocketAddr;

use reqwest::Client;
use serde_json::json;
use tokio_util::sync::CancellationToken;
use yft_sandbox_mcp::{build_router, default_server_config};

async fn spawn_test_server() -> (Client, String, CancellationToken) {
    let cancellation_token = CancellationToken::new();
    let config = default_server_config(cancellation_token.clone()).with_allowed_hosts(vec![
        "localhost",
        "127.0.0.1",
        "testserver",
    ]);

    let app = build_router(config);
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

fn extract_json_from_sse_or_plain(body: &str) -> serde_json::Value {
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(body) {
        return val;
    }
    for line in body.lines() {
        if let Some(json_str) = line.strip_prefix("data: ") {
            let json_str = json_str.trim();
            if !json_str.is_empty()
                && let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str)
            {
                return val;
            }
        }
    }
    panic!("Could not extract JSON from response body: {body}");
}

#[tokio::test]
async fn test_health_check_endpoints() {
    let (client, base_url, ct) = spawn_test_server().await;

    let res_health = client
        .get(format!("{base_url}/health"))
        .send()
        .await
        .expect("request failed");
    assert_eq!(res_health.status(), 200);
    assert_eq!(res_health.text().await.unwrap(), "OK");

    let res_livez = client
        .get(format!("{base_url}/livez"))
        .send()
        .await
        .expect("request failed");
    assert_eq!(res_livez.status(), 200);
    assert_eq!(res_livez.text().await.unwrap(), "OK");

    ct.cancel();
}

#[tokio::test]
async fn test_mcp_http_initialization_and_tool_call() {
    let (client, base_url, ct) = spawn_test_server().await;

    // 1. Initialize MCP session
    let init_payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "integration-test-client",
                "version": "1.0.0"
            }
        }
    });

    let res_init = client
        .post(format!("{base_url}/mcp"))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .header("MCP-Protocol-Version", "2024-11-05")
        .json(&init_payload)
        .send()
        .await
        .expect("init request failed");

    assert_eq!(res_init.status(), 200);
    let session_id = res_init
        .headers()
        .get("Mcp-Session-Id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let body_text = res_init.text().await.unwrap();
    let init_body = extract_json_from_sse_or_plain(&body_text);
    assert_eq!(init_body["id"], 1);
    assert!(init_body["result"]["capabilities"]["tools"].is_object());

    // 2. Send initialized notification
    let mut notif_builder = client
        .post(format!("{base_url}/mcp"))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .header("MCP-Protocol-Version", "2024-11-05");
    if let Some(ref sid) = session_id {
        notif_builder = notif_builder.header("Mcp-Session-Id", sid);
    }
    let notif_res = notif_builder
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        }))
        .send()
        .await
        .expect("initialized notification failed");
    assert!(notif_res.status().is_success());

    // 3. Call echo tool
    let call_payload = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "echo",
            "arguments": {
                "message": "Cloud Run HTTP Test"
            }
        }
    });

    let mut req_builder = client
        .post(format!("{base_url}/mcp"))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .header("MCP-Protocol-Version", "2024-11-05");

    if let Some(ref sid) = session_id {
        req_builder = req_builder.header("Mcp-Session-Id", sid);
    }

    let res_call = req_builder
        .json(&call_payload)
        .send()
        .await
        .expect("call request failed");

    assert_eq!(res_call.status(), 200);
    let call_body_text = res_call.text().await.expect("read call body");
    let call_body = extract_json_from_sse_or_plain(&call_body_text);
    assert_eq!(call_body["id"], 2);
    let text = call_body["result"]["content"][0]["text"].as_str().unwrap();
    assert_eq!(text, "Echo: Cloud Run HTTP Test");

    ct.cancel();
}

#[tokio::test]
async fn test_disallowed_host_is_rejected() {
    let (client, base_url, ct) = spawn_test_server().await;

    // Send a request with a Host header that is not in the allowlist
    let res = client
        .post(format!("{base_url}/mcp"))
        .header("Host", "malicious-site.com")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&json!({"jsonrpc": "2.0", "id": 1, "method": "initialize"}))
        .send()
        .await
        .expect("request should complete");

    assert_eq!(res.status(), 403);

    ct.cancel();
}
