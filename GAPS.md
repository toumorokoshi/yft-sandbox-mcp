# Project Gaps & Feature Backlog

## P1: Feature Implementation

- [x] **Cloud Run Deployment & Containerization**:
  - Added multi-stage `Dockerfile` and `.dockerignore` producing a minimal, secure container image for `yft-sandbox-mcp`.
  - Added OpenTofu manifests in `tofu/` configuring GCP Cloud Run v2 service, IAM public invoker, Artifact Registry repository, and custom domain mapping for `nachochips.tsutsumi.io` (managed by external DNS).
- [ ] **AnkiConnect Flashcard Action (`anki_create_flashcard`)**:
  - Implement HTTP client for AnkiConnect (`127.0.0.1:8765`).
  - Add `anki_create_flashcard` tool with deck selection, model selection, front/back fields, and tag support.
  - Handle connection failure gracefully when Anki is not running.
  - Add integration tests mocking AnkiConnect API responses.

## P2: Enhancements & Configurations

- [ ] **Verify `tsutsumi.io` for the custom domain mapping**:
  - `google_cloud_run_domain_mapping.domain` fails with "Caller is not authorized to administer the domain". Verify `tsutsumi.io` in Google Search Console as the deploying account, then re-run `tofu apply`.
- [ ] **Allow the `*.run.app` host or document it as unsupported**:
  - MCP requests to the default Cloud Run URL return 403 because the host is not in `ALLOWED_HOSTS`. Only the custom domain works today.
- [ ] **Remote OpenTofu state**:
  - State lives in a local `tofu/terraform.tfstate`. Move it to a GCS backend so it is not lost with the workstation.

- [ ] **Move MCP auth secret out of the repository**:
  - `secrets/mcp-auth-token` is committed and baked into the container image. Move it to Google Secret Manager and mount it into Cloud Run as a file at `MCP_AUTH_TOKEN_FILE`.
- [ ] **OAuth 2.1 authorization for MCP**:
  - Replace the shared secret with the MCP authorization spec (OAuth 2.1 + protected resource metadata) if per-client identity or revocation is needed.

- [ ] **Configuration File & Environment Overrides**:
  - Allow specifying default Anki deck, model name, and custom AnkiConnect URL via config file or environment variables (`ANKI_CONNECT_URL`).
- [ ] **CLI Arguments**:
  - Add optional CLI arguments via `clap` (e.g. `--log-level`, `--version`) while preserving stdio transport for MCP.
