output "cluster_id" {
  description = "OCID của Kubernetes Cluster"
  value       = oci_containerengine_cluster.game_cluster.id
}

output "kubeconfig_command" {
  description = "Câu lệnh để tạo kubeconfig và kết nối kubectl tới OKE"
  value       = "oci ce cluster create-kubeconfig --cluster-id ${oci_containerengine_cluster.game_cluster.id} --file $HOME/.kube/config --region ${var.region} --token-version 2.0.0 --kube-endpoint PUBLIC_ENDPOINT"
}

output "cloudflare_tunnel_token" {
  description = "Token xác thực của Cloudflare Tunnel (Dùng để đưa vào Kubernetes Secret)"
  value       = cloudflare_tunnel.oke_tunnel.tunnel_token
  sensitive   = true
}

output "k8s_secret_command" {
  description = "Lệnh tạo Kubernetes Secret cho Cloudflare Tunnel"
  value       = "kubectl create secret generic cloudflare-tunnel-secret --from-literal=token='${cloudflare_tunnel.oke_tunnel.tunnel_token}' --dry-run=client -o yaml | kubectl apply -f -"
  sensitive   = true
}

output "game_url" {
  description = "Đường dẫn HTTPS chơi game qua Cloudflare (Không bị bóp 10 Mbps)"
  value       = "https://${var.subdomain == "@" ? var.domain_name : "${var.subdomain}.${var.domain_name}"}"
}
