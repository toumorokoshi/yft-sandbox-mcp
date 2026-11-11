# Project Gaps & Feature Backlog

## P1: Feature Implementation

- [x] **Cloud Run Deployment & Containerization**:
  - Added multi-stage `Dockerfile` and `.dockerignore` producing a minimal, secure container image for `yft-sandbox-mcp`.
  - Added OpenTofu manifests in `tofu/` configuring GCP Cloud Run v2 service, IAM public invoker, Artifact Registry repository, and custom domain mapping for `nachochips.tsutsumi.io` (managed by external DNS).
- [x] **Anki Flashcard Bundling Action (`bundle-anki-flashcard`)**:
  - Implemented in-memory `.apkg` generation via `genanki-rs` and `base64`.
  - Added `bundle-anki-flashcard` tool accepting `deck_name` (optional, defaults to `"Default"`), `question`, and `answer`.
  - Returns binary `.apkg` archive via MCP embedded resource blob (`ResourceContents::BlobResourceContents`).
  - Added comprehensive unit tests and MCP integration test.

## P2: Enhancements & Configurations

- [ ] **Batch Flashcard Bundling**:
  - Allow passing an array of questions/answers to bundle multiple flashcards into a single `.apkg` package.
- [ ] **Anki Tags & Cloze Deletions**:
  - Add optional `tags` array and Cloze deletion model support in `bundle-anki-flashcard`.
- [ ] **Verify `tsutsumi.io` for the custom domain mapping**:
  - `google_cloud_run_domain_mapping.domain` fails with "Caller is not authorized to administer the domain". Verify `tsutsumi.io` in Google Search Console as the deploying account, then re-run `tofu apply`.
- [ ] **Remote OpenTofu state**:
  - State lives in a local `tofu/terraform.tfstate`. Move it to a GCS backend so it is not lost with the workstation.
- [ ] **Move MCP auth secret out of the repository**:
  - `secrets/mcp-auth-token` is committed and baked into the container image. Move it to Google Secret Manager and mount it into Cloud Run as a file at `MCP_AUTH_TOKEN_FILE`.
- [ ] **OAuth 2.1 authorization for MCP**:
  - Replace the shared secret with the MCP authorization spec (OAuth 2.1 + protected resource metadata) if per-client identity or revocation is needed.
- [ ] **CLI Arguments**:
  - Add optional CLI arguments via `clap` (e.g. `--log-level`, `--version`) while preserving stdio transport for MCP.
