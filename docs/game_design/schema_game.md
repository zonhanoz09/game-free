# DATABASE SPECIFICATION & PERSISTENCE ARCHITECTURE: 3X3 TACTICAL CARD ENGINE

*Target: AI Implementation Agents, Backend Engineers, Game Systems Programmers*  
*Dialect: PostgreSQL 15+ (với JSONB Support) / Compatible với SQLite cho Client Offline Storage*  
*ORM / Query Layer: SQLx (Async Pure Rust) kết hợp Serde trong `crates/data_schema` và `apps/server`*

---

## 1. TỔNG QUAN KIẾN TRÚC DỮ LIỆU (OVERVIEW)

Hệ thống lưu trữ được phân chia thành 2 phân hệ độc lập:
1. **Master Game Data (Metadata tĩnh & Rules Engine):** Cấu hình gốc của trò chơi (Rarity, Card Templates, Base Stats, Skills, Status Effects, Board Cells & Line Buffs). Dữ liệu này có thể nạp sẵn vào cache in-memory, hot-reload qua Admin API hoặc xuất ra JSON cho Client.
2. **Player Runtime Data (Dữ liệu động):** Dữ liệu tài khoản người dùng, thẻ bài sở hữu, cấp độ, số sao, trang bị kỹ năng phụ, và ma trận bố trí đội hình $3 \times 3$ trên bàn cờ.

```
+-------------------------------------------------------------------------------------------------------+
| MASTER GAME DATA (Admin CRUD / Hot-Reload Cache Engine)                                               |
|                                                                                                       |
|  [rarity_configs]                                                                                     |
|        │                                                                                              |
|        ▼                                                                                              |
|  [card_templates] ─── (1:N) ───► [card_template_skills] ────► [skills]                               |
|        │                                                        │                                     |
|        ├─── (1:N) ───► [card_template_buffs] ◄───┐              │ (1:N)                               |
|        │                                         │              ▼                                     |
|        │                                         ├──────► [status_effects] (Buff/Debuff/CC/DoT)       |
|        ▼                                         │              ▲                                     |
|  [board_cell_configs] (1..9 Slots) ──────────────┴──────────────┘ (Tile Buffs / Hazards)              |
|        ▲                                                                                              |
|        └─── (Mapped via line_type) ─── [formation_line_configs]                                      |
+───────────────────────────────────────────────────────────────────────────────────────────────────────+
                                                   │
+──────────────────────────────────────────────────┴────────────────────────────────────────────────────+
| PLAYER RUNTIME DATA (Gameplay State & Formations)                                                     |
|                                                                                                       |
|  [users] ─── (1:N) ───► [player_cards] ─── (Equip Skill) ───► [skills]                                |
|    │                          │                                                                       |
|    └─── (1:N) ───────────────►┴───► [player_formations] (Slots 1..9 Mapping on 3x3 Grid)             |
+-------------------------------------------------------------------------------------------------------+
```

---

## 2. POSTGRESQL DDL SCHEMA

### 2.1. Phân hệ Master Data: Hiệu ứng, Bàn cờ, Kỹ năng & Tướng

