# Cấu Trúc Hệ Thống & Hướng Dẫn Phát Triển (2D Game Architecture & Dev Guide)

Tài liệu này hệ thống hóa toàn bộ mã nguồn của game theo từng module độc lập sau khi chuyển đổi hoàn toàn sang **2D Tactical Auto-Battler** trên nền tảng **Bevy Engine 0.15 (Rust)**. Khi cần sửa lỗi, cân bằng chỉ số, thêm tướng mới, thêm ải mới hoặc tùy chỉnh đồ họa, bạn chỉ cần mở đúng file phụ trách tương ứng.

---

## 📁 1. Sơ Đồ Cấu Trúc File & Trách Nhiệm

```
assets/
└── textures/       # Hình ảnh 2D sprite độ phân giải cao:
    ├── background.png   # Hình nền đấu trường La Mã cổ kính
    ├── knight.png       # Avatar Hiệp sĩ
    ├── archer.png       # Avatar Cung thủ
    ├── mage.png         # Avatar Pháp sư
    ├── assassin.png     # Avatar Sát thủ
    └── cleric.png       # Avatar Mục sư

src/
├── main.rs         # Điểm khởi chạy (Entry Point), cấu hình Camera2d, Resource & Systems
├── types.rs        # Định nghĩa Dữ liệu chung: GameState, Faction, UnitClass, UnitStats, GridPos, GameTextures
├── board.rs        # Sàn đấu 2D: Khối bệ đá Colosseum, vạch phân chia hoàng kim, 4 đuốc lửa ember, 18 ô sàn đấu & hover chuột 2D
├── units.rs        # Khởi tạo Tướng 2D: Thẻ bài Token sprite, khung viền phe phái, thanh Máu & Thể Lực 2D, Bobbing
├── battle.rs       # Cơ chế Chiến Đấu: Turn Bar (ATB), AI chọn mục tiêu, Lướt chém 2D, Đạn đạo 2D, Hit Stop, Rung màn hình 2D
├── stages.rs       # Dữ liệu Ải PvE: Cấu hình 5 Màn chơi (Stage 1 -> 5), Đội hình quái, Vị trí xuất hiện
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
  - `GameTextures`: Nạp các ảnh PNG 2D nhân vật và hình nền đấu trường.
- **Khi nào cần sửa**:
  - Khi muốn thêm class/hệ phái mới (vd: `Berserker`, `Paladin`, `Necromancer`).
  - Khi muốn thêm chỉ số mới (vd: `mana`, `range`, `dodge_rate`, `shield`).

---

### 2. `src/board.rs` — Không Gian Đấu Trường & Tương Tác 2D
- **Chức năng**:
  - Khởi tạo hình nền 2D (`background.png`), bệ đá đấu trường (Arena Dais), vạch phân cách hoàng kim phát sáng.
  - 4 Đuốc lửa ở 4 góc với hiệu ứng phát hạt tàn lửa bốc lên (`TorchFlameEmitter`, `TorchParticle`).
  - 18 Ô đất chiến đấu 2D (9 ô xanh cho Ta, 9 ô đỏ cho Địch).
  - `update_cursor_hover`: Chuyển đổi tọa độ chuột màn hình sang tọa độ thế giới 2D (`camera.viewport_to_world_2d`), tự động phát hiện ô đang hover, phóng to nhẹ (1.05x) và làm sáng viền neon.
- **Khi nào cần sửa**:
  - Muốn mở rộng bàn cờ (vd: từ 3x3 lên 4x4).
  - Muốn thay đổi kích thước sàn hoặc vị trí bố trí đấu trường.

---

### 3. `src/units.rs` — Thẻ Bài & Hiển Thị Tướng 2D
- **Chức năng**:
  - Hàm `spawn_unit()`: Tạo token tướng 2D gồm bóng mờ dưới chân, viền thẻ bài phe phái (Xanh cho Ta, Đỏ cho Địch), sprite chân dung nhân vật từ `GameTextures` (lật mặt đối diện với địch), nhãn tên hệ phái.
  - Thanh máu kép 2D trôi nổi trên đầu: Thanh HP (đổi màu Xanh -> Vàng -> Đỏ theo lượng máu) và Thanh thể lực (Cyan Action Bar).
  - Hoạt ảnh thở phập phồng (`IdleBobbing`) và co dãn hình thể (`ChibiSquashStretch`).
- **Khi nào cần sửa**:
  - Tinh chỉnh kích thước token, viền khung, hoặc thanh máu 2D.
  - Thêm hoạt ảnh visual mới cho tướng.

---

### 4. `src/battle.rs` — Cơ Chế Chiến Đấu & Hiệu Ứng Combat 2D
- **Chức năng**:
  - **Action Gauge**: Tướng nạp thanh thể lực dựa trên chỉ số `speed`. Đạt 100 điểm thì ra chiêu độc lập.
  - **AI Nhắm Mục Tiêu**:
    - `Knight`: Chém tiền tuyến gần nhất.
    - `Archer`: Bắn tỉa mục tiêu máu thấp nhất trên sàn đấu.
    - `Mage`: Tung sét nổ diện rộng cả hàng mục tiêu (Row Splash AOE).
    - `Assassin`: Nhảy bóng đêm ám sát tướng hàng sau cùng.
    - `Cleric`: Phóng tinh tú ban phước hồi máu cho đồng minh nguy cấp nhất.
  - **Game Feel & VFX 2D**:
    - `HitStopManager`: Khựng hình khi va chạm có lực.
    - `CameraShake2d`: Rung màn hình 2D theo mức độ chấn động.
    - `CombatVfx2d` & `SparkParticle2d`: Vệt chém lưỡi liềm vàng, vệt chém chữ X huyết sắc, vòng xung lực sóng nổ, chùm tia lửa bay rơi theo trọng lực.
    - `FloatingText2d`: Chữ số sát thương nhảy ngẫu nhiên vòng cung với trọng lực và co giãn pop-in/fade-out mượt mà.
    - `UnitHitRecoil2d`: Hiệu ứng giật lùi khi trúng đòn rồi nảy về vị trí cũ.
- **Khi nào cần sửa**:
  - Cân bằng công thức tính sát thương, phòng thủ hoặc tỷ lệ bạo kích.
  - Thay đổi logic chọn mục tiêu của từng binh chủng.

---

### 5. `src/stages.rs` — Cấu Hình Màn Chơi PvE
- **Chức năng**:
  - Lưu trữ danh sách 5 Ải chiến dịch (Stage 1 -> Stage 5): Tiền tuyến hiệp sĩ, Đột kích xạ thủ, Hỏa ngục pháp sư, Sát thủ bóng đêm, và Đại hội chiến binh hoàng kim.
- **Khi nào cần sửa**:
  - Thêm ải 6, 7, 8... hoặc thay đổi loại quái, vị trí quái vật trên sàn đấu.

---

### 6. `src/ui.rs` — Giao Diện Người Dùng (HUD & Controls)
- **Chức năng**:
  - Top Bar: Tiêu đề trận đánh, tên ải, đếm số quân (Ta / 5), nút tăng tốc 1x / 2x.
  - Bảng Thẻ Tướng Bên Phải: Rê chuột vào tướng hoặc nút hàng ghế chờ để xem chi tiết chân dung, hệ phái, chỉ số HP/ATK/DEF/SPD, tên và mô tả chiêu thức.
  - Hàng Ghế Chờ: Chọn 1 trong 5 tướng để click đặt lên sàn đấu (tối đa 5 tướng).
  - Nút Đội hình mẫu, Xóa hết, Bắt đầu chiến đấu.
  - Màn hình kết thúc: Thắng (VICTORY) chuyển sang màn kế tiếp, Thua (DEFEAT) cho phép thử lại.
- **Khi nào cần sửa**:
  - Thêm nút chức năng mới, nâng cấp bố cục UI hoặc phông chữ hiển thị.

---

## 🚀 3. Hướng Dẫn Biên Dịch & Chạy Thử Nghiệm

Chạy game:
```bash
cargo run
```

Chạy kiểm tra tự động:
```bash
cargo test
cargo clippy
```
