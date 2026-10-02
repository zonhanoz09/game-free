use axum::{
    Router,
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
};
use futures_util::{SinkExt, StreamExt};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::{RwLock, mpsc};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};

// ==========================================
// DATA MODELS & PERSISTENCE
// ==========================================
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub display_name: String,
    pub avatar: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub elo: i32,
    pub wins: u32,
    pub losses: u32,
    pub matches: u32,
    pub created_at: String,
    pub last_login: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MatchRecord {
    pub match_id: String,
    pub host: String,
    pub guest: String,
    pub winner: Option<String>,
    pub rounds: usize,
    pub timestamp: String,
}

#[derive(Default, Serialize, Deserialize)]
pub struct DatabaseData {
    pub users: HashMap<String, User>,
    pub matches: Vec<MatchRecord>,
}

pub struct Database {
    file_path: PathBuf,
    data: DatabaseData,
}

impl Database {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let file_path = path.as_ref().to_path_buf();
        if let Some(parent) = file_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let data = if file_path.exists() {
            fs::read_to_string(&file_path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            DatabaseData::default()
        };

        println!(
            "[RUST DATABASE] Loaded {} users and {} matches from disk.",
            data.users.len(),
            data.matches.len()
        );

        Self { file_path, data }
    }

    fn save(&self) {
        let tmp = self.file_path.with_extension("tmp");
        if let Ok(json) = serde_json::to_string_pretty(&self.data) {
            let _ = fs::write(&tmp, json);
            let _ = fs::rename(&tmp, &self.file_path);
        }
    }

    fn hash_password(pwd: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}_tac_arena_salt", pwd).as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn register(
        &mut self,
        username: &str,
        password: &str,
        display_name: &str,
        avatar: &str,
    ) -> Result<User, String> {
        let clean = username.trim().to_lowercase();
        if clean.len() < 3 || clean.len() > 20 {
            return Err("Tên tài khoản phải từ 3 đến 20 ký tự!".to_string());
        }
        if !clean.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err("Tên tài khoản chỉ chứa chữ cái, số và dấu gạch dưới!".to_string());
        }
        if password.len() < 4 {
            return Err("Mật khẩu phải từ 4 ký tự trở lên!".to_string());
        }
        if self.data.users.contains_key(&clean) {
            return Err("Tên tài khoản này đã được sử dụng!".to_string());
        }

        let now = chrono_now();
        let user = User {
            username: clean.clone(),
            display_name: if display_name.trim().is_empty() {
                clean.clone()
            } else {
                display_name.trim().chars().take(24).collect()
            },
            avatar: if avatar.is_empty() {
                "knight".to_string()
            } else {
                avatar.to_string()
            },
            password_hash: Self::hash_password(password),
            elo: 1000,
            wins: 0,
            losses: 0,
            matches: 0,
            created_at: now.clone(),
            last_login: now,
        };

        self.data.users.insert(clean, user.clone());
        self.save();
        Ok(user)
    }

    pub fn login(&mut self, username: &str, password: &str) -> Result<User, String> {
        let clean = username.trim().to_lowercase();
        let user = self
            .data
            .users
            .get_mut(&clean)
            .ok_or_else(|| "Tài khoản không tồn tại!".to_string())?;

        let hash = Self::hash_password(password);
        if user.password_hash != hash {
            return Err("Mật khẩu không chính xác!".to_string());
        }

        user.last_login = chrono_now();
        let res = user.clone();
        self.save();
        Ok(res)
    }

    pub fn get_user(&self, username: &str) -> Option<User> {
        self.data.users.get(&username.to_lowercase()).cloned()
    }

    pub fn record_match(
        &mut self,
        match_id: &str,
        host_user: &str,
        guest_user: &str,
        winner_user: Option<&str>,
        rounds: usize,
    ) {
        let rec = MatchRecord {
            match_id: match_id.to_string(),
            host: host_user.to_string(),
            guest: guest_user.to_string(),
            winner: winner_user.map(str::to_string),
            rounds,
            timestamp: chrono_now(),
        };

        if let Some(win) = winner_user {
            let is_host_win = win.eq_ignore_ascii_case(host_user);
            if let Some(h) = self.data.users.get_mut(&host_user.to_lowercase()) {
                h.matches += 1;
                if is_host_win {
                    h.wins += 1;
                    h.elo += 25;
                } else {
                    h.losses += 1;
                    h.elo = (h.elo - 20).max(500);
                }
            }
            if let Some(g) = self.data.users.get_mut(&guest_user.to_lowercase()) {
                g.matches += 1;
                if !is_host_win {
                    g.wins += 1;
                    g.elo += 25;
                } else {
                    g.losses += 1;
                    g.elo = (g.elo - 20).max(500);
                }
            }
        }

        self.data.matches.insert(0, rec);
        if self.data.matches.len() > 200 {
            self.data.matches.truncate(200);
        }
        self.save();
    }

    pub fn get_leaderboard(&self, limit: usize) -> Vec<User> {
        let mut list: Vec<User> = self.data.users.values().cloned().collect();
        list.sort_by(|a, b| b.elo.cmp(&a.elo).then_with(|| b.wins.cmp(&a.wins)));
        list.into_iter().take(limit).collect()
    }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", dur.as_secs())
}

