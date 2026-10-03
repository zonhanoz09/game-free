# TÀI LIỆU THIẾT KẾ HỆ THỐNG GAME THẺ BÀI CHIẾN THUẬT 3x3 TAM QUỐC (GDD)

*Phiên bản: 1.0.0*  
*Mục đích: Cung cấp đầy đủ thông số logic, quy tắc thuật toán, dữ liệu thẻ bài, bảng cân bằng và cơ chế Gacha để AI hoặc Game Engine triển khai trực tiếp.*

## Trạng thái và phạm vi

Đây là nguồn sự thật cho luật gameplay, nhưng không phải toàn bộ luật trong tài
liệu đã được triển khai. `docs/game_design/roadmap.md` quy định thứ tự bật
từng phần. Phase hiện tại phải hoàn tất board/combat headless và server
authority trước khi bật gacha, synergy, tactics hoặc equipment trong sản phẩm.

- **Đang giữ:** lưới 3x3, targeting, damage/rage/action value và roster Tam Quốc
  ở mức prototype/archetype hiện tại.
- **Data contract đã hoàn tất:** `ContentCatalog` version `combat-v1` chứa 12
  tướng, có validation ID/skill/slot/stats và fingerprint cho replay/config
  compatibility.
- **Đã triển khai:** `crates/game_logic` có board slot, targeting, damage integer
  formula, action gauge, battle state, deterministic battle loop, tự chọn
  ultimate ở 100 nộ, effect primitives damage/heal/buff/debuff/stun và replay
  events.
- **Đang refactor:** chuyển toàn bộ combat từ client vào `crates/game_logic`.
- **Authority vertical slice:** server có runner `combat-v1`, fingerprint
  `release-v1`, command idempotency, timeout/turn cap, replay event và tự tính
  meta-round damage/reward/persistence. `BATTLE_FINISHED` từ client không còn
  được tin cậy; production hardening để ở phase release.
- **Chưa bật:** gacha pity, progression mảnh/sao, synergy, tactics và equipment.
- **Nguyên tắc:** không hiển thị skill như đã hoạt động nếu effect chưa có trong
  simulation và test.

---

## 1. HỆ THỐNG BÀN CỜ CHIẾN THUẬT 3x3 (GRID SPECIFICATION)

### 1.1. Ma trận tọa độ (Grid Matrix)
Bàn cờ mỗi bên gồm 9 ô vuông arranged theo lưới $3 \times 3$, quy định tọa độ $(x, y)$ và Slot ID $(1 \dots 9)$:

| Tầng \ Cột | Cột 1: Hàng trước (Frontline) | Cột 2: Hàng giữa (Midline) | Cột 3: Hàng sau (Backline) |
| :--- | :--- | :--- | :--- |
| **Tầng 1 (Top)** | **Slot 1** $(x=0, y=0)$ | **Slot 4** $(x=1, y=0)$ | **Slot 7** $(x=2, y=0)$ |
| **Tầng 2 (Center)** | **Slot 2** $(x=0, y=1)$ | **Slot 5** $(x=1, y=1)$ | **Slot 8** $(x=2, y=1)$ |
| **Tầng 3 (Bottom)** | **Slot 3** $(x=0, y=2)$ | **Slot 6** $(x=1, y=2)$ | **Slot 9** $(x=2, y=2)$ |

* **Đội hình Công (Attacker):** Tọa độ $x \in [0, 2]$, hướng tấn công sang phải ($+x$).
* **Đội hình Thủ (Defender):** Tọa độ đối xứng, hướng tấn công sang trái ($-x$).

---

### 1.2. Thuật toán chọn mục tiêu (Targeting Logic)
Mỗi đơn vị khi đến lượt hành động sẽ tìm mục tiêu theo thứ tự ưu tiên:

1. **Standard Direct Line (Mặc định):**
   * Ưu tiên mục tiêu còn sống ở **cùng tầng ($y$)** gần nhất (xét từ Cột 1 $\rightarrow$ Cột 2 $\rightarrow$ Cột 3).
   * Nếu tầng hiện tại không có mục tiêu, tìm sang tầng liền kề theo quy tắc khoảng cách Manhattan:
     $$d((x_1, y_1), (x_2, y_2)) = |x_1 - x_2| + |y_1 - y_2|$$
   * Ô có khoảng cách $d$ nhỏ nhất sẽ bị nhắm tới. Nếu bằng nhau, ưu tiên tầng trung tâm (Tầng 2).

