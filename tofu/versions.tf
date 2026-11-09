terraform {
  # OpenTofu version constraint (1.6.0+ is open source under MPL-2.0)
  required_version = ">= 1.6.0"

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = ">= 5.0.0, < 7.0.0"
    }
  }
}
