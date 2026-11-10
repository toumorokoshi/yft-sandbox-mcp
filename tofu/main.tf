provider "google" {
  project = var.project_id
  region  = var.region
}

# ------------------------------------------------------------------------------
# Enable Required GCP APIs
# ------------------------------------------------------------------------------
resource "google_project_service" "run" {
  service            = "run.googleapis.com"
  disable_on_destroy = false
}

resource "google_project_service" "artifactregistry" {
  service            = "artifactregistry.googleapis.com"
  disable_on_destroy = false
}

# ------------------------------------------------------------------------------
# Artifact Registry: Docker Repository
# ------------------------------------------------------------------------------
resource "google_artifact_registry_repository" "docker_repo" {
  repository_id = var.artifact_registry_repo_id
  format        = "DOCKER"
  location      = var.region
  description   = "Docker container image repository for yft-sandbox-mcp"

  depends_on = [google_project_service.artifactregistry]
}

locals {
  container_image = var.container_image != "" ? var.container_image : "${var.region}-docker.pkg.dev/${var.project_id}/${google_artifact_registry_repository.docker_repo.repository_id}/server:latest"
}

# ------------------------------------------------------------------------------
# Cloud Run (v2) Service
# ------------------------------------------------------------------------------
resource "google_cloud_run_v2_service" "mcp_service" {
  name     = var.service_name
  location = var.region
  ingress  = "INGRESS_TRAFFIC_ALL"

  template {
    scaling {
      min_instance_count = var.min_instance_count
      max_instance_count = var.max_instance_count
    }

    containers {
      image = local.container_image

      ports {
        container_port = 8080
      }

      resources {
        limits = {
          cpu    = var.cpu_limit
          memory = var.memory_limit
        }
      }

      # PORT is reserved: Cloud Run injects it from container_port above.
      env {
        name  = "HOST"
        value = "0.0.0.0"
      }

      env {
        name  = "ALLOWED_HOSTS"
        value = "${var.domain_name},localhost,127.0.0.1,0.0.0.0"
      }

      env {
        name  = "MCP_TRANSPORT"
        value = "http"
      }

      startup_probe {
        http_get {
          path = "/health"
          port = 8080
        }
        initial_delay_seconds = 0
        period_seconds        = 3
        failure_threshold     = 3
      }

      liveness_probe {
        http_get {
          path = "/health"
          port = 8080
        }
        period_seconds    = 10
        failure_threshold = 3
      }
    }
  }

  depends_on = [google_project_service.run]
}

# ------------------------------------------------------------------------------
# IAM: Public Ingress Access for MCP Clients
# ------------------------------------------------------------------------------
resource "google_cloud_run_v2_service_iam_member" "public_access" {
  project  = var.project_id
  location = google_cloud_run_v2_service.mcp_service.location
  name     = google_cloud_run_v2_service.mcp_service.name
  role     = "roles/run.invoker"
  member   = "allUsers"
}

# ------------------------------------------------------------------------------
# Custom Domain Mapping: nachochips.tsutsumi.io -> Cloud Run Service
# ------------------------------------------------------------------------------
resource "google_cloud_run_domain_mapping" "domain" {
  location = var.region
  name     = var.domain_name

  metadata {
    namespace = var.project_id
  }

  spec {
    route_name = google_cloud_run_v2_service.mcp_service.name
  }

  depends_on = [google_cloud_run_v2_service.mcp_service]
}