2. **Special Targeting Tags:**
   * `TARGET_BACKLINE`: Ưu tiên Slot có $x=2$ (Hàng sau).
   * `TARGET_LOWEST_HP_RATIO`: Mục tiêu có $\frac{\text{Current HP}}{\text{Max HP}}$ thấp nhất.
   * `TARGET_HIGHEST_ATK`: Mục tiêu có chỉ số $\text{ATK}$ cao nhất bàn cờ.
   * `TARGET_CROSS`: Mục tiêu chính và 4 ô kề cạnh dạng hình chữ thập: $(x\pm1, y), (x, y\pm1)$.
   * `TARGET_PIERCE_ROW`: Toàn bộ mục tiêu trên cùng hàng ngang $y$ ($x=0, 1, 2$).
   * `TARGET_COLUMN`: Toàn bộ mục tiêu trên cùng cột dọc $x$ ($y=0, 1, 2$).

---

## 2. CƠ CHẾ COMBAT VÀ CÔNG THỨC TOÁN HỌC (COMBAT ENGINE MATH)

### 2.1. Công thức tính sát thương (Damage Formulas)

1. **Sát thương thực nhận:**
   $$\text{Damage} = \max\left(1, \frac{\text{ATK} \times \text{SkillMultiplier}}{1 + \frac{\text{DEF}}{1000}}\right) \times \text{CritMultiplier} \times \text{TypeModifier}$$

2. **Hệ số Bạo kích (Crit):**
   * Xác suất bạo kích: $P(\text{Crit}) = \text{CritRate} - \text{CritResist}$.
   * Nếu kích hoạt bạo kích:
     $$\text{CritMultiplier} = 1.5 + \text{CritDamageBoost}$$
   * Nếu không bạo kích: $\text{CritMultiplier} = 1.0$.

3. **Hệ số Khắc chế / Tương sinh (TypeModifier):**
   * Cùng hệ/Không khắc chế: $1.0$.
   * Tương khắc phe phái: $1.2$ ($+20\%$).

---

### 2.2. Cơ chế Năng lượng & Nộ khí (Rage Engine)
* **Giới hạn Nộ:** Mọi tướng đều có mức Nộ tối đa là $100$.
* **Cơ chế nạp Nộ:**
  * Đánh thường trúng mục tiêu: $+25$ Nộ.
  * Bị nhận sát thương trực tiếp: $+15$ Nộ.
  * Hạ gục một mục tiêu bất kỳ: $+20$ Nộ.
* **Quy tắc xả nộ:** Khi thanh Nộ đạt $\ge 100$ ở đầu lượt hành động của tướng, tướng bắt buộc tiêu hao toàn bộ $100$ Nộ để thực hiện **Tuyệt Kỹ (Ultimate)** thay vì Đánh Thường (Normal Attack).

---

### 2.3. Thứ tự lượt đi (Turn Order Algorithm)
Sử dụng hệ thống Thanh nạp hành động (Action Value - AV):
$$\Delta \text{AV} = \frac{10000}{\text{SPD}}$$
* Đơn vị có $\Delta \text{AV}$ tích lũy đạt $10000$ sớm nhất sẽ giành lượt đi.
* Sau khi hành động xong, bộ đếm AV của đơn vị đó được reset về $0$.

---

## 3. THIẾT KẾ HỆ THỐNG PHE PHÁI & TƯƠNG TÁC DUYÊN PHẬN (SYNERGIES)

