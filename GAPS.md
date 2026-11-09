# Project Gaps & Feature Backlog

## P1: Feature Implementation

- [ ] **Cloud Run Deployment & Containerization**:
  - Add multi-stage `Dockerfile` producing a minimal, secure container image for `yft-sandbox-mcp`.
  - Add Terraform manifests in `terraform/` configuring GCP Cloud Run v2 service, IAM public invoker, and custom domain mapping for `nachochips.yusuketsutsumi.com`.
- [ ] **AnkiConnect Flashcard Action (`anki_create_flashcard`)**:
  - Implement HTTP client for AnkiConnect (`127.0.0.1:8765`).
  - Add `anki_create_flashcard` tool with deck selection, model selection, front/back fields, and tag support.
  - Handle connection failure gracefully when Anki is not running.
  - Add integration tests mocking AnkiConnect API responses.

## P2: Enhancements & Configurations

- [ ] **Configuration File & Environment Overrides**:
  - Allow specifying default Anki deck, model name, and custom AnkiConnect URL via config file or environment variables (`ANKI_CONNECT_URL`).
- [ ] **CLI Arguments**:
  - Add optional CLI arguments via `clap` (e.g. `--log-level`, `--version`) while preserving stdio transport for MCP.
