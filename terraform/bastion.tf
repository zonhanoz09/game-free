# ==============================================================================
# OCI BASTION SERVICE (ALWAYS FREE)
# Cho phép SSH & Rsync deploy mã nguồn vào Private VM mà KHÔNG CẦN Public IP
# ==============================================================================

resource "oci_bastion_bastion" "game_bastion" {
  bastion_type     = "STANDARD"
  compartment_id   = local.target_compartment_id
  target_subnet_id = oci_core_subnet.game_subnet.id
  name             = "game-arena-bastion"

  client_cidr_block_allow_list = ["0.0.0.0/0"]
  max_session_ttl_in_seconds   = 10800 # 3 giờ
}

output "bastion_id" {
  description = "OCID của OCI Bastion Service"
  value       = oci_bastion_bastion.game_bastion.id
}
