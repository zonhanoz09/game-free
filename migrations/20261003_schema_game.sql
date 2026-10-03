-- ============================================================================
-- DATABASE SPECIFICATION & PERSISTENCE ARCHITECTURE: 3X3 TACTICAL CARD ENGINE
-- Dialect: PostgreSQL 15+ (với JSONB Support)
-- Reference: docs/game_design/schema_game.md
-- ============================================================================

-- ----------------------------------------------------------------------------
-- 1. Phân hệ Master Game Data
-- ----------------------------------------------------------------------------

-- 1.1. Bảng cấu hình phẩm cấp
CREATE TABLE IF NOT EXISTS rarity_configs (
    rarity_id VARCHAR(16) PRIMARY KEY, -- 'R', 'SR', 'SSR', 'UR'
    display_name VARCHAR(64) NOT NULL,
    base_stat_multiplier NUMERIC(4, 2) NOT NULL, -- 1.00, 1.20, 1.50, 1.85
    growth_rate_per_level NUMERIC(4, 3) NOT NULL, -- 0.050, 0.075, 0.100, 0.135
    max_star_rating SMALLINT NOT NULL, -- 5, 6, 7, 9
    max_skills_allowed SMALLINT NOT NULL DEFAULT 4,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 1.2. Bảng mẫu thẻ tướng gốc
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

-- 1.3. Bảng kỹ năng và hiệu ứng
CREATE TABLE IF NOT EXISTS skills (
    skill_id VARCHAR(64) PRIMARY KEY, -- 'skill_guanyu_ult'
    name VARCHAR(128) NOT NULL,
    skill_quality VARCHAR(32) NOT NULL, -- 'COMMON', 'RARE', 'EPIC', 'LEGENDARY'
    is_skill_common BOOLEAN DEFAULT FALSE,
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

    -- Danh sách hiệu ứng đính kèm (JSONB Schema: Type, Value, Chance, Duration)
    effects_applied JSONB DEFAULT '[]'::jsonb,

    description TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 1.4. Bảng liên kết Tướng - Kỹ năng mặc định
CREATE TABLE IF NOT EXISTS card_template_skills (
    card_template_id VARCHAR(64) REFERENCES card_templates(card_template_id) ON DELETE CASCADE,
    skill_id VARCHAR(64) REFERENCES skills(skill_id) ON DELETE CASCADE,
    unlock_at_star SMALLINT DEFAULT 1,
    PRIMARY KEY (card_template_id, skill_id)
);

-- 1.5. Bảng cấu hình Buff Hàng và Quy tắc vị trí trên lưới 3x3
CREATE TABLE IF NOT EXISTS formation_line_configs (
    line_type VARCHAR(16) PRIMARY KEY, -- 'FRONT', 'MID', 'BACK'
    display_name VARCHAR(64) NOT NULL,
    slot_indices JSONB NOT NULL, -- [1, 2, 3] cho FRONT, [4, 5, 6] cho MID, [7, 8, 9] cho BACK
    allowed_roles JSONB NOT NULL, -- ["VANGUARD", "WARRIOR"]
    penalty_roles JSONB NOT NULL, -- ["MARKSMAN", "SUPPORT"]
    penalty_stats JSONB DEFAULT '{}'::jsonb, -- {"damage_taken_pct": 0.20}
    line_buffs JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- ----------------------------------------------------------------------------
-- 2. Phân hệ Runtime Data (Player Progress)
-- ----------------------------------------------------------------------------

-- 2.1. Bảng người dùng
CREATE TABLE IF NOT EXISTS users (
    user_id BIGSERIAL PRIMARY KEY,
    username VARCHAR(64) UNIQUE NOT NULL,
    display_name VARCHAR(128) NOT NULL,
    level INT DEFAULT 1,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 2.2. Bảng thẻ bài người chơi sở hữu
CREATE TABLE IF NOT EXISTS player_cards (
    player_card_id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    card_template_id VARCHAR(64) NOT NULL REFERENCES card_templates(card_template_id),
    current_level INT DEFAULT 1,
    current_exp BIGINT DEFAULT 0,
    current_star SMALLINT DEFAULT 1,
    breakthrough_tier SMALLINT DEFAULT 0,
    equipped_common_skill_id VARCHAR(64) REFERENCES skills(skill_id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT uq_user_card_instance UNIQUE (user_id, card_template_id)
);

CREATE INDEX IF NOT EXISTS idx_player_cards_user ON player_cards(user_id);

-- 2.3. Bảng bố trí đội hình lưới 3x3 của người chơi
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

-- ----------------------------------------------------------------------------
-- 3. Master Seed Data
-- ----------------------------------------------------------------------------

-- Khởi tạo Phẩm cấp
INSERT INTO rarity_configs (rarity_id, display_name, base_stat_multiplier, growth_rate_per_level, max_star_rating, max_skills_allowed)
VALUES
('R',   'Thường',     1.00, 0.050, 5, 2),
('SR',  'Hiếm',       1.20, 0.075, 6, 3),
('SSR', 'Cực Phẩm',   1.50, 0.100, 7, 4),
('UR',  'Thần Tướng', 1.85, 0.135, 9, 5)
ON CONFLICT (rarity_id) DO NOTHING;

-- Khởi tạo Line Buffs cho Bàn cờ 3x3
INSERT INTO formation_line_configs (line_type, display_name, slot_indices, allowed_roles, penalty_roles, penalty_stats, line_buffs)
VALUES
(
    'FRONT', 'Hàng Đầu', '[1, 2, 3]'::jsonb,
    '["VANGUARD", "WARRIOR"]'::jsonb,
    '["MARKSMAN", "SUPPORT"]'::jsonb,
    '{"damage_taken_pct": 0.20}'::jsonb,
    '{"hp_pct": 0.15, "def_pct": 0.12, "crit_res_pct": 0.05}'::jsonb
),
(
    'MID', 'Hàng Giữa', '[4, 5, 6]'::jsonb,
    '["WARRIOR", "ASSASSIN", "MARKSMAN", "MAGE"]'::jsonb,
    '[]'::jsonb,
    '{}'::jsonb,
    '{"dmg_dealt_pct": 0.08, "crit_rate_pct": 0.10, "armor_pen_pct": 0.05}'::jsonb
),
(
    'BACK', 'Hàng Sau', '[7, 8, 9]'::jsonb,
    '["MARKSMAN", "MAGE", "TACTICIAN", "SUPPORT"]'::jsonb,
    '["VANGUARD"]'::jsonb,
    '{"aggro_reduction_pct": 0.30, "speed_reduction_pct": 0.15}'::jsonb,
    '{"speed_flat": 10, "initial_morale_flat": 25, "cc_success_pct": 0.15}'::jsonb
)
ON CONFLICT (line_type) DO NOTHING;

-- Khởi tạo Kỹ năng mẫu
INSERT INTO skills (skill_id, name, skill_quality, is_skill_common, trigger_type, execution_priority, cost_morale, cooldown_turns, target_pattern, target_rule, damage_type, damage_multiplier, can_crit, effects_applied, description)
VALUES
(
    'skill_guanyu_ult', 'Uy Chấn Hoa Hạ', 'LEGENDARY', FALSE, 'ULTIMATE', 1, 100, 0,
    'PIERCE_ROW', 'ENEMY_NEAREST', 'PHYSICAL', 2.80, TRUE,
    '[{"status_type": "Stun", "target": "PrimaryTarget", "chance": 0.40, "duration_turns": 1}, {"status_type": "ArmorBreak", "target": "AllTargetsInScope", "chance": 1.00, "value": -0.30, "duration_turns": 2}]'::jsonb,
    'Chém dọc toàn bộ một hàng ngang, gây sát thương lớn và giảm giáp'
),
(
    'skill_guanyu_normal', 'Thanh Long Trảm', 'COMMON', FALSE, 'NORMAL_ATTACK', 3, 0, 0,
    'DIRECT_LANE', 'ENEMY_NEAREST', 'PHYSICAL', 1.00, TRUE,
    '[]'::jsonb,
    'Đòn đánh thường tích lũy 25 sĩ khí'
)
ON CONFLICT (skill_id) DO NOTHING;

-- Khởi tạo Tướng Quan Vũ
INSERT INTO card_templates (card_template_id, name, title, faction, rarity_id, recommended_role, base_hp, base_atk, base_def, base_speed, base_crit_rate, base_crit_dmg, initial_morale, max_morale)
VALUES
('hero_guan_yu', 'Quan Vũ', 'Võ Thánh', 'SHU', 'SSR', 'WARRIOR', 4200, 780, 360, 105, 0.150, 1.500, 50, 100)
ON CONFLICT (card_template_id) DO NOTHING;

-- Gán Kỹ năng cho Quan Vũ
INSERT INTO card_template_skills (card_template_id, skill_id, unlock_at_star)
VALUES
('hero_guan_yu', 'skill_guanyu_normal', 1),
('hero_guan_yu', 'skill_guanyu_ult', 1)
ON CONFLICT (card_template_id, skill_id) DO NOTHING;
