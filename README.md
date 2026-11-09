# yft-sandbox-mcp

An extensible [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) server written in Rust, designed to host personal actions and productivity automations (such as automated flashcard creation in Anki, note syncing, and custom developer workflows).

## Features

- **Dual Transport (HTTP & Stdio)**: Streamable HTTP transport hosted on Axum (Cloud Run ready on `$PORT`) with built-in health check probes (`/health`, `/livez`) and backward compatibility for stdio transport via `MCP_TRANSPORT=stdio`.
- **Clean Protocol Separation**: All diagnostic and runtime traces are routed to `stderr` via `tracing_subscriber`, keeping JSON-RPC protocol framing pristine.
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

Run the server as an HTTP server (default, Cloud Run ready on `0.0.0.0:8080`):

```bash
cargo run
```

Run the server with custom port or allowed hosts:

```bash
PORT=8080 ALLOWED_HOSTS="nachochips.tsutsumi.io,localhost" cargo run
```

Run the server in local stdio mode (for local desktop MCP clients):

```bash
MCP_TRANSPORT=stdio cargo run
```

## Client Configuration

### Connecting via Streamable HTTP (Remote / Cloud Run)

To connect to this server over HTTP (e.g. deployed on Cloud Run at `nachochips.tsutsumi.io` or running locally on port 8080):

```json
{
  "mcpServers": {
    "yft-sandbox": {
      "url": "https://nachochips.tsutsumi.io/mcp"
    }
  }
}
```

### Connecting via Stdio (Local Desktop)

To register this server with a local MCP client communicating over standard input/output:

```json
{
  "mcpServers": {
    "yft-sandbox": {
      "command": "/absolute/path/to/yft-sandbox-mcp/target/debug/yft-sandbox-mcp",
      "env": {
        "MCP_TRANSPORT": "stdio"
      }
    }
  }
}
```

## Cloud Run Deployment

To deploy `yft-sandbox-mcp` to Google Cloud Run and serve it at `nachochips.tsutsumi.io` with zero idle infrastructure costs, refer to:
- [Cloud Run Deployment Guide](docs/cloud-run-deployment.md)
- [OpenTofu Manifests](tofu/README.md)

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
