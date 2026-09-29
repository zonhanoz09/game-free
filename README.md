# Chiến Thuật 3x3 - Đấu Trường 3D (3D Auto-Battler)

Game chiến thuật dàn trận tự động xây dựng trên nền tảng **Bevy Engine (Rust)** với **sàn đấu 3D (3D Colosseum Arena)** và **các nhân vật mô hình 3D** chiến đấu trực tiếp trên sàn đấu.

---

## 🏛️ Đấu Trường & Nhân Vật 3D Sinh Động

- **Sàn Đấu 3D (3D Arena Dais)**: 
  - Khối bệ đá đấu trường nổi 3D với viền đồng/vàng cổ kính.
  - Vạch năng lượng ma pháp hoàng kim chia đôi chiến tuyến ở trung tâm.
  - 4 Trụ đá giác đấu đồ sộ ở 4 góc gắn đỉnh chảo lửa bập bùng phát sáng thực tế (**Point Lights & Emissive Fire**).
  - Hệ thống ánh sáng mặt trời (**Directional Light**) đổ bóng thực tế lên các khối nhân vật và sàn đấu.
- **18 Ô Đất Sàn Đấu 3D Tương Tác**:
  - 9 ô xanh lam thẫm phong ấn cho Phe Ta và 9 ô hắc diện thạch đỏ cho Phe Địch.
  - Khi rê chuột vào ô bất kỳ, ô sàn 3D sẽ **nhẹ nhàng nâng cao lên và phát sáng viền neon rực rỡ** nhờ cơ chế Raycasting 3D thời gian thực.
- **5 Binh Chủng Mô Hình 3D Độc Bản Đứng Trên Sàn Đấu**:
  - 🛡️ **Hiệp Sĩ (Knight)**: Giáp thép toàn thân sáng loáng ánh kim, thắt lưng vàng, đại khiên thép hộ vệ tay trái, trường kiếm thép tuốt trần tay phải, mũ thiết giáp gắn lông mao hoàng gia.
  - 🏹 **Cung Thủ (Archer)**: Áo da thợ săn lục bảo, mũ trùm gắn lông vũ, bao tên sau lưng, tay giương trường cung gỗ cùng mũi tên phát sáng sẵn sàng nhắm bắn.
  - 🔮 **Pháp Sư (Mage)**: Áo choàng pháp sư tím huyền bí xòe rộng, nón chóp cao viền sao vàng, tay cầm quyền trượng cổ thụ đỉnh ngọc ma pháp xoay tròn phát quang tím.
  - 🗡️ **Sát Thủ (Assassin)**: Tư thế thủ thế trườn thấp trong bóng tối, giáp da than huyền bí, khăn quàng huyết sắc bay phất phơ, mắt sáng đỏ rực, song đoản đao tẩm độc lục bảo cầm ngược cực ngầu.
  - ✨ **Mục Sư (Cleric)**: Lễ phục nữ tu trắng tinh khôi viền vàng, vòng thánh quang **Golden Halo** lơ lửng xoay tròn trên đỉnh đầu, tay nâng quyền trượng thánh giá mặt trời tỏa hào quang ấm áp.
- **Bệ Tướng & Thanh Máu 3D (3D Overhead Health Bar)**:
  - Dưới chân mỗi tướng là bệ đá tròn viền vòng rune phát sáng (Xanh lam cho Ta, Đỏ cho Địch).
  - Phía trên đầu mỗi tướng là thanh máu 3D nghiêng theo góc nhìn camera, đổi màu linh hoạt theo lượng máu (Xanh lá -> Vàng cam -> Đỏ thẫm).
  - Tướng sống có hoạt ảnh thở phập phồng (Idle Breathing Bobbing) và xoay các vật phẩm ma pháp lơ lửng.

---

## ⚔️ Hiệu Ứng Chiến Đấu 3D

- **Lướt chém cận chiến (Knight & Assassin)**: Tướng lao mình lướt qua mặt sàn đấu 3D theo quỹ đạo vòng cung, vung vũ khí chém kẻ địch và lùi về vị trí. Sát thủ kích hoạt bước bóng đêm nhảy thẳng ra hàng sau với tỷ lệ bạo kích cao.
- **Quỹ đạo đạn đạo 3D (Projectiles)**:
  - Cung thủ bắn ra mũi tên 3D bay theo đường parabol trên không trung cắm thẳng vào mục tiêu yếu máu nhất.
  - Pháp sư phóng quả cầu lôi điện tím phát nổ gây sát thương lan cho toàn bộ hàng địch.
  - Mục sư phóng ngôi sao thánh quang vàng kim hồi phục sinh lực cho đồng minh nguy cấp nhất.
- **Chữ sát thương nhảy 3D (Floating Combat Text)**: Hiển thị lượng sát thương, đòn chí mạng hoặc hồi máu trôi dần lên không trung trên đầu các tướng.
- **Gục ngã 3D**: Tướng bị hạ gục sẽ chìm dần xuống lòng đất sàn đấu đá cẩm thạch.

---

## 🎮 Cách Chơi & Điều Khiển

### 1. Bố cục sàn đấu:
- **Bên Trái (Phe Ta)**:
  - **Hậu phương (Cột 0)**: Đặt tướng tầm xa, hỗ trợ (Cung thủ, Pháp sư, Mục sư).
  - **Trung tuyến (Cột 1)**: Vị trí linh hoạt hoặc Sát thủ.
  - **Tiền tuyến (Cột 2)**: Tiếp giáp vạch chia tâm, ưu tiên Hiệp sĩ chống chịu.
- **Bên Phải (Phe Địch)**: Đội hình quái/AI gồm 5 Màn chơi từ dễ đến Trùm Cuối Hoàng Kim.

### 2. Thao tác chuột:
- **Chuột trái**: 
  - Chọn tướng ở thanh tướng dưới đáy màn hình.
  - Click vào ô sàn đấu 3D bên phe ta (tối đa 5 quân) để đặt hoặc đổi quân.
- **Chuột phải**: Click vào ô đã có quân để gỡ quân khỏi sàn đấu.
- **Nút "Đội hình mẫu"**: Tự động dàn nhanh đội hình chuẩn mẫu 5 tướng lên sàn đấu.
- **Nút "Xóa hết"**: Gỡ toàn bộ quân ta trên sàn đấu.
- **Nút "BẮT ĐẦU CHIẾN ĐẤU"**: Khởi động trận chiến 3D tự động.
- **Nút "Tốc độ: 1x / 2x"**: Tăng tốc trận đánh.
- **Nút "MÀN TIẾP THEO >>"**: Sau khi diệt sạch địch, chuyển sang màn kế tiếp.

---

## 🚀 Khởi Chạy Game

Từ thư mục dự án `D:\src\game-free`:

```powershell
cargo run
```
hoặc mở trực tiếp file exe đã biên dịch:
```powershell
.\target\debug\game-free.exe
```
