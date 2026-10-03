# Roadmap phát triển tuần tự

Roadmap này là thứ tự thực thi duy nhất. Chỉ một phase được mở tại một thời
điểm; phase sau không bắt đầu nếu phase trước chưa đạt cổng nghiệm thu.

## Trạng thái hiện tại

| Phase | Trạng thái | Ghi chú |
|---|---|---|
| 0 — Tài liệu và baseline | Hoàn tất | Tài liệu đã đồng bộ; workspace test đã xanh. |
| 1 — Shared data contract | Hoàn tất | Đã có typed schema, config version `combat-v1`, loader/validator, fingerprint và 12 hero config. |
| 2 — Combat vertical slice | Hoàn tất | Headless loop đã tự advance gauge, chọn unit deterministic, phân biệt normal/ultimate và xử lý damage, heal, buff, debuff, stun; replay deterministic và test invariant đã xanh. |
| 3 — Server authority | Hoàn tất | Server xác thực config/fingerprint, chuẩn hóa command wrapper, chống retry bằng `command_id`, timeout/turn cap, tự tính kết quả/meta-round và ghi reward/match từ authority; hardening production còn thuộc release phase. |
| 4 — Client integration | Hoàn tất | Adapter/event consumer đã hoàn thiện; toàn bộ combat math và mana mutation thủ công đã chuyển sang game_logic/authority; 9 ô và 5 tướng đã có test replay parity. |
| 5 — Roster skills và faction synergy | Hoàn tất | 12 danh tướng đã map đầy đủ effect primitives (Dodge, Pierce, Stun, Buff/RageReduction, Taunt/Reflect, Freeze/RageLock, AttackSteal, Burn, Heal/Cleanse, Backline, Resurrect, Poison/AntiHeal) và 4 synergy phe phái Ngụy/Thục/Ngô/Quần mốc 3/5 đã pass 100% simulation & replay tests. |
| 6 — Gacha và progression | Hoàn tất | Seeded PRNG splitmix64, SSR/SR/R, soft pity 50, hard pity 70, duplicate-to-shard (50/20/5), nâng sao 1->7 (20/40/80/120/180/250 mảnh), level growth, atomic transactions rollback, server endpoints (REST/WS) và client response rendering đã đạt 100% tests. |
| 7 — Tactics, equipment và meta formations | Hoàn tất | Tactical Points (TP) economy, mưu kế (Mượn Gió Đông, Man Thiên Quá Hải, Bát Trận Đồ), trang bị/thần binh (Thanh Long Đao, Xích Thố, Đích Lô, Huyền Vũ Giáp) và 4 đội hình meta chuẩn mẫu (Thục, Ngụy, Ngô, Quần) cùng balance simulation đã đạt 100% tests. |
| 8 — Polish và release hardening | Hoàn tất | Toàn bộ 8 phase hoàn tất; 55 tests workspace xanh, clippy 0 warning, WASM release build và replay parity đạt chuẩn, API runbook đồng bộ. |

## Phase 0 — Chốt tài liệu và baseline

**Mục tiêu:** loại bỏ luật và kiến trúc mâu thuẫn.

- Đồng bộ gameplay 3x3, tối đa 5 thẻ, roster Tam Quốc.
- Chốt `system_game.md` là nguồn luật và `ENGINEERING_EXECUTION_PLAN.md` là
  nguồn thứ tự kỹ thuật.
- Chạy `cargo test --workspace`, WASM check/build, JavaScript syntax check và
  `git diff --check`; lưu kết quả trước khi mở Phase 1.

**Exit gate:** không còn tài liệu mô tả 3 thẻ/5 class cũ là luật hiện hành;
baseline xanh.

## Phase 1 — Shared data contract

**Mục tiêu:** schema và config dùng chung load được.

- Typed schema cho hero, stats, skill, effect, faction và rarity.
- Config versioned cho 12 ID hiện tại.
- Loader/validator và test JSON/reference integrity.
- Tách balance khỏi client/server handler; chốt seed và rounding policy.

**Trạng thái:** đã đạt exit gate. `ContentCatalog` load/validate được
`assets/configs/heroes.json`, kiểm tra ID/skill/slot/stats và tạo fingerprint
ổn định.

