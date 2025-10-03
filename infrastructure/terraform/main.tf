resource "hcloud_network" "slurm_cluster_private_network" {
  name     = "slurm_cluster"
  ip_range = "10.0.0.0/16"
}

resource "hcloud_network_subnet" "slurm_cluster_private_network_subnet" {
  type         = "cloud"
  network_id   = hcloud_network.slurm_cluster_private_network.id
  network_zone = "eu-central"
  ip_range     = "10.0.1.0/24"
}

resource "hcloud_server" "slurm_cluster_controller" {
  name        = "slurm-cluster-controller"
  image       = "ubuntu-24.04"
  server_type = "cax11"
  location    = "fsn1"
  public_net {
    ipv4_enabled = true
    ipv6_enabled = true
  }
  user_data = templatefile("${path.module}/files/slurm-controller-init.yaml", {
    bastion_public_key = var.bastion_public_key
    compute_public_key = var.compute_public_key
  })
}

resource "hcloud_server_network" "slurm_cluster_controller" {
  server_id = hcloud_server.slurm_cluster_controller.id
  subnet_id = hcloud_network_subnet.slurm_cluster_private_network_subnet.id
}

resource "hcloud_server" "slurm_cluster_compute" {
  count       = 1
  name        = "slurm-cluster-compute-${count.index + 1}"
  image       = "ubuntu-24.04"
  server_type = "cax11"
  location    = "fsn1"
  public_net {
    ipv4_enabled = true
    ipv6_enabled = true
  }
  user_data = templatefile("${path.module}/files/slurm-compute-init.yaml", {
    bastion_public_key  = var.bastion_public_key
    compute_private_key = var.compute_private_key
  })
}

resource "hcloud_server_network" "slurm_cluster_compute" {
  count     = length(hcloud_server.slurm_cluster_compute)
  server_id = hcloud_server.slurm_cluster_compute[count.index].id
  subnet_id = hcloud_network_subnet.slurm_cluster_private_network_subnet.id
}
