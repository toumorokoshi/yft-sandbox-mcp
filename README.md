# yft-sandbox-mcp

An extensible [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) server written in Rust, designed to host personal actions and productivity automations (such as automated flashcard creation in Anki, note syncing, and custom developer workflows).

## Features

- **Built on Official `rmcp` SDK**: Uses the official Model Context Protocol Rust SDK (`rmcp`) with async stdio transport.
- **Clean Transport Separation**: Stdio JSON-RPC protocol transport on standard I/O; all server logs and diagnostic traces are strictly routed to `stderr` to prevent protocol stream corruption.
- **Modular Action Architecture**: Functional business logic is isolated into pure helper functions and tested independently from MCP serialization.
- **Local CI & Zero Cost**: To avoid unnecessary GitHub Actions costs on private repositories, CI is validated locally before every push via Git pre-push hooks and `just ci`.

## Included MCP Tools

| Tool | Description | Status |
|------|-------------|--------|
| `echo` | Example MCP action that echoes back an input message. | Ready |
| `anki_create_flashcard` | Create flashcards in Anki via AnkiConnect. | Planned (see `GAPS.md` and `design/anki-flashcard-tool.md`) |

## Quickstart

### Prerequisites

- [Rust & Cargo](https://rustup.rs/) (edition 2024 / 1.88+)
- [Just](https://github.com/casey/just) command runner

### Setup & Install Pre-Push Hooks

Initialize the repository and configure the local pre-push git hook:

```bash
just install-hooks
```

This sets `git config core.hooksPath .githooks`, ensuring all commits are verified against the local CI suite before pushing.

### Build and Run

Build the server binary:

```bash
just build
```

Run the server directly (communicates over stdio):

```bash
cargo run
```

## Client Configuration

To register this server with an MCP client (such as Claude Desktop, Antigravity, or Cursor), add the following to your MCP client configuration:

```json
{
  "mcpServers": {
    "yft-sandbox": {
      "command": "/absolute/path/to/yft-sandbox-mcp/target/debug/yft-sandbox-mcp"
    }
  }
}
```

Or using `cargo run`:

```json
{
  "mcpServers": {
    "yft-sandbox": {
      "command": "cargo",
      "args": ["run", "--quiet", "--manifest-path", "/absolute/path/to/yft-sandbox-mcp/Cargo.toml"]
    }
  }
}
```

## Development & Local CI

Available commands via `just`:

- `just ci`: Runs formatting checks, linter (`clippy` with `-D warnings`), and all unit & integration tests.
- `just lint`: Runs `cargo clippy --all-targets --all-features -- -D warnings`.
- `just format`: Checks code formatting with `cargo fmt --all -- --check`.
- `just fix`: Automatically formats code and applies safe clippy suggestions.
- `just test`: Runs all unit and integration tests.
- `just install-hooks`: Configures Git to use `.githooks/` for pre-push validation.

## Continuous Integration Policy

GitHub Actions CI is disabled for this private repository to avoid incurred costs. All agents and contributors must run `just ci` locally before pushing. The Git pre-push hook automatically enforces this check on `git push`.