```sql
-- ============================================================================
-- 1. BẢNG CẤU HÌNH PHẨM CẤP (RARITY)
-- ============================================================================
CREATE TABLE IF NOT EXISTS rarity_configs (
    rarity_id VARCHAR(16) PRIMARY KEY, -- 'R', 'SR', 'SSR', 'UR'
    display_name VARCHAR(64) NOT NULL,
    base_stat_multiplier NUMERIC(4, 2) NOT NULL, -- 1.00, 1.20, 1.50, 1.85
    growth_rate_per_level NUMERIC(4, 3) NOT NULL, -- 0.050, 0.075, 0.100, 0.135
    max_star_rating SMALLINT NOT NULL, -- 5, 6, 7, 9
    max_skills_allowed SMALLINT NOT NULL DEFAULT 4,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- 2. BẢNG HIỆU ỨNG (STATUS EFFECTS / BUFF / DEBUFF / CC / DOT / HOT)
-- ============================================================================
CREATE TABLE IF NOT EXISTS status_effects (
    effect_id VARCHAR(64) PRIMARY KEY, -- 'eff_burn', 'eff_atk_up', 'eff_stun', 'eff_holy_aura'
    name VARCHAR(128) NOT NULL,
    effect_category VARCHAR(32) NOT NULL, -- 'BUFF', 'DEBUFF', 'CROWD_CONTROL', 'DOT', 'HOT', 'TRIGGER_PASSIVE'
    stat_target VARCHAR(32) NOT NULL,    -- 'HP', 'ATK', 'DEF', 'SPEED', 'CRIT_RATE', 'CRIT_DMG', 'MORALE', 'SHIELD', 'ACTION_LOCK'
    calculation_type VARCHAR(32) NOT NULL, -- 'PERCENTAGE', 'FLAT', 'ABSOLUTE_STATE', 'CUSTOM_FORMULA'
    base_value NUMERIC(7, 3) DEFAULT 0.0, -- vd: 0.20 (+20% ATK), 50.0 (+50 Shield flat)
    max_stacks SMALLINT NOT NULL DEFAULT 1, -- Giới hạn cộng dồn hiệu ứng
    duration_turns INT NOT NULL DEFAULT 1,  -- Số hiệp duy trì (0 = tức thì/vĩnh viễn tùy trigger)
    tick_trigger VARCHAR(32) NOT NULL DEFAULT 'TURN_START', -- 'TURN_START', 'TURN_END', 'ON_HIT', 'ON_DAMAGED', 'ON_DEATH', 'INSTANT'
    is_dispellable BOOLEAN NOT NULL DEFAULT TRUE, -- Có thể giải trừ (Cleanse) được không
    icon VARCHAR(64) DEFAULT '✨',
    vfx_prefab VARCHAR(128) DEFAULT 'vfx_default_glow',
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- 3. BẢNG CẤU HÌNH Ô BÀN CỜ 3X3 & ĐỊA HÌNH (BOARD CELL CONFIGS)
-- ============================================================================
CREATE TABLE IF NOT EXISTS board_cell_configs (
    slot_id SMALLINT PRIMARY KEY, -- 1..9 (Quy chuẩn ô 3x3)
    col SMALLINT NOT NULL CHECK (col >= 0 AND col <= 2), -- 0: Cột Trái, 1: Giữa, 2: Phải
    row SMALLINT NOT NULL CHECK (row >= 0 AND row <= 2), -- 0: Hàng Trước (Front), 1: Giữa (Mid), 2: Sau (Back)
    line_type VARCHAR(16) NOT NULL, -- 'FRONT', 'MID', 'BACK'
    terrain_type VARCHAR(32) NOT NULL DEFAULT 'NORMAL', -- 'NORMAL', 'HIGH_GROUND', 'LAVA_ZONE', 'HOLY_SANCTUARY', 'SHADOW_VEIL'
    tile_buff_effect_id VARCHAR(64) REFERENCES status_effects(effect_id) ON DELETE SET NULL, -- Buff nội tại khi đứng trên ô
    hazard_damage_pct NUMERIC(4, 3) DEFAULT 0.000, -- Sát thương môi trường trừ máu mỗi hiệp
    preferred_roles JSONB DEFAULT '[]'::jsonb, -- Ví dụ: ["VANGUARD", "WARRIOR"]
    penalty_roles JSONB DEFAULT '[]'::jsonb,   -- Ví dụ: ["MARKSMAN"] (bị giảm chỉ số nếu đứng ô này)
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- 4. BẢNG CẤU HÌNH BUFF THEO HÀNG TRẬN (FORMATION LINE CONFIGS)
-- ============================================================================
CREATE TABLE IF NOT EXISTS formation_line_configs (
    line_type VARCHAR(16) PRIMARY KEY, -- 'FRONT', 'MID', 'BACK'
    display_name VARCHAR(64) NOT NULL,
    slot_indices JSONB NOT NULL, -- [1, 2, 3] cho FRONT, [4, 5, 6] cho MID, [7, 8, 9] cho BACK
    allowed_roles JSONB NOT NULL, -- ["VANGUARD", "WARRIOR"]
    penalty_roles JSONB NOT NULL, -- ["MARKSMAN", "SUPPORT"]
    penalty_stats JSONB DEFAULT '{}'::jsonb, -- {"damage_taken_pct": 0.20}
    line_buffs JSONB NOT NULL, -- {"hp_pct": 0.15, "def_pct": 0.12}
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- 5. BẢNG MẪU THẺ TƯỚNG (CARD TEMPLATES)
-- ============================================================================
CREATE TABLE IF NOT EXISTS card_templates (
    card_template_id VARCHAR(64) PRIMARY KEY, -- 'hero_guan_yu'
    name VARCHAR(128) NOT NULL,
    title VARCHAR(128),
    faction VARCHAR(32) NOT NULL, -- 'SHU', 'WEI', 'WU', 'QUN'
    rarity_id VARCHAR(16) NOT NULL REFERENCES rarity_configs(rarity_id),
    recommended_role VARCHAR(32) NOT NULL, -- 'VANGUARD', 'WARRIOR', 'ASSASSIN', 'MARKSMAN', 'MAGE', 'TACTICIAN', 'SUPPORT'
    
    -- Chỉ số cơ bản tại Level 1 (Base Stats)
    base_hp INT NOT NULL,
    base_atk INT NOT NULL,
    base_def INT NOT NULL,
    base_speed INT NOT NULL,
    base_crit_rate NUMERIC(4, 3) DEFAULT 0.050,
    base_crit_dmg NUMERIC(4, 3) DEFAULT 1.500,
    base_block_rate NUMERIC(4, 3) DEFAULT 0.000,
    base_dodge_rate NUMERIC(4, 3) DEFAULT 0.050,
    
    -- Sĩ khí / Nộ khí
    initial_morale INT DEFAULT 50,
    max_morale INT DEFAULT 100,
    
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- 6. BẢNG KỸ NĂNG (SKILLS)
-- ============================================================================
CREATE TABLE IF NOT EXISTS skills (
    skill_id VARCHAR(64) PRIMARY KEY, -- 'skill_guanyu_ult'
    name VARCHAR(128) NOT NULL,
    skill_quality VARCHAR(32) NOT NULL, -- 'COMMON', 'RARE', 'EPIC', 'LEGENDARY'
    is_skill_common BOOLEAN DEFAULT FALSE, -- Kỹ năng chung có thể tháo lắp tự do
    trigger_type VARCHAR(32) NOT NULL, -- 'NORMAL_ATTACK', 'ACTIVE_CD', 'ULTIMATE', 'PASSIVE'
    execution_priority SMALLINT NOT NULL, -- 1: Ultimate, 2: Active CD, 3: Normal, 4: Passive
    
    cost_morale INT DEFAULT 0,
    cooldown_turns INT DEFAULT 0,
    
    -- Phạm vi mục tiêu trên lưới 3x3
    target_pattern VARCHAR(32) NOT NULL, -- 'DIRECT_LANE', 'FRONT_ROW', 'CROSS_PLUS', 'ALL_BOARD', 'BACKLINE_ASSASSIN', 'PIERCE_ROW'
    target_rule VARCHAR(32) NOT NULL,    -- 'ENEMY_NEAREST', 'ENEMY_LOWEST_HP', 'ENEMY_HIGHEST_ATK', 'SELF_ALLY'
    
    -- Công thức sát thương
    damage_type VARCHAR(16) NOT NULL, -- 'PHYSICAL', 'MAGIC', 'TRUE_DAMAGE', 'NONE'
    damage_multiplier NUMERIC(5, 2) DEFAULT 1.00,
    can_crit BOOLEAN DEFAULT TRUE,
    
    -- JSONB lưu nhanh các hiệu ứng (Hybrid Storage)
    effects_applied JSONB DEFAULT '[]'::jsonb,
    
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- 7. BẢNG QUAN HỆ KỸ NĂNG - HIỆU ỨNG (SKILL_EFFECTS MAPPING)
-- ============================================================================
CREATE TABLE IF NOT EXISTS skill_effects (
    mapping_id BIGSERIAL PRIMARY KEY,
    skill_id VARCHAR(64) NOT NULL REFERENCES skills(skill_id) ON DELETE CASCADE,
    effect_id VARCHAR(64) NOT NULL REFERENCES status_effects(effect_id) ON DELETE CASCADE,
    target_scope VARCHAR(32) NOT NULL DEFAULT 'PRIMARY_TARGET', -- 'SELF', 'PRIMARY_TARGET', 'ALL_TARGETS_IN_SCOPE', 'ALL_ALLIES'
    proc_chance NUMERIC(4, 3) NOT NULL DEFAULT 1.000, -- 1.00 = 100% tỉ lệ kích hoạt
    custom_value NUMERIC(7, 3), -- Ghi đè giá trị hiệu ứng nếu kỹ năng có hệ số riêng
    custom_duration INT,        -- Ghi đè số hiệp duy trì
    CONSTRAINT uq_skill_effect_scope UNIQUE (skill_id, effect_id, target_scope)
);

-- ============================================================================
-- 8. BẢNG LIÊN KẾT TƯỚNG - KỸ NĂNG MẶC ĐỊNH (CARD_TEMPLATE_SKILLS)
-- ============================================================================
CREATE TABLE IF NOT EXISTS card_template_skills (
    card_template_id VARCHAR(64) REFERENCES card_templates(card_template_id) ON DELETE CASCADE,
    skill_id VARCHAR(64) REFERENCES skills(skill_id) ON DELETE CASCADE,
    slot_type VARCHAR(16) NOT NULL DEFAULT 'NORMAL' CHECK (slot_type IN ('NORMAL', 'ULTIMATE', 'PASSIVE')),
    unlock_at_star SMALLINT DEFAULT 1,
    PRIMARY KEY (card_template_id, skill_id)
);

-- ============================================================================
-- 9. BẢNG BUFF NỘI TẠI CỦA TƯỚNG (CARD_TEMPLATE_BUFFS)
-- ============================================================================
CREATE TABLE IF NOT EXISTS card_template_buffs (
    mapping_id BIGSERIAL PRIMARY KEY,
    card_template_id VARCHAR(64) REFERENCES card_templates(card_template_id) ON DELETE CASCADE,
    effect_id VARCHAR(64) REFERENCES status_effects(effect_id) ON DELETE CASCADE,
    unlock_at_level INT DEFAULT 1,
    unlock_at_star SMALLINT DEFAULT 1,
    CONSTRAINT uq_card_buff UNIQUE (card_template_id, effect_id)
);

-- ============================================================================
-- 10. BẢNG KÍCH DUYÊN ĐỘI HÌNH (FORMATION_SYNERGIES)
-- ============================================================================
CREATE TABLE IF NOT EXISTS formation_synergies (
    synergy_id VARCHAR(64) PRIMARY KEY, -- 'synergy_vanguard', 'synergy_shu'
    name VARCHAR(128) NOT NULL,
    synergy_type VARCHAR(32) NOT NULL, -- 'CLASS', 'FACTION', 'SPECIAL'
    trigger_count SMALLINT NOT NULL DEFAULT 2, -- Số tướng cùng hệ/phe yêu cầu
    icon VARCHAR(64) DEFAULT '🔰',
    description TEXT,
    hero_ids JSONB DEFAULT '[]'::jsonb, -- Danh sách hero_id tham gia kích duyên
    stat_buffs JSONB NOT NULL DEFAULT '{}'::jsonb, -- Buff chỉ số cho toàn đội: {"def": 35.0, "hp": 150.0}
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
```

