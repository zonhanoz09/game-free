# Kiến trúc game

## Sơ đồ workspace

```text
apps/client/                 Bevy client native/WASM
  src/main.rs                App, plugins và system schedule
  src/types.rs               GameState, UnitClass, UnitStats, resources
  src/board.rs               Bàn đấu 3x3, hover và vị trí quân
  src/units.rs               Spawn unit, HP/action bars, stat bonus
  src/battle.rs              ATB, target selection, combat/VFX
  src/ui.rs                  HUD, shop, bench, placement và result
  src/net.rs                 WebSocket bridge và deck sync
apps/server/                 Axum HTTP/WebSocket dedicated server
crates/core/                 Hằng số và tiện ích nền
crates/protocol/             Schema mạng dùng chung
crates/game_logic/           Rule deterministic dùng chung
crates/data_schema/          Schema dữ liệu cân bằng
dist/wasm/                   HTML/JS/WASM/assets production
```

## Client runtime

Bevy khởi tạo camera, board, UI, stage và network bridge. `GameState` điều khiển
các pha placement, battle, victory và defeat. Board dùng lưới 3x3 mỗi phe;
người chơi có thể đặt tối đa năm unit trên bàn và dùng bench để dự bị.

`UnitStats` chứa `max_hp`, `hp`, `mana`, `max_mana`, `atk`, `def`, `speed`,
`crit_rate` và `shield`. Hàm spawn nhận bonus từ card, gồm HP, ATK và initiative
(được cộng vào speed). `battle.rs` đọc action gauge để quyết định lượt, thay vì
đồng bộ từng frame với server.

## Web client và server

Server Axum vừa phục vụ API tài khoản/collection/deck, vừa chạy WebSocket PvP,
và fallback static tới `dist/wasm`. Dữ liệu người dùng hiện được serialize theo
schema trong `apps/server/src/main.rs`; protocol dùng `serde` JSON.

Browser UI quản lý lobby, modal, collection, đội hình, shop, profile,
leaderboard và kết quả trận. JavaScript gọi API HTTP, còn bridge WASM chuyển
`SET_DECK`, `START_BATTLE` và các event PvP vào Bevy.

## Luồng deploy

```text
scripts/build-wasm.sh
        │
        ▼
dist/wasm + assets
        │
        ├── deploy/docker/Dockerfile.server
        ├── deploy/docker/docker-compose.yml
        └── deploy/k8s hoặc deploy/terraform
```

OCI dùng Bastion tunnel để sync `dist/wasm`, `apps/server` và Docker context.
OKE dùng manifests trong `deploy/k8s/` và Cloudflare Tunnel để expose HTTPS/WSS.
