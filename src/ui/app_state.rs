use crate::game_logic::battlefield::Battlefield;
use crate::rendering::viewport::Camera;
use crate::utils::input_mode::InputMode;
use specs::World;
use std::collections::{HashMap, HashSet};

pub enum AppState {
    MainMenu,
    NewGameConfig,
    InGame(GameState),
    Paused(GameState),
    Settings,
}

pub struct GameState {
    pub world: World,
    pub battlefield: Battlefield,
    pub camera: Camera,
    pub running: bool,
    pub input_mode: InputMode,
    pub cursor_pos: crate::game_logic::battlefield::Position,
    pub config: crate::config::game_config::GameConfig,
    pub peripheral_tiles: HashMap<crate::game_logic::battlefield::Position, bool>,
    pub spotter_map: HashMap<crate::game_logic::battlefield::Position, specs::Entity>,
    pub last_seen_markers: HashMap<specs::Entity, crate::components::last_seen::LastSeenMarker>,
    pub visible_entities: HashSet<specs::Entity>,
}