---

### 2.2. Phân hệ Runtime Data: Người chơi, Thẻ bài & Đội hình

```sql
-- ============================================================================
-- 10. BẢNG NGƯỜI DÙNG (USERS)
-- ============================================================================
CREATE TABLE IF NOT EXISTS users (
    user_id BIGSERIAL PRIMARY KEY,
    username VARCHAR(64) UNIQUE NOT NULL,
    display_name VARCHAR(128) NOT NULL,
    avatar VARCHAR(64) DEFAULT 'knight',
    gold BIGINT DEFAULT 1000,
    gems INT DEFAULT 50,
    level INT DEFAULT 1,
    elo INT DEFAULT 1000,
    rank_tier VARCHAR(32) DEFAULT 'Bronze',
    battle_slots SMALLINT DEFAULT 3,
    password_hash VARCHAR(256) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- 11. BẢNG THẺ BÀI NGƯỜI CHƠI SỞ HỮU (PLAYER_CARDS)
-- ============================================================================
CREATE TABLE IF NOT EXISTS player_cards (
    player_card_id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    card_template_id VARCHAR(64) NOT NULL REFERENCES card_templates(card_template_id),
    current_level INT DEFAULT 1,
    current_exp BIGINT DEFAULT 0,
    current_star SMALLINT DEFAULT 1,
    breakthrough_tier SMALLINT DEFAULT 0,
    is_foil BOOLEAN DEFAULT FALSE,
    quantity INT DEFAULT 1,
    equipped_common_skill_id VARCHAR(64) REFERENCES skills(skill_id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT uq_user_card_instance UNIQUE (user_id, card_template_id)
);

CREATE INDEX IF NOT EXISTS idx_player_cards_user ON player_cards(user_id);

-- ============================================================================
-- 12. BẢNG BỐ TRÍ ĐỘI HÌNH LƯỚI 3X3 (PLAYER_FORMATIONS)
-- ============================================================================
CREATE TABLE IF NOT EXISTS player_formations (
    formation_id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    formation_type VARCHAR(32) NOT NULL, -- 'ARENA_DEFENSE', 'PVE_CAMPAIGN', 'PVP_ATTACK'
    
    -- Slot 1..3: Hàng đầu (Front), Slot 4..6: Hàng giữa (Mid), Slot 7..9: Hàng sau (Back)
    slot_1_card_id BIGINT REFERENCES player_cards(player_card_id) ON DELETE SET NULL,
    slot_2_card_id BIGINT REFERENCES player_cards(player_card_id) ON DELETE SET NULL,
    slot_3_card_id BIGINT REFERENCES player_cards(player_card_id) ON DELETE SET NULL,
    slot_4_card_id BIGINT REFERENCES player_cards(player_card_id) ON DELETE SET NULL,
    slot_5_card_id BIGINT REFERENCES player_cards(player_card_id) ON DELETE SET NULL,
    slot_6_card_id BIGINT REFERENCES player_cards(player_card_id) ON DELETE SET NULL,
    slot_7_card_id BIGINT REFERENCES player_cards(player_card_id) ON DELETE SET NULL,
    slot_8_card_id BIGINT REFERENCES player_cards(player_card_id) ON DELETE SET NULL,
    slot_9_card_id BIGINT REFERENCES player_cards(player_card_id) ON DELETE SET NULL,
    
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT uq_user_formation_type UNIQUE (user_id, formation_type)
);
```

