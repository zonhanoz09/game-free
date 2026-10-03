# HTTP và WebSocket API

Server chạy ở port `8080`, phục vụ static web từ `dist/wasm` và endpoint WebSocket
`/ws`. Các route HTTP chính:

| Method | Route | Mục đích |
|---|---|---|
| POST | `/api/auth/register` | Tạo tài khoản |
| POST | `/api/auth/login` | Đăng nhập |
| GET | `/api/user/profile` | Lấy profile và collection |
| POST | `/api/user/customize` | Đổi avatar/cardback/board skin |
| GET | `/api/decks` | Lấy các deck |
| POST | `/api/decks/save` | Validate và lưu deck |
| POST | `/api/decks/delete` | Xóa deck |
| POST | `/api/cards/foil` | Foil card |
| POST | `/api/cards/upgrade` | Nâng level hoặc sao |
| POST | `/api/cards/sell` | Bán card |
| POST | `/api/shop/buy_card` | Mua card |
| GET | `/api/leaderboard` | Bảng xếp hạng |
| POST | `/api/match/reward` | Nhận reward PvE |
| POST | `/api/match/record` | Ghi nhận trận đấu |
| POST | `/api/gacha/pull` | Triệu hồi tướng (Gacha, soft pity 50, hard pity 70) |
| GET | `/api/progression` | Lấy tiến trình người chơi (tiền, pity, tướng, mảnh) |
| POST | `/api/progression/upgrade_star` | Nâng sao tướng bằng mảnh (1 đến 7 sao) |
| POST | `/api/progression/upgrade_level` | Nâng cấp level tướng |
| GET | `/api/schema/master_data` | Lấy cấu hình master data (rarities, line buffs, skills, templates) |
| POST | `/api/formation/calculate_stats` | Tính toán chỉ số tướng theo Level, Sao, Slot 3x3 và Line Buffs/Penalties |
| GET | `/api/formation/layout` | Lấy bố trí đội hình lưới 3x3 (Slot 1..9) |
| POST | `/api/formation/save_layout` | Lưu bố trí đội hình lưới 3x3 (Slot 1..9) |

## WebSocket

Client `RustPvPClient` tự kết nối tới `/ws`, tự reconnect khi mất kết nối và
gửi/nhận message JSON. Các message gameplay chính gồm:

- `CREATE_ROOM`, `JOIN_ROOM`, `QUICK_MATCH`, `REFRESH_LOBBY`
- `PLAYER_READY` với lineup gồm `col`, `row`, `class`, `star_level`
- `START_BATTLE`, `ROUND_TURN_COMPLETED`, `STRIKE_ACTION`
- `UPDATE_MATCH_HP`, `MATCH_END`
- `BATTLE_COMMAND` (hoặc `{ type: "BATTLE_COMMAND", command: ... }`) với
  `command_id`, `battle_id`, `turn`, `actor_id`, `target_rule`,
  `damage_rate_bps`, `rage_cost`, `critical`
- `BATTLE_RESULT` chứa `winner`, `turn` và replay gồm `seed`,
  `config_version`, `config_fingerprint`, `events`
- `SET_DECK`, `EXIT_MATCH`, `SET_SPEED`
- `GACHA_PULL` (client gửi số lượt `count`), nhận `GACHA_PULL_RESULT` chứa danh sách tướng/mảnh, pity counter và tiền còn lại
- `HERO_UPGRADE_STAR`, nhận `HERO_UPGRADE_STAR_RESULT`
- `HERO_UPGRADE_LEVEL`, nhận `HERO_UPGRADE_LEVEL_RESULT`
- `PROGRESSION_SYNC` đồng bộ toàn bộ tiến trình người chơi

Khi hai người chơi cùng `PLAYER_READY`, server tạo authoritative battle với
`config_version = combat-v1`, fingerprint `release-v1` và seed ổn định theo
room/round. Client chỉ gửi command; server bắt buộc fingerprint đúng, kiểm tra
battle ID, turn, actor, target rule, skill policy và timeout, sau đó chạy
`game_logic` để tạo event/replay. `command_id` được lưu để retry không chạy lại
lượt; `critical` trong payload bị bỏ qua và không có hiệu lực. Kết quả, damage
meta-round, reward và `MATCH_END` chỉ phát sinh từ authority server.
`BATTLE_FINISHED` từ client bị từ chối.

Bridge WASM dùng `js_to_rust_pvp` để gửi message vào Bevy và
`window._handle_rust_pvp` để nhận event từ Bevy. Dữ liệu card cũ không có
`initiative_bonus` vẫn tương thích nhờ giá trị mặc định bằng 0.
