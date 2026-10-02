locals {
  target_compartment_id = var.compartment_ocid != null ? var.compartment_ocid : var.tenancy_ocid
}

# ==============================================================================
# KIẾN TRÚC MẠNG BẢO MẬT: INTERNET -> CLOUDFLARE -> TUNNEL -> OCI PRIVATE VCN
# ==============================================================================

# 1. Virtual Cloud Network (VCN Riêng Tư)
resource "oci_core_vcn" "game_vcn" {
  compartment_id = local.target_compartment_id
  cidr_blocks    = ["10.0.0.0/16"]
  display_name   = "game-arena-vcn"
  dns_label      = "gamearena"
}

# 2. NAT Gateway (Chỉ cho phép kết nối Outbound từ OCI ra Cloudflare Edge)
# Không có Internet Gateway -> Không một thiết bị nào trên Internet có thể gọi trực tiếp vào VCN
resource "oci_core_nat_gateway" "game_nat" {
  compartment_id = local.target_compartment_id
  vcn_id         = oci_core_vcn.game_vcn.id
  display_name   = "game-arena-nat-gateway"
}

# 3. Route Table (Chỉ đường Outbound đi qua NAT Gateway)
resource "oci_core_route_table" "game_rt" {
  compartment_id = local.target_compartment_id
  vcn_id         = oci_core_vcn.game_vcn.id
  display_name   = "game-arena-route-table"

  route_rules {
    destination       = "0.0.0.0/0"
    destination_type  = "CIDR_BLOCK"
    network_entity_id = oci_core_nat_gateway.game_nat.id
  }
}

# 4. Security List (Tường lửa VCN - Khóa chặt Inbound, chỉ mở Outbound)
resource "oci_core_security_list" "game_sl" {
  compartment_id = local.target_compartment_id
  vcn_id         = oci_core_vcn.game_vcn.id
  display_name   = "game-arena-security-list"

  # Outbound: Cho phép gửi ra Internet (Để Cloudflare Tunnel thiết lập kênh truyền QUIC/TLS)
  egress_security_rules {
    destination = "0.0.0.0/0"
    protocol    = "all"
    description = "Allow all outbound traffic for Cloudflare Tunnel and OS updates"
  }

  # Inbound: Chỉ cho phép lưu lượng nội bộ bên trong VCN (10.0.0.0/16)
  ingress_security_rules {
    protocol    = "all"
    source      = "10.0.0.0/16"
    description = "Allow internal VCN traffic only"
  }
}

# 5. Private Subnet (Không cấp phát Public IP)
resource "oci_core_subnet" "game_subnet" {
  compartment_id             = local.target_compartment_id
  vcn_id                     = oci_core_vcn.game_vcn.id
  cidr_block                 = "10.0.1.0/24"
  display_name               = "game-arena-private-subnet"
  dns_label                  = "public"
  route_table_id             = oci_core_route_table.game_rt.id
  security_list_ids          = [oci_core_security_list.game_sl.id]
  prohibit_public_ip_on_vnic = false
}