---

## 3. SEED DATA MẪU (MASTER SEEDS)

```sql
-- 1. Phẩm cấp
INSERT INTO rarity_configs (rarity_id, display_name, base_stat_multiplier, growth_rate_per_level, max_star_rating, max_skills_allowed)
VALUES 
('R',   'Thường',     1.00, 0.050, 5, 2),
('SR',  'Hiếm',       1.20, 0.075, 6, 3),
('SSR', 'Cực Phẩm',   1.50, 0.100, 7, 4),
('UR',  'Thần Tướng', 1.85, 0.135, 9, 5)
ON CONFLICT (rarity_id) DO NOTHING;

-- 2. Đăng ký kho Hiệu ứng mẫu (Status Effects)
INSERT INTO status_effects (effect_id, name, effect_category, stat_target, calculation_type, base_value, max_stacks, duration_turns, tick_trigger, is_dispellable, icon, vfx_prefab, description)
VALUES
('eff_stun',         'Choáng',         'CROWD_CONTROL', 'ACTION_LOCK', 'ABSOLUTE_STATE', 1.0, 1, 1, 'TURN_START', FALSE, '💫', 'vfx_stun_stars',   'Mục tiêu mất lượt hành động trong 1 hiệp'),
('eff_armor_break',  'Phá Giáp',       'DEBUFF',        'DEF',         'PERCENTAGE',    -0.30, 3, 2, 'TURN_START', TRUE,  '🛡️', 'vfx_armor_break', 'Giảm 30% phòng thủ, tối đa cộng dồn 3 tầng'),
('eff_burn',         'Thiêu Đốt',      'DOT',           'HP',          'PERCENTAGE',    -0.08, 5, 3, 'TURN_END',   TRUE,  '🔥', 'vfx_burn_flame',  'Mỗi hiệp chịu sát thương lửa bằng 8% công của người thi triển'),
('eff_freeze',       'Đóng Băng',      'CROWD_CONTROL', 'SPEED',       'PERCENTAGE',    -0.50, 1, 2, 'TURN_START', TRUE,  '❄️', 'vfx_ice_shards',  'Giảm 50% tốc độ đánh và hồi sĩ khí'),
('eff_atk_up',       'Cuồng Nộ',       'BUFF',          'ATK',         'PERCENTAGE',     0.25, 3, 2, 'TURN_START', TRUE,  '⚔️', 'vfx_atk_buff',    'Tăng 25% lực công kích'),
('eff_shield',       'Lá Chắn Linh Hồn','BUFF',         'SHIELD',      'FLAT',          300.0, 1, 2, 'ON_DAMAGED', TRUE,  '🔰', 'vfx_holy_shield', 'Tạo khiên hấp thụ 300 điểm sát thương'),
('eff_provoke',      'Khiêu Khích',    'CROWD_CONTROL', 'ACTION_LOCK', 'ABSOLUTE_STATE', 1.0, 1, 1, 'ON_HIT',     TRUE,  '💢', 'vfx_taunt_roar',  'Buộc mục tiêu phải tấn công người thi triển'),
('eff_holy_regen',   'Thần Ân Trị Liệu','HOT',          'HP',          'PERCENTAGE',     0.10, 1, 3, 'TURN_START', TRUE,  '✨', 'vfx_holy_regen',  'Mỗi hiệp hồi phục 10% lượng máu tối đa')
ON CONFLICT (effect_id) DO NOTHING;

-- 3. Khởi tạo 9 Ô Bàn Cờ 3x3 (Board Cells)
INSERT INTO board_cell_configs (slot_id, col, row, line_type, terrain_type, tile_buff_effect_id, hazard_damage_pct, preferred_roles, penalty_roles, description)
VALUES
(1, 0, 0, 'FRONT', 'HIGH_GROUND',    'eff_armor_break', 0.0, '["VANGUARD", "WARRIOR"]'::jsonb, '["MARKSMAN"]'::jsonb, 'Tiền tuyến góc trái - Cao điểm phòng hộ'),
(2, 1, 0, 'FRONT', 'NORMAL',         NULL,              0.0, '["VANGUARD"]'::jsonb,            '["MARKSMAN", "MAGE"]'::jsonb, 'Tiền tuyến chính diện - Hứng sát thương trung tâm'),
(3, 2, 0, 'FRONT', 'HIGH_GROUND',    'eff_armor_break', 0.0, '["VANGUARD", "WARRIOR"]'::jsonb, '["MARKSMAN"]'::jsonb, 'Tiền tuyến góc phải'),
(4, 0, 1, 'MID',   'NORMAL',         NULL,              0.0, '["WARRIOR", "ASSASSIN"]'::jsonb,  '[]'::jsonb, 'Trung quân sườn trái'),
(5, 1, 1, 'MID',   'HOLY_SANCTUARY', 'eff_holy_regen',  0.0, '["WARRIOR", "MAGE", "SUPPORT"]'::jsonb, '[]'::jsonb, 'Trọng tâm bàn cờ - Thánh đường ban phước hồi máu'),
(6, 2, 1, 'MID',   'NORMAL',         NULL,              0.0, '["WARRIOR", "ASSASSIN"]'::jsonb,  '[]'::jsonb, 'Trung quân sườn phải'),
(7, 0, 2, 'BACK',  'SHADOW_VEIL',    NULL,              0.0, '["MARKSMAN", "ASSASSIN"]'::jsonb, '["VANGUARD"]'::jsonb, 'Hậu quân trái - Ẩn nấp trong sương mù né aggro'),
(8, 1, 2, 'BACK',  'NORMAL',         NULL,              0.0, '["MAGE", "TACTICIAN"]'::jsonb,    '["VANGUARD"]'::jsonb, 'Hậu quân trung tâm - Đứng dàn kỹ năng diện rộng'),
(9, 2, 2, 'BACK',  'SHADOW_VEIL',    NULL,              0.0, '["MARKSMAN", "SUPPORT"]'::jsonb,  '["VANGUARD"]'::jsonb, 'Hậu quân phải - Vị trí cự xạ chuẩn chỉ')
ON CONFLICT (slot_id) DO NOTHING;

-- 4. Line Buffs cho Bàn cờ 3x3
INSERT INTO formation_line_configs (line_type, display_name, slot_indices, allowed_roles, penalty_roles, penalty_stats, line_buffs)
VALUES
('FRONT', 'Hàng Đầu', '[1, 2, 3]'::jsonb, '["VANGUARD", "WARRIOR"]'::jsonb, '["MARKSMAN", "SUPPORT"]'::jsonb, '{"damage_taken_pct": 0.20}'::jsonb, '{"hp_pct": 0.15, "def_pct": 0.12, "crit_res_pct": 0.05}'::jsonb),
('MID',   'Hàng Giữa', '[4, 5, 6]'::jsonb, '["WARRIOR", "ASSASSIN", "MARKSMAN", "MAGE"]'::jsonb, '[]'::jsonb, '{}'::jsonb, '{"dmg_dealt_pct": 0.08, "crit_rate_pct": 0.10, "armor_pen_pct": 0.05}'::jsonb),
('BACK',  'Hàng Sau', '[7, 8, 9]'::jsonb, '["MARKSMAN", "MAGE", "TACTICIAN", "SUPPORT"]'::jsonb, '["VANGUARD"]'::jsonb, '{"aggro_reduction_pct": 0.30, "speed_reduction_pct": 0.15}'::jsonb, '{"speed_flat": 10, "initial_morale_flat": 25, "cc_success_pct": 0.15}'::jsonb)
ON CONFLICT (line_type) DO NOTHING;

-- 5. Kỹ năng mẫu & liên kết hiệu ứng
INSERT INTO skills (skill_id, name, skill_quality, is_skill_common, trigger_type, execution_priority, cost_morale, cooldown_turns, target_pattern, target_rule, damage_type, damage_multiplier, can_crit, effects_applied, description)
VALUES
('skill_guanyu_ult', 'Uy Chấn Hoa Hạ', 'LEGENDARY', FALSE, 'ULTIMATE', 1, 100, 0, 'PIERCE_ROW', 'ENEMY_NEAREST', 'PHYSICAL', 2.80, TRUE, '[{"effect_id": "eff_stun", "target": "PrimaryTarget", "chance": 0.40}, {"effect_id": "eff_armor_break", "target": "AllTargetsInScope", "chance": 1.00}]'::jsonb, 'Chém dọc toàn bộ một hàng ngang, gây sát thương khủng, giảm 30% giáp và tỷ lệ choáng'),
('skill_guanyu_normal', 'Thanh Long Trảm', 'COMMON', FALSE, 'NORMAL_ATTACK', 3, 0, 0, 'DIRECT_LANE', 'ENEMY_NEAREST', 'PHYSICAL', 1.00, TRUE, '[]'::jsonb, 'Đòn đánh đơn hàng gần nhất, tích lũy 25 điểm sĩ khí')
ON CONFLICT (skill_id) DO NOTHING;

-- Liên kết vào bảng quan hệ chuẩn skill_effects
INSERT INTO skill_effects (skill_id, effect_id, target_scope, proc_chance, custom_value, custom_duration)
VALUES
('skill_guanyu_ult', 'eff_stun',        'PRIMARY_TARGET',       0.40, NULL,  1),
('skill_guanyu_ult', 'eff_armor_break', 'ALL_TARGETS_IN_SCOPE', 1.00, -0.30, 2)
ON CONFLICT DO NOTHING;

-- 6. Tướng mẫu Quan Vũ
INSERT INTO card_templates (card_template_id, name, title, faction, rarity_id, recommended_role, base_hp, base_atk, base_def, base_speed, base_crit_rate, base_crit_dmg, initial_morale, max_morale)
VALUES
('hero_guan_yu', 'Quan Vũ', 'Võ Thánh', 'SHU', 'SSR', 'WARRIOR', 4200, 780, 360, 105, 0.150, 1.500, 50, 100)
ON CONFLICT (card_template_id) DO NOTHING;

INSERT INTO card_template_skills (card_template_id, skill_id, unlock_at_star)
VALUES
('hero_guan_yu', 'skill_guanyu_normal', 1),
('hero_guan_yu', 'skill_guanyu_ult', 1)
ON CONFLICT DO NOTHING;

INSERT INTO card_template_buffs (card_template_id, effect_id, unlock_at_level, unlock_at_star)
VALUES
('hero_guan_yu', 'eff_atk_up', 1, 3)
ON CONFLICT DO NOTHING;
```

