# Lộ trình phát triển

**Tactical Arena** là game CCG dàn trận tự động chạy trên Bevy 0.15/Rust,
với client native/WASM và server Axum.

## Trạng thái hiện tại

### Đã hoàn thành

- Client Bevy chạy native và WebAssembly; build chuẩn dùng
  `scripts/build-wasm.sh`.
- PvE stage 1–10 và chế độ endless sau stage 10.
- PvP 1v1 qua room code hoặc quick match, đồng bộ bằng WebSocket.
- Collection, shop, nâng cấp, foil, bán thẻ và lưu deck qua HTTP API.
- UI web có tab **Kho thẻ** và **Đội hình**.
- Đội hình gồm ba vị trí:
  - **Đầu:** chỉ `Đỡ Đòn`, +25% HP.
  - **Giữa:** chỉ `Sát Thủ` hoặc `Xạ Thủ`, +20% ATK.
  - **Cuối:** chỉ `Hỗ Trợ` hoặc `Pháp Sư`, +30 initiative/speed.
- Server Rust/Axum phục vụ HTTP, WebSocket và `dist/wasm`.
- Local Docker Compose và triển khai OCI qua Bastion/Cloudflare đã có.

### Đang phát triển / chưa bắt đầu

1. **Trang bị và cổ vật**
   - Data item, inventory, rơi vật phẩm theo mốc PvE và gắn item cho thẻ.
2. **Combat nâng cao**
   - Buff/debuff như stun, burn, poison, taunt.
   - Thống kê damage/healing sau trận.
3. **Mở rộng thẻ và synergy**
   - Thêm class/thẻ mới và hệ tộc có mốc kích hoạt.
4. **Âm thanh và tùy chọn**
   - BGM theo phase, SFX bus, volume/mute và accessibility options.
5. **Tiến trình tài khoản**
   - Save/load tiến trình PvE, achievement và đồng bộ dữ liệu dài hạn.

## Nguyên tắc triển khai

- Luật gameplay dùng kiểu dữ liệu rõ ràng và giữ tương thích với payload deck
  cũ.
- UI dùng layout responsive, card compact và validate role trước khi sync deck.
- Tính năng PvP mới phải cập nhật đồng thời HTTP/WebSocket API và tài liệu
  [`docs/api/http-websocket.md`](../api/http-websocket.md).
- Tính năng dùng chung cho client/server ưu tiên đặt trong `crates/`; các crate
  hiện là nền tảng ban đầu, chưa thay thế toàn bộ module app-local.

## Kiểm tra trước khi phát hành

```bash
cargo test --workspace
cargo check -p game-free --target wasm32-unknown-unknown
./scripts/build-wasm.sh
git diff --check
```
