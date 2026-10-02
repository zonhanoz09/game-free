#!/usr/bin/env bash
set -e

# ==============================================================================
# SCRIPT DEPLOY 3V3 TACTICAL ARENA (RUST WEBSOCKET SERVER) LÊN OCI
# ==============================================================================

export PATH="$HOME/.cargo/bin:$PATH"
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_DIR"

echo "======================================================="
echo " 🚀 BẮT ĐẦU TRIỂN KHAI RUST WEBSOCKET SERVER LÊN OCI"
echo "======================================================="

# 1. Build WASM
echo "📦 Đang biên dịch bản WebAssembly mới nhất (Release)..."
cargo build -p game-free --target wasm32-unknown-unknown --release
wasm-bindgen --out-dir dist/wasm --target web target/wasm32-unknown-unknown/release/game-free.wasm
mkdir -p dist/wasm/assets
cp -r assets/* dist/wasm/assets/ 2>/dev/null || true

# 2. Tạo OCI Bastion Session & Thiết lập SSH Tunnel
echo "🛡️ Khởi tạo OCI Bastion Port-Forwarding Session..."
python3 scripts/bastion_session.py

TUNNEL_CMD=$(cat /tmp/bastion_tunnel_cmd.txt)
LOCAL_TUNNEL_PORT=2222

echo "🔗 Đang mở SSH Tunnel qua OCI Bastion..."
# Chạy tunnel ngầm
eval "$TUNNEL_CMD" &
TUNNEL_PID=$!

# Bắt trap để dọn dẹp tunnel khi script kết thúc
cleanup() {
    echo "🧹 Dọn dẹp kết nối Bastion Tunnel..."
    kill $TUNNEL_PID 2>/dev/null || true
    fuser -k ${LOCAL_TUNNEL_PORT}/tcp 2>/dev/null || true
}
trap cleanup EXIT

# Đợi port tunnel sẵn sàng
echo "⏳ Đợi SSH Tunnel sẵn sàng..."
for i in {1..15}; do
    if nc -z 127.0.0.1 ${LOCAL_TUNNEL_PORT} 2>/dev/null; then
        echo "✅ Bastion SSH Tunnel đã kết nối thành công (port ${LOCAL_TUNNEL_PORT})!"
        break
    fi
    sleep 1
done

# 3. Đồng bộ files lên máy chủ
echo "📤 Đang đồng bộ files lên OCI Private Instance (10.0.1.60)..."
rsync -avz -e "ssh -i $HOME/.ssh/id_ed25519 -o StrictHostKeyChecking=no -p ${LOCAL_TUNNEL_PORT}" \
    --exclude ".git" \
    --exclude "target" \
    --exclude "apps/server/target" \
    --exclude "node_modules" \
    --exclude ".terraform" \
    --exclude "*.tfstate*" \
    dist/wasm apps/server deploy/docker/Dockerfile.server Cargo.toml Cargo.lock crates \
    ubuntu@127.0.0.1:/opt/game-free/app/

# 4. Build và khởi động Rust WebSocket Server trên OCI
echo "🦀 Đang biên dịch và khởi động Rust WebSocket Server trên OCI..."
ssh -i "$HOME/.ssh/id_ed25519" -o StrictHostKeyChecking=no -p ${LOCAL_TUNNEL_PORT} \
    ubuntu@127.0.0.1 "
        cd /opt/game-free/app && \
        echo '📦 Đang build Rust Server Docker image...' && \
        sudo docker build -t tactical-arena-rust:latest -f deploy/docker/Dockerfile.server . && \
        sudo docker stop tactical-arena-pvp 2>/dev/null || true && \
        sudo docker rm tactical-arena-pvp 2>/dev/null || true && \
        sudo docker run -d \
            --name tactical-arena-pvp \
            --restart always \
            --network game-free_default \
            --network-alias game-server \
            -p 8080:8080 \
            -v /opt/game-free/app/dist/wasm:/app/dist/wasm \
            -v /opt/game-free/app/data:/app/data \
            -w /app \
            tactical-arena-rust:latest && \
        sudo docker ps
    "

echo "======================================================="
echo " 🎉 DEPLOY THÀNH CÔNG!"
echo " 🌐 Game đang chạy trực tiếp tại: https://game.annhan.me/"
echo " ⚡ Backend: 100% Rust WebSocket Server (Không Firebase)"
echo "======================================================="
