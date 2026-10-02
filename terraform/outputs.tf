output "server_private_ip" {
  description = "Địa chỉ Private IP nội bộ của máy ảo (Không có Public IP)"
  value       = oci_core_instance.game_server.private_ip
}

output "game_domain" {
  description = "Tên miền truy cập Game qua Cloudflare Tunnel"
  value       = var.subdomain
}

output "game_url" {
  description = "Đường dẫn HTTPS trực tiếp để chơi Game online qua Cloudflare"
  value       = "https://${var.subdomain}"
}
