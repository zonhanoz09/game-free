# Cấu Trúc Hệ Thống & Hướng Dẫn Phát Triển (Game Architecture & Dev Guide)

Tài liệu này hệ thống hóa toàn bộ mã nguồn của game theo từng module độc lập. Khi cần sửa lỗi, cân bằng chỉ số, thêm tướng mới, thêm ải mới hoặc nâng cấp đồ họa, bạn chỉ cần mở đúng file phụ trách tương ứng.

---

## 📁 1. Sơ Đồ Cấu Trúc File & Trách Nhiệm

```
assets/
└── models/         # Thư mục chứa các file mô hình 3D ngoài (.glb) tự động nạp
    └── README.md   # Hướng dẫn chi tiết cách tải và xuất file từ VRoid / Mixamo / Quaternius

src/
├── main.rs         # Điểm khởi chạy (Entry Point), cấu hình Cửa sổ, Camera & Đăng ký Hệ thống (Systems)
├── types.rs        # Định nghĩa Dữ liệu chung: GameState, Faction, UnitClass, UnitStats, GridPos
├── assets_3d.rs    # Kho tài nguyên 3D: Meshes (khối hình 3D), Vật liệu (Materials, PBR, Màu sắc, Emissive)
├── board.rs        # Sàn đấu 3D: Đấu trường Colosseum, Trụ đá, Ngọn lửa, 18 Ô đất chiến đấu, Raycast chuột
├── units.rs        # Khởi tạo Tướng 3D: Ghép nối giải phẫu trang bị, Thanh Máu & Thể Lực 3D, Bobbing & Squash-Stretch
├── battle.rs       # Cơ chế Chiến Đấu: Turn Bar (ATB), AI chọn mục tiêu, Lướt chém, Đạn đạo, Game Feel, Số sát thương
├── stages.rs       # Dữ liệu Ải PvE: Cấu hình 5 Màn chơi (Stage 1 -> 5), Đội hình quái, Vị trí xuất hiện
├── model_loader.rs # Bộ Nạp Mô Hình 3D Ngoại (.glb) & Quản Lý Animation Xương (VRoid / Mixamo / Quaternius)
└── ui.rs           # Giao diện người dùng 2D (HUD): Thanh trên, Bảng thông tin tướng, Hàng ghế chờ, Thắng/Thua
```

---

## 🛠️ 2. Chi Tiết Từng File & Khi Nào Cần Chỉnh Sửa

### 1. `src/types.rs` — Trọng tâm Dữ liệu Toàn Cục
- **Chức năng**:
  - `GameState`: Vòng đời game (`Placement` xếp tướng, `Battle` chiến đấu, `Victory` thắng, `Defeat` thua).
  - `Faction`: Phe ta (`Player`) và Phe địch (`Enemy`).
  - `UnitClass`: Danh sách 5 hệ phái (`Knight`, `Archer`, `Mage`, `Assassin`, `Cleric`).
  - `UnitStats`: Chỉ số cơ bản của tướng (`hp`, `max_hp`, `atk`, `def`, `speed`, `crit_rate`).
  - `GridPos`: Tọa độ bàn cờ 3x3 (`col`, `row`, `faction`).
- **Khi nào cần sửa**:
  - Khi muốn thêm class/hệ phái mới (vd: `Berserker`, `Paladin`, `Necromancer`).
  - Khi muốn thêm chỉ số mới (vd: `mana`, `range`, `dodge_rate`, `shield`).

---

### 2. `src/assets_3d.rs` — Kho Vật Liệu & Mesh 3D
- **Chức năng**:
  - Tạo và lưu trữ tất cả các khối hình 3D chuẩn: `Cylinder`, `Cube`, `Sphere`, `Cone`, `Torus`, `Capsule3d`.
  - Định nghĩa toàn bộ bảng màu và độ bóng bẩy PBR (`StandardMaterial`): Màu áo giáp kim loại, da thú, áo choàng pháp sư, tia lửa, hiệu ứng vệt kiếm, vòng sét, ánh sáng chớp.