// ==========================================
// MULTIPLAYER ROOM & MATCHMAKING
// ==========================================
type Tx = mpsc::UnboundedSender<Message>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnitData {
    pub col: usize,
    pub row: usize,
    pub class: String,
    pub star_level: u8,
}

#[derive(Clone, Debug)]
pub struct PlayerSession {
    pub id: String,
    pub name: String,
    pub avatar: String,
    pub elo: i32,
    pub hp: i32,
    pub ready: bool,
    pub lineup: Vec<UnitData>,
    pub tx: Tx,
}

#[derive(Clone, Debug, Serialize)]
pub struct PublicRoomInfo {
    pub code: String,
    pub host_name: String,
    pub host_avatar: String,
    pub host_elo: i32,
}

pub struct Room {
    pub code: String,
    pub round: usize,
    pub host: PlayerSession,
    pub guest: Option<PlayerSession>,
}

pub struct QuickMatchEntry {
    pub session: PlayerSession,
}

pub struct AppState {
    pub db: Arc<RwLock<Database>>,
    pub rooms: Arc<RwLock<HashMap<String, Room>>>,
    pub quick_match: Arc<RwLock<Option<QuickMatchEntry>>>,
}

fn generate_room_code() -> String {
    let chars = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut rng = rand::thread_rng();
    (0..4)
        .map(|_| chars[rng.gen_range(0..chars.len())] as char)
        .collect()
}

// ==========================================
// REST API HANDLERS
// ==========================================
#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    password: String,
    display_name: Option<String>,
    avatar: Option<String>,
}

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct ProfileQuery {
    username: String,
}

#[derive(Deserialize)]
struct RecordMatchRequest {
    match_id: Option<String>,
    host: String,
    guest: String,
    winner: Option<String>,
    rounds: Option<usize>,
}

async fn handle_register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    let mut db = state.db.write().await;
    match db.register(
        &payload.username,
        &payload.password,
        payload.display_name.as_deref().unwrap_or(""),
        payload.avatar.as_deref().unwrap_or("knight"),
    ) {
        Ok(user) => (
            StatusCode::OK,
            Json(serde_json::json!({ "success": true, "user": user })),
        ),
        Err(msg) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "message": msg })),
        ),
    }
}

async fn handle_login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let mut db = state.db.write().await;
    match db.login(&payload.username, &payload.password) {
        Ok(user) => (
            StatusCode::OK,
            Json(serde_json::json!({ "success": true, "user": user })),
        ),
        Err(msg) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "success": false, "message": msg })),
        ),
    }
}

async fn handle_profile(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ProfileQuery>,
) -> impl IntoResponse {
    let db = state.db.read().await;
    match db.get_user(&query.username) {
        Some(user) => (
            StatusCode::OK,
            Json(serde_json::json!({ "success": true, "user": user })),
        ),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "success": false, "message": "Không tìm thấy người chơi!" })),
        ),
    }
}