---

## 4. ÁNH XẠ STRUCTS RUST (`crates/data_schema`)

```rust
// crates/data_schema/src/models.rs
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EffectCategory {
    Buff,
    Debuff,
    CrowdControl,
    Dot,
    Hot,
    TriggerPassive,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StatTarget {
    Hp,
    Atk,
    Def,
    Speed,
    CritRate,
    CritDmg,
    Morale,
    Shield,
    ActionLock,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CalculationType {
    Percentage,
    Flat,
    AbsoluteState,
    CustomFormula,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct StatusEffectRow {
    pub effect_id: String,
    pub name: String,
    pub effect_category: EffectCategory,
    pub stat_target: StatTarget,
    pub calculation_type: CalculationType,
    #[serde(default)]
    pub base_value: f64,
    #[serde(default = "default_stack")]
    pub max_stacks: i16,
    #[serde(default = "default_duration")]
    pub duration_turns: i32,
    pub tick_trigger: String,
    #[serde(default = "default_true")]
    pub is_dispellable: bool,
    pub icon: Option<String>,
    pub vfx_prefab: Option<String>,
    pub description: Option<String>,
}

fn default_stack() -> i16 { 1 }
fn default_duration() -> i32 { 1 }
fn default_true() -> bool { true }

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BoardCellConfigRow {
    pub slot_id: u8, // 1..9
    pub col: u8,     // 0..2
    pub row: u8,     // 0..2
    pub line_type: String,
    pub terrain_type: String,
    pub tile_buff_effect_id: Option<String>,
    #[serde(default)]
    pub hazard_damage_pct: f64,
    #[serde(default)]
    pub preferred_roles: Vec<String>,
    #[serde(default)]
    pub penalty_roles: Vec<String>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SkillEffectRow {
    pub mapping_id: i64,
    pub skill_id: String,
    pub effect_id: String,
    pub target_scope: String,
    pub proc_chance: f64,
    pub custom_value: Option<f64>,
    pub custom_duration: Option<i32>,
}
```

