# Gameplay hiện tại

## Vòng lặp chính

1. Người chơi vào sảnh web, đăng nhập hoặc chơi guest.
2. Mở **Kho thẻ bài** để xem collection, lọc theo vai trò/phẩm cấp, tìm kiếm,
   nâng cấp cấp độ/sao, bán thẻ hoặc foil thẻ.
3. Mở **Đội hình ra trận** và xếp tối đa ba thẻ vào đúng vị trí.
4. Chọn PvE để đấu AI hoặc PvP để ghép trận nhanh/tạo phòng/tham gia bằng mã phòng.
5. Trận đấu diễn ra theo action gauge (ATB). Kết quả cập nhật phần thưởng, ELO và
   tiến trình người chơi.

## Năm lớp tướng

| Lớp | Vai trò | Hành vi chính |
|---|---|---|
| Knight | Đỡ Đòn | Đánh tuyến trước, chịu sát thương |
| Archer | Xạ Thủ | Bắn mục tiêu có HP thấp |
| Mage | Pháp Sư | Sát thương phép lan theo hàng |
| Assassin | Sát Thủ | Đột kích tuyến sau, có chí mạng |
| Cleric | Hỗ Trợ | Hồi máu đồng minh nguy cấp |

## Luật đội hình và bonus vị trí

| Vị trí | Vai trò hợp lệ | Bonus |
|---|---|---|
| Tiên phong | Đỡ Đòn | +25% HP hiệu dụng |
| Chủ lực | Sát Thủ, Xạ Thủ | +20% ATK hiệu dụng |
| Hỗ trợ | Hỗ Trợ, Pháp Sư | +30 Speed/tiên cơ |

UI chặn việc xếp sai vai trò. Khi đồng bộ sang Bevy, bonus được truyền vào
`DeckCardData` và cộng trực tiếp vào `UnitStats` lúc spawn. Thẻ không nằm trong
ba vị trí vẫn được giữ làm dự bị.

## Combat

Combat dùng action gauge dựa trên `speed`; tướng đầy gauge sẽ hành động. Hệ thống
hiện có HP, ATK, DEF, mana, speed, crit, khiên, hồi máu, mục tiêu theo lớp,
floating combat text, hit-stop, camera shake, projectile và hiệu ứng va chạm.

PvE có các stage tăng dần, stage 10 có boss và các stage sau đó chuyển sang
endless. PvP truyền lineup, vị trí, class và sao qua WebSocket; server xử lý
phòng, trạng thái ready, round và sát thương giữa hai người chơi.

## Collection và kinh tế

Mỗi `UserCard` có id, hero class, sao, level, HP/ATK bonus, starter/foil và
quantity. Người chơi có thể mua thẻ bằng gold, nâng cấp level/sao, foil bằng
gems và bán thẻ không phải starter. Deck được lưu theo `DeckCardEntry` và
được server validate trước khi lưu.

## UI web

`dist/wasm/index.html` chứa lobby, modal profile/shop/collection/deck/PvP,
leaderboard, result screen và bridge JavaScript ↔ Bevy WASM. Modal deck đã tách
thành hai tab **Kho thẻ bài** và **Đội hình ra trận** để giảm mật độ card.