async fn handle_leaderboard(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db = state.db.read().await;
    let list = db.get_leaderboard(20);
    Json(serde_json::json!({ "success": true, "leaderboard": list }))
}

async fn handle_record_match(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RecordMatchRequest>,
) -> impl IntoResponse {
    let mut db = state.db.write().await;
    let match_id = payload
        .match_id
        .unwrap_or_else(|| format!("m_{}", chrono_now()));
    db.record_match(
        &match_id,
        &payload.host,
        &payload.guest,
        payload.winner.as_deref(),
        payload.rounds.unwrap_or(1),
    );
    Json(serde_json::json!({ "success": true }))
}

// ==========================================
// WEBSOCKET HANDLER
// ==========================================
async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // Forward outgoing messages to client
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let mut current_room: Option<String> = None;
    let mut current_role: Option<String> = None;

    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            let parsed: serde_json::Value = match serde_json::from_str(&text) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let raw_type = parsed
                .get("type")
                .or_else(|| parsed.get("action"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let msg_type = raw_type.to_uppercase();

            match msg_type.as_str() {
                "GET_ROOMS" | "LOBBY_ROOMS" => {
                    let rooms_guard = state.rooms.read().await;
                    let list: Vec<PublicRoomInfo> = rooms_guard
                        .values()
                        .filter(|r| r.guest.is_none())
                        .map(|r| PublicRoomInfo {
                            code: r.code.clone(),
                            host_name: r.host.name.clone(),
                            host_avatar: r.host.avatar.clone(),
                            host_elo: r.host.elo,
                        })
                        .collect();

                    let res = serde_json::json!({
                        "type": "LOBBY_ROOMS_RESPONSE",
                        "rooms": list
                    });
                    let _ = tx.send(Message::Text(res.to_string().into()));
                }

                "CREATE_ROOM" => {
                    let player_id = parsed
                        .get("player_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("guest");
                    let player_name = parsed
                        .get("player_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Player 1");
                    let avatar = parsed
                        .get("avatar")
                        .and_then(|v| v.as_str())
                        .unwrap_or("knight");
                    let elo = parsed.get("elo").and_then(|v| v.as_i64()).unwrap_or(1000) as i32;

                    let mut rooms_guard = state.rooms.write().await;
                    let mut code = generate_room_code();
                    while rooms_guard.contains_key(&code) {
                        code = generate_room_code();
                    }

                    let session = PlayerSession {
                        id: player_id.to_string(),
                        name: player_name.to_string(),
                        avatar: avatar.to_string(),
                        elo,
                        hp: 100,
                        ready: false,
                        lineup: vec![],
                        tx: tx.clone(),
                    };

                    let room = Room {
                        code: code.clone(),
                        round: 1,
                        host: session,
                        guest: None,
                    };

                    rooms_guard.insert(code.clone(), room);
                    current_room = Some(code.clone());
                    current_role = Some("host".to_string());

                    let res = serde_json::json!({
                        "type": "ROOM_CREATED",
                        "room_code": code,
                        "role": "host",
                        "player_name": player_name
                    });
                    let _ = tx.send(Message::Text(res.to_string().into()));
                    println!("[RUST WS] Room {} created by {}", code, player_name);
                }

                "JOIN_ROOM" => {
                    let code = parsed
                        .get("room_code")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim()
                        .to_uppercase();

                    let player_id = parsed
                        .get("player_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("guest");
                    let player_name = parsed
                        .get("player_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Player 2");
                    let avatar = parsed
                        .get("avatar")
                        .and_then(|v| v.as_str())
                        .unwrap_or("knight");
                    let elo = parsed.get("elo").and_then(|v| v.as_i64()).unwrap_or(1000) as i32;

                    let mut rooms_guard = state.rooms.write().await;
                    if let Some(room) = rooms_guard.get_mut(&code) {
                        if room.guest.is_some() {
                            let _ = tx.send(Message::Text(
                                serde_json::json!({ "type": "ERROR", "message": "Phòng đã đủ 2 người chơi!" })
                                    .to_string()
                                    .into(),
                            ));
                            continue;
                        }

                        let guest_session = PlayerSession {
                            id: player_id.to_string(),
                            name: player_name.to_string(),
                            avatar: avatar.to_string(),
                            elo,
                            hp: 100,
                            ready: false,
                            lineup: vec![],
                            tx: tx.clone(),
                        };

                        let host_name = room.host.name.clone();
                        room.guest = Some(guest_session);

                        current_room = Some(code.clone());
                        current_role = Some("guest".to_string());

                        // Send confirmation to guest
                        let _ = tx.send(Message::Text(
                            serde_json::json!({
                                "type": "ROOM_JOINED",
                                "room_code": code,
                                "role": "guest",
                                "player_name": player_name,
                                "opponent_name": host_name,
                                "round": room.round
                            })
                            .to_string()
                            .into(),
                        ));

                        // Notify host
                        let _ = room.host.tx.send(Message::Text(
                            serde_json::json!({
                                "type": "OPPONENT_JOINED",
                                "room_code": code,
                                "role": "host",
                                "player_name": host_name,
                                "opponent_name": player_name,
                                "round": room.round
                            })
                            .to_string()
                            .into(),
                        ));

                        println!(
                            "[RUST WS] {} joined room {} vs {}",
                            player_name, code, host_name
                        );
                    } else {
                        let _ = tx.send(Message::Text(
                            serde_json::json!({ "type": "ERROR", "message": format!("Phòng {} không tồn tại!", code) })
                                .to_string()
                                .into(),
                        ));
                    }
                }

                "QUICK_MATCH" => {
                    let player_id = parsed
                        .get("player_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("guest");
                    let player_name = parsed
                        .get("player_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Player");
                    let avatar = parsed
                        .get("avatar")
                        .and_then(|v| v.as_str())
                        .unwrap_or("knight");
                    let elo = parsed.get("elo").and_then(|v| v.as_i64()).unwrap_or(1000) as i32;

                    let session = PlayerSession {
                        id: player_id.to_string(),
                        name: player_name.to_string(),
                        avatar: avatar.to_string(),
                        elo,
                        hp: 100,
                        ready: false,
                        lineup: vec![],
                        tx: tx.clone(),
                    };

                    let mut qm_guard = state.quick_match.write().await;
                    if let Some(waiting) = qm_guard.take() {
                        let code = generate_room_code();
                        let host_name = waiting.session.name.clone();
                        let guest_name = session.name.clone();

                        let mut rooms_guard = state.rooms.write().await;
                        let room = Room {
                            code: code.clone(),
                            round: 1,
                            host: waiting.session.clone(),
                            guest: Some(session),
                        };
                        rooms_guard.insert(code.clone(), room);

                        current_room = Some(code.clone());
                        current_role = Some("guest".to_string());

                        // Notify both
                        let _ = waiting.session.tx.send(Message::Text(
                            serde_json::json!({
                                "type": "ROOM_JOINED",
                                "room_code": code,
                                "role": "host",
                                "player_name": host_name,
                                "opponent_name": guest_name,
                                "round": 1
                            })
                            .to_string()
                            .into(),
                        ));

                        let _ = tx.send(Message::Text(
                            serde_json::json!({
                                "type": "ROOM_JOINED",
                                "room_code": code,
                                "role": "guest",
                                "player_name": guest_name,
                                "opponent_name": host_name,
                                "round": 1
                            })
                            .to_string()
                            .into(),
                        ));

                        println!(
                            "[RUST WS] Quick match formed! Room {} between {} and {}",
                            code, host_name, guest_name
                        );
                    } else {
                        *qm_guard = Some(QuickMatchEntry { session });
                        let _ = tx.send(Message::Text(
                            serde_json::json!({ "type": "WAITING_FOR_MATCH", "message": "Đang tìm đối thủ..." })
                                .to_string()
                                .into(),
                        ));
                    }
                }

                "PLAYER_READY" => {
                    let lineup_val = parsed
                        .get("lineup")
                        .cloned()
                        .unwrap_or(serde_json::json!([]));
                    let lineup: Vec<UnitData> =
                        serde_json::from_value(lineup_val).unwrap_or_default();

                    if let Some(code) = &current_room {
                        let mut rooms_guard = state.rooms.write().await;
                        if let Some(room) = rooms_guard.get_mut(code) {
                            if current_role.as_deref() == Some("host") {
                                room.host.ready = true;
                                room.host.lineup = lineup;
                                if let Some(ref g) = room.guest {
                                    let _ = g.tx.send(Message::Text(
                                        serde_json::json!({ "type": "OPPONENT_READY" })
                                            .to_string()
                                            .into(),
                                    ));
                                }
                            } else if current_role.as_deref() == Some("guest") {
                                if let Some(ref mut g) = room.guest {
                                    g.ready = true;
                                    g.lineup = lineup;
                                    let _ = room.host.tx.send(Message::Text(
                                        serde_json::json!({ "type": "OPPONENT_READY" })
                                            .to_string()
                                            .into(),
                                    ));
                                }
                            }

                            // If both players are ready, trigger round start!
                            let host_ready = room.host.ready;
                            let guest_ready = room.guest.as_ref().map(|g| g.ready).unwrap_or(false);

                            if host_ready && guest_ready {
                                room.host.ready = false;
                                if let Some(ref mut g) = room.guest {
                                    g.ready = false;
                                }

                                let host_lineup = room.host.lineup.clone();
                                let guest_lineup = room.guest.as_ref().unwrap().lineup.clone();
                                let host_hp = room.host.hp;
                                let guest_hp = room.guest.as_ref().unwrap().hp;
                                let round = room.round;

                                // Send start round to Host
                                let _ = room.host.tx.send(Message::Text(
                                    serde_json::json!({
                                        "type": "START_ROUND",
                                        "round": round,
                                        "opponent_lineup": guest_lineup,
                                        "player_hp": host_hp,
                                        "opponent_hp": guest_hp
                                    })
                                    .to_string()
                                    .into(),
                                ));

                                // Send start round to Guest
                                if let Some(ref g) = room.guest {
                                    let _ = g.tx.send(Message::Text(
                                        serde_json::json!({
                                            "type": "START_ROUND",
                                            "round": round,
                                            "opponent_lineup": host_lineup,
                                            "player_hp": guest_hp,
                                            "opponent_hp": host_hp
                                        })
                                        .to_string()
                                        .into(),
                                    ));
                                }

                                println!(
                                    "[RUST WS] Battle started for round {} in room {}",
                                    round, code
                                );
                            }
                        }
                    }
                }

                "BATTLE_FINISHED" => {
                    let winner_role = parsed
                        .get("winner_role")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let survivors = parsed
                        .get("player_survivors")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(1) as i32;

                    if let Some(code) = &current_room {
                        let mut rooms_guard = state.rooms.write().await;
                        if let Some(room) = rooms_guard.get_mut(code) {
                            let damage = 10 + survivors * 3;
                            if winner_role == "host" {
                                if let Some(ref mut g) = room.guest {
                                    g.hp = (g.hp - damage).max(0);
                                }
                            } else if winner_role == "guest" {
                                room.host.hp = (room.host.hp - damage).max(0);
                            }

                            let host_hp = room.host.hp;
                            let guest_hp = room.guest.as_ref().map(|g| g.hp).unwrap_or(0);

                            // Send HP updates
                            let _ = room.host.tx.send(Message::Text(
                                serde_json::json!({
                                    "type": "UPDATE_MATCH_HP",
                                    "player_hp": host_hp,
                                    "opponent_hp": guest_hp,
                                    "damage_dealt": damage
                                })
                                .to_string()
                                .into(),
                            ));

                            if let Some(ref g) = room.guest {
                                let _ = g.tx.send(Message::Text(
                                    serde_json::json!({
                                        "type": "UPDATE_MATCH_HP",
                                        "player_hp": guest_hp,
                                        "opponent_hp": host_hp,
                                        "damage_dealt": damage
                                    })
                                    .to_string()
                                    .into(),
                                ));
                            }

                            // Check match end
                            if host_hp <= 0 || guest_hp <= 0 {
                                let winner_name = if host_hp > 0 {
                                    room.host.name.clone()
                                } else {
                                    room.guest
                                        .as_ref()
                                        .map(|g| g.name.clone())
                                        .unwrap_or_default()
                                };
                                let winner_id = if host_hp > 0 {
                                    room.host.id.clone()
                                } else {
                                    room.guest
                                        .as_ref()
                                        .map(|g| g.id.clone())
                                        .unwrap_or_default()
                                };

                                let end_msg = serde_json::json!({
                                    "type": "MATCH_END",
                                    "winner": winner_name
                                })
                                .to_string();

                                let _ = room.host.tx.send(Message::Text(end_msg.clone().into()));
                                if let Some(ref g) = room.guest {
                                    let _ = g.tx.send(Message::Text(end_msg.into()));
                                }

                                // Update DB
                                let mut db = state.db.write().await;
                                db.record_match(
                                    &format!("m_{}", code),
                                    &room.host.id,
                                    &room.guest.as_ref().map(|g| g.id.as_str()).unwrap_or(""),
                                    Some(&winner_id),
                                    room.round,
                                );

                                rooms_guard.remove(code);
                                println!(
                                    "[RUST WS] Match ended in room {}. Winner: {}",
                                    code, winner_name
                                );
                            } else {
                                room.round += 1;
                            }
                        }
                    }
                }

                _ => {}
            }
        }
    }

    // Cleanup on disconnect
    if let Some(code) = current_room {
        let mut rooms_guard = state.rooms.write().await;
        if let Some(room) = rooms_guard.get_mut(&code) {
            if current_role.as_deref() == Some("host") {
                if let Some(ref g) = room.guest {
                    let _ = g.tx.send(Message::Text(
                        serde_json::json!({ "type": "ERROR", "message": "Chủ phòng đã thoát trận đấu." })
                            .to_string()
                            .into(),
                    ));
                }
                rooms_guard.remove(&code);
            } else if current_role.as_deref() == Some("guest") {
                let _ = room.host.tx.send(Message::Text(
                    serde_json::json!({ "type": "ERROR", "message": "Đối thủ đã rời khỏi phòng." })
                        .to_string()
                        .into(),
                ));
                room.guest = None;
            }
        }
    }

    let mut qm_guard = state.quick_match.write().await;
    if let Some(ref entry) = *qm_guard {
        if entry.session.tx.same_channel(&tx) {
            *qm_guard = None;
        }
    }
}

// ==========================================
// MAIN ENTRY POINT
// ==========================================
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let data_dir = PathBuf::from("data");
    let db_path = data_dir.join("game_db.json");
    let db = Database::new(db_path);

    let state = Arc::new(AppState {
        db: Arc::new(RwLock::new(db)),
        rooms: Arc::new(RwLock::new(HashMap::new())),
        quick_match: Arc::new(RwLock::new(None)),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // REST & WebSocket Routes
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/api/auth/register", post(handle_register))
        .route("/api/auth/login", post(handle_login))
        .route("/api/user/profile", get(handle_profile))
        .route("/api/leaderboard", get(handle_leaderboard))
        .route("/api/match/record", post(handle_record_match))
        .fallback_service(ServeDir::new("wasm_dist"))
        .layer(cors)
        .with_state(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    println!("=======================================================");
    println!(" 🦀 3v3 TACTICAL ARENA - RUST WEBSOCKET SERVER");
    println!(" 🌐 Server Listening on: http://{}", addr);
    println!(" ⚡ WebSocket Endpoint: ws://{}/ws", addr);
    println!(" 📦 Static Assets: wasm_dist");
    println!("=======================================================");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