---

## 5. THIẾT KẾ QUY CHUẨN CRUD API (FULL CRUD SPECIFICATION)

Tất cả các API quản trị Master Data yêu cầu header xác thực:
`X-Admin-Key: <ADMIN_SECRET_KEY>` hoặc `Authorization: Bearer <ADMIN_SECRET_KEY>`.

```
Base Route: /api/admin/
```

### 5.1. Bàn cờ & Cấu hình ô cờ (Board & Cell Configs)

| Hành động | Phương thức | Endpoint | Mô tả |
| :--- | :--- | :--- | :--- |
| **List Cells** | `GET` | `/api/admin/board/cells` | Lấy danh sách toàn bộ 9 ô bàn cờ $3 \times 3$ kèm hiệu ứng ô |
| **Get Cell** | `GET` | `/api/admin/board/cells/:slot_id` | Lấy chi tiết cấu hình và buff ô cờ theo `slot_id` (1..9) |
| **Upsert Cell** | `POST` | `/api/admin/board/cells` | Tạo mới hoặc cập nhật ô cờ (Terrain, Buff ID, Roles) |
| **Update Cell** | `PUT` | `/api/admin/board/cells/:slot_id` | Cập nhật nhanh địa hình, buff gán vào ô cụ thể |
| **Delete Cell Buff** | `DELETE` | `/api/admin/board/cells/:slot_id/buff` | Gỡ bỏ hiệu ứng môi trường khỏi ô cờ |
| **List Line Configs** | `GET` | `/api/admin/lines` | Lấy cấu hình Front/Mid/Back |
| **Save Line Config** | `POST` | `/api/admin/lines` | Tạo/Cập nhật cấu hình buff hàng trận và penalty |

#### Payload Upsert Cell (`POST /api/admin/board/cells`):
```json
{
  "slot_id": 5,
  "col": 1,
  "row": 1,
  "line_type": "MID",
  "terrain_type": "HOLY_SANCTUARY",
  "tile_buff_effect_id": "eff_holy_regen",
  "hazard_damage_pct": 0.0,
  "preferred_roles": ["WARRIOR", "MAGE", "SUPPORT"],
  "penalty_roles": [],
  "description": "Thánh địa trung tâm hồi máu"
}
```

---

### 5.2. Hiệu ứng & Buff/Debuff (Status Effects Registry)

| Hành động | Phương thức | Endpoint | Mô tả |
| :--- | :--- | :--- | :--- |
| **List Effects** | `GET` | `/api/admin/effects?category=BUFF` | Danh sách hiệu ứng, hỗ trợ filter theo danh mục |
| **Get Effect** | `GET` | `/api/admin/effects/:effect_id` | Lấy chi tiết thông số của 1 hiệu ứng |
| **Create Effect** | `POST` | `/api/admin/effects` | Đăng ký một hiệu ứng mới vào hệ thống |
| **Update Effect** | `PUT` | `/api/admin/effects/:effect_id` | Sửa công thức, thời gian, stacks, vfx của hiệu ứng |
| **Delete Effect** | `DELETE` | `/api/admin/effects/:effect_id` | Xóa hiệu ứng khỏi master registry |

#### Payload Create/Update Effect (`POST /api/admin/effects`):
```json
{
  "effect_id": "eff_freeze_curse",
  "name": "Băng Nguyền Hàn Băng",
  "effect_category": "CROWD_CONTROL",
  "stat_target": "SPEED",
  "calculation_type": "PERCENTAGE",
  "base_value": -0.40,
  "max_stacks": 2,
  "duration_turns": 2,
  "tick_trigger": "TURN_START",
  "is_dispellable": true,
  "icon": "🧊",
  "vfx_prefab": "vfx_deep_freeze",
  "description": "Giảm 40% tốc độ hành động, làm chậm nhịp ra đòn"
}
```

---

### 5.3. Kỹ năng của Tướng & Gán Hiệu ứng (Skills & Skill Effects)

| Hành động | Phương thức | Endpoint | Mô tả |
| :--- | :--- | :--- | :--- |
| **List Skills** | `GET` | `/api/admin/skills` | Lấy danh sách toàn bộ kỹ năng |
| **Get Skill** | `GET` | `/api/admin/skills/:skill_id` | Lấy chi tiết kỹ năng cùng các hiệu ứng đính kèm |
| **Create/Save Skill** | `POST` | `/api/admin/skills` | Thêm mới hoặc ghi đè cấu hình kỹ năng |
| **Update Skill** | `PUT` | `/api/admin/skills/:skill_id` | Sửa tham số cooldown, morale, damage mult, pattern |
| **Delete Skill** | `DELETE` | `/api/admin/skills/:skill_id` | Xóa kỹ năng khỏi hệ thống |
| **Attach Effect** | `POST` | `/api/admin/skills/:skill_id/effects` | Gán hiệu ứng buff/debuff vào kỹ năng với tỉ lệ proc |
| **Detach Effect** | `DELETE` | `/api/admin/skills/:skill_id/effects/:effect_id` | Gỡ hiệu ứng khỏi kỹ năng |

