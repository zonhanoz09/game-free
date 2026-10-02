# Hướng Dẫn Tích Hợp Mô Hình 3D & Animation (.GLB / .GLTF)

Hệ thống **Dynamic 3D Model Loader & Skeletal Animation System** đã được tích hợp hoàn chỉnh vào game.

Game hỗ trợ tự động nhận diện và nạp các mô hình nhân vật 3D có gắn xương (Rigged) và chuyển động (Animated) từ các nguồn phổ biến nhất hiện nay: **VRoid Studio, Mixamo, Quaternius, Kenney**.

---

## 📁 1. Quy Tắc Đặt Tên File Mô Hình

Chỉ cần đặt file `.glb` vào thư mục này (`assets/models/`) theo đúng tên hệ phái:

| Tên File | Hệ Phái | Ghi Chú |
| :--- | :--- | :--- |
| `knight.glb` | **Hiệp Sĩ (Knight)** | Cận chiến khiên kiếm / giáp nặng |
| `archer.glb` | **Cung Thủ (Archer)** | Tầm xa trường cung / áo da |
| `mage.glb` | **Pháp Sư (Mage)** | Tầm xa pháp trượng / áo choàng phép |
| `assassin.glb` | **Sát Thủ (Assassin)** | Cận chiến song đoản đao / giáp nhẹ |
| `cleric.glb` | **Mục Sư (Cleric)** | Hỗ trợ scepter thánh quang / lễ phục |

> [!TIP]
> **Cơ chế Fallback thông minh**: Nếu bạn chưa tải đủ cả 5 tướng, class nào có file `.glb` sẽ hiển thị mô hình ngoài, class nào chưa có sẽ tự động hiển thị mô hình 3D Chibi có sẵn của game mà không gây lỗi!

---

## 🎬 2. Quy Chuẩn Animation Trong File .GLB

Hệ thống `AnimationGraph` tự động liên kết các hoạt ảnh theo thứ tự Clip Index hoặc theo tên:

| Animation Index | Tên Hoạt Ảnh (Tùy chọn) | Mô Tả |
| :---: | :--- | :--- |
| **0** | `Idle` | Trạng thái đứng thở chờ lượt |
| **1** | `Attack` / `Slash` / `Shoot` | Đòn đánh hoặc tung chiêu |
| **2** | `Run` / `Walk` / `Dash` | Di chuyển lướt áp sát mục tiêu |
| **3** | `Hit` / `Damage` | Phản ứng giật lùi khi bị trúng đòn |
| **4** | `Die` / `Death` | Hoạt ảnh gục ngã |

---

## 🌐 3. Hướng Dẫn Nạp Từ Từng Nguồn Miễn Phí

### 🎨 A. VRoid Studio & VRoid Hub (Phong Cách Anime Đẹp Nhất)
1. Tải phần mềm miễn phí [VRoid Studio](https://vroid.com/en/studio).
2. Tạo nhân vật Anime với tỉ lệ Chibi (đầu to, thân ngắn tỉ lệ 1:2.5 hoặc 1:3).
3. Xuất file dạng `.vrm`.
4. Mở Blender (cài add-on miễn phí **VRM Add-on for Blender**):
   - `File -> Import -> VRM (.vrm)`.
   - `File -> Export -> glTF 2.0 (.glb)`.
5. Đổi tên thành `knight.glb` (hoặc hệ phái bạn muốn) và copy vào thư mục `assets/models/`.

### ⚡ B. Quaternius & Kenney (Low-Poly / Chibi RPG - CC0 Miễn Phí)
1. Truy cập [quaternius.com](https://quaternius.com) hoặc [kenney.nl](https://kenney.nl/assets).
2. Tải các gói:
   - *Ultimate Modular Characters* (Quaternius).
   - *Animated Characters* (Kenney).
3. Các pack này đã có sẵn file `.glb` kèm đầy đủ animations (Idle, Walk, Attack, Death).
4. Chỉ cần đổi tên file `.glb` theo bảng trên và thả vào thư mục `assets/models/`.

### 🕺 C. Adobe Mixamo (Gán Xương & Hàng Ngàn Animation Miễn Phí)
1. Vào [mixamo.com](https://www.mixamo.com/) (miễn phí với tài khoản Adobe).
2. Bấm **Upload Character** và kéo model `.obj` / `.fbx` của bạn lên.
3. Chấm 5 điểm khớp (cằm, cổ tay, khuỷu tay, đầu gối, háng) để hệ thống tự động gán khung xương (Auto-Rigging).
4. Tìm kiếm các animation mong muốn:
   - `Idle` -> Tải về (chọn format FBX with Skin).
   - `Sword And Shield Slash` hoặc `Standing Draw Arrow` -> Tải về.
5. Dùng Blender nạp các file animation này vào cùng 1 file `.glb` (hoặc dùng tool mã nguồn mở [Mixamo2GLB](https://github.com/mixamo2glb/Mixamo2GLBAnimationMerger)).
6. Lưu file vào `assets/models/<tên-hệ-phái>.glb`.
