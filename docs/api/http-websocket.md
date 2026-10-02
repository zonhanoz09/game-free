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

## WebSocket

Client `RustPvPClient` tự kết nối tới `/ws`, tự reconnect khi mất kết nối và
gửi/nhận message JSON. Các message gameplay chính gồm:

- `CREATE_ROOM`, `JOIN_ROOM`, `QUICK_MATCH`, `REFRESH_LOBBY`
- `PLAYER_READY` với lineup gồm `col`, `row`, `class`, `star_level`
- `START_BATTLE`, `ROUND_TURN_COMPLETED`, `STRIKE_ACTION`
- `UPDATE_MATCH_HP`, `BATTLE_FINISHED`, `MATCH_END`
- `SET_DECK`, `EXIT_MATCH`, `SET_SPEED`

Bridge WASM dùng `js_to_rust_pvp` để gửi message vào Bevy và
`window._handle_rust_pvp` để nhận event từ Bevy. Dữ liệu card cũ không có
`initiative_bonus` vẫn tương thích nhờ giá trị mặc định bằng 0.
