###############################################################################
# Inputs for the RAAS-OCJS judge deployment.
###############################################################################

variable "project_id" {
  description = "GCP project that will own the judge VM. Billing must be enabled."
  type        = string
}

variable "region" {
  description = "Region to deploy into. asia-south1 (Mumbai) is closest to the benchmark client."
  type        = string
  default     = "asia-south1"
}

variable "zone" {
  description = "Zone within the region."
  type        = string
  default     = "asia-south1-a"
}

variable "instance_name" {
  description = "Name of the judge VM. Must be a valid GCE name (lowercase, digits, hyphens)."
  type        = string
  default     = "raas-judge"
}

variable "machine_type" {
  description = <<-EOT
    Machine type. e2-standard-4 is 4 vCPU / 16 GiB.

    Sizing note: a Low-tier submission container is capped at low_tier_mb plus
    1 vCPU. A container promoted to High is uncapped and may consume the whole
    host, so the concurrency the judge can safely absorb is set by how many
    promotions overlap, not by dividing 16 GiB by the Low tier alone.
  EOT
  type        = string
  default     = "e2-standard-4"
}

variable "boot_disk_size_gb" {
  description = <<-EOT
    Boot disk size. The Rust build tree plus the three runtime images need room;
    20 GiB is workable but tight, 30 GiB leaves headroom.
  EOT
  type        = number
  default     = 30
}

variable "boot_disk_type" {
  description = "Boot disk type. pd-balanced is the cost/performance default."
  type        = string
  default     = "pd-balanced"
}

variable "low_tier_mb" {
  description = "Memory ceiling in MiB for the Low tier. The judge defaults to 256."
  type        = number
  default     = 256
}

variable "repo_url" {
  description = "Public git URL the VM clones. Must be reachable without credentials."
  type        = string
  default     = "https://github.com/Hemanthkumar2k04/RAAS-OCJS.git"
}

variable "repo_ref" {
  description = "Branch or tag to check out on the VM."
  type        = string
  default     = "main"
}

variable "ssh_source_ranges" {
  description = <<-EOT
    CIDRs allowed to reach tcp/22 directly. Empty by default, which leaves IAP
    as the only SSH path (Google's 35.235.240.0/20 range is always permitted).

    Prefer leaving this empty. Adding 0.0.0.0/0 would undo the main protection.
  EOT
  type        = list(string)
  default     = []
}

variable "judge_source_ranges" {
  description = <<-EOT
    CIDRs allowed to reach the judge on tcp/3000. Empty by default, meaning the
    judge is reachable only over an IAP tunnel.

    The judge compiles and executes arbitrary submitted source. A shared-secret
    header is enforced when RAAS_AUTH_TOKEN is set, but that is a second layer,
    not a licence to publish the port. Populate this with a single narrow CIDR
    (a home or office address) if the React frontend must call the API directly.
  EOT
  type        = list(string)
  default     = []
}
