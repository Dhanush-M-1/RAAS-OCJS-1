###############################################################################
# RAAS-OCJS judge on a single GCE VM.
#
# The judge cannot run on a serverless platform. It writes to the host cgroup
# tree as root and drives the Docker socket to spawn sibling containers, so it
# needs a real kernel and a real Docker daemon. Cloud Run, Cloud Functions, App
# Engine and GKE Autopilot all withhold one or both. A plain VM is the correct
# primitive here, not a compromise.
###############################################################################

terraform {
  required_version = ">= 1.5"

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 6.0"
    }
  }
}

provider "google" {
  project = var.project_id
  region  = var.region
  zone    = var.zone
}

# Compute and IAP are the only APIs this configuration needs. Enabling them here
# keeps the apply self-contained on a fresh project.
resource "google_project_service" "compute" {
  project            = var.project_id
  service            = "compute.googleapis.com"
  disable_on_destroy = false
}

resource "google_project_service" "iap" {
  project            = var.project_id
  service            = "iap.googleapis.com"
  disable_on_destroy = false
}

###############################################################################
# Network
#
# A dedicated VPC rather than the default network, because the default network
# ships permissive rules (notably ssh from 0.0.0.0/0) that would quietly expose
# the VM. Ingress here is default-deny and only two rules ever open anything.
###############################################################################

resource "google_compute_network" "raas" {
  name                    = "${var.instance_name}-net"
  auto_create_subnetworks = false
  depends_on              = [google_project_service.compute]
}

resource "google_compute_subnetwork" "raas" {
  name          = "${var.instance_name}-subnet"
  network       = google_compute_network.raas.id
  region        = var.region
  ip_cidr_range = "10.10.0.0/24"
}

# IAP's fixed forwarding range. This is the only SSH path by default; the VM is
# never reachable on 22 from the public internet.
resource "google_compute_firewall" "iap_ssh" {
  name    = "${var.instance_name}-allow-iap-ssh"
  network = google_compute_network.raas.name

  allow {
    protocol = "tcp"
    ports    = ["22"]
  }

  source_ranges = ["35.235.240.0/20"]
  target_tags   = ["raas-judge"]

  description = "SSH via Identity-Aware Proxy only."
}

# Optional escape hatch. Created only when judge_source_ranges is non-empty, so
# the default posture leaves tcp/3000 closed to the world.
resource "google_compute_firewall" "judge_api" {
  count   = length(var.judge_source_ranges) > 0 ? 1 : 0
  name    = "${var.instance_name}-allow-judge-api"
  network = google_compute_network.raas.name

  allow {
    protocol = "tcp"
    ports    = ["3000"]
  }

  source_ranges = var.judge_source_ranges
  target_tags   = ["raas-judge"]

  description = "Judge API. Keep this to a single narrow CIDR if used at all."
}

resource "google_compute_firewall" "user_ssh" {
  count   = length(var.ssh_source_ranges) > 0 ? 1 : 0
  name    = "${var.instance_name}-allow-user-ssh"
  network = google_compute_network.raas.name

  allow {
    protocol = "tcp"
    ports    = ["22"]
  }

  source_ranges = var.ssh_source_ranges
  target_tags   = ["raas-judge"]

  description = "Direct SSH. Prefer IAP; only add a narrow CIDR here."
}

###############################################################################
# Judge VM
###############################################################################

resource "google_compute_instance" "judge" {
  name         = var.instance_name
  machine_type = var.machine_type
  zone         = var.zone
  tags         = ["raas-judge"]

  # Spot is deliberately not used. Preemption mid-run would silently corrupt a
  # benchmark that measures promotion timing, and the saving is a few cents.
  scheduling {
    provisioning_model  = "STANDARD"
    automatic_restart   = true
    on_host_maintenance = "MIGRATE"
  }

  # Lets the judge survive a host maintenance event without a hard stop, which
  # matters because a stopped VM still bills for its disk.
  allow_stopping_for_update = true

  boot_disk {
    initialize_params {
      image = "ubuntu-os-cloud/ubuntu-2404-lts-amd64"
      size  = var.boot_disk_size_gb
      type  = var.boot_disk_type
    }
  }

  network_interface {
    subnetwork = google_compute_subnetwork.raas.id

    # An ephemeral external address is attached so the VM can apt-get and pull
    # images without Cloud NAT. It does not make the judge reachable: ingress is
    # default-deny and only the IAP range has a rule.
    access_config {}
  }

  metadata_startup_script = templatefile("${path.module}/startup.sh.tftpl", {
    repo_url    = var.repo_url
    repo_ref    = var.repo_ref
    low_tier_mb = var.low_tier_mb
  })

  service_account {
    # The judge needs no GCP API access at all. No scopes are granted.
    scopes = []
  }

  labels = {
    app = "raas-ocjs"
  }

  depends_on = [
    google_project_service.compute,
    google_compute_firewall.iap_ssh,
  ]
}
