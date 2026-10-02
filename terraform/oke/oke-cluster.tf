data "oci_identity_availability_domains" "ads" {
  compartment_id = local.target_compartment_id
}

# 1. OKE Cluster (Basic Cluster - Hoàn toàn không mất phí quản trị)
resource "oci_containerengine_cluster" "game_cluster" {
  compartment_id     = local.target_compartment_id
  kubernetes_version = var.k8s_version
  name               = var.cluster_name
  vcn_id             = oci_core_vcn.oke_vcn.id
  type               = "BASIC_CLUSTER"

  endpoint_config {
    is_public_ip_enabled = true
    subnet_id            = oci_core_subnet.oke_endpoint_subnet.id
  }

  options {
    service_lb_subnet_ids = [oci_core_subnet.oke_lb_subnet.id]

    add_ons {
      is_kubernetes_dashboard_enabled = false
      is_tiller_enabled               = false
    }

    admission_controller_options {
      is_pod_security_policy_enabled = false
    }

    kubernetes_network_config {
      pods_cidr     = "10.244.0.0/16"
      services_cidr = "10.96.0.0/16"
    }
  }
}

# Lấy danh sách Image tối ưu cho OKE Node Pool
data "oci_containerengine_node_pool_option" "node_pool_options" {
  node_pool_option_id = "all"
}

locals {
  # Lọc image Oracle Linux OKE tương thích với ARM64 (A1.Flex) và phiên bản K8s đã chọn
  arm_images = [
    for source in data.oci_containerengine_node_pool_option.node_pool_options.sources :
    source.image_id if length(regexall("aarch64", lower(source.source_name))) > 0
  ]
  node_image_id = length(local.arm_images) > 0 ? local.arm_images[0] : data.oci_containerengine_node_pool_option.node_pool_options.sources[0].image_id
}

# 2. Worker Node Pool (Chạy trên Always Free ARM Ampere A1.Flex)
resource "oci_containerengine_node_pool" "game_node_pool" {
  cluster_id         = oci_containerengine_cluster.game_cluster.id
  compartment_id     = local.target_compartment_id
  kubernetes_version = var.k8s_version
  name               = "arm-node-pool-free"
  node_shape         = var.node_shape

  node_shape_config {
    ocpus         = var.node_ocpus
    memory_in_gbs = var.node_memory_in_gbs
  }

  node_source_details {
    image_id    = local.node_image_id
    source_type = "IMAGE"
  }

  node_config_details {
    size = var.node_count

    placement_configs {
      availability_domain = data.oci_identity_availability_domains.ads.availability_domains[0].name
      subnet_id           = oci_core_subnet.oke_node_subnet.id
    }
  }

  ssh_public_key = file(pathexpand(var.ssh_public_key_path))
}
