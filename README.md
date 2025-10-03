# Slurm Actions - GitHub Actions CI/CD on Slurm

A basic CI/CD platform that runs GitHub Actions workflows as Slurm jobs, providing scalable compute resources for your GitHub repositories.

## Features

- **GitHub Actions Integration**: Run GitHub Actions workflows on Slurm compute clusters
- **Event Triggers**: Supports push events and other GitHub webhook events
- **Third-party Actions**: Compatible some popular actions like `actions/checkout`
- **Scalable**: Leverage Slurm's job scheduling and resource management
- **Self-hosted**: Run on your own infrastructure with full control

## Prerequisites

Before setting up the project, ensure you have the following installed:

- **Docker** - For running RabbitMQ message queue
- **Rust** - For building and running the webhook API and worker services
- **Terraform** - For provisioning cloud infrastructure
- **Ansible** - For configuring Slurm cluster nodes
- **ngrok** - For exposing the API server to the public internet
- **Hetzner Cloud Account** - For cloud infrastructure (or adapt for your preferred provider)

## Setup Instructions

### 1. RabbitMQ Message Queue

Start a RabbitMQ container for message queuing:

```bash
docker run -it --rm --name rabbitmq -p 5552:5552 -p 15672:15672 -p 5672:5672 \
    -e RABBITMQ_SERVER_ADDITIONAL_ERL_ARGS='-rabbitmq_stream advertised_host localhost' \
    rabbitmq:4-management
```

In a separate terminal, enable required plugins:

```bash
docker exec rabbitmq rabbitmq-plugins enable rabbitmq_stream rabbitmq_stream_management
```

### 2. SSH Key Generation

Create SSH key pairs for infrastructure access:

```bash
# Generate bastion key (for initial access)
ssh-keygen -t rsa -b 4096 -f ~/.ssh/bastion -C "bastion-key"

# Generate compute node key (for inter-node communication)
ssh-keygen -t rsa -b 4096 -f ~/.ssh/slurm-compute -C "slurm-compute-key"
```

### 3. Infrastructure Provisioning

Navigate to the Terraform directory:

```bash
cd infrastructure/terraform
```

Create a `.tfvars` file with your configuration:

```hcl
# .tfvars
hcloud_token = "your-hetzner-cloud-token"
bastion_public_key = "ssh-rsa AAAAB3NzaC1yc2E... bastion-key"  # Contents of ~/.ssh/bastion.pub
compute_private_key = "-----BEGIN OPENSSH PRIVATE KEY-----\n..."  # Contents of ~/.ssh/slurm-compute
compute_public_key = "ssh-rsa AAAAB3NzaC1yc2E... slurm-compute-key"  # Contents of ~/.ssh/slurm-compute.pub
```

Deploy the infrastructure:

```bash
terraform init
terraform apply -var-file=.tfvars
```

**Important**: Note down the output values:
- `controller_node_public_ip`
- `compute_node_public_ip`

### 4. Ansible Configuration

Create an inventory file:

```bash
cd ../ansible
```

Create `inventory.ini`:

```ini
[controller]
slurm-cluster-controller ansible_host=<controller_node_public_ip> ansible_user=slurm ansible_ssh_private_key_file=<path_to_bastion_private_key> ansible_ssh_common_args='-o StrictHostKeyChecking=no'

[compute]
slurm-cluster-compute-1 ansible_host=<compute_node_public_ip> ansible_user=slurm ansible_ssh_private_key_file=<path_to_bastion_private_key> ansible_ssh_common_args='-o StrictHostKeyChecking=no'
```

Replace placeholders with actual values:
- `<controller_node_public_ip>`: Output from Terraform
- `<compute_node_public_ip>`: Output from Terraform
- `<path_to_bastion_private_key>`: Path to your bastion private key (e.g., `/Users/username/.ssh/bastion`)

### 5. Slurm Configuration Update

Before running the Ansible playbooks, update the Slurm configuration file:

1. Edit `infrastructure/ansible/files/slurm/slurm.conf` and replace `SLURM_COMPUTE_NODE_IP` with the actual IP address of your compute node (the `compute_node_public_ip` from Terraform output).

### 6. Slurm Cluster Setup

Set up the controller node:

```bash
ansible-playbook -i inventory.ini setup-slurm-controller.yaml
```

**Note**: If the controller setup fails due to MySQL package issues, SSH into the controller node and run:

```bash
ssh -i <path_to_bastion_private_key> slurm@<controller_node_public_ip>
sudo apt --fix-broken install -y
```

Then re-run the controller playbook.

Set up the compute node:

```bash
ansible-playbook -i inventory.ini setup-slurm-compute.yaml \
  --extra-vars slurm_controller_ip=<controller_node_public_ip> \
  --extra-vars slurm_controller_hostname=slurm-cluster-controller
```

