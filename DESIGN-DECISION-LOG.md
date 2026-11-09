# Design Decision Log

## 2026-11-08: Repository Bootstrap & Local CI Architecture

### 1. Model Context Protocol Framework Selection
- **Decision**: Adopt the official `rmcp` Rust SDK (v3.9.0) with stdio transport.
- **Context**: The user requested a Rust MCP server in this repository for personal actions (such as creating flashcards in Anki), starting with a bootstrapped repository and an example MCP command.
- **Rationale**: `rmcp` is the official Model Context Protocol Rust SDK. It offers declarative `#[tool_router]` and `#[tool]` macros, automatic JSON Schema generation via `schemars`, and robust async stdio transport.

### 2. Standard I/O Protocol and Log Routing
- **Decision**: Route all `tracing` output strictly to `stderr` and keep `stdout` exclusively for JSON-RPC messages.
- **Context**: MCP servers communicating over stdio rely on raw newline-delimited JSON-RPC messages on `stdout`. Any accidental print to `stdout` breaks client protocol framing.
- **Rationale**: Directing `tracing_subscriber::fmt().with_writer(std::io::stderr)` guarantees that logging never pollutes `stdout`.

### 3. Local CI & Git Pre-Push Hook Policy
- **Decision**: Disable remote GitHub Actions CI to prevent recurring runner costs on private repositories. Enforce local CI via a Git `pre-push` hook executing `just ci`.
- **Context**: The user specified that GitHub CI for private repositories costs money, and instructed to modify `AGENTS.md` to run CI locally and add a pre-push hook.
- **Rationale**: `just ci` runs formatting checks (`cargo fmt`), strict clippy analysis (`-D warnings`), and the full test suite locally. The `.githooks/pre-push` hook guarantees that no commit can be pushed without passing local CI first.

### 4. Functional Separation of MCP Actions
- **Decision**: Separate the functional core logic of each action into pure functions from the MCP parameter extraction and handler routing.
- **Context**: Follow codebase tenets emphasizing functional programming, testability, and modularity.
- **Rationale**: Pure functions (such as `format_echo`) can be unit tested without async runtime or serialization overhead.

## 2026-11-08: Streamable HTTP Transport & Cloud Run Integration

### 1. HTTP Server Adoption for Cloud Run Hosting
- **Decision**: Convert the server's default runtime to a Streamable HTTP server hosted on `axum` with `rmcp::transport::streamable_http_server`, while maintaining backward compatibility with `stdio` via `MCP_TRANSPORT=stdio`.
- **Context**: Serving `yft-sandbox-mcp` on Google Cloud Run at `nachochips.yusuketsutsumi.com` requires an HTTP service listening on `$PORT` (default `8080`) that responds to HTTP health check probes and handles standard MCP JSON-RPC over HTTP.
- **Rationale**: `rmcp` v3.9.0 provides native `transport-streamable-http-server` using Tower and Axum. It natively implements session management, SSE event streaming, and JSON-RPC dispatch conforming to the Model Context Protocol 2024-11-05 Streamable HTTP transport specification.

### 2. Cloud Health Check Endpoints
- **Decision**: Expose `GET /health` and `GET /livez` returning HTTP 200 OK.
- **Context**: Cloud Run requires health checks for container startup and liveness probes.
- **Rationale**: Prevents probe failures and container restarts without interfering with MCP protocol routes at `/mcp` or `/`.

### 3. Allowed Hosts Defense for Custom Domain
- **Decision**: Support host validation with defaults including `nachochips.yusuketsutsumi.com`, `localhost`, and loopback interfaces, configurable via `ALLOWED_HOSTS`.
- **Context**: The MCP Streamable HTTP specification rejects unallowed Host headers to guard against DNS rebinding attacks against local services.
- **Rationale**: Ensures the service is secure by default while functioning immediately upon deployment to `nachochips.yusuketsutsumi.com`.
