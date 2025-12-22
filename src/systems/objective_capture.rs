use crate::components::{dead::Dead, position::Position, soldier::Soldier};
use crate::game_logic::objectives::Objectives;
use crate::utils::event_log::EventLog;
use specs::{Entities, Join, ReadStorage, System, Write};

pub struct ObjectiveCaptureSystem;

impl<'a> System<'a> for ObjectiveCaptureSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Soldier>,
        ReadStorage<'a, Dead>,
        Write<'a, Objectives>,
        Write<'a, EventLog>,
    );

    fn run(
        &mut self,
        (entities, positions, soldiers, dead_markers, mut objectives, mut event_log): Self::SystemData,
    ) {
        let mut check_victory = false;

        for (flag_id, flag) in objectives.flags.iter_mut() {
            let mut entities_in_radius = Vec::new();

            for (entity, pos, soldier) in (&entities, &positions, &soldiers).join() {
                if dead_markers.get(entity).is_some() {
                    continue;
                }

                if flag.is_position_in_radius(&pos.as_battlefield_pos()) {
                    entities_in_radius.push((entity, soldier.faction));
                }
            }

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

                if flag.capture_progress == 1 {
                    event_log.add(format!(
                        "{} flag is being contested! ({}/{})",
                        match flag.owning_faction {
                            crate::components::soldier::Faction::Allies => "Allied",
                            crate::components::soldier::Faction::CentralPowers => "Central Powers",
                        },
                        flag.capture_progress,
                        flag.required_turns
                    ));
                }

                if flag.is_captured() {
                    let capturing_faction = attackers[0].1;
                    flag.capture(capturing_faction);

                    event_log.add(format!(
                        "{} captured {}!",
                        match capturing_faction {
                            crate::components::soldier::Faction::Allies => "Allies",
                            crate::components::soldier::Faction::CentralPowers => "Central Powers",
                        },
                        flag_id
                    ));

                    check_victory = true;
                }
            } else if !defenders.is_empty() || attackers.is_empty() {
                if flag.capture_progress > 0 {
                    event_log.add(format!("{} flag defended!", flag_id));
                }
                flag.reset_progress();
            }
        }

        if check_victory {
            if let Some(victor) = objectives.check_victory() {
                let victor_name = match victor {
                    crate::components::soldier::Faction::Allies => "Allies",
                    crate::components::soldier::Faction::CentralPowers => "Central Powers",
                };

                // ALWAYS log victory messages (critical game state information)
                event_log.add("==========================================".to_string());
                event_log.add(format!("VICTORY! {} have captured all objectives!", victor_name));
                event_log.add("==========================================".to_string());
            }
        }
    }
}