### 7. Configure Environment Variables

Create a `.env` file in the `ghwebhooks/crates/rabbitmq-worker/` directory with the following variables:

```bash
cd ../../ghwebhooks/crates/rabbitmq-worker
```

Create `.env` file:

```env
GHWEBHOOKS_RMQ_CONSUMER_GITHUB_TOKEN=your_github_token_here
GHWEBHOOKS_RMQ_CONSUMER_SLURMRESTD_HOST=<controller_node_public_ip>
GHWEBHOOKS_RMQ_CONSUMER_SLURMRESTD_PORT=6820
GHWEBHOOKS_RMQ_CONSUMER_SLURMRESTD_USER=slurm
GHWEBHOOKS_RMQ_CONSUMER_SLURMRESTD_TOKEN=your_slurm_token_here
```

**To get the Slurm token**:
1. SSH into the controller node: `ssh -i <path_to_bastion_private_key> slurm@<controller_node_public_ip>`
2. Run: `scontrol token`
3. Copy the token value and paste it in the `.env` file

### 8. GitHub Webhook Service

Navigate to the webhook service directory:

```bash
cd ../../../ghwebhooks
```

Start the API server:

```bash
cargo run --bin api
```

In a separate terminal, start the RabbitMQ worker:

```bash
cargo run --bin rabbitmq-worker
```

### 9. Expose API with ngrok

To make your API accessible to GitHub webhooks, expose it using ngrok.

In a separate terminal, run:

```bash
ngrok http 8000
```

This will provide you with a public URL (e.g., `https://abc123.ngrok.io`) that forwards to your local API server.

**Note**: Copy the ngrok URL as you'll need it for configuring GitHub webhooks.

## Architecture

```
GitHub Repository
       ↓ (webhook)
   API Server
       ↓ (message queue)
   RabbitMQ
       ↓ (job processing)
   Worker Service
       ↓ (job submission)
   Slurm Controller
       ↓ (job execution)
   Compute Nodes
```

## Usage

1. **Set up GitHub App**:
   - Go to GitHub Settings → Developer settings → GitHub Apps
   - Click "New GitHub App"
   - Fill in the app details:
     - **App name**: Choose a unique name for your app
     - **Homepage URL**: Your ngrok URL (e.g., `https://abc123.ngrok.io`)
     - **Webhook URL**: Your ngrok URL with `/webhook` endpoint (e.g., `https://abc123.ngrok.io/webhook`)
     - **Webhook secret**: Generate a secure secret (optional but recommended)
   - Under "Repository permissions", grant necessary permissions (e.g., Contents: Read, Metadata: Read)
   - Under "Subscribe to events", select relevant events (e.g., Push)
   - Create the GitHub App

2. **Install the GitHub App**:
   - After creating the app, install it on your target repository
   - **Important**: Note down the **App Installation ID** from the installation URL or settings
   - The installation ID will be needed for API authentication

3. **Push Code**: Push commits to trigger workflow execution
4. **Monitor Jobs**: Use Slurm commands (`squeue`, `sacct`) or the Slurm REST API to monitor job status
5. **View Logs**: Check job outputs in Slurm log directories

## Configuration

### Environment Variables

The services can be configured using environment variables:

- `RABBITMQ_URL`: RabbitMQ connection string (default: `amqp://localhost:5672`)
- `SLURM_CONTROLLER_HOST`: Slurm controller hostname
- `API_PORT`: API server port (default: 8000)
- `GITHUB_APP_ID`: Your GitHub App ID
- `GITHUB_APP_INSTALLATION_ID`: The installation ID noted during app installation
- `GITHUB_APP_PRIVATE_KEY`: Path to your GitHub App's private key file

### Slurm Configuration

The Slurm cluster is configured via Ansible playbooks. Key configuration files:

- `infrastructure/ansible/files/slurm/slurm.conf`: Main Slurm configuration
- `infrastructure/ansible/files/slurm/slurmdbd.conf`: Slurm database daemon configuration

## Troubleshooting

### Common Issues

1. **Database Connection Errors**: Ensure MySQL is running and slurmdbd can connect
2. **Authentication Failures**: Check Munge key synchronization between nodes
3. **Job Submission Failures**: Verify Slurm controller and compute nodes are communicating

### Useful Commands

```bash
# Check Slurm cluster status
sinfo

# View job queue
squeue

# Check job history
sacct

# Test Munge authentication
munge -n | unmunge

# Check service status
systemctl status slurmctld
systemctl status slurmd
systemctl status slurmdbd
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## Support

For issues and questions:
1. Check the troubleshooting section above
2. Review Slurm documentation
3. Create an issue in the GitHub repository
