use rand::Rng;
use rand::prelude::IndexedRandom;
use crate::components::soldier::{Rank, Faction};
use crate::components::soldier_stats::SoldierStats;

pub fn generate_soldier_stats(rank: Rank, rng: &mut impl Rng) -> SoldierStats {
    let base = rank.base_stats();

    let (acc_var, move_var, hp_var) = match rank {
        Rank::Private => (0.10, 0.15, 15),
        Rank::Corporal => (0.08, 0.12, 12),
        Rank::Sergeant => (0.06, 0.10, 10),
        Rank::Lieutenant => (0.04, 0.08, 8),
        Rank::Captain => (0.03, 0.05, 5),
    };

    SoldierStats {
        accuracy_modifier: base.accuracy_base + rng.gen_range(-acc_var..=acc_var),
        movement_speed_modifier: base.movement_speed_base * (1.0 + rng.gen_range(-move_var..=move_var)),
        max_hp_modifier: rng.gen_range(-hp_var..=hp_var),
        carrying_capacity: base.carrying_capacity_base,
    }
}

pub fn select_random_rank(rng: &mut impl Rng) -> Rank {
    let total_weight: u32 = Rank::all()
        .iter()
        .map(|r| r.distribution_weight())
        .sum();

    let mut roll = rng.gen_range(0..total_weight);

    for rank in &Rank::all() {
        let weight = rank.distribution_weight();
        if roll < weight {
            return *rank;
        }
        roll -= weight;
    }

    Rank::Private
}

pub fn generate_name(faction: Faction, rank: Rank) -> String {
    let first_names_allies = [
        "John", "William", "James", "George", "Thomas", "Robert", "Edward", "Arthur",
        "Charles", "Henry", "Albert", "Frederick", "Walter", "Harold", "Ernest", "Alfred"
    ];
    let last_names_allies = [
        "Smith", "Jones", "Taylor", "Brown", "Davis", "Wilson", "Evans", "Thomas",
        "Roberts", "Johnson", "Lewis", "Walker", "Robinson", "Wood", "Thompson", "White"
    ];

    let first_names_central = [
        "Hans", "Franz", "Karl", "Wilhelm", "Friedrich", "Otto", "Heinrich", "Ernst",
        "Ludwig", "Gustav", "Walter", "Hermann", "Kurt", "Fritz", "Max", "Paul"
    ];
    let last_names_central = [
        "Mueller", "Schmidt", "Weber", "Fischer", "Bauer", "Meyer", "Wagner", "Klein",
        "Hoffmann", "Schulz", "Becker", "Koch", "Richter", "Wolf", "Schroeder", "Neumann"
    ];

    let mut rng = rand::rng();

    let (first, last) = match faction {
        Faction::Allies => (
            first_names_allies.as_slice().choose(&mut rng).unwrap(),
            last_names_allies.as_slice().choose(&mut rng).unwrap(),
        ),
        Faction::CentralPowers => (
            first_names_central.as_slice().choose(&mut rng).unwrap(),
            last_names_central.as_slice().choose(&mut rng).unwrap(),
        ),
    };

    format!("{} {}", rank.as_str(), last)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rank_distribution() {
        let mut rng = rand::rng();
        let mut counts = std::collections::HashMap::new();

        for _ in 0..1000 {
            let rank = select_random_rank(&mut rng);
            *counts.entry(rank).or_insert(0) += 1;
        }

        let private_count = counts.get(&Rank::Private).unwrap_or(&0);
        assert!(*private_count > 600);

        let captain_count = counts.get(&Rank::Captain).unwrap_or(&0);
        assert!(*captain_count < 50);
    }

    #[test]
    fn test_stat_generation() {
        let mut rng = rand::rng();

        for rank in &Rank::all() {
            let stats = generate_soldier_stats(*rank, &mut rng);
            let base = rank.base_stats();

            assert!(stats.accuracy_modifier >= base.accuracy_base - 0.2);
            assert!(stats.accuracy_modifier <= base.accuracy_base + 0.2);
            assert!(stats.movement_speed_modifier > 0.5);
            assert!(stats.movement_speed_modifier < 1.5);
        }
    }
}
