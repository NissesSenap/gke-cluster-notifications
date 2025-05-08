variable "project_id" {
  description = "The Google Cloud Project ID"
  type        = string
}

variable "region" {
  description = "The Google Cloud region where resources will be created"
  type        = string
  default     = "us-central1"
}

variable "slack_webhook_url" {
  description = "The Slack webhook URL for notifications"
  type        = string
  default     = ""
  sensitive   = true
}

# Loki integration variables
variable "loki_endpoint" {
  description = "The endpoint URL for your Loki instance"
  type        = string
}

variable "loki_username" {
  description = "Username for Loki authentication"
  type        = string
  default     = ""
  sensitive   = true
}

variable "loki_password" {
  description = "Password for Loki authentication"
  type        = string
  default     = ""
  sensitive   = true
}
