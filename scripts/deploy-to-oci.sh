#!/usr/bin/env bash
set -e

# ==============================================================================
# Script tự động đồng bộ mã nguồn & khởi chạy Game Server lên Oracle Cloud (OCI)
# ==============================================================================

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_DIR"

echo "======================================================="
echo " 🚀 BẮT ĐẦU TRIỂN KHAI 3V3 TACTICAL ARENA LÊN OCI"
echo "======================================================="

# 1. Lấy IP máy chủ từ Terraform hoặc tham số dòng lệnh
SERVER_IP="$1"
if [ -z "$SERVER_IP" ]; then
    if [ -d "terraform" ] && [ -f "terraform/terraform.tfstate" ]; then
        echo "🔍 Đang lấy Public IP từ Terraform State..."
        SERVER_IP=$(cd terraform && terraform output -raw server_public_ip 2>/dev/null || echo "")
    fi
fi

if [ -z "$SERVER_IP" ]; then
    echo "❌ Lỗi: Không tìm thấy IP máy chủ."
    echo "Cách dùng: ./scripts/deploy-to-oci.sh <IP_MÁY_CHỦ_OCI>"
    exit 1
fi

SSH_USER="ubuntu"
SSH_KEY="$HOME/.ssh/id_ed25519"
SSH_CMD="ssh -i $SSH_KEY -o StrictHostKeyChecking=no -o ConnectTimeout=15"
REMOTE_TARGET="${SSH_USER}@${SERVER_IP}"
REMOTE_APP_DIR="/opt/game-free/app"

echo "🎯 Máy chủ đích: $REMOTE_TARGET"

# 2. Biên dịch WASM mới nhất trước khi deploy
echo "📦 Đang biên dịch bản WebAssembly mới nhất (Release)..."
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --out-dir wasm_dist --target web target/wasm32-unknown-unknown/release/game-free.wasm
mkdir -p wasm_dist/assets
cp -r assets/* wasm_dist/assets/ 2>/dev/null || true

# 3. Kiểm tra kết nối SSH
echo "🔑 Đang kiểm tra kết nối SSH tới $REMOTE_TARGET..."
$SSH_CMD "$REMOTE_TARGET" "echo '✅ Kết nối SSH thành công!'"

# 4. Đợi cloud-init hoàn tất (nếu máy ảo vừa tạo)
echo "⏳ Đợi hệ thống Cloud-Init trên OCI hoàn tất cấu hình Docker..."
$SSH_CMD "$REMOTE_TARGET" "cloud-init status --wait 2>/dev/null || true"

# 5. Tạo thư mục đích trên máy chủ
$SSH_CMD "$REMOTE_TARGET" "sudo mkdir -p $REMOTE_APP_DIR && sudo chown -R $SSH_USER:$SSH_USER /opt/game-free"

# 6. Đồng bộ files lên máy chủ
echo "📤 Đang đồng bộ files lên $REMOTE_TARGET:$REMOTE_APP_DIR..."
rsync -avz --delete -e "$SSH_CMD" \
    --exclude '.git' \
    --exclude 'target' \
    --exclude 'node_modules' \
    --exclude 'terraform' \
    "$PROJECT_DIR/server.js" \
    "$PROJECT_DIR/package.json" \
    "$PROJECT_DIR/package-lock.json" \
    "$PROJECT_DIR/wasm_dist" \
    "$PROJECT_DIR/assets" \
    "$REMOTE_TARGET:$REMOTE_APP_DIR/"

# 7. Cài đặt npm dependencies và khởi động Docker Container trên OCI
echo "⚙️ Đang cài đặt node modules và khởi động Docker Compose..."
$SSH_CMD "$REMOTE_TARGET" bash << 'EOF'
set -e
cd /opt/game-free/app

# Cài đặt node_modules bằng container node:20-alpine
sudo docker run --rm -v /opt/game-free/app:/app -w /app node:20-alpine npm install --omit=dev

# Khởi chạy Docker Compose (gồm game-server và cloudflared tunnel)
cd /opt/game-free
sudo docker compose down 2>/dev/null || true
sudo docker compose up -d --force-recreate
echo "🐳 Docker Container đã khởi chạy thành công!"
sudo docker compose ps
EOF

echo ""
echo "======================================================="
echo " 🎉 TRIỂN KHAI THÀNH CÔNG!"
echo " 🌐 Địa chỉ truy cập trực tiếp: http://${SERVER_IP}:8080"
echo " 🛡️ Địa chỉ qua Cloudflare Tunnel: https://game.annhan.me"
echo "======================================================="
