terraform {
  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 5.0"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.5"
    }
  }
  required_version = ">= 1.0"
}

provider "google" {
  project = var.project_id
  region  = var.region
}

# Random string for uniqueness
resource "random_id" "suffix" {
  byte_length = 4
}

# Pub/Sub topic
resource "google_pubsub_topic" "gke_notifications" {
  name = "gke-cluster-notifications-${random_id.suffix.hex}"
}

# Cloud Run service
resource "google_cloud_run_v2_service" "gke_notifications_service" {
  name     = "gke-cluster-notifications-${random_id.suffix.hex}"
  location = var.region

  template {
    containers {
      image = "us.gcr.io/${var.project_id}/gke-cluster-notifications:latest"

      env {
        name  = "JSON_LOG"
        value = "true"
      }
      env {
        name  = "GCP_PROJECT"
        value = var.project_id
      }
      env {
        name  = "SLACK_WEBHOOK"
        value = var.slack_webhook_url
      }
      # Configure to send logs to STDOUT where the OTEL collector can pick them up
      env {
        name  = "RUST_LOG"
        value = "info"
      }

      # Configure volume mounts for the application container
      volume_mounts {
        name       = "shared-logs"
        mount_path = "/var/log"
      }
    }

    # OpenTelemetry Collector sidecar container
    containers {
      name  = "otel-collector"
      image = "otel/opentelemetry-collector-contrib:latest"
      
      # Pass Loki endpoint and credentials as environment variables
      env {
        name  = "LOKI_ENDPOINT"
        value = var.loki_endpoint
      }
      env {
        name  = "LOKI_USERNAME"
        value = var.loki_username
      }
      env {
        name  = "LOKI_PASSWORD"
        value = var.loki_password
      }

      # Mount the configuration file
      volume_mounts {
        name       = "otel-config"
        mount_path = "/etc/otelcol-contrib"
      }
      
      # Mount shared logs volume
      volume_mounts {
        name       = "shared-logs"
        mount_path = "/var/log"
      }

      # The command to run the collector (optional, defaults to the image's entrypoint)
      command = ["/otelcol-contrib"]
      args = ["--config=/etc/otelcol-contrib/config.yaml"]
    }

    # Define volumes
    volumes {
      name = "otel-config"
      secret {
        secret = google_secret_manager_secret.otel_config.secret_id
        items {
          key  = "latest"
          path = "config.yaml"
        }
      }
    }
    
    volumes {
      name = "shared-logs"
      empty_dir {}
    }
  }
}

# Secret for OTEL Collector configuration
resource "google_secret_manager_secret" "otel_config" {
  secret_id = "otel-config-${random_id.suffix.hex}"
  
  replication {
    auto {}
  }
}

# Create the OTEL collector configuration
resource "google_secret_manager_secret_version" "otel_config_version" {
  secret      = google_secret_manager_secret.otel_config.id
  secret_data = <<EOT
receivers:
  filelog:
    include:
      - /var/log/*.log
    start_at: beginning
    include_file_path: true
    include_file_name: true
    operators:
      - type: json_parser
        
processors:
  batch:
    timeout: 1s
    send_batch_size: 1024
  resourcedetection:
    detectors: [env, gcp]
    timeout: 2s
  resource:
    attributes:
      - action: insert
        key: service.name
        value: "gke-cluster-notifications"
  attributes:
    actions:
      - key: loki.attribute.labels
        action: insert
        value: "service_name,severity,level"

exporters:
  loki:
    endpoint: ${env:LOKI_ENDPOINT}
    auth:
      authenticator: basic
    basic_auth:
      username: ${env:LOKI_USERNAME}
      password: ${env:LOKI_PASSWORD}
    labels:
      attributes:
        service_name: "resource.attributes.service.name"
        severity: "severity"
        level: "level"
    tenant_id: "gke-cluster-notifications"

service:
  pipelines:
    logs:
      receivers: [filelog]
      processors: [resourcedetection, resource, attributes, batch]
      exporters: [loki]
EOT
}

# Add IAM permission for Cloud Run to access the secret
resource "google_secret_manager_secret_iam_member" "otel_config_access" {
  secret_id = google_secret_manager_secret.otel_config.id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${google_service_account.cloudrun_sa.email}"
}

# Service account for Cloud Run
resource "google_service_account" "cloudrun_sa" {
  account_id   = "cloudrun-sa-${random_id.suffix.hex}"
  display_name = "Cloud Run Service Account"
}

# Service account for Pub/Sub to push to Cloud Run
resource "google_service_account" "pubsub_sa" {
  account_id   = "pubsub-cloud-run-sa-${random_id.suffix.hex}"
  display_name = "Pub/Sub to Cloud Run Service Account"
}

# Grant the service account permission to invoke the Cloud Run service
resource "google_cloud_run_service_iam_binding" "pubsub_invoker" {
  location = google_cloud_run_v2_service.gke_notifications_service.location
  service  = google_cloud_run_v2_service.gke_notifications_service.name
  role     = "roles/run.invoker"
  members  = ["serviceAccount:${google_service_account.pubsub_sa.email}"]
}

# Pub/Sub subscription that pushes to Cloud Run
resource "google_pubsub_subscription" "gke_notifications_subscription" {
  name  = "gke-notifications-push-subscription-${random_id.suffix.hex}"
  topic = google_pubsub_topic.gke_notifications.name

  push_config {
    push_endpoint = google_cloud_run_v2_service.gke_notifications_service.uri

    oidc_token {
      service_account_email = google_service_account.pubsub_sa.email
    }
  }

  # Configure retry policy - messages will be redelivered if the Cloud Run service fails
  retry_policy {
    minimum_backoff = "10s"
    maximum_backoff = "600s" # 10 minutes
  }

  # Acknowledge deadline - how long the subscriber has to acknowledge the message
  ack_deadline_seconds = 60

  # Enable message ordering if needed
  enable_message_ordering = false

  # Configure expiration policy - subscription will expire if there is no activity
  expiration_policy {
    ttl = "2592000s" # 30 days
  }

  depends_on = [
    google_cloud_run_service_iam_binding.pubsub_invoker
  ]
}

# Output the Cloud Run URL
output "cloud_run_url" {
  value = google_cloud_run_v2_service.gke_notifications_service.uri
}

# Output the Pub/Sub topic name
output "pubsub_topic" {
  value = google_pubsub_topic.gke_notifications.name
}

# Output the Pub/Sub subscription name
output "pubsub_subscription" {
  value = google_pubsub_subscription.gke_notifications_subscription.name
}
