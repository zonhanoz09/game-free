# Engineering Execution Plan

## Mục đích

Đây là kế hoạch kỹ thuật duy nhất để biến GDD thành sản phẩm chạy được. Kế
hoạch áp dụng theo phase tuần tự; mỗi phase có đầu vào, đầu ra và exit gate.
Không bắt đầu phase sau khi phase trước chưa đạt gate.

## Trạng thái nền tảng hiện tại

- **Phase hiện tại:** Phase 4 đã đạt exit gate. Phase 5 (Skills và synergy) đang mở.
- Workspace đã có `apps/client`, `apps/server`, `crates/core`,
  `crates/protocol`, `crates/game_logic` và `crates/data_schema`.
- `crates/game_logic` đã có board slot mapping, targeting, damage integer
  formula, action gauge, battle state, battle loop, ultimate policy, effect
  resolution và replay events.
- `crates/data_schema` đã có type nền tảng cho faction, rarity, role, hero,
  stats và skill; `ContentCatalog` load/validate config `combat-v1` và tạo
  fingerprint ổn định.
- Server đã có profile/card/deck/battle-slot và migration roster Tam Quốc; các
  phần này phải được giữ tương thích trong quá trình refactor.
- PvE combat hiện còn được điều khiển ở client; PvP đã có transport/room state
  nhưng chưa phải headless authoritative simulation theo mục tiêu cuối.

## Quy tắc boundary

| Thành phần | Trách nhiệm | Không làm |
|---|---|---|
| `crates/core` | RNG seed, fixed math, common errors | Không biết Bevy/app |
| `crates/data_schema` | Struct config và validation | Không chạy combat |
| `crates/game_logic` | Board, targeting, damage, turn, effects, replay | Không phụ thuộc render/window |
| `crates/protocol` | Command, snapshot, event, replay DTO | Không chứa implementation combat |
| `apps/server` | Authority, persistence, HTTP/WebSocket | Không tin damage/result từ client |
| `apps/client` | Input, Bevy render, audio, VFX, UI | Không tính damage/target/reward |

## Phase 0 — Tài liệu và baseline

**Đầu vào:** code và tài liệu hiện tại.  
**Đầu ra:** gameplay, architecture và roadmap không mâu thuẫn.

- Chốt `system_game.md` là nguồn luật.
- Chốt `gameplay.md` là mô tả runtime hiện hành.
- Chốt `roadmap.md` là thứ tự sản phẩm.
- Chạy baseline test/build và ghi các giới hạn đang tồn tại.

**Exit gate:** không còn tài liệu mô tả 3 thẻ/5 class cũ là luật; baseline xanh;
không còn mâu thuẫn giữa trạng thái “đã có” và “mục tiêu kiến trúc”.

## Phase 1 — Data contract

**Đầu vào:** roster ID hiện tại và luật trong GDD.  
**Đầu ra:** config versioned load được ở native/WASM.

- Tạo typed schema cho hero, stats, skill, effect, faction, rarity và synergy.
- Tạo loader/validator và config release cho 12 danh tướng.
- Chốt fixed-point/rounding, seed, config version/hash.
- Viết migration/round-trip/reference tests.

**Trạng thái:** đã đạt exit gate; config release 12 tướng được load/validate,
round-trip test, invalid config test và fingerprint test đều có.

**Exit gate:** config hợp lệ load thành công; config sai bị từ chối rõ ràng.

## Phase 2 — Headless combat vertical slice

**Đầu vào:** data contract đã khóa.  
**Đầu ra:** một simulation 3x3 deterministic không Bevy.

- `BoardSlot`, `BoardState`, placement validation.
- Direct-line, Manhattan fallback, backline, low-HP, high-ATK, cross, row, column.
- Damage, defense, crit, faction modifier, rage, kill gain, action value, ultimate.
- Effect primitive trước: damage, heal, buff, debuff, stun.
- `SimulationEvent` và replay seed/version.
- Unit/property tests và 1.000 trận headless.

**Trạng thái:** đã đạt exit gate; `BattleState::step` điều khiển vòng lặp
deterministic và chọn ultimate ở 100 nộ. Effect primitives damage, heal, buff,
debuff, stun có event replay tương ứng; test 1.000 trận và test chọn ultimate
đều chạy headless.

**Exit gate:** đạt; cùng input cho cùng event/result và invariant
HP/rage/occupancy không bị phá.

## Phase 3 — Server authority

**Đầu vào:** simulation và replay contract.  
**Đầu ra:** server tự xác thực và tính kết quả.

- Protocol battle setup, command, replay và result đã có config fingerprint.
- Server authority runner dùng `game_logic`, không nhận damage/critical/reward do
  client tính; WebSocket hỗ trợ command wrapper hoặc payload phẳng.
- `command_id` chống retry lặp, turn cap/thời gian timeout, config enforcement,
  authoritative meta-round damage và match reward/write-back đã được nối.
- Migration write-back cho local/ADB/cloud.

**Trạng thái:** đã đạt exit gate vertical slice. Production hardening (distributed
idempotency, reconnect resume và observability) được để cho release hardening,
không chặn boundary authority hiện tại.

**Exit gate:** đạt — client không thể sửa outcome/reward bằng damage/critical
payload; replay xác minh được bằng seed/version/fingerprint.

## Phase 4 — Client adapter

**Đầu vào:** server protocol ổn định.  
**Đầu ra:** Bevy client render event.

- Chuyển `battle/turn.rs` và `animations.rs` sang adapter/event consumer từng
  bước.