| Phe phái | Định vị chiến thuật | Kích hoạt (3 Tướng) | Kích hoạt (5 Tướng) |
| :--- | :--- | :--- | :--- |
| **Ngụy (Wei)** | Chống chịu, giảm nộ, tạo khiên | Giảm $10\%$ sát thương nhận vào; khi bị đánh có $20\%$ tỉ lệ trừ $15$ Nộ của địch. | Giảm $20\%$ sát thương nhận vào; phản lại $20\%$ sát thương nguyên bản. |
| **Thục (Shu)** | Dồn sát thương đơn, Bạo kích, Đâm xuyên | Tăng $15\%$ Tỉ lệ Bạo kích; khi đồng đội tiêu diệt mục tiêu, hồi $15$ Nộ toàn đội. | Tăng $25\%$ Tỉ lệ Bạo kích; đòn chí mạng bỏ qua $30\%$ DEF mục tiêu. |
| **Ngô (Wu)** | DoT Lửa, Sát thương hàng ngang, Bào mòn | Sát thương gây ra lên mục tiêu bị "Thiêu đốt" tăng $20\%$. | Hiệu ứng Thiêu đốt khi kết thúc thời gian phát nổ gây $100\%$ ATK ra 4 ô kề cạnh. |
| **Quần (Qun)** | Sát thủ, Đột kích hậu phương, Độc dược | Tăng $15\%$ Hút máu toàn diện. | Sát thương lên mục tiêu dưới $40\%$ HP chuyển thành Sát thương chuẩn (bỏ qua DEF). |

---

## 4. BẢNG CÂN BẰNG THÔNG SỐ (STAT BALANCE MATRIX)

### 4.1. Hệ số phẩm cấp (Rarity Multipliers)
* Base Stat Multiplier:
  * **R:** $1.00\times$ (Mức chuẩn)
  * **SR:** $1.25\times$
  * **SSR:** $1.55\times$
  * **UR:** $1.85\times$
* Tăng trưởng theo level:
  $$\text{Stat}_{\text{Level}} = \text{BaseStat} \times \left(1 + \text{GrowthRate} \times (\text{Level} - 1)\right)$$

| Phẩm cấp | Growth Rate / Level | Giới hạn Sao | Số kỹ năng mở khóa |
| :--- | :--- | :--- | :--- |
| **R** | $+5.0\%$ | 5 Sao | 2 (1 Đánh thường, 1 Tuyệt kỹ) |
| **SR** | $+7.5\%$ | 6 Sao | 3 (1 Thường, 1 Nộ, 1 Nội tại) |
| **SSR** | $+10.0\%$ | 7 Sao | 4 (1 Thường, 1 Nộ, 2 Nội tại/Duyên phận) |
| **UR** | $+12.5\%$ | 7 Sao + Thức tỉnh | 4 Kỹ năng cấp cao + 1 Hợp kích |

---

### 4.2. Bảng cơ sở Level 1 theo Role (Chuẩn SSR Base)

```json
{
  "stat_budget_level_1_ssr": {
    "TANKER": {
      "base_hp": 5000,
      "base_atk": 350,
      "base_def": 420,
      "base_spd": 90,
      "crit_rate": 0.05,
      "crit_damage": 1.50
    },
    "WARRIOR": {
      "base_hp": 3800,
      "base_atk": 550,
      "base_def": 300,
      "base_spd": 105,
      "crit_rate": 0.10,
      "crit_damage": 1.50
    },
    "ASSASSIN": {
      "base_hp": 2400,
      "base_atk": 850,
      "base_def": 180,
      "base_spd": 130,
      "crit_rate": 0.25,
      "crit_damage": 1.75
    },
    "MAGE_MARKSMAN": {
      "base_hp": 2200,
      "base_atk": 920,
      "base_def": 160,
      "base_spd": 100,
      "crit_rate": 0.15,
      "crit_damage": 1.50
    },
    "SUPPORT_HEALER": {
      "base_hp": 3000,
      "base_atk": 450,
      "base_def": 240,
      "base_spd": 115,
      "crit_rate": 0.05,
      "crit_damage": 1.50
    }
  }
}
```

---

## 5. DANH MỤC THẺ TƯỚNG (HERO REGISTRY & SKILLSETS)

