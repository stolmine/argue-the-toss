use crate::components::soldier::Faction;
use crate::game_logic::battlefield::Position;
use specs::Entity;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ObjectiveFlag {
    pub position: Position,
    pub owning_faction: Faction,
    pub capture_progress: i32,
    pub required_turns: i32,
    pub radius: i32,
}

impl ObjectiveFlag {
    pub fn new(position: Position, owning_faction: Faction) -> Self {
        Self {
            position,
            owning_faction,
            capture_progress: 0,
            required_turns: 5,
            radius: 2,
        }
    }

    pub fn reset_progress(&mut self) {
        self.capture_progress = 0;
    }

    pub fn increment_progress(&mut self) {
        self.capture_progress += 1;
    }

    pub fn is_captured(&self) -> bool {
        self.capture_progress >= self.required_turns
    }

    pub fn capture(&mut self, new_faction: Faction) {
        self.owning_faction = new_faction;
        self.capture_progress = 0;
    }

    pub fn is_position_in_radius(&self, pos: &Position) -> bool {
        self.position.manhattan_distance_to(pos) <= self.radius
    }
}

#[derive(Debug, Clone)]
pub struct Objectives {
    pub flags: HashMap<String, ObjectiveFlag>,
}

impl Objectives {
    pub fn new() -> Self {
        Self {
            flags: HashMap::new(),
        }
    }

    pub fn add_flag(&mut self, id: String, flag: ObjectiveFlag) {
        self.flags.insert(id, flag);
    }

    pub fn get_flag(&self, id: &str) -> Option<&ObjectiveFlag> {
        self.flags.get(id)
    }

    pub fn get_flag_mut(&mut self, id: &str) -> Option<&mut ObjectiveFlag> {
        self.flags.get_mut(id)
    }

    pub fn get_enemy_flag_position(&self, faction: Faction) -> Option<Position> {
        self.flags
            .values()
            .find(|flag| flag.owning_faction != faction)
            .map(|flag| flag.position)
    }

    pub fn check_victory(&self) -> Option<Faction> {
        let allies_flags: Vec<_> = self.flags
            .values()
            .filter(|flag| flag.owning_faction == Faction::Allies)
            .collect();

        let central_flags: Vec<_> = self.flags
            .values()
            .filter(|flag| flag.owning_faction == Faction::CentralPowers)
            .collect();

        if allies_flags.len() == self.flags.len() {
            Some(Faction::Allies)
        } else if central_flags.len() == self.flags.len() {
            Some(Faction::CentralPowers)
        } else {
            None
        }
    }
}

impl Default for Objectives {
    fn default() -> Self {
        Self::new()
    }
}

pub fn check_flag_occupation(
    flag: &mut ObjectiveFlag,
    entities_in_radius: &[(Entity, Faction)],
) -> Option<Faction> {
    let defenders: Vec<_> = entities_in_radius
        .iter()
        .filter(|(_, faction)| *faction == flag.owning_faction)
        .collect();

    let attackers: Vec<_> = entities_in_radius
        .iter()
        .filter(|(_, faction)| *faction != flag.owning_faction)
        .collect();

    if !attackers.is_empty() && defenders.is_empty() {
        flag.increment_progress();

        if flag.is_captured() {
            let capturing_faction = attackers[0].1;
            flag.capture(capturing_faction);
            return Some(capturing_faction);
        }
    } else if !defenders.is_empty() || attackers.is_empty() {
        flag.reset_progress();
    }

    None
}
