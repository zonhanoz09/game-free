locals {
  target_compartment_id = var.compartment_ocid != null ? var.compartment_ocid : var.tenancy_ocid
}

# 1. Virtual Cloud Network (VCN) cho OKE
resource "oci_core_vcn" "oke_vcn" {
  compartment_id = local.target_compartment_id
  cidr_blocks    = ["10.0.0.0/16"]
  display_name   = "oke-game-vcn"
  dns_label      = "okegame"
}

# 2. Gateways (Internet Gateway, NAT Gateway, Service Gateway)
resource "oci_core_internet_gateway" "oke_igw" {
  compartment_id = local.target_compartment_id
  vcn_id         = oci_core_vcn.oke_vcn.id
  display_name   = "oke-internet-gateway"
  enabled        = true
}

resource "oci_core_nat_gateway" "oke_nat" {
  compartment_id = local.target_compartment_id
  vcn_id         = oci_core_vcn.oke_vcn.id
  display_name   = "oke-nat-gateway"
}

data "oci_core_services" "all_services" {
  filter {
    name   = "name"
    values = ["All .* Services In Oracle Services Network"]
    regex  = true
  }
}

resource "oci_core_service_gateway" "oke_sgw" {
  compartment_id = local.target_compartment_id
  vcn_id         = oci_core_vcn.oke_vcn.id
  display_name   = "oke-service-gateway"

  services {
    service_id = data.oci_core_services.all_services.services[0].id
  }
}

# 3. Route Tables
# Route Table cho Public Subnets (Load Balancer & API Endpoint)
resource "oci_core_route_table" "oke_public_rt" {
  compartment_id = local.target_compartment_id
  vcn_id         = oci_core_vcn.oke_vcn.id
  display_name   = "oke-public-route-table"

  route_rules {
    destination       = "0.0.0.0/0"
    destination_type  = "CIDR_BLOCK"
    network_entity_id = oci_core_internet_gateway.oke_igw.id
  }
}

# Route Table cho Worker Node Subnet (Truy cập Internet qua NAT & Service Gateway)
resource "oci_core_route_table" "oke_node_rt" {
  compartment_id = local.target_compartment_id
  vcn_id         = oci_core_vcn.oke_vcn.id
  display_name   = "oke-node-route-table"

  route_rules {
    destination       = "0.0.0.0/0"
    destination_type  = "CIDR_BLOCK"
    network_entity_id = oci_core_nat_gateway.oke_nat.id
  }

  route_rules {
    destination       = data.oci_core_services.all_services.services[0].cidr_block
    destination_type  = "SERVICE_CIDR_BLOCK"
    network_entity_id = oci_core_service_gateway.oke_sgw.id
  }
}

# 4. Security Lists

# Security List cho Kubernetes API Endpoint (Port 6443)
resource "oci_core_security_list" "oke_endpoint_sl" {
  compartment_id = local.target_compartment_id
  vcn_id         = oci_core_vcn.oke_vcn.id
  display_name   = "oke-endpoint-security-list"

  egress_security_rules {
    destination = "0.0.0.0/0"
    protocol    = "all"
  }

  ingress_security_rules {
    protocol    = "6" # TCP
    source      = "0.0.0.0/0"
    description = "Allow Kubernetes API access"
    tcp_options {
      min = 6443
      max = 6443
    }
  }
}

# Security List cho Public Load Balancer (Port 80, 443, 8080)
resource "oci_core_security_list" "oke_lb_sl" {
  compartment_id = local.target_compartment_id
  vcn_id         = oci_core_vcn.oke_vcn.id
  display_name   = "oke-lb-security-list"

  egress_security_rules {
    destination = "0.0.0.0/0"
    protocol    = "all"
  }

  ingress_security_rules {
    protocol    = "6"
    source      = "0.0.0.0/0"
    description = "HTTP Traffic"
    tcp_options {
      min = 80
      max = 80
    }
  }

  ingress_security_rules {
    protocol    = "6"
    source      = "0.0.0.0/0"
    description = "HTTPS Traffic"
    tcp_options {
      min = 443
      max = 443
    }
  }

  ingress_security_rules {
    protocol    = "6"
    source      = "0.0.0.0/0"
    description = "Game WebSocket Traffic"
    tcp_options {
      min = 8080
      max = 8080
    }
  }
}

# Security List cho Worker Nodes
resource "oci_core_security_list" "oke_node_sl" {
  compartment_id = local.target_compartment_id
  vcn_id         = oci_core_vcn.oke_vcn.id
  display_name   = "oke-node-security-list"

  egress_security_rules {
    destination = "0.0.0.0/0"
    protocol    = "all"
  }

  ingress_security_rules {
    protocol    = "all"
    source      = "10.0.0.0/16"
    description = "Allow all internal VCN communication"
  }

  ingress_security_rules {
    protocol    = "6"
    source      = "0.0.0.0/0"
    description = "SSH Access"
    tcp_options {
      min = 22
      max = 22
    }
  }
}

# 5. Subnets (Kubernetes API Endpoint, Public Load Balancers, Worker Nodes)
resource "oci_core_subnet" "oke_endpoint_subnet" {
  compartment_id             = local.target_compartment_id
  vcn_id                     = oci_core_vcn.oke_vcn.id
  cidr_block                 = "10.0.0.0/28"
  display_name               = "oke-endpoint-subnet"
  dns_label                  = "endpoint"
  route_table_id             = oci_core_route_table.oke_public_rt.id
  security_list_ids          = [oci_core_security_list.oke_endpoint_sl.id]
  prohibit_public_ip_on_vnic = false
}

resource "oci_core_subnet" "oke_lb_subnet" {
  compartment_id             = local.target_compartment_id
  vcn_id                     = oci_core_vcn.oke_vcn.id
  cidr_block                 = "10.0.0.32/27"
  display_name               = "oke-lb-subnet"
  dns_label                  = "lb"
  route_table_id             = oci_core_route_table.oke_public_rt.id
  security_list_ids          = [oci_core_security_list.oke_lb_sl.id]
  prohibit_public_ip_on_vnic = false
}

resource "oci_core_subnet" "oke_node_subnet" {
  compartment_id             = local.target_compartment_id
  vcn_id                     = oci_core_vcn.oke_vcn.id
  cidr_block                 = "10.0.1.0/24"
  display_name               = "oke-node-subnet"
  dns_label                  = "node"
  route_table_id             = oci_core_route_table.oke_node_rt.id
  security_list_ids          = [oci_core_security_list.oke_node_sl.id]
  prohibit_public_ip_on_vnic = true
}