```json
[
  {
    "id": "HERO_SHU_001",
    "name": "Quan Vũ",
    "faction": "SHU",
    "rarity": "SSR",
    "role": "WARRIOR",
    "optimal_slots": [1, 4],
    "skills": {
      "normal": {
        "name": "Thanh Long Trảm",
        "target_rule": "DIRECT_LINE",
        "damage_rate": 1.0,
        "rage_gain": 25
      },
      "ultimate": {
        "name": "Uy Chấn Hoa Hạ",
        "cost_rage": 100,
        "target_rule": "TARGET_PIERCE_ROW",
        "damage_rate": 2.5,
        "effects": [
          { "type": "DEBUFF_DEF", "value": 0.30, "duration_turns": 2 }
        ]
      }
    }
  },
  {
    "id": "HERO_SHU_002",
    "name": "Trương Phi",
    "faction": "SHU",
    "rarity": "SSR",
    "role": "TANKER",
    "optimal_slots": [2],
    "skills": {
      "normal": {
        "name": "Xà Mâu Kích",
        "target_rule": "DIRECT_LINE",
        "damage_rate": 0.8,
        "rage_gain": 25
      },
      "ultimate": {
        "name": "Nộ Hống Trường Bản",
        "cost_rage": 100,
        "target_rule": "TARGET_COLUMN",
        "damage_rate": 1.4,
        "effects": [
          { "type": "CROWD_CONTROL_STUN", "chance": 0.50, "duration_turns": 1 }
        ]
      },
      "passive": {
        "name": "Kim Cương Bất Hoại",
        "condition": "ON_SLOT_2",
        "effects": [
          { "type": "DAMAGE_REDUCTION", "value": 0.20 },
          { "type": "RAGE_ON_HIT", "value": 10 }
        ]
      }
    }
  },
  {
    "id": "HERO_WU_001",
    "name": "Chu Du",
    "faction": "WU",
    "rarity": "SSR",
    "role": "MAGE_MARKSMAN",
    "optimal_slots": [7, 8],
    "skills": {
      "normal": {
        "name": "Hỏa Vũ",
        "target_rule": "DIRECT_LINE",
        "damage_rate": 0.9,
        "rage_gain": 25
      },
      "ultimate": {
        "name": "Xích Bích Liệt Hỏa",
        "cost_rage": 100,
        "target_rule": "TARGET_CROSS",
        "damage_rate": 1.8,
        "effects": [
          { "type": "DOT_BURN", "dot_percent_hp": 0.08, "duration_turns": 3 }
        ]
      }
    }
  },
  {
    "id": "HERO_QUN_001",
    "name": "Lữ Bố",
    "faction": "QUN",
    "rarity": "UR",
    "role": "WARRIOR",
    "optimal_slots": [2, 4, 5],
    "skills": {
      "normal": {
        "name": "Phương Thiên Họa Kích",
        "target_rule": "DIRECT_LINE",
        "damage_rate": 1.2,
        "rage_gain": 25
      },
      "ultimate": {
        "name": "Thiên Bá Phong Vân",
        "cost_rage": 100,
        "target_rule": "TARGET_LOWEST_HP_RATIO",
        "damage_rate": 3.2,
        "special_trigger": "ON_KILL_REFRESH_RAGE_100"
      }
    }
  }
]
```

---

## 6. THẺ MƯU KẾ & TRANG BỊ CHIẾN THUẬT (TACTICS & ARTIFACTS)

### 6.1. Thẻ Mưu Kế (Tactic Spells)
Người chơi tích lũy điểm **Chiến thuật (Tactical Points - TP)** trong trận để kích hoạt:

```json
[
  {
    "id": "TACTIC_001",
    "name": "Mượn Gió Đông",
    "cost_tp": 3,
    "effect_type": "AMPLIFY_BURN",
    "description": "Tăng gấp đôi toàn bộ sát thương Thiêu đốt hiện có trên sân đối thủ."
  },
  {
    "id": "TACTIC_002",
    "name": "Man Thiên Quá Hải",
    "cost_tp": 2,
    "effect_type": "SWAP_SLOTS",
    "description": "Hoán đổi vị trí lập tức giữa 2 ô bất kỳ thuộc phe mình trên lưới 3x3."
  },
  {
    "id": "TACTIC_003",
    "name": "Bát Trận Đồ",
    "cost_tp": 4,
    "effect_type": "SCRAMBLE_TURN_ORDER",
    "description": "Đảo ngược thứ tự hành động (Speed) của toàn bộ phe địch trong 1 hiệp."
  }
]
```