- **Khi nào cần sửa**:
  - Muốn đổi màu sắc/độ sáng của phe ta, phe địch, vũ khí hoặc hiệu ứng chiêu.
  - Thêm mesh/vật liệu mới cho tướng mới hoặc skill mới.

---

### 3. `src/board.rs` — Không Gian Đấu Trường & Tương Tác Ô Đất
- **Chức năng**:
  - Dựng bệ đá đấu trường La Mã (Colosseum Arena), 4 trụ đá góc, chảo lửa bập bùng với ánh sáng thực tế.
  - 18 Ô đất chiến đấu 3D (9 ô xanh cho Ta, 9 ô đỏ cho Địch).
  - Hệ thống Raycast chiếu từ con trỏ chuột xuống sàn đấu 3D: Tự động phát hiện ô đang hover, nhấc nhẹ ô lên cao và phát quang viền neon.
- **Khi nào cần sửa**:
  - Muốn mở rộng bàn cờ (vd: từ 3x3 lên 4x4 hoặc lục giác Hex-grid).
  - Muốn thay đổi kích thước sàn, thêm chướng ngại vật hoặc hiệu ứng thời tiết (mưa, sương mù).

---

### 4. `src/model_loader.rs` — Bộ Nạp Mô Hình 3D & Animation (.GLB / .GLTF)
- **Chức năng**:
  - Quét thư mục `assets/models/` tìm kiếm các file `.glb` của 5 hệ phái (`knight.glb`, `archer.glb`, `mage.glb`, `assassin.glb`, `cleric.glb`).
  - Xây dựng `AnimationGraph` tự động kết nối các clip hoạt ảnh (Idle, Attack, Run, Hit, Die).
  - Tự động gán `AnimationPlayer` và kích hoạt vòng lặp Idle mượt mà cho mô hình ngoại nạp từ VRoid Studio, Quaternius, Kenney hoặc Mixamo.
- **Khi nào cần sửa**:
  - Thêm hệ phái mới cần nạp mô hình riêng.
  - Thay đổi ánh xạ (mapping) hoặc tốc độ của các clip hoạt ảnh.

---

### 5. `src/units.rs` — Lắp Ráp Ngoại Hình & Animation Tướng
- **Chức năng**:
  - Hàm `spawn_unit()`: Tự động phát hiện nếu có mô hình 3D ngoại (`.glb`) thì nạp mô hình đó, nếu chưa có thì tự động fallback hiển thị mô hình 3D Chibi thủ công (không bao giờ crash game).
  - Quản lý phân cấp hiển thị (`Visibility`, `InheritedVisibility`, `ViewVisibility`).
  - Thanh máu kép 3D trôi nổi trên đầu: Thanh HP (đổi màu Xanh -> Vàng -> Đỏ) và Thanh thể lực (Cyan Action Bar).
  - Animation thở phập phồng (`IdleBobbing`), xoay vật phẩm (`SpinningItem`), hạt ma thuật bay quanh trượng (`OrbitingMote`), co giãn hình thể Anime (`ChibiSquashStretch`).
- **Khi nào cần sửa**:
  - Muốn chỉnh sửa hình dáng, vũ khí, mũ giáp của tướng thủ công.
  - Tinh chỉnh vị trí bệ tướng hoặc thanh máu 3D trên đầu.

---

