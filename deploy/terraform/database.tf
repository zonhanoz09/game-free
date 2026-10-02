# ==============================================================================
# ORACLE AUTONOMOUS DATABASE (ALWAYS FREE TIER)
# ==============================================================================

resource "oci_database_autonomous_database" "game_adb" {
  compartment_id              = var.compartment_ocid != null ? var.compartment_ocid : var.tenancy_ocid
  db_name                     = "gamedb"
  display_name                = "gamedb"
  db_workload                 = "OLTP"
  is_free_tier                = true
  admin_password              = var.adb_admin_password
  cpu_core_count              = 1
  data_storage_size_in_tbs    = 1
  license_model               = "LICENSE_INCLUDED"
  is_auto_scaling_enabled     = false
  is_mtls_connection_required = false
  whitelisted_ips             = ["0.0.0.0/0"]
}

output "adb_id" {
  value       = oci_database_autonomous_database.game_adb.id
  description = "OCID của Oracle Autonomous Database Always Free"
}

output "adb_connection_urls" {
  value       = oci_database_autonomous_database.game_adb.connection_urls
  description = "Connection URLs (APEX, ORDS, SQL Dev Web)"
}

output "adb_connection_strings" {
  value       = oci_database_autonomous_database.game_adb.connection_strings
  description = "Connection Strings (TLS and mTLS)"
}
