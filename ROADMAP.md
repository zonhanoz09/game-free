# 🗺️ Lộ Trình Phát Triển Trò Chơi (Project Roadmap)
**Game: 3v3 Tactical Arena - 2D Auto-Battler (Bevy Engine 0.15 - Rust)**

Tài liệu này ghi lại các giai đoạn phát triển chiến lược để đưa game thành một tựa game Auto-Battler 2D hoàn chỉnh, có chiều sâu chiến thuật và trải nghiệm chơi lôi cuốn.

---

## 📌 Tổng Quan Các Giai Đoạn (Milestones)

| Giai Đoạn | Tên Mục Tiêu | Trạng Thái | Mô Tả Trọng Tâm |
| :---: | :--- | :---: | :--- |
| **Online PvP** | **Đấu Online 1v1 (PvP) & Web Browser Deployment** | 🟢 **Hoàn thành** | Chơi trực tiếp trên trình duyệt Web (WASM), phòng đấu đối kháng 1v1 (Room Code), đồng bộ đội hình thời gian thực qua WebSocket, Docker container sẵn sàng deploy VPS. |
| **Giai đoạn 1** | **Cửa Hàng Ngẫu Nhiên & Hàng Ghế Dự Bị** | 🟢 **Hoàn thành** | 4 thẻ bài Rolling Shop (2G/3G), Hàng ghế dự bị (Bench 6 ô), mua/bán/chuyển quân, ghép sao tự động trên cả sàn và ghế. |
| **Giai đoạn 2** | **Hệ Thống Trang Bị & Cổ Vật (Items & Relics)** | ⚪ Chưa bắt đầu | Rơi hòm đồ sau boss/round, trang bị cơ bản (+ATK, Giáp phản đòn, Hút máu, Nạp mana), kho đồ và gắn trang bị cho tướng. |
| **Giai đoạn 3** | **Combat Chiều Sâu (Buff/Debuff) & Bảng DPS Meter** | ⚪ Chưa bắt đầu | Trạng thái Stun (choáng), Burn/Poison (rút máu), Taunt (khiêu khích). Bảng thống kê sát thương/hồi phục sau round đấu. |
| **Giai đoạn 4** | **Tướng Mới & Mở Rộng Hệ Tộc (Synergies)** | ⚪ Chưa bắt đầu | Thêm 2 tướng (Berserker, Paladin), mở rộng mốc hệ tộc (2)/(4), hệ mới (Cuồng Chiến, Kiên Cố). |
| **Giai đoạn 5** | **Nhạc Nền (BGM), Tùy Chọn & Lưu Dữ Liệu (Save System)**| ⚪ Chưa bắt đầu | Nhạc nền prep phase & battle phase, lưu điểm kỷ lục / tiến trình màn chơi, menu cài đặt âm lượng. |

---

## 📋 Chi Tiết Từng Giai Đoạn

### 🟢 Online PvP & Web Browser Deployment (Chơi Trực Tuyến & Máy Chủ) - [ĐÃ HOÀN THÀNH]
- [x] **Biên dịch WebAssembly (WASM)**:
  - Tối ưu Bevy 0.15 chạy hoàn hảo trên nền tảng Web (`wasm32-unknown-unknown`) với `WebGL2` / `WebGPU`.
  - Cấu hình tự động flag `getrandom_backend="wasm_js"` và `wasm-bindgen-cli`.
- [x] **Đấu Đối Kháng Trực Tuyến 1v1 (Player vs Player)**:
  - Hệ thống phòng riêng biệt (Room Code 4 ký tự ngẫu nhiên: ví dụ `A9X2`, `RA7F`).
  - Hỗ trợ Ghép trận nhanh (Quick Match) tự động bắt cặp người chơi.
  - Giao diện thanh máu đối kháng (100 HP vs 100 HP), hiển thị Blue Host vs Red Challenger.
  - Đồng bộ hoá thời gian thực: Cả 2 người chơi chuẩn bị đội hình, bấm Khoá / Sẵn sàng, máy chủ WebSocket trao đổi đội hình (vị trí, cấp sao, class) và bắt đầu trận đấu cùng lúc!
  - Tính toán sát thương sau mỗi hiệp đấu (5 sát thương gốc + 2 sát thương cho mỗi tướng sống sót), trừ máu đối phương và chuyển vòng tiếp theo cho đến khi có người thắng cuộc.
- [x] **WebSocket Server & Web Client (`server.js`)**:
  - Máy chủ WebSocket chuyên dụng tích hợp HTTP server phục vụ trực tiếp file tĩnh WASM và assets.
  - Cơ chế tự động kết nối lại khi mất mạng, xử lý ngắt kết nối an toàn.
- [x] **Triển Khai Máy Chủ (Docker & VPS)**:
  - `Dockerfile` siêu nhẹ (Node.js Alpine) chứa sẵn gói bundle WASM và WebSocket server.
  - `docker-compose.yml` hỗ trợ khởi chạy 1 lệnh duy nhất: `docker compose up -d` trên bất kỳ VPS nào (port 8080).

---