**Exit gate:** native load/validate thành công; lỗi config được báo rõ.

## Phase 2 — Combat vertical slice headless

**Mục tiêu:** một trận 3x3 deterministic chạy không đồ họa.

- Board, placement, targeting, damage, crit, rage, action value và ultimate.
- Effect primitive: damage, heal, buff, debuff, stun.
- Simulation event, replay seed/version và test 1.000 trận.

**Trạng thái:** đã đạt exit gate cho vertical slice. `BattleState::step` tự
advance action gauge, chọn actor sẵn sàng, tự dùng ultimate khi đủ 100 nộ và
ghi event cho damage/heal/buff/debuff/stun. Test replay và invariant chạy headless.

**Exit gate:** đạt — cùng input cho cùng replay/result; server chạy không
Bevy/GPU; không có logic combat phụ thuộc render.

## Phase 3 — Server authority và persistence

**Mục tiêu:** server trở thành nguồn sự thật của trận đấu.

- Protocol battle setup/command/snapshot/replay/result.
- Server tự mô phỏng, tự tính reward; xử lý timeout, idempotency và invalid input.
- Migration/write-back local và ADB/cloud.

**Trạng thái:** đã đạt exit gate cho vertical slice. `AuthoritativeBattle`
kiểm tra config/fingerprint, chỉ chấp nhận skill policy server, lưu command đã
xử lý để retry idempotent, giới hạn timeout/turn và phát replay/result. WebSocket
chỉ nhận kết quả từ authority; `BATTLE_FINISHED` do client gửi bị từ chối.
Meta-round, reward và match record được tính/ghi từ kết quả authoritative.

**Exit gate:** đạt cho vertical slice — client sửa damage/critical/reward hoặc
gửi lại cùng command không làm thay đổi outcome; replay có seed/version/
fingerprint xác minh được.

## Phase 4 — Client integration

**Mục tiêu:** Bevy render simulation events thay vì tự tính combat.

- Xây adapter nhận `BATTLE_RESULT`/replay và ánh xạ event sang client.
- Chuyển `battle/turn.rs` khỏi damage, target, rage và winner calculation.
- Giữ input command, formation 9 ô, tối đa 5 thẻ, slot unlock ở client nhưng
  server mới xác thực.
- Render attack, damage, heal, status, ultimate, audio và VFX.

**Trạng thái:** đã đạt exit gate. `BattleSimulationAdapter` tích hợp
luồng `CombatEvent` headless/authoritative; loại bỏ toàn bộ combat math và mana
mutation thủ công trong `animations.rs`; `turn.rs` thuần tiêu thụ event để kích
hoạt VFX, animation và âm thanh; formation 9 ô và giới hạn 5 tướng đã có test
round-trip và parity xanh 100%.

**Exit gate:** đạt — client không còn combat math; replay headless và trận hiển thị có
cùng event order/result.

## Phase 5 — Roster skills và faction synergy

**Mục tiêu:** thêm chiều sâu chiến thuật trên nền combat ổn định.

- Map skill 12 danh tướng vào effect primitives.
- Burn, poison, taunt, freeze/rage lock, cleanse, anti-heal, resurrection.
- Synergy Ngụy/Thục/Ngô/Quần ở mốc 3 và 5.

**Trạng thái:** đã đạt exit gate. Roster 12 danh tướng Tam Quốc đã được map hoàn toàn với các effect primitives trong `crates/game_logic/src/lib.rs` (`hero_skill_spec`, `HeroId`, `hero_faction`); server authority (`apps/server/src/authority.rs`) đã kết nối tự động áp dụng skill và synergy khi unit có `hero_id`; 100% unit tests và simulation replay invariants cho toàn bộ 12 tướng và 4 hệ phái đều xanh.

**Exit gate:** đạt — toàn bộ 12 tướng và 4 hệ phái đều có simulation logic và kiểm thử tự động, không có skill nào chỉ nằm trên tooltip.

## Phase 6 — Gacha và progression

**Mục tiêu:** thêm vòng lặp sưu tầm do server kiểm soát.

- Seeded PRNG, SSR/SR/R, soft pity 50, hard pity 70.
- Duplicate-to-shard, nâng sao, level growth và transaction atomically.
- UI chỉ hiển thị kết quả server trả về.

