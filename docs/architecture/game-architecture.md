# Kiến trúc game

## Nguyên tắc nguồn sự thật

`crates/data_schema` định nghĩa dữ liệu và balance. `crates/game_logic` định
nghĩa simulation deterministic. `apps/server` là authority cho trận đấu,
progression và reward. `apps/client` chỉ là adapter input + lớp hiển thị Bevy.

Không đặt công thức damage, targeting, rage, turn order hoặc reward trong UI,
VFX, WebSocket handler hay database handler.

## Sơ đồ workspace

```text
crates/core          RNG seed, fixed math, common errors
crates/data_schema   Hero/Skill/Effect/Gacha config structs
crates/game_logic    Board, targeting, combat, effects, replay
crates/protocol      Client/server commands, snapshots, events
        │
        ├── apps/server   HTTP, WebSocket, persistence, authority
        └── apps/client   Bevy input, render, audio, VFX, responsive UI
```

## Data flow trận đấu

```text
formation + seed + config version
              │
              ▼
      server/game_logic simulation
              │
              ├── authoritative result/reward/replay
              └── SimulationEvent stream
                              │
                              ▼
                    client render/audio/UI
```

Client native và WASM có thể dự đoán hoặc phát lại event, nhưng kết quả chính
thức do server simulation quyết định. Replay phải chứa seed, config hash và
version để tái hiện được.

## Boundary theo phase

- **Phase 0–1:** không thay đổi UI combat; chỉ chốt tài liệu và data contract.
- **Phase 2:** `game_logic` chạy headless độc lập với Bevy render/window.
- **Phase 3:** server dùng `game_logic` để validate command và reward.
- **Phase 4:** client chuyển sang consume snapshot/event qua adapter.
- **Phase 5 trở đi:** chỉ thêm content/effect đã có contract và test.

## Persistence và migration

Profile, card, deck, slot unlock, pity, shard và equipment được version hóa.
Migration phải giữ dữ liệu hợp lệ, ghi lại kết quả thành công và báo lỗi rõ khi
gặp ID/config không còn tồn tại; không tự động đổi tướng không rõ nguyên nhân.

## Deploy

Build WASM tạo artifact trong `dist/wasm`; server phục vụ static artifact và
HTTP/WebSocket. Production deploy chỉ chạy ở Phase 8 sau khi test, config
validation, replay parity và smoke check đạt.
