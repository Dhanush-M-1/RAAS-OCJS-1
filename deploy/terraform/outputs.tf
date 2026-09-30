output "project_id" {
  description = "Project that owns the judge VM."
  value       = var.project_id
}

output "instance_name" {
  description = "Name of the judge VM."
  value       = google_compute_instance.judge.name
}

output "zone" {
  description = "Zone the VM runs in."
  value       = google_compute_instance.judge.zone
}

output "internal_ip" {
  description = "Internal address of the judge inside the VPC."
  value       = google_compute_instance.judge.network_interface[0].network_ip
}

output "external_ip" {
  description = <<-EOT
    Ephemeral public address, present for outbound package and image pulls.

    This does NOT make the judge reachable. Ingress is default-deny and the only
    rule targets Google's IAP range on tcp/22, so nothing on the internet can
    connect to 3000.
  EOT
  value       = google_compute_instance.judge.network_interface[0].access_config[0].nat_ip
}

output "tunnel_command" {
  description = "Forwards local port 3000 to the judge over IAP. Run this, then talk to http://localhost:3000."
  value       = "gcloud compute start-iap-tunnel ${google_compute_instance.judge.name} 3000 --local-host-port=localhost:3000 --zone=${google_compute_instance.judge.zone} --project=${var.project_id}"
}

output "ssh_command" {
  description = "Interactive SSH over IAP."
  value       = "gcloud compute ssh ${google_compute_instance.judge.name} --zone=${google_compute_instance.judge.zone} --project=${var.project_id} --tunnel-through-iap"
}

output "bootstrap_log_command" {
  description = "Follow the first-boot bootstrap log, which is where Docker, Rust and image builds report progress."
  value       = "gcloud compute ssh ${google_compute_instance.judge.name} --zone=${google_compute_instance.judge.zone} --project=${var.project_id} --tunnel-through-iap --command='sudo tail -f /var/log/raas-bootstrap.log'"
}

output "judge_api_exposed" {
  description = "Whether tcp/3000 has any firewall rule at all. False means IAP tunnel only."
  value       = length(var.judge_source_ranges) > 0
}
