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
- **Decision**: Support host validation with defaults including `nachochips.tsutsumi.io`, `localhost`, and loopback interfaces, configurable via `ALLOWED_HOSTS`.
- **Context**: The MCP Streamable HTTP specification rejects unallowed Host headers to guard against DNS rebinding attacks against local services.
- **Rationale**: Ensures the service is secure by default while functioning immediately upon deployment to `nachochips.tsutsumi.io`.

## 2026-11-08: Terraform Manifests & External Custom Domain (nachochips.tsutsumi.io)

### 1. Zero-Cost Infrastructure with Cloud Run v2 and Domain Mapping
- **Decision**: Declare all GCP infrastructure in `terraform/` using `google_cloud_run_v2_service`, `google_cloud_run_domain_mapping`, and `google_artifact_registry_repository`.
- **Context**: The server needs to be deployed to Cloud Run and accessible at `nachochips.tsutsumi.io`.
- **Rationale**: Cloud Run Domain Mapping (`google_cloud_run_domain_mapping`) provisions free Google-managed SSL/TLS certificates and incurs $0/month in idle load balancer fees, pairing with Cloud Run's scale-to-zero (`min_instance_count = 0`).

### 2. External DNS Integration for `tsutsumi.io`
- **Decision**: Do not manage Cloud DNS records via Terraform since the domain `tsutsumi.io` is managed externally. Instead, export structured outputs in Terraform detailing the exact CNAME record (`nachochips` -> `ghs.googlehosted.com.`) for external DNS configuration.
- **Context**: User confirmed `tsutsumi.io` is not managed by GCP.
- **Rationale**: Keeps Terraform isolated to GCP resources while giving the operator clear, copy-paste DNS values for their external DNS provider.

### 3. Consolidated Cloud Run Deployment Documentation
- **Decision**: Provide complete step-by-step instructions in `docs/cloud-run-deployment.md` covering prerequisites, Search Console domain verification, Terraform setup, container builds, and DNS configuration.
- **Context**: Operator requested explicit deployment and DNS setup instructions in the `docs/` directory.
- **Rationale**: Centralizing comprehensive deployment instructions in `docs/` guarantees operators have a single reference guide for deployment lifecycle and maintenance.

## 2026-11-09: Transition Infrastructure as Code to OpenTofu

### 1. OpenTofu Adoption for GCP Infrastructure
- **Decision**: Migrate IaC manifests exclusively to OpenTofu (`tofu/`), remove legacy `terraform/` directory/symlinks, update version constraint to `>= 1.6.0`, adopt `tofu.tfvars.example`, and update CLI workflows to `tofu`.
- **Context**: Operator requested adapting the infrastructure files to OpenTofu and removing the legacy `terraform` directory.
- **Rationale**: OpenTofu is a truly open-source (MPL-2.0, Linux Foundation) drop-in replacement with full compatibility with the Google provider (`hashicorp/google`) and provides a clean open-source foundation without BSL license restrictions.

## 2026-11-09: Shared-Secret Authentication for the HTTP Transport

### 1. Bearer Header with Query Parameter Fallback
- **Decision**: Require a shared secret on all MCP routes, accepted as `Authorization: Bearer <token>` or `?token=<token>`. Health checks stay open.
- **Context**: The Cloud Run service is public (`allUsers` invoker), so anyone could call MCP tools.
- **Rationale**: The MCP authorization spec uses Bearer tokens on HTTP transports, so the header is the standard path. Some clients can only be configured with a URL, so the query parameter covers them. Full OAuth 2.1 is heavier than a single-user server needs today.

### 2. Secret Loaded from a File in the Repository
- **Decision**: Store the secret in `secrets/mcp-auth-token`, copy it into the image, and load it at startup from `MCP_AUTH_TOKEN_FILE`. Fail to start in HTTP mode if it is missing.
- **Context**: Operator asked for the secret to live in the repository for now.
- **Rationale**: Loading from a file path means moving to a Secret Manager volume mount later only changes deployment, not code. Failing closed avoids accidentally serving without auth.
