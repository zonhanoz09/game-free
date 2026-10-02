# Chiến Thuật 3x3 - Đấu Trường 2D (2D Tactical Auto-Battler)

## Cấu trúc workspace

```text
apps/client       Bevy client native/WASM
apps/server       Dedicated HTTP/WebSocket server
crates/           Các thư viện dùng chung và schema
assets/           Runtime assets
data/             Dữ liệu cân bằng raw/generated
deploy/           Docker, Kubernetes, Terraform và local compose
dist/wasm         Artifact web được tạo bởi wasm-bindgen
scripts/          Build/deploy automation
```

Build WASM và đồng bộ asset:

```bash
./scripts/build-wasm.sh
```

Chạy kiểm tra workspace:

```bash
cargo test --workspace
```

Game chiến thuật dàn trận tự động xây dựng trên nền tảng **Bevy Engine 0.15 (Rust)** với **sàn đấu 2D (2D Tactical Arena)**, đồ họa nhân vật sprite sắc nét và hệ thống combat thời gian thực (ATB).

---

## 🏛️ Đấu Trường & Đồ Họa 2D Đẹp Mắt

- **Sàn Đấu 2D (2D Arena Dais)**:
  - Khối bệ đá đấu trường 2D với viền đồng/vàng cổ kính.
  - Hình nền đấu trường nghệ thuật (`assets/textures/background.png`).
  - Vạch năng lượng ma pháp hoàng kim chia đôi chiến tuyến ở trung tâm.
  - 4 Đuốc lửa đấu trường ở 4 góc với hiệu ứng lửa bập bùng và tàn lửa (ember particles) bay lơ lửng.
- **18 Ô Đất Sàn Đấu 2D Tương Tác**:
  - 9 ô xanh lam thẫm phong ấn cho Phe Ta và 9 ô hắc diện thạch đỏ cho Phe Địch.
  - Khi rê chuột vào ô bất kỳ, ô sàn 2D sẽ **phóng to nhẹ (1.05x) và phát sáng viền neon rực rỡ** (Cyan cho Ta, Cam/Đỏ cho Địch).
- **5 Binh Chủng Thẻ Bài / Sprite 2D Sắc Nét Đứng Trên Sàn Đấu**:
  - 🛡️ **Hiệp Sĩ (Knight)**: Giáp thép toàn thân sáng loáng, đại khiên hộ vệ, trường kiếm thép tuốt trần.
  - 🏹 **Cung Thủ (Archer)**: Áo da thợ săn lục bảo, trường cung gỗ cùng mũi tên phát sáng sẵn sàng nhắm bắn kẻ yếu máu nhất.
  - 🔮 **Pháp Sư (Mage)**: Áo choàng pháp sư tím huyền bí, quyền trượng cổ thụ đỉnh ngọc ma pháp phóng lôi điện lan toàn hàng.
  - 🗡️ **Sát Thủ (Assassin)**: Tư thế luồn bóng đêm, song đoản đao tẩm độc lục bảo, nhắm thẳng hàng sau địch với tỷ lệ bạo kích cực cao.
  - ✨ **Mục Sư (Cleric)**: Lễ phục nữ tu trắng tinh khôi viền vàng, hồi phục lượng máu lớn cho đồng minh bị thương nặng nhất.
- **Thanh Máu & Năng Lượng 2D Kép (Overhead Dual Bars)**:
  - Phía trên mỗi tướng có thanh máu động (Xanh lá -> Vàng cam -> Đỏ thẫm khi thấp máu).
  - Thanh năng lượng xanh lơ (Action Gauge) tích lũy theo chỉ số Tốc độ (Speed), khi đầy 100% tướng lập tức xuất chiêu!
  - Hoạt ảnh thở bồng bềnh (Sinusoidal Idle Bobbing) và co dãn biến dạng (Squash & Stretch) tạo cảm giác sống động.

---

## ⚔️ Hiệu Ứng Chiến Đấu 2D Sống Động

- **Lướt chém cận chiến (Knight & Assassin)**: Tướng lao mình lướt qua mặt sàn đấu 2D theo quỹ đạo vòng cung, vung vũ khí chém kẻ địch, kích hoạt Hit Stop, rung màn hình (2D Camera Shake), vệt chém lưỡi liềm vàng/chữ X và giật lùi đối thủ.
- **Quỹ đạo đạn đạo 2D (Projectiles)**:
  - Cung thủ bắn ra mũi tên 2D bay theo đường cong parabol xoay theo góc bay, găm vào mục tiêu kèm chùm tia lửa xuyên giáp.
  - Pháp sư phóng quả cầu lôi điện tím phát nổ chấn động gây sát thương lan cho toàn bộ hàng địch (Row AoE Splash).
  - Mục sư phóng ngôi sao thánh quang vàng kim hồi phục sinh lực cho đồng minh nguy cấp nhất.
- **Chữ sát thương nhảy 2D (Floating Combat Text)**: Hiển thị lượng sát thương đỏ, đòn chí mạng vàng rực "CRIT!", sát thương lan tím "SPLASH", hoặc hồi máu xanh lá "+HEAL" nảy lên mượt mà.
- **Vòng tròn hào quang xuất chiêu (Turn Spotlight)**: Vòng sáng hoàng kim xoay tròn dưới chân tướng đang hành động.

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
  - Click vào ô sàn đấu 2D bên phe ta (tối đa 5 quân) để đặt hoặc đổi quân.
- **Chuột phải**: Click vào ô đã có quân để gỡ quân khỏi sàn đấu.
- **Nút "Đội hình mẫu"**: Tự động dàn nhanh đội hình chuẩn mẫu 5 tướng lên sàn đấu.
- **Nút "Xóa hết"**: Gỡ toàn bộ quân ta trên sàn đấu.
- **Nút "BẮT ĐẦU CHIẾN ĐẤU"**: Khởi động trận chiến 2D tự động.
- **Nút "Tốc độ: 1x / 2x"**: Tăng tốc trận đánh.
- **Nút "MÀN TIẾP THEO >>"**: Sau khi diệt sạch địch, chuyển sang màn kế tiếp.

---

## 🚀 Khởi Chạy Game

Từ thư mục dự án:

```bash
cargo run
```
hoặc chạy kiểm tra:
```bash
cargo test
```
