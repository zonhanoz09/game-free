# Tạo chuỗi secret ngẫu nhiên bảo mật cho Tunnel
resource "random_id" "tunnel_secret" {
  byte_length = 32
}

# 1. Khởi tạo Cloudflare Tunnel (Đi thẳng qua hạ tầng Edge toàn cầu của Cloudflare)
resource "cloudflare_tunnel" "oke_tunnel" {
  account_id = var.cloudflare_account_id
  name       = "oke-game-arena-tunnel"
  secret     = random_id.tunnel_secret.b64_std
}

# 2. Cấu hình Ingress Routing của Cloudflare Tunnel tới K8s Service nội bộ
resource "cloudflare_tunnel_config" "oke_tunnel_config" {
  account_id = var.cloudflare_account_id
  tunnel_id  = cloudflare_tunnel.oke_tunnel.id

  config {
    ingress_rule {
      hostname = var.subdomain == "@" ? var.domain_name : "${var.subdomain}.${var.domain_name}"
      service  = "http://tactical-arena-service:8080"
    }

    # Ingress rule mặc định kết thúc (bắt buộc bởi Cloudflare)
    ingress_rule {
      service = "http_status:404"
    }
  }
}

# 3. Bản ghi CNAME trỏ Subdomain về Tunnel (Bật sẵn Proxied để nhận SSL & CDN miễn phí)
resource "cloudflare_record" "oke_tunnel_cname" {
  zone_id = var.cloudflare_zone_id
  name    = var.subdomain
  content = "${cloudflare_tunnel.oke_tunnel.id}.cfargotunnel.com"
  type    = "CNAME"
  proxied = true
  comment = "3v3 Tactical Arena OKE Tunnel - Bypass OCI 10 Mbps Limit"
}
