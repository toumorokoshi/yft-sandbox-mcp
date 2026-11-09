# Cloud Run Deployment & Custom Domain Guide (OpenTofu)

This guide provides end-to-end instructions for deploying `yft-sandbox-mcp` to **Google Cloud Run (v2)** and exposing it under the custom domain **`nachochips.tsutsumi.io`** using **OpenTofu**.

Because `tsutsumi.io` is registered and hosted externally (outside of Google Cloud DNS), OpenTofu provisions the serverless compute, container registry, IAM permissions, and Cloud Run domain mapping, while external DNS is configured via a standard CNAME record.

---

## 1. Architecture Overview

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
                  |  - $0/month idle load balancer fee   |
                  +--------------------------------------+
                                     |
                                     v
                  +--------------------------------------+
                  |        Google Cloud Run (v2)         |
                  |  - Service: yft-sandbox-mcp          |
                  |  - Scale-to-zero (0 idle cost)       |
                  |  - Axum Streamable HTTP on port 8080 |
                  |  - Probes: GET /health               |
                  +--------------------------------------+
                                     ^
                                     | Pulls image
                  +--------------------------------------+
                  |       Google Artifact Registry       |
                  |  - us-central1-docker.pkg.dev/...    |
                  +--------------------------------------+
```

---

## 2. Prerequisites

Before starting, ensure the following tools are installed:

1. **[Google Cloud CLI (`gcloud`)](https://cloud.google.com/sdk/docs/install)**
2. **[OpenTofu](https://opentofu.org/docs/intro/install/)** (version `>= 1.6.0`)
3. **[Docker](https://docs.docker.com/get-docker/)** (for building container images)
4. **Google Search Console Domain Verification**:
   - Google requires domain ownership verification before allowing Cloud Run custom domain mappings.
   - If `tsutsumi.io` (or `nachochips.tsutsumi.io`) has not been verified in your Google account:
     1. Open [Google Search Console](https://search.google.com/search-console).
     2. Add your domain property (`tsutsumi.io` or `nachochips.tsutsumi.io`).
     3. Copy the verification TXT record provided by Google.
     4. Add the TXT record to your external DNS registrar (Cloudflare, Route53, Namecheap, etc.).
     5. Click **Verify** in Google Search Console.

---

## 3. Step-by-Step Deployment

### Step 1: Authenticate with Google Cloud

Authenticate your user account and Application Default Credentials (ADC) for OpenTofu:

```bash
gcloud auth login
gcloud auth application-default login
gcloud config set project YOUR_PROJECT_ID
```

*(Replace `YOUR_PROJECT_ID` with your target GCP Project ID).*

---

### Step 2: Configure OpenTofu Variables

Navigate into the `tofu/` directory:

```bash
cd tofu
cp tofu.tfvars.example tofu.tfvars
```

Edit `tofu.tfvars` and set your GCP Project ID:

```hcl
project_id  = "YOUR_PROJECT_ID"
region      = "us-central1"
service_name = "yft-sandbox-mcp"
domain_name = "nachochips.tsutsumi.io"
```

---

### Step 3: Initialize and Apply with OpenTofu

Initialize the OpenTofu Google provider and provision the infrastructure:

```bash
tofu init
tofu apply
```

Type `yes` when prompted. OpenTofu will:
- Enable the Cloud Run API (`run.googleapis.com`) and Artifact Registry API (`artifactregistry.googleapis.com`).
- Create the Artifact Registry Docker repository (`yft-sandbox-mcp`).
- Deploy the Cloud Run v2 service with scale-to-zero (`min_instance_count = 0`).
- Configure health probes on `/health` (port 8080).
- Grant public invoker permissions (`roles/run.invoker` to `allUsers`).
- Create the Cloud Run Domain Mapping for `nachochips.tsutsumi.io`.

---

### Step 4: Configure External DNS

Log in to your external DNS provider (e.g., Cloudflare, AWS Route 53, Namecheap, Squarespace, Porkbun) and create the following record:

| Type | Host / Name | Value / Target | TTL | Proxy Status (Cloudflare) |
|---|---|---|---|---|
| **CNAME** | `nachochips` | `ghs.googlehosted.com.` | Auto / 300 | **DNS only** (grey cloud) |

> [!IMPORTANT]
> If you manage DNS with **Cloudflare**, you must set the proxy toggle to **DNS only (grey cloud)** initially so that Google Cloud can directly verify domain routing and issue the managed SSL/TLS certificate.

---

### Step 5: Build and Deploy the Container Image

From the root of the repository, build and push the release Docker image to Google Artifact Registry:

```bash
# 1. Return to the repository root
cd ..

# 2. Authenticate Docker with your Artifact Registry region
gcloud auth configure-docker us-central1-docker.pkg.dev

# 3. Build the container image using the multi-stage Dockerfile
docker build -t us-central1-docker.pkg.dev/YOUR_PROJECT_ID/yft-sandbox-mcp/server:latest .

# 4. Push the image to Artifact Registry
docker push us-central1-docker.pkg.dev/YOUR_PROJECT_ID/yft-sandbox-mcp/server:latest

# 5. Deploy the image to the Cloud Run service
gcloud run deploy yft-sandbox-mcp \
  --image us-central1-docker.pkg.dev/YOUR_PROJECT_ID/yft-sandbox-mcp/server:latest \
  --region us-central1
```

---

## 4. Verification & Testing

Once DNS propagates and Google provisions the SSL certificate (usually within 2–10 minutes):

### 1. Test the HTTP Health Check

```bash
curl -i https://nachochips.tsutsumi.io/health
```

**Expected Response**:
```http
HTTP/2 200 
content-type: text/plain; charset=utf-8
content-length: 2

OK
```

### 2. Test the MCP Streamable HTTP Protocol Handshake

Send an MCP `initialize` request to the `/mcp` endpoint:

```bash
curl -i -X POST https://nachochips.tsutsumi.io/mcp \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -H "MCP-Protocol-Version: 2024-11-05" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2024-11-05",
      "capabilities": {},
      "clientInfo": {
        "name": "curl-client",
        "version": "1.0.0"
      }
    }
  }'
```

**Expected Response**:
- HTTP `200 OK`
- Header `mcp-session-id: <UUID>`
- JSON-RPC response payload indicating available tools (`echo`, etc.).

---

## 5. Client Configuration

To connect MCP-compatible AI clients (such as Claude Desktop, Antigravity, or Cursor) to your deployed server, update your client configuration:

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

## 6. Maintenance & Updates

### Re-deploying Code Changes
When updating tools or server logic:

```bash
# 1. Ensure local CI checks pass
just ci

# 2. Build and push new container tag
docker build -t us-central1-docker.pkg.dev/YOUR_PROJECT_ID/yft-sandbox-mcp/server:latest .
docker push us-central1-docker.pkg.dev/YOUR_PROJECT_ID/yft-sandbox-mcp/server:latest

# 3. Redeploy to Cloud Run
gcloud run deploy yft-sandbox-mcp \
  --image us-central1-docker.pkg.dev/YOUR_PROJECT_ID/yft-sandbox-mcp/server:latest \
  --region us-central1
```

### Teardown
To destroy all GCP resources and stop any potential resource consumption:

```bash
cd tofu
tofu destroy
```
