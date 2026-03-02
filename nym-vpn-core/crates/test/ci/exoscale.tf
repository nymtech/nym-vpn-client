terraform {
  required_providers {
    exoscale = {
      source  = "exoscale/exoscale"
    }
  }
}

variable "exoscale_api_key" {
  type = string
  sensitive = true
}
variable "exoscale_api_secret" {
  type = string
  sensitive = true
}

variable "run_id" {
  type        = string
  default     = "local"
  description = "Unique run identifier (GH Actions run_id or 'local' for dev)"
}

variable "template_id" {
  type        = string
  sensitive = true
  description = "Exoscale template ID for the VM image"

  validation {
    condition     = var.template_id != ""
    error_message = "template_id must be set. Configure the EXOSCALE_TEMPLATE_ID repository variable."
  }
}

variable "instance_type" {
  type        = string
  description = "Exoscale compute instance type"
}

provider "exoscale" {
  key    = var.exoscale_api_key
  secret = var.exoscale_api_secret
}

locals {
  zone_ch = "ch-gva-2"
}

resource "exoscale_security_group" "test_harness_sg" {
  name = "test-harness-sg-${var.run_id}"
}

resource "exoscale_security_group_rule" "ssh" {
  security_group_id = exoscale_security_group.test_harness_sg.id
  type              = "INGRESS"
  protocol          = "TCP"
  cidr              = "0.0.0.0/0"
  start_port        = 22
  end_port          = 22
}

# Data volume is managed outside Terraform: you create it ONCE, save volume ID
# as TF_VAR_<variable name below>, then it'll get automatically attached to every new VM.
# On first VM launch, you initialize the block device (format, mount etc.).
# Since the volume persists through VM lifecycles, on subsequent runs, you don't have to do anything with it
# Caveat: you also have to deprovision it manually (via CLI) if you decide you don't need it anymore.
# But you do NOT destroy it every time along the VM.
variable "data_volume_id" {
  type        = string
  sensitive = true
  description = "ID of a pre-existing block storage volume (for persistent data like qemu images)"
}

resource "exoscale_compute_instance" "test_harness_instance" {
  zone        = local.zone_ch
  name        = "test-harness-${var.run_id}"

  template_id = var.template_id
  type        = var.instance_type
  disk_size   = 80

  security_group_ids = [exoscale_security_group.test_harness_sg.id]

  block_storage_volume_ids = [var.data_volume_id]

  user_data   = templatefile("${path.module}/cloud-init.yml.tftpl", {})
}

output "instance_ip" {
  value = exoscale_compute_instance.test_harness_instance.public_ip_address
}