### 6.2. Trang bị & Thần binh (Equipment Multipliers)
* **Vũ khí - Thanh Long Đao:** Đòn đánh thường tăng thêm $+15\%$ Xuyên giáp ($\text{Ignore DEF}$).
* **Chiến mã - Xích Thố:** Tăng vĩnh viễn $+30$ Tốc độ cơ bản ($\text{SPD}$).
* **Chiến mã - Đích Lô:** Khi nhận đòn có sát thương vượt quá $50\%$ HP hiện tại, tự động đổi vị trí với một ô trống liền kề để giảm $50\%$ lượng sát thương đó.

---

## 7. CƠ CHẾ KINH TẾ GACHA & BẢO HIỂM (GACHA SPECIFICATION)

### 7.1. Bảng xác suất gốc (Base Probability)
* **SSR:** $2.50\%$
* **SR:** $15.50\%$
* **R:** $82.00\%$

### 7.2. Thuật toán Bảo hiểm (Pity Algorithm)

```python
def roll_gacha(pity_counter, hard_pity=70, soft_pity=50):
    base_ssr_rate = 0.025
    current_ssr_rate = base_ssr_rate
    
    if pity_counter >= soft_pity:
        # Tăng thêm 2.5% mỗi lượt sau mốc 50
        current_ssr_rate += 0.025 * (pity_counter - soft_pity + 1)
        
    if pity_counter >= hard_pity:
        current_ssr_rate = 1.0  # Chắc chắn ra SSR
        
    random_val = random_uniform(0.0, 1.0)
    
    if random_val <= current_ssr_rate:
        pity_counter = 0
        return "PULL_SSR", pity_counter
    elif random_val <= (current_ssr_rate + 0.155):
        pity_counter += 1
        return "PULL_SR", pity_counter
    else:
        pity_counter += 1
        return "PULL_R", pity_counter
```

### 7.3. Quy đổi Mảnh tướng trùng lặp (Shard Duplicate System)
* Rút trùng **SSR:** Nhận $50$ Mảnh tướng.
* Rút trùng **SR:** Nhận $20$ Mảnh tướng.
* Rút trùng **R:** Nhận $5$ Mảnh tướng.
* Bảng nâng sao:
  * 1 Sao $\rightarrow$ 2 Sao: $20$ Mảnh
  * 2 Sao $\rightarrow$ 3 Sao: $40$ Mảnh
  * 3 Sao $\rightarrow$ 4 Sao: $80$ Mảnh
  * 4 Sao $\rightarrow$ 5 Sao: $120$ Mảnh
  * 5 Sao $\rightarrow$ 6 Sao: $180$ Mảnh
  * 6 Sao $\rightarrow$ 7 Sao (Thức tỉnh): $250$ Mảnh

---

## 8. CẤU TRÚC ĐỘI HÌNH META 3x3 CHUẨN MẪU (META FORMATIONS)

### 8.1. Đội hình Thục Quốc Xuyên Phá (Single-target Piercing)
```
[Slot 1: Trống]      [Slot 4: Triệu Vân]     [Slot 7: Quan Vũ]
[Slot 2: Trương Phi] [Slot 5: Lưu Bị]        [Slot 8: Hoàng Trung]
[Slot 3: Trống]      [Slot 6: Trống]         [Slot 9: Gia Cát Lượng]
```
* **Chiến thuật:** Trương Phi đứng ở Slot 2 chịu trọn hỏa lực và phản nộ. Triệu Vân và Quan Vũ đâm xuyên một hàng. Hoàng Trung và Gia Cát Lượng khống chế và dứt điểm mục tiêu yếu máu ở Slot 8 và 9.

### 8.2. Đội hình Ngô Quốc Thiêu Đốt Phân Tầng (Spread DoT)
```
[Slot 1: Hoàng Cái]  [Slot 4: Trống]         [Slot 7: Lục Tốn]
[Slot 2: Trống]      [Slot 5: Trống]         [Slot 8: Chu Du]
[Slot 3: Tôn Sách]   [Slot 6: Trống]         [Slot 9: Đại Kiều]
```
* **Chiến thuật:** Bỏ trống hoàn toàn cột giữa (Slot 2, 5) để né kỹ năng quét hình chữ thập. Tôn Sách và Hoàng Cái chịu đòn hai cánh. Bộ ba Chu Du - Lục Tốn - Đại Kiều xả hỏa thiêu liên tục từ hàng sau.