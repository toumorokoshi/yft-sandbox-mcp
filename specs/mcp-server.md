# MCP Server Specification

## Protocol & SDK

- **SDK**: `rmcp` (v3.9.0 or compatible official Rust SDK) with `transport-streamable-http-server` and `transport-io`.
- **Transports**:
  - **HTTP (Default)**: Streamable HTTP MCP transport hosted on `axum`, compliant with the Model Context Protocol 2024-11-05 spec. Mounted at `/mcp` and fallback `/`.
  - **Stdio**: Standard input/output JSON-RPC transport activated when `MCP_TRANSPORT=stdio`.
- **Protocol Version**: `2024-11-05` / standard Model Context Protocol.

## HTTP Endpoints & Configuration

- `GET /health` & `GET /livez`: Health check endpoints returning HTTP 200 OK for Cloud Run liveness and startup probes.
- `POST /mcp` (and `GET /mcp` for SSE): MCP JSON-RPC protocol endpoint handling session initialization, notifications, and tool invocations.
- **Environment Variables**:
  - `PORT`: HTTP port to bind to (defaults to `8080` for Cloud Run compatibility).
  - `HOST`: Bind address (defaults to `0.0.0.0`).
  - `ALLOWED_HOSTS`: Comma-separated list of permitted host headers (defaults to `nachochips.tsutsumi.io,localhost,127.0.0.1,0.0.0.0`; set to `*` to disable host validation).
  - `MCP_TRANSPORT`: Set to `stdio` for local client stdio mode; defaults to `http`.

## I/O & Logging Rules

1. When running over stdio, `stdout` is strictly reserved for JSON-RPC MCP framing.
2. `stderr` receives all logs, metrics, diagnostic traces, and debug prints via `tracing_subscriber`.
3. In HTTP mode, logs continue to stream to `stderr` for clean ingestion by Google Cloud Logging.

## Tool Definition Convention

1. Every MCP tool action should reside in its own submodule under `src/tools/`.
2. Input argument structs must derive `Deserialize`, `Serialize`, and `schemars::JsonSchema`.
3. Business logic must be implemented in pure, standalone helper functions separated from MCP parameter unwrapping.
4. Each tool module must include automated unit tests for its core transformation logic.
