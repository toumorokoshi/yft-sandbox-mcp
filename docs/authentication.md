# Authentication

The HTTP transport requires a shared secret on every MCP request. Health
checks (`/health`, `/livez`) stay unauthenticated so Cloud Run probes work.

## Presenting the secret

Clients may send the secret in either of two ways:

1. **`Authorization: Bearer <token>` header (preferred).** This is the
   mechanism the MCP authorization spec uses for HTTP transports.
2. **`?token=<token>` query parameter.** A fallback for clients that only
   accept a URL and cannot set headers.

If an `Authorization: Bearer` header is present it takes precedence over the
query parameter. Requests without a valid token receive `401 Unauthorized`
with a `WWW-Authenticate: Bearer` header.

> Query strings can end up in request logs (including Cloud Run request logs).
> Prefer the header when the client supports it.

## Where the secret lives

The secret is a single line in [`secrets/mcp-auth-token`](../secrets/mcp-auth-token).
On startup the HTTP server reads the file named by `MCP_AUTH_TOKEN_FILE`,
defaulting to `secrets/mcp-auth-token` relative to the working directory. The
server refuses to start in HTTP mode if the file is missing or empty.

The `Dockerfile` copies the file to `/etc/yft-sandbox-mcp/mcp-auth-token` and
sets `MCP_AUTH_TOKEN_FILE` to that path.

Stdio mode does not use the secret.

## Rotating the secret

```bash
openssl rand -hex 32 > secrets/mcp-auth-token
```

Then rebuild and redeploy the container, and update every client config.

## Client configuration

Header (preferred):

```json
{
  "mcpServers": {
    "yft-sandbox": {
      "url": "https://nachochips.tsutsumi.io/mcp",
      "headers": {
        "Authorization": "Bearer <token>"
      }
    }
  }
}
```

URL only:

```json
{
  "mcpServers": {
    "yft-sandbox": {
      "url": "https://nachochips.tsutsumi.io/mcp?token=<token>"
    }
  }
}
```

Test with curl:

```bash
curl -i -X POST https://nachochips.tsutsumi.io/mcp \
  -H "Authorization: Bearer $(cat secrets/mcp-auth-token)" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"curl","version":"1.0"}}}'
```
