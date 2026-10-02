# Tactical Arena — 2D CCG Auto-Battler

Game chiến thuật dàn trận tự động trên Bevy 0.15/Rust, chạy native và
WebAssembly. Người chơi thu thập, nâng cấp và xếp thẻ tướng vào ba vị trí,
sau đó đấu PvE hoặc PvP realtime qua Rust WebSocket server.

## Tài liệu

- [Gameplay hiện tại](docs/game_design/gameplay.md)
- [Kiến trúc mã nguồn](docs/architecture/game-architecture.md)
- [HTTP/WebSocket API](docs/api/http-websocket.md)
- [Lộ trình](docs/game_design/roadmap.md)
- [Kiến trúc hạ tầng](docs/infrastructure.md)
- [Triển khai OCI](docs/infrastructure-oci.md)
- [Triển khai OKE](docs/infrastructure-oke.md)
- [Runtime assets](docs/assets.md)
- [Tooling](docs/tooling.md)

## Cấu trúc workspace

```text
apps/client       Bevy client native/WASM
apps/server       Dedicated HTTP/WebSocket server
crates/           Shared libraries và schema
assets/           Runtime assets
data/             Dữ liệu cân bằng raw/generated
deploy/           Docker, Kubernetes, Terraform
dist/wasm         Artifact web do wasm-bindgen tạo
scripts/          Build và deploy automation
docs/             Tài liệu dự án
```

## Lệnh thường dùng

```bash
cargo test --workspace
cargo run -p game-free
./scripts/build-wasm.sh
./scripts/run-local.sh
```

Server phục vụ trên port `8080`. Xem hướng dẫn triển khai trong
[`docs/`](docs/).