### 🟢 Giai Đoạn 1: Cửa Hàng Ngẫu Nhiên & Hàng Ghế Dự Bị (Auto-Battler Core) - [ĐÃ HOÀN THÀNH]
- [x] **Hàng ghế dự bị 2D (Reserve Bench 6 ô)**:
  - Hiển thị 6 bệ đá dự bị bo góc phát sáng (`bench_world_pos`) căn lề cạnh sàn đấu phe ta.
  - Hỗ trợ chọn tướng (vòng hào quang Selection Halo vàng kim lấp lánh).
  - Hoán đổi vị trí thông minh (Swap): Ghế <-> Ghế, Sàn <-> Sàn, Ghế <-> Sàn chỉ với 1 click.
  - Chuyển quân ra ô trống và kiểm soát giới hạn tối đa 5 tướng trên sàn đấu (`MAX_PLAYER_UNITS`).
- [x] **Hệ thống Cửa hàng Thẻ Bài Ngẫu Nhiên (Rolling Shop 4 Slots)**:
  - Giao diện 4 thẻ bài sang trọng hiển thị chân dung tướng, tên, hệ phái, giá tiền (2G/3G).
  - Nút `Roll 2G [D]` hỗ trợ cả click và phím tắt `D` (chuẩn Auto-Chess / TFT).
  - Nút `Lock [E]` hỗ trợ click và phím tắt `E` để khoá cửa hàng giữ nguyên cho vòng sau.
  - Tự động tuyển mộ và đưa tướng mới mua vào ô ghế dự bị trống đầu tiên.
  - Cảnh báo đầy đủ khi không đủ vàng hoặc hàng ghế dự bị đầy (6/6).
- [x] **Cơ chế Bán tướng (Sell / Refund)**:
  - Click chuột phải vào bất kỳ tướng nào trên sàn hoặc trên ghế để bán lấy lại 100% vàng (hoàn tiền theo cấp sao).
  - Nút `Clear Board` hoàn tiền toàn bộ cả sàn lẫn ghế dự bị.
- [x] **Ghép Sao Toàn Diện (Cross-Bench Star Fusion)**:
  - Tự động quét 3 tướng cùng loại 1★ trên cả sàn và ghế -> Tự động ghép thành 2★ với chỉ số vượt trội (+80% HP, +60% ATK).
  - Tự động quét 3 tướng 2★ -> Ghép thành 3★ MAX POWER với vòng hào quang hoàng gia và hiệu ứng SFX Ultimate!

---

### ⚪ Giai Đoạn 2: Hệ Thống Trang Bị & Cổ Vật (Items & Relics)
- [ ] **Hệ thống Item Data & Inventory**:
  - `Bloodthirster Sword`: +15 ATK, 15% Hút máu.
  - `Thornmail Vest`: +25 DEF, phản lại 20% sát thương nhận vào thành sát thương phép.
  - `Tear of the Goddess`: +40 Mana khởi đầu trận.
  - `Wind Boots`: +30% Tốc độ ra chiêu (Speed).
  - `Archangel Scepter`: +35% Uy lực kỹ năng Ultimate.
- [ ] **Hòm Đồ & Rơi Trang Bị**:
  - Sau khi diệt sạch quái ở các ải cột mốc (Stage 3, 5, 8, 10), rơi 1 trang bị ngẫu nhiên vào Hòm đồ.
  - Người chơi click chọn trang bị rồi gắn vào thẻ tướng (mỗi tướng tối đa 3 trang bị).

---

### ⚪ Giai Đoạn 3: Combat Chiều Sâu (Buff/Debuff) & Bảng DPS Meter
- [ ] **Hiệu ứng khống chế & Bất lợi**:
  - `Stun`: Bị tê liệt trong 1.5s, thanh thể lực (Action Gauge) ngừng nạp.
  - `Poison/Burn`: Gây sát thương theo % mỗi giây với số màu tím/cam nhảy lên.
  - `Taunt`: Kẻ địch bị buộc phải chọn mục tiêu là tướng khiêu khích.
- [ ] **Bảng Thống Kê Sát Thương (Combat Metrics)**:
  - Theo dõi tổng lượng Damage gây ra và Healing hồi phục của từng tướng.
  - Hiển thị bảng thanh ngang trực quan ở màn hình Victory / Defeat.

---

### ⚪ Giai Đoạn 4: Tướng Mới & Mở Rộng Hệ Tộc
- [ ] **2 Tướng mới**:
  - `Berserker` (Cuồng Chiến): Máu càng thấp tốc đánh và sát thương càng tăng vọt.
  - `Paladin` (Thánh Kỵ Sĩ): Tạo khiên hộ mệnh liên kết cho đồng minh lân cận.
- [ ] **Mở rộng Hệ tộc**:
  - Vanguard: Mốc (2) và Mốc (4).
  - Sharpshooter: Mốc (2) và Mốc (4).
  - Hệ tộc mới: Bastion, Bloodlust.

---

### ⚪ Giai Đoạn 5: Nhạc Nền (BGM), Tùy Chọn & Lưu Tiến Trình
- [ ] BGM Prep Phase (nhạc suy tính chiến thuật) & Battle Phase (nhạc giao tranh dồn dập).
- [ ] Menu Cài đặt: Âm lượng BGM, Âm lượng SFX, Mute.
- [ ] Save System: Ghi file `save_game.json` lưu giữ stage cao nhất, vàng tích lũy, bộ sưu tập tướng.