#### Payload Attach Effect (`POST /api/admin/skills/skill_guanyu_ult/effects`):
```json
{
  "effect_id": "eff_burn",
  "target_scope": "ALL_TARGETS_IN_SCOPE",
  "proc_chance": 0.85,
  "custom_value": -0.10,
  "custom_duration": 2
}
```

---

### 5.4. Mẫu Tướng & Thẻ bài (Card Templates & Innate Buffs)

| Hành động | Phương thức | Endpoint | Mô tả |
| :--- | :--- | :--- | :--- |
| **List Templates** | `GET` | `/api/admin/templates?faction=SHU` | Lấy danh sách mẫu tướng, hỗ trợ lọc theo Phe/Phẩm |
| **Get Template** | `GET` | `/api/admin/templates/:id` | Lấy toàn bộ chỉ số, danh sách skill và buff nội tại |
| **Create Template** | `POST` | `/api/admin/templates` | Tạo tướng mới |
| **Update Template** | `PUT` | `/api/admin/templates/:id` | Sửa chỉ số cơ bản, role, HP/ATK/DEF/SPD |
| **Delete Template** | `DELETE` | `/api/admin/templates/:id` | Xóa tướng mẫu |
| **Bind Skill** | `POST` | `/api/admin/templates/:id/skills` | Gán kỹ năng cho tướng và điều kiện sao mở khóa |
| **Bind Innate Buff**| `POST` | `/api/admin/templates/:id/buffs` | Gán hiệu ứng bị động vĩnh viễn cho tướng |

#### Payload Create Template (`POST /api/admin/templates`):
```json
{
  "card_template_id": "hero_zhao_yun",
  "name": "Triệu Vân",
  "title": "Thường Thắng Tướng Quân",
  "faction": "SHU",
  "rarity_id": "SSR",
  "recommended_role": "WARRIOR",
  "base_hp": 3800,
  "base_atk": 820,
  "base_def": 340,
  "base_speed": 115,
  "base_crit_rate": 0.20,
  "base_crit_dmg": 1.60,
  "base_block_rate": 0.05,
  "base_dodge_rate": 0.15,
  "initial_morale": 60,
  "max_morale": 100
}
```

---

### 5.5. Phẩm cấp (Rarity Configs)

| Hành động | Phương thức | Endpoint | Mô tả |
| :--- | :--- | :--- | :--- |
| **List Rarities** | `GET` | `/api/admin/rarities` | Danh sách cấu hình R, SR, SSR, UR |
| **Save Rarity** | `POST` | `/api/admin/rarities` | Tạo hoặc cập nhật hệ số tăng trưởng chỉ số theo phẩm |
| **Delete Rarity** | `DELETE` | `/api/admin/rarities/:rarity_id` | Xóa cấu hình phẩm cấp |

---

### 5.6. Bố trí Đội hình Người chơi (Player Formations)

| Hành động | Phương thức | Endpoint | Mô tả |
| :--- | :--- | :--- | :--- |
| **Get Formation** | `GET` | `/api/user/formation?type=PVP_ATTACK` | Lấy đội hình 9 ô của người chơi |
| **Save Formation** | `POST` | `/api/user/formation` | Cập nhật vị trí thẻ bài trên lưới 3x3 |
| **Clear Formation**| `POST` | `/api/user/formation/clear` | Tháo toàn bộ thẻ bài về kho dự bị |

#### Payload Save Formation (`POST /api/user/formation`):
```json
{
  "formation_type": "PVP_ATTACK",
  "slots": {
    "slot_1": 1024,
    "slot_2": 1025,
    "slot_3": null,
    "slot_4": null,
    "slot_5": 1028,
    "slot_6": null,
    "slot_7": 1030,
    "slot_8": null,
    "slot_9": 1032
  }
}
```

---

### 5.7. Người chơi & Thẻ bài Sở hữu (Users & Inventory)

| Hành động | Phương thức | Endpoint | Mô tả |
| :--- | :--- | :--- | :--- |
| **List Users** | `GET` | `/api/admin/users` | Danh sách tài khoản |
| **Create User** | `POST` | `/api/admin/users` | Tạo tài khoản kèm cấp vàng/ngọc ban đầu |
| **Update User** | `PUT` | `/api/admin/users/:username` | Cập nhật Vàng, Ngọc, Level, Rank ELO, Mật khẩu |
| **Delete User** | `DELETE` | `/api/admin/users/:username` | Xóa tài khoản và dữ liệu liên quan |
| **Grant Card** | `POST` | `/api/admin/users/grant-card` | Thêm thẻ tướng trực tiếp vào túi đồ người chơi |
| **Remove Card** | `POST` | `/api/admin/users/remove-card` | Xóa thẻ tướng khỏi túi đồ người chơi |

---


### 5.8. Quản Lý Kỹ Năng Tướng (Hero Skills Mapping CRUD)

| Hành động | Phương thức | Endpoint | Mô tả |
| :--- | :--- | :--- | :--- |
| **List Template Skills** | `GET` | `/api/admin/template_skills` | Danh sách liên kết kỹ năng gán cho từng mẫu tướng |
| **Save Template Skill** | `POST` | `/api/admin/template_skills/save` | Gán kỹ năng mới hoặc đổi slot/số sao mở khóa |
| **Delete Template Skill** | `POST` | `/api/admin/template_skills/delete` | Gỡ kỹ năng khỏi tướng |

#### Payload Save Template Skill (`POST /api/admin/template_skills/save`):
```json
{
  "template_id": "hero_zhao_yun",
  "skill_id": "skill_zhaoyun_ult",
  "slot_type": "ULTIMATE",
  "unlock_star": 1
}
```

