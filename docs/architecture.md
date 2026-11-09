# Architecture

## High-Level Overview

`yft-sandbox-mcp` is an extensible Model Context Protocol (MCP) server designed to run either in cloud environments (such as Google Cloud Run over HTTP) or locally (over stdio).

```
+------------------------------------------------+      +------------------------------------------------+
|             Remote AI / Web Client             |      |             Local Desktop Client               |
|           (e.g., nachochips.tsutsumi.io)           |      |       (e.g., Claude Desktop, Antigravity)      |
+------------------------------------------------+      +------------------------------------------------+
                       |                                                        |
             HTTP / SSE (/mcp)                                         JSON-RPC over stdio
                       |                                                        |
                       v                                                        v
+------------------------------------------------+      +------------------------------------------------+
|               Axum HTTP Router                 |      |              rmcp Stdio Transport              |
|  - GET /health, GET /livez                     |      |               (MCP_TRANSPORT=stdio)            |
|  - StreamableHttpService (/mcp)                |      |                                                |
+------------------------------------------------+      +------------------------------------------------+
                       \                                                       /
                        \                                                     /
                         v                                                   v
                      +---------------------------------------------------------+
                      |                      SandboxServer                      |
                      |                 (Tool Router & Handler)                 |
                      +---------------------------------------------------------+
                                     |                           |
                                     v                           v
                            +-----------------+         +-----------------+
                            |    echo tool    |         | anki tool (TBD) |
                            +-----------------+         +-----------------+
                                                                 |
                                                                 v
                                                         HTTP (AnkiConnect)
                                                                 |
                                                                 v
                                                         +-----------------+
                                                         |  Anki Desktop   |
                                                         +-----------------+
```

## Module Structure

- `src/lib.rs`: Library entrypoint, exposing `SandboxServer`, HTTP router utilities, and tools.
- `src/http.rs`: HTTP router (`build_router`), health checks (`/health`, `/livez`), and server lifecycle (`run_http_server`) integrating `rmcp::transport::streamable_http_server` with Axum.
- `src/main.rs`: Binary entrypoint. Supports HTTP server mode by default (listening on `$PORT` / `0.0.0.0`) and optional stdio mode when `MCP_TRANSPORT=stdio`.
- `src/server.rs`: Server struct, tool registration macros, and MCP capabilities.
- `src/tools/`: Submodules containing individual tools and their isolated functional logic.
