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
