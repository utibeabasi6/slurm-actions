output "compute_node_public_ip" {
  value = hcloud_server.slurm_cluster_compute.ipv4_address
}

output "controller_node_public_ip" {
  value = hcloud_server.slurm_cluster_controller.ipv4_address
}
