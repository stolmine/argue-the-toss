use crate::components::soldier::Faction;
use crate::game_logic::battlefield::{Battlefield, Position, TerrainType};
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

pub fn create_strategic_objectives(
    battlefield: &Battlefield,
) -> (Position, Position) {
    let ally_spawn = battlefield.ally_spawn.as_ref();
    let enemy_spawn = battlefield.enemy_spawn.as_ref();

    if ally_spawn.is_none() || enemy_spawn.is_none() {
        let width = battlefield.width() as i32;
        let height = battlefield.height() as i32;
        return (
            Position::new(width / 4, height * 3 / 4),
            Position::new(width * 3 / 4, height / 4),
        );
    }

    let ally_spawn = ally_spawn.unwrap();
    let enemy_spawn = enemy_spawn.unwrap();

    let ally_flag_pos = find_strategic_position(
        battlefield,
        ally_spawn.center,
        25,
        true,
    );

    let enemy_flag_pos = find_strategic_position(
        battlefield,
        enemy_spawn.center,
        25,
        true,
    );

    (ally_flag_pos, enemy_flag_pos)
}

fn find_strategic_position(
    battlefield: &Battlefield,
    near: Position,
    radius: i32,
    prefer_fortifications: bool,
) -> Position {
    let mut best_position = near;
    let mut best_score = -1000.0;

    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let pos = Position::new(near.x + dx, near.y + dy);

            if !battlefield.in_bounds(&pos) {
                continue;
            }

            let distance = ((dx * dx + dy * dy) as f32).sqrt();
            if distance > radius as f32 {
                continue;
            }

            if let Some(tile) = battlefield.get_tile(&pos) {
                if !tile.terrain.is_passable() {
                    continue;
                }

                if matches!(tile.terrain, TerrainType::Water | TerrainType::DeepWater) {
                    continue;
                }

                let mut score = 0.0;

                if prefer_fortifications {
                    let fortification_score = match tile.terrain {
                        TerrainType::TrenchFloor | TerrainType::TrenchParapet | TerrainType::Trench => 50.0,
                        TerrainType::Bunker => 60.0,
                        TerrainType::MgNest => 55.0,
                        TerrainType::Sandbags => 40.0,
                        TerrainType::CommTrench => 45.0,
                        TerrainType::Fortification => 50.0,
                        TerrainType::ShellCrater => 30.0,
                        TerrainType::Ruins => 35.0,
                        _ => 0.0,
                    };
                    score += fortification_score;
                }

                let cover_score = tile.terrain.cover_bonus() * 20.0;
                score += cover_score;

                let distance_penalty = distance * 0.5;
                score -= distance_penalty;

                let nearby_fortifications = count_nearby_fortifications(battlefield, &pos, 3);
                score += nearby_fortifications as f32 * 5.0;

                if score > best_score {
                    best_score = score;
                    best_position = pos;
                }
            }
        }
    }

    best_position
}

fn count_nearby_fortifications(battlefield: &Battlefield, pos: &Position, radius: i32) -> usize {
    let mut count = 0;

    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx == 0 && dy == 0 {
                continue;
            }

            let check_pos = Position::new(pos.x + dx, pos.y + dy);

            if let Some(tile) = battlefield.get_tile(&check_pos) {
                match tile.terrain {
                    TerrainType::TrenchFloor | TerrainType::TrenchParapet | TerrainType::Trench |
                    TerrainType::Bunker | TerrainType::MgNest | TerrainType::Sandbags |
                    TerrainType::CommTrench | TerrainType::Fortification => {
                        count += 1;
                    }
                    _ => {}
                }
            }
        }
    }

    count
}
