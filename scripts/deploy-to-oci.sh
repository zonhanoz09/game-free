#!/usr/bin/env bash
set -e

# ==============================================================================
# Script tự động đồng bộ mã nguồn & khởi chạy Game Server lên OCI qua Bastion
# ==============================================================================

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_DIR"

echo "======================================================="
echo " 🚀 BẮT ĐẦU TRIỂN KHAI 3V3 TACTICAL ARENA LÊN OCI"
echo "======================================================="

# 1. Biên dịch WASM mới nhất
echo "📦 Đang biên dịch bản WebAssembly mới nhất (Release)..."
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --out-dir wasm_dist --target web target/wasm32-unknown-unknown/release/game-free.wasm
mkdir -p wasm_dist/assets
cp -r assets/* wasm_dist/assets/ 2>/dev/null || true

# 2. Tạo OCI Bastion Session & Thiết lập SSH Tunnel
echo "🛡️ Khởi tạo OCI Bastion Port-Forwarding Session..."
python3 scripts/bastion_session.py

LOCAL_TUNNEL_PORT=2222
fuser -k ${LOCAL_TUNNEL_PORT}/tcp 2>/dev/null || true

BASTION_CMD=$(cat /tmp/bastion_tunnel_cmd.txt)
echo "🔗 Đang mở SSH Tunnel qua OCI Bastion..."
eval "$BASTION_CMD &"
TUNNEL_PID=$!
sleep 2

cleanup() {
    echo "🧹 Dọn dẹp kết nối Bastion Tunnel..."
    kill $TUNNEL_PID 2>/dev/null || true
    fuser -k ${LOCAL_TUNNEL_PORT}/tcp 2>/dev/null || true
}
trap cleanup EXIT

# 3. Đồng bộ files lên máy chủ
echo "📤 Đang đồng bộ files lên OCI Private Instance (10.0.1.60)..."
rsync -avz -e "ssh -i $HOME/.ssh/id_ed25519 -o StrictHostKeyChecking=no -p ${LOCAL_TUNNEL_PORT}" \
    --exclude ".git" \
    --exclude "target" \
    --exclude "node_modules" \
    --exclude ".terraform" \
    --exclude "*.tfstate*" \
    wasm_dist server.js package.json \
    ubuntu@127.0.0.1:/opt/game-free/app/

# 4. Khởi động lại Docker container
echo "🔄 Khởi động lại dịch vụ game trên OCI..."
ssh -i "$HOME/.ssh/id_ed25519" -o StrictHostKeyChecking=no -p ${LOCAL_TUNNEL_PORT} \
    ubuntu@127.0.0.1 "sudo docker restart tactical-arena-pvp && sudo docker ps"

echo "======================================================="
echo " 🎉 DEPLOY THÀNH CÔNG!"
echo " 🌐 Game đang chạy trực tiếp tại: https://game.annhan.me/"
echo "======================================================="
