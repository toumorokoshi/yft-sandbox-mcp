# Architecture

## High-Level Overview

`yft-sandbox-mcp` is an extensible Model Context Protocol (MCP) server designed to run locally.

```
+------------------------------------------------+
|                   AI Client                    |
|   (e.g., Claude Desktop, Antigravity, Cursor)  |
+------------------------------------------------+
                       |
             JSON-RPC over stdio
                       |
                       v
+------------------------------------------------+
|               yft-sandbox-mcp                  |
|                                                |
|  +------------------------------------------+  |
|  |             SandboxServer                |  |
|  |         (Tool Router & Handler)          |  |
|  +------------------------------------------+  |
|             |                       |          |
|             v                       v          |
|    +-----------------+     +-----------------+ |
|    |    echo tool    |     | anki tool (TBD) | |
|    +-----------------+     +-----------------+ |
|                                     |          |
+-------------------------------------|----------+
                                      v
                             HTTP (AnkiConnect)
                                      |
                                      v
                             +-----------------+
                             |  Anki Desktop   |
                             +-----------------+
```

## Module Structure

- `src/lib.rs`: Library entrypoint, exposing `SandboxServer` and `tools` module.
- `src/main.rs`: CLI binary entrypoint, initializing logging to stderr and launching the stdio transport.
- `src/server.rs`: Server struct, tool registration macros, and MCP capabilities.
- `src/tools/`: Submodules containing individual tools and their isolated logic.
