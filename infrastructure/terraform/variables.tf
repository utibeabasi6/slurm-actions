variable "hcloud_token" {
  description = "Hetzner Cloud token"
  type        = string
  sensitive   = true
}

variable "bastion_public_key" {
  description = "Public key of the bastion"
  type        = string
}

variable "compute_private_key" {
  description = "Private key of the compute nodes"
  type        = string
  sensitive   = true
}

variable "compute_public_key" {
  description = "Public key of the compute nodes"
  type        = string
}
