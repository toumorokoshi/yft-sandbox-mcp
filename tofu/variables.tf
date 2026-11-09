variable "project_id" {
  type        = string
  description = "The GCP project ID to deploy resources into."
}

variable "region" {
  type        = string
  default     = "us-central1"
  description = "The GCP region for Cloud Run and Artifact Registry."
}

variable "service_name" {
  type        = string
  default     = "yft-sandbox-mcp"
  description = "Name of the Cloud Run v2 service."
}

variable "domain_name" {
  type        = string
  default     = "nachochips.tsutsumi.io"
  description = "The custom domain name to map to the Cloud Run service (e.g., nachochips.tsutsumi.io)."
}

variable "container_image" {
  type        = string
  default     = ""
  description = "The container image to run. If left empty, defaults to the latest image tag in the created Artifact Registry repository."
}

variable "artifact_registry_repo_id" {
  type        = string
  default     = "yft-sandbox-mcp"
  description = "The ID of the Artifact Registry Docker repository."
}

variable "min_instance_count" {
  type        = number
  default     = 0
  description = "Minimum number of Cloud Run instances (0 enables scale-to-zero for zero idle cost)."
}

variable "max_instance_count" {
  type        = number
  default     = 10
  description = "Maximum number of Cloud Run instances."
}

variable "cpu_limit" {
  type        = string
  default     = "1"
  description = "CPU limit for the Cloud Run instance (e.g., '1' or '2')."
}

variable "memory_limit" {
  type        = string
  default     = "512Mi"
  description = "Memory limit for the Cloud Run instance (e.g., '256Mi', '512Mi', '1Gi')."
}
