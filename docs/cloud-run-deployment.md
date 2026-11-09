# Cloud Run Deployment Guide

This document describes how `yft-sandbox-mcp` is hosted on Google Cloud Run and exposed via the custom domain `nachochips.tsutsumi.io`.

---

## Architecture

```
                          nachochips.tsutsumi.io
                                     |
                       (CNAME in External DNS Provider)
                                     |
                                     v
                           ghs.googlehosted.com
                                     |
                                     v
                  +--------------------------------------+
                  |      Cloud Run Domain Mapping        |
                  |  - Free Google-managed TLS cert      |
                  |  - Maps to yft-sandbox-mcp service   |
                  +--------------------------------------+
                                     |
                                     v
                  +--------------------------------------+
                  |          Google Cloud Run            |
                  |  - Service: yft-sandbox-mcp          |
                  |  - Scale-to-zero (0 idle cost)       |
                  |  - Port 8080 (Axum HTTP server)      |
                  |  - Probes: GET /health               |
                  +--------------------------------------+
```

---

## Infrastructure Management with Terraform

All infrastructure resources are declared as code in `terraform/`:

- **Cloud Run (v2)**: Configured in `terraform/main.tf` with automatic scaling from 0 to 10 instances.
- **Artifact Registry**: Docker repository storing the multi-stage release image.
- **Public Ingress IAM**: `roles/run.invoker` granted to `allUsers`.
- **Domain Mapping**: `google_cloud_run_domain_mapping` mapping `nachochips.tsutsumi.io` to the service.

---

## External DNS Configuration

Since `tsutsumi.io` is registered and hosted externally (not in Google Cloud DNS), DNS records are configured in your external DNS provider:

```
Type:  CNAME
Name:  nachochips
Value: ghs.googlehosted.com.
```

Google will automatically issue a managed SSL/TLS certificate once the CNAME resolves to Google Cloud.

---

## Container Build & Runtime

The container is built via the multi-stage [Dockerfile](file:///home/yusuke/workspace/yft-sandbox-mcp/Dockerfile):
- **Builder**: Official `rust:bookworm` compiling release binary with full optimizations.
- **Runner**: Minimal `debian:bookworm-slim` running as an unprivileged user (`appuser`, UID 10001).
- **Environment**:
  - `PORT`: Automatically populated by Cloud Run (default `8080`).
  - `HOST`: `0.0.0.0`.
  - `ALLOWED_HOSTS`: `nachochips.tsutsumi.io,localhost,127.0.0.1,0.0.0.0`.
  - `MCP_TRANSPORT`: `http`.
