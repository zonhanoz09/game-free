# Kiến trúc game

## Sơ đồ workspace

```text
apps/client/                 Bevy client native/WASM
  src/main.rs                Entry point tối giản
  src/app.rs                 Composition root, plugin/resource/system wiring
  src/types.rs               GameState, UnitClass, UnitStats, resources
  src/board.rs               Bàn đấu 3x3, hover và vị trí quân
  src/units.rs               Spawn unit, HP/action bars, stat bonus
  src/battle/                Battle state, turn selection, animations, VFX và outcomes
  src/ui/                    UI components, setup, placement, input và result
  src/net.rs                 WebSocket bridge và deck sync
apps/server/                 Axum HTTP/WebSocket dedicated server
  src/main.rs                Application composition root và route wiring
  src/models.rs              User/card/deck models, persistence và domain services
  src/multiplayer.rs         Room, player session và matchmaking state
  src/api.rs                 REST request DTOs và HTTP handlers
  src/websocket.rs           WebSocket lifecycle và PvP message handling
  src/tests.rs               Server domain tests
  src/config.rs              Defaults và clock abstraction cho persistence
crates/core/                 Hằng số và tiện ích nền
crates/protocol/             Schema mạng dùng chung
crates/game_logic/           Rule deterministic dùng chung
crates/data_schema/          Schema dữ liệu cân bằng
dist/wasm/                   HTML/JS/WASM/assets production
```

## Client runtime

`main.rs` chỉ xử lý platform bootstrap; `app.rs` là composition root duy nhất để
đăng ký plugin, resource và system theo từng phase. Bevy khởi tạo camera, board,
UI, stage và network bridge. `GameState` điều khiển
các pha placement, battle, victory và defeat. Board dùng lưới 3x3 mỗi phe;
người chơi có thể đặt tối đa năm unit trên bàn và dùng bench để dự bị.

`UnitStats` chứa `max_hp`, `hp`, `mana`, `max_mana`, `atk`, `def`, `speed`,
`crit_rate` và `shield`. Hàm spawn nhận bonus từ card, gồm HP, ATK và initiative
(được cộng vào speed). `battle.rs` đọc action gauge để quyết định lượt, thay vì
đồng bộ từng frame với server.

## Web client và server

Server Axum vừa phục vụ API tài khoản/collection/deck, vừa chạy WebSocket PvP,
và fallback static tới `dist/wasm`. Composition root chỉ lắp route và state;
models, REST API, multiplayer và WebSocket nằm ở các module riêng. Dữ liệu
người dùng hiện được serialize theo schema domain trong server; các giá trị mặc
định và thời gian được gom vào `config.rs`; protocol dùng `serde` JSON.

## Nguyên tắc tổ chức code

- **Composition root:** chỉ `app.rs`/`main.rs` được wiring hệ thống; module gameplay
  chỉ cung cấp system/resource và không tự khởi tạo application.
- **Single responsibility:** board, units, combat, economy, UI và network giữ
  boundary riêng; thay đổi một feature không cần sửa toàn bộ entrypoint.
- **DRY/KISS:** dùng helper/domain type hiện có, tránh thêm abstraction khi chưa
  có quy tắc hoặc hành vi cần dùng chung.
- **SOLID:** phụ thuộc hướng vào types/domain; adapter HTTP, WebSocket và WASM
  chỉ chuyển đổi dữ liệu, không chứa luật combat.

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
