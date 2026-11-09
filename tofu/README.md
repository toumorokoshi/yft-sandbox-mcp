# OpenTofu Deployment for `yft-sandbox-mcp` on Google Cloud Run

This directory contains OpenTofu manifests to deploy `yft-sandbox-mcp` onto **Google Cloud Run (v2)** mapped to the custom domain **`nachochips.tsutsumi.io`**.

Because the domain `tsutsumi.io` is managed externally (outside GCP), OpenTofu outputs the required DNS CNAME records to add in your DNS provider without needing GCP Cloud DNS.

---

## Architecture & Features

- **Google Cloud Run v2**: Serverless container execution with automated scale-to-zero (`min_instance_count = 0`) to guarantee **$0 idle infrastructure cost**.
- **Native Domain Mapping**: Uses `google_cloud_run_domain_mapping` to automatically provision and renew Google-managed SSL/TLS certificates at no extra cost (no expensive load balancer idle fees).
- **Google Artifact Registry**: Dedicated Docker repository for holding production container images.
- **Health Checks & Probes**: Automated startup and liveness probes hitting `/health` on port 8080.
- **External DNS Compatibility**: Clean outputs providing the exact CNAME record (`nachochips` -> `ghs.googlehosted.com.`) for your external DNS registrar.
- **OpenTofu**: Uses open-source OpenTofu (`tofu`) licensed under MPL-2.0.

---

## Prerequisites

1. **[Google Cloud CLI (`gcloud`)](https://cloud.google.com/sdk/docs/install)** installed and authenticated.
2. **[OpenTofu](https://opentofu.org/docs/intro/install/)** (version `>= 1.6.0`) installed:
   ```bash
   # On Debian/Ubuntu:
   curl --proto '=https' --tlsv1.2 -fsSL https://get.opentofu.org/install-opentofu.sh -o install-opentofu.sh
   chmod +x install-opentofu.sh
   ./install-opentofu.sh --install-method deb
   rm -f install-opentofu.sh

   # On macOS via Homebrew:
   brew install opentofu
   ```
3. **[Docker](https://docs.docker.com/get-docker/)** installed for building container images.
4. **Domain Ownership Verification**:
   - Google requires domain ownership verification before mapping a custom domain to Cloud Run.
   - If you have not verified `tsutsumi.io` or `nachochips.tsutsumi.io` in Google Webmaster Central / Search Console:
     1. Visit [Google Search Console](https://search.google.com/search-console).
     2. Add your domain property (`tsutsumi.io` or `nachochips.tsutsumi.io`).
     3. Add the TXT verification record to your external DNS provider.
     4. Click **Verify**.

---

## Step-by-Step Deployment Guide

### 1. Authenticate with Google Cloud

Ensure your local environment has application default credentials configured:

```bash
gcloud auth login
gcloud auth application-default login
gcloud config set project YOUR_PROJECT_ID
```

### 2. Configure OpenTofu Variables

Create a `tofu.tfvars` file from the provided example:

```bash
cp tofu.tfvars.example tofu.tfvars
```

Edit `tofu.tfvars` and set your GCP Project ID:

```hcl
project_id  = "YOUR_PROJECT_ID"
region      = "us-central1"
domain_name = "nachochips.tsutsumi.io"
```

### 3. Initialize & Apply with OpenTofu

```bash
tofu init
tofu apply
```

Review the plan and type `yes` to confirm. OpenTofu will:
- Enable the Cloud Run and Artifact Registry APIs.
- Create the Artifact Registry Docker repository.
- Deploy the Cloud Run service.
- Grant public invoker permissions (`roles/run.invoker` to `allUsers`).
- Create the custom domain mapping for `nachochips.tsutsumi.io`.

### 4. Build and Push the Container Image

After applying OpenTofu, authenticate Docker with Google Artifact Registry and build/push the image:

```bash
# Authenticate Docker with your Artifact Registry region
gcloud auth configure-docker us-central1-docker.pkg.dev

# From the root of the repository:
docker build -t us-central1-docker.pkg.dev/YOUR_PROJECT_ID/yft-sandbox-mcp/server:latest .
docker push us-central1-docker.pkg.dev/YOUR_PROJECT_ID/yft-sandbox-mcp/server:latest

# Update Cloud Run to use the newly pushed image
gcloud run deploy yft-sandbox-mcp \
  --image us-central1-docker.pkg.dev/YOUR_PROJECT_ID/yft-sandbox-mcp/server:latest \
  --region us-central1
```

*(Note: OpenTofu outputs these exact commands tailored to your project ID and region upon completion.)*

### 5. Configure Your External DNS Provider

Check OpenTofu outputs for DNS records:

```bash
tofu output dns_records_to_configure
```

Log in to your external DNS provider (e.g., Cloudflare, Route53, Namecheap, Porkbun) and create the following record:

| Record Type | Host / Name | Target / Value | TTL |
|---|---|---|---|
| **CNAME** | `nachochips` | `ghs.googlehosted.com.` | Auto / 300 |

> [!NOTE]
> If you are using Cloudflare, set the proxy status to **DNS only (grey cloud)** initially so Google can issue and verify the SSL/TLS certificate directly.

### 6. Verify Deployment

Once DNS propagates (usually 1–5 minutes):

```bash
# 1. Verify healthcheck endpoint
curl -s https://nachochips.tsutsumi.io/health
# Expected output: OK

# 2. Test MCP JSON-RPC protocol endpoint
curl -s -X POST https://nachochips.tsutsumi.io/mcp \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -H "MCP-Protocol-Version: 2024-11-05" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0"}}}'
```

---

## Client Configuration

To configure your AI client (e.g. Claude Desktop, Antigravity, Cursor) to use your deployed Cloud Run server, add the following to your MCP client config:

```json
{
  "mcpServers": {
    "yft-sandbox": {
      "url": "https://nachochips.tsutsumi.io/mcp"
    }
  }
}
```

---

## Teardown / Cleanup

To delete all deployed resources:

```bash
tofu destroy
```
