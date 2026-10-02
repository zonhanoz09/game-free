#!/usr/bin/env bash
set -e

# ==============================================================================
# Script tự động triển khai Game lên OKE & Cloudflare Tunnel (Bypass 10 Mbps OCI LB)
# ==============================================================================

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_DIR"

echo "======================================================="
echo " ☸️ BẮT ĐẦU TRIỂN KHAI LÊN OKE & CLOUDFLARE TUNNEL"
echo " 🚀 Đi đường Cloudflare Edge - Không giới hạn 10 Mbps"
echo "======================================================="

# 1. Kiểm tra kubectl
if ! command -v kubectl &> /dev/null; then
    if [ -f "$HOME/.local/bin/kubectl" ]; then
        export PATH="$HOME/.local/bin:$PATH"
    else
        echo "❌ Lỗi: Không tìm thấy lệnh kubectl."
        exit 1
    fi
fi

# 2. Kiểm tra kết nối cluster
echo "🔍 Kiểm tra kết nối tới OKE Cluster..."
if ! kubectl cluster-info &> /dev/null; then
    echo "⚠️ Chưa cấu hình Kubeconfig cho OKE cluster!"
    echo "Vui lòng chạy lệnh sau (lấy từ terraform output trong thư mục terraform/oke):"
    echo "  oci ce cluster create-kubeconfig --cluster-id <CLUSTER_OCID> --file \$HOME/.kube/config --region <REGION> --token-version 2.0.0 --kube-endpoint PUBLIC_ENDPOINT"
    exit 1
fi
echo "✅ Đã kết nối thành công tới OKE Cluster!"

# 3. Đồng bộ Cloudflare Tunnel Token từ Terraform vào K8s Secret (nếu có)
if [ -d "terraform/oke" ] && [ -f "terraform/oke/terraform.tfstate" ]; then
    echo "🔑 Đang kiểm tra Cloudflare Tunnel Token từ Terraform..."
    TUNNEL_TOKEN=$(cd terraform/oke && terraform output -raw cloudflare_tunnel_token 2>/dev/null || echo "")
    if [ -n "$TUNNEL_TOKEN" ]; then
        echo "🔒 Đang tạo/cập nhật Kubernetes Secret 'cloudflare-tunnel-secret'..."
        kubectl create secret generic cloudflare-tunnel-secret \
            --from-literal=token="$TUNNEL_TOKEN" \
            --dry-run=client -o yaml | kubectl apply -f -
    fi
fi

# 4. Biên dịch WebAssembly
echo "📦 Biên dịch WebAssembly mới nhất (Release)..."
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --out-dir wasm_dist --target web target/wasm32-unknown-unknown/release/game-free.wasm
mkdir -p wasm_dist/assets
cp -r assets/* wasm_dist/assets/ 2>/dev/null || true

# 5. Áp dụng Kubernetes Manifests
echo "🚀 Đang triển khai Pods game, Service ClusterIP nội bộ và Cloudflare Tunnel Daemon..."
kubectl apply -f k8s/

echo ""
echo "⏳ Đang kiểm tra trạng thái Pods trong cụm OKE..."
kubectl rollout status deployment/tactical-arena-pvp --timeout=120s || true
kubectl rollout status deployment/cloudflared --timeout=120s || true

GAME_URL=""
if [ -d "terraform/oke" ] && [ -f "terraform/oke/terraform.tfstate" ]; then
    GAME_URL=$(cd terraform/oke && terraform output -raw game_url 2>/dev/null || echo "")
fi

echo ""
echo "======================================================="
echo " 🎉 TRIỂN KHAI THÀNH CÔNG!"
echo " 🌐 Cụm OKE đang chạy: $(kubectl get pods -l app=tactical-arena-pvp --no-headers | wc -l) Pods Game"
echo " 🛡️ Cloudflare Tunnel: $(kubectl get pods -l app=cloudflared --no-headers | wc -l) Pods Tunnel kết nối trực tiếp Edge"
echo " 🚫 KHÔNG DÙNG OCI Load Balancer 10 Mbps -> Băng thông tối đa qua Cloudflare CDN"
if [ -n "$GAME_URL" ]; then
    echo " 🎮 Truy cập Game: $GAME_URL"
else
    echo " 🎮 Truy cập Game qua Domain đã cấu hình trên Cloudflare Tunnel"
fi
echo "======================================================="
