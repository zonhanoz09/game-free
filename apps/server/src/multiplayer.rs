use super::*;

type Tx = mpsc::UnboundedSender<Message>;

pub(crate) type UnitData = game_protocol::PvpUnitData;

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

pub struct Room {
    pub code: String,
    pub round: usize,
    pub settled_round: usize,
    pub host: PlayerSession,
    pub guest: Option<PlayerSession>,
    pub authority: Option<crate::authority::AuthoritativeBattle>,
}

#[derive(Serialize)]
pub struct PublicRoomInfo {
    pub code: String,
    pub host_name: String,
    pub host_avatar: String,
    pub host_elo: i32,
}

pub struct QuickMatchEntry {
    pub session: PlayerSession,
}

pub struct AppState {
    pub db: Arc<RwLock<Database>>,
    pub rooms: Arc<RwLock<HashMap<String, Room>>>,
    pub quick_match: Arc<RwLock<Option<QuickMatchEntry>>>,
    pub player_progression: Arc<RwLock<HashMap<String, game_logic::gacha::PlayerProgressionState>>>,
}

pub(crate) fn generate_room_code() -> String {
    let mut rng = rand::thread_rng();
    let num: u32 = rng.gen_range(1000..9999);
    format!("{:04}", num)
}