### 6. `src/battle.rs` — Trọng Tâm Cơ Chế Chiến Đấu & Game Feel
- **Chức năng**:
  - **Action Gauge**: Tướng nạp thanh thể lực dựa trên chỉ số `speed`. Đạt 100 điểm thì ra chiêu độc lập.
  - **AI Nhắm Mục Tiêu**:
    - `Knight`: Chém tiền tuyến gần nhất.
    - `Archer`: Bắn tỉa mục tiêu máu thấp nhất trên sàn đấu.
    - `Mage`: Tung sét nổ diện rộng cả hàng mục tiêu (Row Splash AOE).
    - `Assassin`: Nhảy bóng đêm ám sát tướng hàng sau cùng.
    - `Cleric`: Phóng tinh tú ban phước hồi máu cho đồng minh nguy cấp nhất.
  - **Game Feel**:
    - `HitStopManager`: Khựng hình 3-5 frame khi đòn đánh va chạm có lực.
    - `CameraShake`: Rung màn hình khi nổ chí mạng hoặc nổ chiêu Mage.
    - `spawn_floating_text`: Chữ số sát thương nhảy ngẫu nhiên không đè hàng, hiệu ứng nảy vòng cung (arc hop) với trọng lực và co giãn pop-in/fade-out mượt mà.
    - `PointLightFlash`: Chớp sáng đa sắc cực đại tại vị trí trúng đòn.
- **Khi nào cần sửa**:
  - Cân bằng công thức tính sát thương, phòng thủ, tỷ lệ chí mạng.
  - Sửa logic nhắm mục tiêu của các hệ phái.
  - Tinh chỉnh thời gian khựng (Hit Stop) hoặc độ rung camera (Camera Shake).

---

### 7. `src/stages.rs` — Quản Lý Màn Chơi (PvE Campaign)
- **Chức năng**:
  - Lưu trữ danh sách cấu hình của các ải từ 1 đến 5:
    - Tên ải & Mô tả chiến thuật gợi ý.
    - Đội hình phe địch: Vị trí cột/hàng và class quái.
- **Khi nào cần sửa**:
  - Muốn thêm Ải 6, 7, 8... hoặc chế độ Endless/Vô Tận.
  - Cân chỉnh độ khó của quái ở từng ải.

---

### 8. `src/ui.rs` — Giao Diện Người Dùng 2D (HUD & Menu)
- **Chức năng**:
  - **Top Bar**: Tên ải hiện tại, số lượng tướng đã xuất trận (tối đa 3), nút điều chỉnh tốc độ trận đấu (1x / 2x / 3x).
  - **Hero Inspection Card (Kính lúp SOI TƯỚNG)**: Bảng kính mờ góc trên phải hiển thị Avatar, Tên, Tộc hệ, Cột máu/công/thủ/tốc và Mô tả kỹ năng chi tiết khi hover chuột vào tướng.
  - **Bench Tray (Hàng ghế chờ)**: Thẻ bài 5 tướng để người chơi bấm chọn và click lên sàn đấu.
  - **Nút Chức Năng**: Bắt đầu trận (`Start Battle`), Xóa đội hình (`Clear`), Đội hình mẫu (`Preset Team`).
  - **Popup Kết Thúc**: Bảng thông báo Chiến Thắng / Thất Bại với nút Qua Màn tiếp theo hoặc Thử lại.
- **Khi nào cần sửa**:
  - Muốn thêm nút bấm, thanh máu boss, bảng vàng thành tích, đổi font chữ hoặc layout UI.

---

### 9. `src/main.rs` — Bộ Khởi Động & Điều Phối
- **Chức năng**:
  - Thiết lập cửa sổ game (1280x720, resizable).
  - Cấu hình Camera 3D và Camera 2D UI Overlay.
  - Khởi tạo tất cả Resources (`BattleTurnManager`, `CameraShake`, `HitStopManager`, `GltfModelAssets`, `CurrentStage`...).
  - Đăng ký vòng lặp các hệ thống Bevy (`Startup`, `Update`, `OnEnter`, `in_state`).
- **Khi nào cần sửa**:
  - Thêm một hệ thống mới (`add_systems`) hoặc resource mới.
  - Đổi màu nền trời (`ClearColor`) hoặc cường độ ánh sáng môi trường (`AmbientLight`).
