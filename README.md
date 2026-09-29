# Chiến Thuật 3x3 - Dàn Trận Đối Kháng (Auto-Battler)

Game chiến thuật dàn trận tự động xây dựng trên nền tảng **Bevy Engine (Rust)** với bàn cờ 3x3 đối kháng cùng đồ họa Fantasy sinh động.

---

## 🎨 Đồ Họa & Giao Diện Mới

- **Đấu trường Arena**: Phông nền đấu trường đấu trường cổ kính với vòng tròn phong ấn ma thuật rực rỡ ở tâm.
- **5 Bộ Chân Dung Binh Chủng (Avatar Art)**:
  - 🛡️ **Hiệp Sĩ (Knight)**: Giáp thép hộ vệ, khiên ánh kim kiên cố.
  - 🏹 **Cung Thủ (Archer)**: Thợ săn bóng đêm với cung tên ngọc bích phát sáng.
  - 🔮 **Pháp Sư (Mage)**: Đại pháp sư áo tím ma thuật phóng sét lan tỏa.
  - 🗡️ **Sát Thủ (Assassin)**: Thích khách mặt nạ huyết sắc với song đao tẩm độc.
  - ✨ **Mục Sư (Cleric)**: Nữ tu thánh điện với quyền trượng ánh sáng ban phước hồi phục.
- **Thẻ Bài Tướng**: Khung viền phát sáng (Xanh lam cho Phe Ta, Đỏ cho Phe Địch) kèm nhãn tên và thanh máu trực quan.
- **Thanh Đặt Quân Bổ Sung Ảnh Đại Diện (Thumbnails)**: Trực quan, dễ quan sát và chọn quân.

---

## 🎮 Cách Chơi & Luật Game

### 1. Bàn cờ 3x3 mỗi bên:
- **Bên Trái (Phe Ta)**: Lưới 3x3 gồm:
  - **Hậu phương (Cột 0)**: Phù hợp đặt Cung thủ, Pháp sư, Mục sư.
  - **Trung tuyến (Cột 1)**: Đội hình hỗ trợ hoặc sát thủ.
  - **Tiền tuyến (Cột 2)**: Vị trí tiếp giáp địch, ưu tiên Hiệp sĩ (Tanker).
- **Bên Phải (Phe Địch)**: Lưới 3x3 của máy/AI theo từng màn chơi.

### 2. Các Binh Chủng:
1. **Hiệp Sĩ (KNG)**: HP cao (160), giáp dày, đánh cận chiến (lướt kiếm chém).
2. **Cung Thủ (ARC)**: Tấn công tầm xa, tự động bắn mục tiêu **thấp máu nhất**.
3. **Pháp Sư (MAG)**: Bắn quả cầu ma pháp gây sát thương mục tiêu chính đồng thời **gây 45% sát thương lan cho toàn bộ kẻ địch cùng hàng**.
4. **Sát Thủ (ASN)**: Tốc độ cao, bạo kích 35%, **nhảy thẳng ra hàng sau (hậu phương) của địch**.
5. **Mục Sư (CLR)**: Tự động tìm đồng minh thấp máu nhất để **hồi máu**.

### 3. Điều khiển:
- **Chuột trái**: 
  - Chọn binh chủng ở thanh bên dưới.
  - Click vào ô bên lưới phe ta (tối đa 5 quân) để đặt hoặc đổi quân.
- **Chuột phải**: Click vào ô đã có quân để gỡ bỏ.
- **Nút "Doi hinh mau"**: Tự động xếp nhanh đội hình chuẩn.
- **Nút "Xoa het"**: Xóa sạch toàn bộ quân ta trên bàn cờ.
- **Nút "BAT DAU CHIEN DAU"**: Bắt đầu trận đấu tự động.
- **Nút "Toc do: 1x / 2x"**: Thay đổi tốc độ trận đánh.
- **Nút "MAN TIEP THEO >>"**: Khi thắng, mở khóa màn chơi mới (Stage 1 đến Stage 5 Trùm Cuối).

---

## 🚀 Khởi Chạy Game

Từ thư mục dự án `D:\src\game-free`:

```powershell
cargo run
```
hoặc mở trực tiếp file exe:
```powershell
.\target\debug\game-free.exe
```