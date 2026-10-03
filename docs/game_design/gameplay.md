# Gameplay hiện hành

## Trạng thái sản phẩm

Game là đấu trường thẻ tướng Tam Quốc 3x3. Người chơi thu thập tướng, chọn tối
đa 5 tướng ra trận, sắp xếp vào 9 ô, sau đó đấu PvE hoặc PvP theo thanh hành
động. Roster hiện tại dùng các ID danh tướng Tam Quốc; các archetype cũ chỉ còn
là adapter trình bày/tương thích trong lúc refactor combat engine.

**Trạng thái triển khai:** collection, shop, deck, slot unlock, formation UI,
PvE và PvP transport đã có. Combat headless dùng chung, server-authoritative
replay/reward và skill data-driven vẫn đang chờ các phase tương ứng; không mô tả
chúng như hành vi đã hoàn tất.

## Vòng lặp người chơi

1. Đăng nhập hoặc chơi guest.
2. Mở kho thẻ để xem, mua và quản lý tướng.
3. Mở đội hình, chọn tối đa 5 tướng và đặt vào lưới 3x3.
4. Chọn PvE hoặc PvP.
5. Trận đấu chạy theo action value; tướng đủ thanh hành động sẽ ra đòn hoặc
   dùng tuyệt kỹ khi đủ 100 Nộ.
6. Nhận kết quả và phần thưởng do server xác thực.

## Bàn cờ và đội hình

Mỗi phe có 9 ô. `col=0` là tiền phong, `col=1` là trung quân, `col=2` là hậu
phương; `row=0/1/2` lần lượt là trên/giữa/dưới. Slot ID được tính:

```text
slot_id = col * 3 + row + 1
```

Đội hình được lưu theo slot, tối đa 5 tướng, mặc định mở 1 slot; các slot tiếp
theo mua bằng tiền trong cửa hàng. UI đội hình và UI trận đấu dùng cùng một
layout 3x3.

## Combat

- Action value đạt 10000 theo tốc độ `SPD` sẽ kích hoạt lượt.
- Đánh thường trúng mục tiêu: +25 Nộ.
- Nhận sát thương trực tiếp: +15 Nộ.
- Hạ gục mục tiêu: +20 Nộ.
- Đủ 100 Nộ ở đầu lượt: bắt buộc dùng tuyệt kỹ và tiêu hao 100 Nộ.
- Targeting mặc định ưu tiên cùng tầng, sau đó dùng khoảng cách Manhattan và
  tie-break tầng giữa.
- Các rule đặc biệt gồm hậu phương, mục tiêu thấp HP, mục tiêu ATK cao, hàng,
  cột và hình dấu thập.

Luật chi tiết và công thức chuẩn nằm trong
[`system_game.md`](system_game.md). Khi combat engine được tách xong, client chỉ
render simulation events; client không tự tính damage, target hoặc reward.

## Roster và collection

Roster hiện tại gồm Triệu Vân, Hoàng Trung, Gia Cát Lượng, Tào Tháo, Điển Vi,
Quách Gia, Tôn Sách, Lục Tốn, Đại Kiều & Tiểu Kiều, Trương Cáp & Nhan Lương,
Hoa Đà và Giả Hủ. Mỗi thẻ có identity, faction, role, rarity, level, sao và
skill data.

Các kỹ năng chỉ được hiển thị là khả dụng khi effect tương ứng đã được mô phỏng
và kiểm thử trong shared engine. Không thêm skill riêng trong UI hoặc Bevy
system.

## PvE, PvP và authority

PvE có stage tăng dần và endless sau stage cuối. PvP truyền đội hình, slot và
trạng thái qua WebSocket. Mục tiêu kiến trúc là server chạy simulation
headless, xác thực replay và tự tính kết quả, phần thưởng, ELO; hiện tại đây là
phần đang refactor, vì vậy client chưa được coi là nguồn authority cuối cùng
cho combat.

## Phạm vi chưa kích hoạt

Gacha pity, nâng sao bằng mảnh, tactics, equipment và faction synergy là các
phase sau của roadmap. Không coi chúng là gameplay hiện hành cho đến khi engine,
protocol, persistence và UI cùng đạt cổng nghiệm thu.