---

### 5.9. Quản Lý Kích Duyên Đội Hình (Formation Synergies CRUD)

| Hành động | Phương thức | Endpoint | Mô tả |
| :--- | :--- | :--- | :--- |
| **List Synergies** | `GET` | `/api/admin/synergies` | Danh sách kích duyên hệ phái, phe phái và đặc biệt |
| **Save Synergy** | `POST` | `/api/admin/synergies/save` | Thêm mới hoặc cập nhật kích duyên, số tướng và stat buffs |
| **Delete Synergy** | `POST` | `/api/admin/synergies/delete` | Xóa kích duyên khỏi hệ thống |

#### Payload Save Synergy (`POST /api/admin/synergies/save`):
```json
{
  "synergy_id": "synergy_vanguard",
  "name": "Thiết Vệ",
  "synergy_type": "CLASS",
  "trigger_count": 2,
  "icon": "🔰",
  "hero_ids": ["zhao_yun", "sun_ce", "dian_wei"],
  "stat_buffs": {
    "def": 35.0,
    "hp": 150.0
  },
  "is_active": true,
  "description": "Tăng 35 Phòng thủ và 150 Máu cho toàn đội khi có ít nhất 2 tướng Thiết Vệ ra trận."
}
```

---

## 5.10. QUY CHUẨN BIỂU TƯỢNG HIỆU ỨNG (STATUS EFFECT ICONS) TRONG CHIẾN ĐẤU

Để người chơi nắm bắt diễn biến trận đấu trực quan, các trạng thái Buff, Debuff, Khống chế và Kích duyên được gắn kèm Icon emoji tương ứng, hiển thị trên đỉnh đầu thẻ bài (`EffectIconsRoot2d`):

| Icon | Tên Trạng Thái / Duyên | Phân Loại | Hiệu Ứng Trong Trận Đấu |
| :---: | :--- | :--- | :--- |
| `🛡️` | **Khiên Hộ Thể (Shield)** | BUFF | Hấp thụ sát thương trực tiếp thay cho HP |
| `🔥` | **Thiêu Đốt (Burn)** | DOT | Gây sát thương hỏa mỗi hiệp / lan tỏa theo hàng |
| `☠️` | **Trúng Độc (Poison)** | DOT | Trừ máu liên tục qua các hiệp đấu |
| `❄️` | **Băng Giá (Freeze / Slow)**| CROWD_CONTROL | Giảm tốc độ hồi thanh hành động ATB / đóng băng lượt |
| `💫` | **Choáng Váng (Stun)** | CROWD_CONTROL | Ngắt lượt hành động, giảm thanh ATB |
| `⚔️` | **Cuồng Nộ / Công Buff** | BUFF | Tăng chỉ số Tấn Công (ATK) |
| `🔰` | **Duyên Thiết Vệ (Vanguard)**| SYNERGY | Kích hoạt khi có từ 2 tướng Thiết Vệ (Tăng Thủ & Máu) |
| `🏹` | **Duyên Thần Xạ (Sharpshooter)**| SYNERGY | Kích hoạt khi có từ 2 tướng Thần Xạ (Tăng Công & Bạo) |
| `⚡` | **Duyên Kỳ Môn (Arcanist)** | SYNERGY | Kích hoạt khi có từ 2 tướng Kỳ Môn (Tăng Công Phép & Sĩ khí) |
| `🗡️` | **Duyên Ám Ảnh (Shadow)** | SYNERGY | Kích hoạt khi có từ 2 tướng Ám Ảnh (Tăng mạnh Bạo Kích) |
| `⚕️` | **Duyên Thần Ân (Divine)** | SYNERGY | Kích hoạt khi có tướng Hỗ Trợ (Hồi phục HP liên tục) |
| `🩸` | **Chảy Máu (Bleed)** | DOT | Sát thương vật lý DoT khi dính đòn bạo kích/sát thủ |
| `🌿` | **Hồi Sinh / Hồi Phục (Heal)** | HOT | Phục hồi HP theo thời gian |

### Cơ Chế Render Icon Overhead:
- Mỗi đơn vị khi xuất hiện trên bàn cờ được gắn `ActiveStatusEffects` và node con `EffectIconsRoot2d`.
- Hệ thống `update_unit_effect_icons` lắng nghe danh sách hiệu ứng đang hoạt động, tự động định dạng chuỗi icon (vd: `🔰 🛡️ 🔥`) và hiển thị chữ nổi 2D phía trên thanh HP.
- Khi hết thời gian duy trì hoặc kết thúc trận đấu, danh sách hiệu ứng được dọn dẹp sạch sẽ.

## 6. RUNTIME COMPUTATION PIPELINE (DATABASE -> ENGINE)

Khi `apps/server` khởi tạo trận đấu, chu trình tính toán tổng hợp dữ liệu chiến đấu diễn ra theo quy trình:

```
[Load Player Formations & Cards]
              │
              ▼
[Apply Growth Formula (Level, Stars, Breakthrough)]
              │
              ▼
[Fetch Board Cell & Line Buffs (Slot 1..9 Mapping)]
       ├─ Kiểm tra Preferred Role  ──► Nhận Tile Buff & Line Buff
       └─ Kiểm tra Penalty Role    ──► Áp dụng Penalty Stat Modifiers
              │
              ▼
[Compile Entity Status Effects]
       ├─ Innate Buffs (Tướng)
       ├─ Tile Environment Buffs (Ô cờ)
       └─ Active Skill Effects Registry
              │
              ▼
[Spawn Battle Units into Combat Engine State]
```

### Công thức chỉ số cuối cùng (Final Stat Formula):
$$\text{Stat}_{\text{final}} = \left( \left(\text{BaseStat} \times \text{RarityMult} \times (1 + \text{GrowthRate} \times (\text{Lv} - 1))\right) \times (1 + \text{StarBonus}) + \text{LineBuffFlat} + \text{TileBuffFlat} \right) \times (1 + \text{LineBuffPct} + \text{TileBuffPct})$$
