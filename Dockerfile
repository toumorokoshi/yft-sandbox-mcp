# ------------------------------------------------------------------------------
# Build Stage: Compile yft-sandbox-mcp in a clean Rust environment
# ------------------------------------------------------------------------------
FROM rust:bookworm AS builder

WORKDIR /app

# Copy dependency manifests and source tree
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

# Compile the release binary
RUN cargo build --release

# ------------------------------------------------------------------------------
# Runtime Stage: Minimal, secure Debian Bookworm runner
# ------------------------------------------------------------------------------
FROM debian:bookworm-slim AS runner

# Install ca-certificates for secure outbound HTTPS connections
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Run as non-privileged system user
RUN useradd -u 10001 -m -s /bin/bash appuser

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/yft-sandbox-mcp /usr/local/bin/yft-sandbox-mcp

# Shared secret required by MCP clients (see docs/authentication.md)
COPY secrets/mcp-auth-token /etc/yft-sandbox-mcp/mcp-auth-token

USER 10001

# Cloud Run defaults: port 8080 on 0.0.0.0
ENV PORT=8080 \
    HOST=0.0.0.0 \
    MCP_TRANSPORT=http \
    MCP_AUTH_TOKEN_FILE=/etc/yft-sandbox-mcp/mcp-auth-token

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/yft-sandbox-mcp"]
