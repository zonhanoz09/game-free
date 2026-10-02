data "oci_identity_availability_domains" "ads" {
  compartment_id = local.target_compartment_id
}

# Tự động tìm kiếm Image Canonical Ubuntu 22.04 mới nhất tương ứng với Shape (ARM64 hoặc x86_64)
data "oci_core_images" "ubuntu_arm" {
  count                    = length(regexall("A1", var.instance_shape)) > 0 ? 1 : 0
  compartment_id           = local.target_compartment_id
  operating_system         = "Canonical Ubuntu"
  operating_system_version = "22.04"
  shape                    = var.instance_shape
  sort_by                  = "TIMECREATED"
  sort_order               = "DESC"
}

data "oci_core_images" "ubuntu_x86" {
  count                    = length(regexall("A1", var.instance_shape)) == 0 ? 1 : 0
  compartment_id           = local.target_compartment_id
  operating_system         = "Canonical Ubuntu"
  operating_system_version = "22.04"
  shape                    = var.instance_shape
  sort_by                  = "TIMECREATED"
  sort_order               = "DESC"
}

locals {
  image_id = length(regexall("A1", var.instance_shape)) > 0 ? data.oci_core_images.ubuntu_arm[0].images[0].id : data.oci_core_images.ubuntu_x86[0].images[0].id
  is_flex  = length(regexall("Flex", var.instance_shape)) > 0
}

# Máy chủ ảo Oracle Cloud Always Free
resource "oci_core_instance" "game_server" {
  availability_domain = data.oci_identity_availability_domains.ads.availability_domains[0].name
  compartment_id      = local.target_compartment_id
  display_name        = "tactical-arena-free-tier"
  shape               = var.instance_shape

  dynamic "shape_config" {
    for_each = local.is_flex ? [1] : []
    content {
      ocpus         = var.instance_ocpus
      memory_in_gbs = var.instance_memory_in_gbs
    }
  }

  create_vnic_details {
    subnet_id        = oci_core_subnet.game_subnet.id
    display_name     = "primary-vnic"
    assign_public_ip = false
    hostname_label   = "gameserver"
  }

  source_details {
    source_type             = "image"
    source_id               = local.image_id
    boot_volume_size_in_gbs = 50 # OCI Free Tier tặng miễn phí lên đến 200GB Boot Volume
  }

  lifecycle {
    ignore_changes = [
      create_vnic_details,
      metadata,
    ]
  }

  metadata = {
    ssh_authorized_keys = file(pathexpand(var.ssh_public_key_path))
    user_data           = base64encode(file("${path.module}/scripts/cloud-init.yaml"))
  }

  preserve_boot_volume = false
}