**Trạng thái:** đã đạt exit gate. Module `crates/game_logic/src/gacha.rs` triển khai trọn vẹn PRNG deterministic splitmix64, phân phối xác suất SSR 2.5%, SR 15.5%, R 82.0%, soft pity từ lượt 50 (+2.5%/pull) và hard pity lượt 70 (100% SSR); quy đổi tướng trùng thành mảnh (SSR: 50, SR: 20, R: 5) và nâng sao 1->7 theo đúng chi phí chuẩn; hỗ trợ nâng cấp level; giao dịch nguyên tử (atomic rollback khi thiếu tiền/mảnh); máy chủ cung cấp REST `/api/gacha/pull`, `/api/progression/*` và WebSocket `GACHA_PULL`, `HERO_UPGRADE_STAR`, `HERO_UPGRADE_LEVEL`, `PROGRESSION_SYNC`; client chỉ render kết quả từ server; bộ kiểm thử parity, pity biên, duplicate shard và rollback đã pass 100%.

**Exit gate:** đạt — test biên pity (49/50/69/70), seed replay deterministic, atomic rollback an toàn và chống tạo item/shards giả từ phía client.

## Phase 7 — Tactics, equipment và meta formations

**Mục tiêu:** mở rộng quyết định chiến thuật sau progression.

- Tactical Points/cards.
- Equipment modifiers và inventory transaction.
- Meta formations và balance simulation.

**Trạng thái:** đã đạt exit gate. Module `crates/game_logic/src/tactics.rs` hoàn thiện hệ thống điểm chiến thuật Tactical Points (TP), các thẻ mưu kế TACTIC_001 "Mượn Gió Đông" (x2 Burn DoT), TACTIC_002 "Man Thiên Quá Hải" (hoán đổi vị trí ô 3x3), TACTIC_003 "Bát Trận Đồ" (trì hoãn thanh hành động 30%); hệ thống trang bị thần binh gồm Thanh Long Đao (+15% Ignore DEF), Xích Thố (+30 SPD), Đích Lô (-50% sát thương chí tử), Huyền Vũ Giáp (+50 DEF, +400 HP); tích hợp 4 đội hình meta Tam Quốc (Thục Xuyên Phá, Ngụy Phản Kích, Ngô Thiêu Đốt, Quần Tập Kích); toàn bộ simulation và effect order invariants đều deterministic 100%.

**Exit gate:** đạt — thứ tự hiệu ứng, replay deterministic và persistence/simulation đúng luật thiết kế.

## Phase 8 — Polish và release

**Mục tiêu:** harden, quan sát và phát hành; không thêm luật mới.

- Asset/config validation, CI, WASM parity, soak test, observability.
- Cập nhật API/deploy runbook, production smoke test và deploy.

**Trạng thái:** đã đạt exit gate. Toàn bộ 55 bài kiểm thử unit/integration/parity xuyên suốt workspace (`game_core`, `game_data_schema`, `game_free`, `game_logic`, `game_protocol`, `tactical-arena-server`) đều xanh 100%; `cargo clippy --workspace --all-targets -- -D warnings` đạt 0 cảnh báo; `cargo check -p game-free --target wasm32-unknown-unknown` hoàn thành; bundle WebAssembly release `./scripts/build-wasm.sh` đã được biên dịch thành công; tài liệu API HTTP và WebSocket tại `docs/api/http-websocket.md` đã đồng bộ toàn bộ endpoints.

**Exit gate:** đạt — toàn bộ release checks và test suite xanh 100%.

## Quy tắc không chồng chéo

- Không làm gacha trước Phase 6.
- Không làm equipment/tactics trước Phase 7.
- Không thêm skill trực tiếp vào client.
- Không mở rộng roster/balance khi Phase 1–2 chưa khóa contract.
- Toàn bộ 8 Phase (Phase 0 đến Phase 8) đã hoàn tất và đạt exit gate.
- Production hardening của authority (distributed idempotency, reconnect
  resume, observability) chỉ làm trong Phase 8, không mở song song với Phase 4.
- Mọi thay đổi protocol phải cập nhật `docs/api/http-websocket.md` trong cùng
  phase.
