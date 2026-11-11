use base64::prelude::*;
use rmcp::ServiceExt;
use serde_json::json;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use yft_sandbox_mcp::SandboxServer;

#[tokio::test]
async fn test_mcp_server_handshake_and_echo() -> Result<(), Box<dyn std::error::Error>> {
    let (server_transport, client_transport) = tokio::io::duplex(4096);

    let server_task = tokio::spawn(async move {
        let server = SandboxServer::new();
        let running = server.serve(server_transport).await?;
        let _ = running.waiting().await;
        Ok::<(), anyhow::Error>(())
    });

    let (client_read, mut client_write) = tokio::io::split(client_transport);
    let mut reader = BufReader::new(client_read);

    // 1. Send initialize request
    let init_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });
    client_write
        .write_all(format!("{init_req}\n").as_bytes())
        .await?;
    client_write.flush().await?;

    let mut line = String::new();
    reader.read_line(&mut line).await?;
    let init_resp: serde_json::Value = serde_json::from_str(&line)?;
    assert_eq!(init_resp["id"], 1);
    assert!(init_resp["result"]["capabilities"]["tools"].is_object());

    // 2. Send initialized notification
    let init_notif = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    client_write
        .write_all(format!("{init_notif}\n").as_bytes())
        .await?;
    client_write.flush().await?;

    // 3. Call echo tool
    let call_req = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "echo",
            "arguments": {
                "message": "Hello MCP!"
            }
        }
    });
    client_write
        .write_all(format!("{call_req}\n").as_bytes())
        .await?;
    client_write.flush().await?;

    line.clear();
    reader.read_line(&mut line).await?;
    let call_resp: serde_json::Value = serde_json::from_str(&line)?;
    assert_eq!(call_resp["id"], 2);
    let text = call_resp["result"]["content"][0]["text"].as_str().unwrap();
    assert_eq!(text, "Echo: Hello MCP!");

    server_task.abort();
    let _ = server_task.await;

    Ok(())
}

#[tokio::test]
async fn test_mcp_server_bundle_anki_flashcard() -> Result<(), Box<dyn std::error::Error>> {
    let (server_transport, client_transport) = tokio::io::duplex(65536);

    let server_task = tokio::spawn(async move {
        let server = SandboxServer::new();
        let running = server.serve(server_transport).await?;
        let _ = running.waiting().await;
        Ok::<(), anyhow::Error>(())
    });

    let (client_read, mut client_write) = tokio::io::split(client_transport);
    let mut reader = BufReader::new(client_read);

    // 1. Send initialize request
    let init_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });
    client_write
        .write_all(format!("{init_req}\n").as_bytes())
        .await?;
    client_write.flush().await?;

    let mut line = String::new();
    reader.read_line(&mut line).await?;

    // 2. Send initialized notification
    let init_notif = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    client_write
        .write_all(format!("{init_notif}\n").as_bytes())
        .await?;
    client_write.flush().await?;

    // 3. Call bundle-anki-flashcard tool
    let call_req = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "bundle-anki-flashcard",
            "arguments": {
                "deck_name": "Rust Trivia",
                "question": "What is ownership in Rust?",
                "answer": "A set of rules governing how memory is managed at compile time."
            }
        }
    });
    client_write
        .write_all(format!("{call_req}\n").as_bytes())
        .await?;
    client_write.flush().await?;

    line.clear();
    reader.read_line(&mut line).await?;
    let call_resp: serde_json::Value = serde_json::from_str(&line)?;
    assert_eq!(call_resp["id"], 2);
    assert_eq!(call_resp["result"]["isError"], false);

    let content = call_resp["result"]["content"].as_array().unwrap();
    assert_eq!(content.len(), 2);
    assert_eq!(content[0]["type"], "text");
    assert!(
        content[0]["text"]
            .as_str()
            .unwrap()
            .contains("Rust_Trivia.apkg")
    );

    assert_eq!(content[1]["type"], "resource");
    let resource = &content[1]["resource"];
    assert_eq!(resource["uri"], "memo://Rust_Trivia.apkg");
    assert_eq!(resource["mimeType"], "application/octet-stream");

    let blob_base64 = resource["blob"].as_str().unwrap();
    let apkg_bytes = BASE64_STANDARD.decode(blob_base64).unwrap();
    assert_eq!(&apkg_bytes[0..4], &[0x50, 0x4B, 0x03, 0x04]);

    server_task.abort();
    let _ = server_task.await;

    Ok(())
}
