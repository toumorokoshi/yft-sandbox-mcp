# MCP Server Specification

## Protocol & SDK

- **SDK**: `rmcp` (v3.9.0 or compatible official Rust SDK).
- **Transport**: Standard input/output (stdio) JSON-RPC protocol transport.
- **Protocol Version**: 2024-11-05 / standard Model Context Protocol.

## I/O & Logging Rules

1. `stdout` is strictly reserved for JSON-RPC MCP framing.
2. `stderr` must receive all logs, metrics, diagnostic traces, and debug prints via `tracing_subscriber`.
3. Panics and standard error output must never write unformatted lines to `stdout`.

## Tool Definition Convention

1. Every MCP tool action should reside in its own submodule under `src/tools/`.
2. Input argument structs must derive `Deserialize`, `Serialize`, and `schemars::JsonSchema`.
3. Business logic must be implemented in pure, standalone helper functions separated from MCP parameter unwrapping.
4. Each tool module must include automated unit tests for its core transformation logic.
