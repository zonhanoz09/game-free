# ☸️ Triển Khai Game Trên OKE Always Free Qua Cloudflare Tunnel (Bỏ Qua Giới Hạn 10 Mbps)

Hệ thống được chuyển đổi sang **Cloudflare Tunnel (`cloudflared`)** chạy trực tiếp bên trong cụm **Oracle Kubernetes Engine (OKE) Always Free**, giúp:
1. **Loại bỏ hoàn toàn giới hạn 10 Mbps**: Không tạo OCI Flexible Load Balancer (tránh bị bóp băng thông 10 Mbps của gói Free Tier).
2. **Tận dụng mạng lưới Cloudflare Global Anycast Edge**: Băng thông không giới hạn, CDN cache toàn bộ WebAssembly (~70MB) và hình ảnh asset tại các máy chủ gần người chơi nhất (Việt Nam, Singapore, Hong Kong...).
3. **Bảo mật tuyệt đối (Zero Open Ports)**: Không cần mở bất kỳ cổng Inbound nào trên OCI Firewall hay VCN Security List. Các pod `cloudflared` tự động kết nối ra ngoài (Outbound) tới Cloudflare Edge.
4. **Tự động cấp SSL/TLS HTTPS & WSS**: Cloudflare tự động xử lý chứng chỉ bảo mật và hỗ trợ giao thức WebSocket mượt mà.

---

## 📁 Cấu Trúc Manifests Kubernetes (`deploy/k8s/`)

| File | Chức năng |
| :--- | :--- |
| [`service.yaml`](../deploy/k8s/service.yaml) | Service ClusterIP nội bộ (`tactical-arena-service:8080`). |
| [`cloudflared.yaml`](../deploy/k8s/cloudflared.yaml) | Chạy Cloudflare Tunnel và chuyển tiếp request tới service game. |
| [`deployment.yaml`](../deploy/k8s/deployment.yaml) | Quản lý game server pods với liveness/readiness probe. |
| [`hpa.yaml`](../deploy/k8s/hpa.yaml) | Autoscale pod theo CPU. |

---

## 🚀 Các Bước Thực Hiện

### Bước 1: Khởi tạo cụm OKE & Cloudflare Tunnel bằng Terraform
1. Vào thư mục `deploy/terraform/oke`:
   ```bash
   cd deploy/terraform/oke
   cp terraform.tfvars.example terraform.tfvars
   ```
2. Điền thông tin OCI và Cloudflare vào `terraform.tfvars`:
   - `tenancy_ocid`, `user_ocid`, `fingerprint`, `private_key_path`
   - `cloudflare_api_token`
   - `cloudflare_account_id` (Lấy tại thanh bên phải trên Cloudflare Dashboard)
   - `cloudflare_zone_id`
   - `domain_name = "yourdomain.com"`
   - `subdomain = "game"`
3. Chạy lệnh:
   ```bash
   terraform init
   terraform apply
   ```
   *(Terraform sẽ tự động tạo cụm OKE Basic Cluster + 2 Node ARM Free Tier, tạo Cloudflare Tunnel và bản ghi CNAME trên Cloudflare)*

### Bước 2: Tải Kubeconfig để kết nối cụm OKE
Chạy câu lệnh hiển thị trong output của Terraform:
```bash
oci ce cluster create-kubeconfig \
  --cluster-id <CLUSTER_OCID> \
  --file $HOME/.kube/config \
  --region <REGION> \
  --token-version 2.0.0 \
  --kube-endpoint PUBLIC_ENDPOINT
```

### Bước 3: Triển khai ứng dụng và Cloudflare Tunnel
Chạy script tự động:
```bash
cd ../..
./scripts/deploy-to-oke.sh
```

Script sẽ tự động:
1. Đọc Tunnel Token từ Terraform và tạo Secret `cloudflare-tunnel-secret` trong Kubernetes.
2. Biên dịch WebAssembly bản mới nhất.
3. Áp dụng toàn bộ `deploy/k8s/` (Deployment, ClusterIP Service, Cloudflared Tunnel, HPA).
4. Kiểm tra trạng thái Pods sẵn sàng.

Bây giờ bạn có thể truy cập thẳng vào: **`https://game.yourdomain.com`**! Tốc độ tải cực nhanh và không bao giờ bị nghẽn 10 Mbps!
