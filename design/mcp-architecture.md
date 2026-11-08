# MCP Server Architecture

## Goals

1. Provide an extensible, performant Rust-based MCP server for local automations and personal tools.
2. Maintain clean separation between the MCP protocol layer and functional business logic.
3. Ensure easy registration of new tools (e.g., Anki flashcard creation, local note syncing).

## Components

- **`src/main.rs`**: Application entrypoint. Configures `tracing` logger to write exclusively to `stderr`, instantiates `SandboxServer`, and binds to `rmcp::transport::stdio()`.
- **`src/server.rs`**: Defines `SandboxServer`, implementing `ServerHandler` with `#[tool_router]` and `#[tool_handler]` attributes. Routes incoming MCP tool calls to specific tool implementations.
- **`src/tools/`**: Domain-specific action modules.
  - `src/tools/echo.rs`: Example echo tool verifying connectivity and argument parsing.
  - `src/tools/anki.rs`: (Planned) AnkiConnect client for creating cards and managing decks.
- **`tests/`**: Integration tests verifying end-to-end JSON-RPC initialization and tool invocation over in-memory duplex streams.

## Trade-offs & Decisions

- **`rmcp` vs Custom JSON-RPC**: `rmcp` is the official Rust SDK providing automated schema generation via `schemars` and robust macro-based tool dispatch.
- **Stdio vs HTTP**: Stdio is optimal for local agent execution (Claude Desktop, Antigravity, local CLI wrappers), eliminating open port conflicts and authentication overhead.