- Giữ camera, VFX, audio, hit-stop, shake và UI ở client.
- Đồng bộ formation 9 ô, tối đa 5 thẻ, slot unlock và `HeroId`.
- Test replay event parity giữa headless và client.

**Trạng thái:** đã đạt exit gate. `BattleSimulationAdapter` tích hợp luồng
`CombatEvent` headless/authoritative; loại bỏ hoàn toàn combat math và mana
mutation thủ công trong `animations.rs`; `turn.rs` thuần tiêu thụ event để kích
hoạt VFX, animation và âm thanh; formation 9 ô và giới hạn 5 tướng đã có test
round-trip và parity xanh 100%.

**Exit gate:** đạt — client không còn combat math; trận hiển thị đúng event
order/result.

## Phase 5 — Skills và synergy

**Đầu vào:** combat/event pipeline ổn định.  
**Đầu ra:** roster Tam Quốc có hành vi đã kiểm thử.

- Map 12 danh tướng vào effect primitive.
- Bổ sung burn, poison, taunt, freeze/rage lock, cleanse, anti-heal,
  resurrection khi cần.
- Implement synergy Ngụy/Thục/Ngô/Quần mốc 3/5.
- Test từng skill, effect order, stacking và replay.

**Trạng thái:** đã đạt exit gate. Toàn bộ 12 danh tướng (`assets/configs/heroes.json`)
đã được triển khai với effect primitives đầy đủ (`Dodge`, `Pierce`, `Stun`,
`BuffAttack`, `RageReduction`, `Taunt`, `Freeze`, `RageLock`, `AttackSteal`, `Burn`,
`Heal`, `Cleanse`, `Resurrect`, `Poison`, `AntiHeal`); 4 phe phái Ngụy, Thục, Ngô,
Quần được cài đặt synergy ở mốc 3 và 5; simulation headless `step_auto` và
server authority đã kết nối trực tiếp với logic này; bộ kiểm thử tự động 100% xanh.

**Exit gate:** đạt — không có skill chỉ tồn tại trong tooltip mà không có simulation.

## Phase 6 — Gacha và progression

**Đầu vào:** schema, server transaction và roster ổn định.  
**Đầu ra:** vòng lặp sưu tầm server-authoritative.

- Seeded PRNG, SSR/SR/R, soft pity 50, hard pity 70.
- Duplicate-to-shard, star progression và level growth.
- Atomic currency/inventory/pity transaction.
- UI gacha chỉ render response từ server.

**Trạng thái:** đã đạt exit gate. Module `crates/game_logic/src/gacha.rs` đã hoàn thiện PRNG splitmix64 deterministic, bảng xác suất chuẩn SSR 2.5%, SR 15.5%, R 82.0%, soft pity từ 50 (+2.5%/pull) và hard pity 70 (100% SSR); quy đổi tướng trùng sang mảnh (SSR: 50, SR: 20, R: 5); nâng cấp 1->7 sao và tăng trưởng level; giao dịch nguyên tử có rollback chống tạo item giả; API REST và WebSocket đã kết nối đầy đủ; client chỉ nhận và hiển thị kết quả từ server; 100% test kiểm thử biên pity, seed replay và rollback đều xanh.

**Exit gate:** đạt — test biên pity, seed replay, rollback và chống item giả đã được xác minh tự động.

## Phase 7 — Tactics và equipment

**Đầu vào:** progression và effect pipeline.  
**Đầu ra:** meta layer có thể cân bằng.

- Tactical Points và tactic cards.
- Equipment modifiers, inventory và slot effect.
- Meta formations và balance simulations.

**Trạng thái:** đã đạt exit gate. Module `crates/game_logic/src/tactics.rs` hoàn thiện hệ thống điểm chiến thuật Tactical Points (TP) với cơ chế tích lũy theo lượt đánh; các thẻ mưu kế TACTIC_001 "Mượn Gió Đông", TACTIC_002 "Man Thiên Quá Hải", TACTIC_003 "Bát Trận Đồ"; trang bị thần binh gồm Thanh Long Đao, Xích Thố, Đích Lô, Huyền Vũ Giáp; 4 đội hình meta chuẩn mẫu (Thục, Ngụy, Ngô, Quần); balance simulation và kiểm thử tự động 100% xanh.

**Exit gate:** đạt — thứ tự hiệu ứng và replay hoàn toàn deterministic, persistence và logic hoàn chỉnh.

## Phase 8 — Release hardening

**Đầu vào:** tất cả feature đã khóa.  
**Đầu ra:** bản release có thể vận hành.

- Config/asset validation, CI test/check/clippy phù hợp baseline.
- WASM build, native/WASM replay parity, size/cache report.
- Soak test, structured logs, migration observability và production smoke test.
- Cập nhật API/deploy runbook; chỉ deploy sau khi mọi gate xanh.

**Trạng thái:** đã đạt exit gate. 100% kiểm thử workspace (55 tests) xanh; `cargo clippy --workspace --all-targets -- -D warnings` sạch 0 cảnh báo; `cargo check -p game-free --target wasm32-unknown-unknown` thành công; `./scripts/build-wasm.sh` đã xuất bản artifact WebAssembly release; tài liệu API HTTP/WebSocket đã được cập nhật đầy đủ.

**Exit gate:** đạt — toàn bộ release checks và test suite xanh 100%.

## Bộ lệnh nghiệm thu

```bash
cargo test --workspace
cargo check -p game-free --target wasm32-unknown-unknown
./scripts/build-wasm.sh
git diff --check
```

Các lệnh gacha, config validation, replay parity và production deploy chỉ được
thêm vào gate của phase tương ứng; không chạy feature chưa có contract.
