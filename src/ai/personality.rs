use crate::ai::{
    actions::{ActionEvaluator, ScoreCombiner},
    considerations::{
        AlliesNearbyConsideration, AmmoLevelConsideration, CoverQualityConsideration,
        DistanceToTargetConsideration, HasLineOfSightConsideration, HealthLevelConsideration,
        ObjectiveProximityConsideration, ThreatLevelConsideration,
        ExposedDangerConsideration, TacticalAdvantageConsideration, ForceBalanceConsideration,
        SupportProximityConsideration, ObjectivePressureConsideration, RetreatNecessityConsideration,
    },
    response_curves::ResponseCurve,
};

pub struct AIPersonality {
    pub name: String,
    pub evaluators: Vec<ActionEvaluator>,
}

impl AIPersonality {
    pub fn new(name: impl Into<String>, evaluators: Vec<ActionEvaluator>) -> Self {
        Self {
            name: name.into(),
            evaluators,
        }
    }

    pub fn balanced() -> Self {
        let evaluators = vec![
            create_balanced_shoot_evaluator(),
            create_balanced_reload_evaluator(),
            create_balanced_move_evaluator(),
            create_balanced_seek_cover_evaluator(),
            create_balanced_seek_objective_evaluator(),
            create_balanced_wait_evaluator(),
        ];

        Self::new("Balanced", evaluators)
    }

    pub fn aggressive() -> Self {
        let evaluators = vec![
            create_aggressive_shoot_evaluator(),
            create_aggressive_reload_evaluator(),
            create_aggressive_move_evaluator(),
            create_aggressive_seek_cover_evaluator(),
            create_aggressive_seek_objective_evaluator(),
            create_aggressive_wait_evaluator(),
        ];

        Self::new("Aggressive", evaluators)
    }

    pub fn defensive() -> Self {
        let evaluators = vec![
            create_defensive_shoot_evaluator(),
            create_defensive_reload_evaluator(),
            create_defensive_move_evaluator(),
            create_defensive_seek_cover_evaluator(),
            create_defensive_seek_objective_evaluator(),
            create_defensive_wait_evaluator(),
        ];

        Self::new("Defensive", evaluators)
    }

    pub fn objective_focused() -> Self {
        let evaluators = vec![
            create_objective_shoot_evaluator(),
            create_objective_reload_evaluator(),
            create_objective_move_evaluator(),
            create_objective_seek_cover_evaluator(),
            create_objective_seek_objective_evaluator(),
            create_objective_wait_evaluator(),
        ];

        Self::new("ObjectiveFocused", evaluators)
    }

    pub fn scout() -> Self {
        let evaluators = vec![
            create_scout_shoot_evaluator(),
            create_scout_reload_evaluator(),
            create_scout_move_evaluator(),
            create_scout_seek_cover_evaluator(),
            create_scout_seek_objective_evaluator(),
            create_scout_wait_evaluator(),
        ];

        Self::new("Scout", evaluators)
    }

    pub fn rearguard() -> Self {
        let evaluators = vec![
            create_rearguard_shoot_evaluator(),
            create_rearguard_reload_evaluator(),
            create_rearguard_move_evaluator(),
            create_rearguard_seek_cover_evaluator(),
            create_rearguard_seek_objective_evaluator(),
            create_rearguard_wait_evaluator(),
        ];

        Self::new("RearGuard", evaluators)
    }
}

fn create_balanced_shoot_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("Shoot", 0.9)
        .with_consideration(Box::new(HasLineOfSightConsideration::new(
            ResponseCurve::Boolean { threshold: 0.5 },
        )))
        .with_consideration(Box::new(DistanceToTargetConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(AmmoLevelConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_consideration(Box::new(ThreatLevelConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_combiner(ScoreCombiner::Average)
}

fn create_balanced_reload_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("Reload", 0.5)
        .with_consideration(Box::new(AmmoLevelConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(DistanceToTargetConsideration::new(
            ResponseCurve::Polynomial { exponent: 2.0 },
        )))
        .with_combiner(ScoreCombiner::Multiplicative)
}

fn create_balanced_move_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("Move", 0.5)
        .with_consideration(Box::new(ExposedDangerConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_consideration(Box::new(TacticalAdvantageConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_consideration(Box::new(ForceBalanceConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_consideration(Box::new(SupportProximityConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(ObjectivePressureConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_consideration(Box::new(RetreatNecessityConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_combiner(ScoreCombiner::Average)
}

fn create_balanced_seek_cover_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("SeekCover", 0.7)
        .with_consideration(Box::new(HealthLevelConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(CoverQualityConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_consideration(Box::new(ThreatLevelConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_combiner(ScoreCombiner::Multiplicative)
}

fn create_balanced_seek_objective_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("SeekObjective", 0.4)
        .with_consideration(Box::new(ObjectiveProximityConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(AlliesNearbyConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_combiner(ScoreCombiner::Average)
}

fn create_balanced_wait_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("Wait", 0.1)
        .with_consideration(Box::new(AmmoLevelConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_combiner(ScoreCombiner::Minimum)
}

fn create_aggressive_shoot_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("Shoot", 1.0)  // Highest priority for aggressive
        .with_consideration(Box::new(HasLineOfSightConsideration::new(
            ResponseCurve::Boolean { threshold: 0.5 },
        )))
        .with_consideration(Box::new(DistanceToTargetConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(AmmoLevelConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_consideration(Box::new(ThreatLevelConsideration::new(
            ResponseCurve::Polynomial { exponent: 2.0 },
        )))
        .with_combiner(ScoreCombiner::Average)
}

fn create_aggressive_reload_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("Reload", 0.6)
        .with_consideration(Box::new(AmmoLevelConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(DistanceToTargetConsideration::new(
            ResponseCurve::Polynomial { exponent: 2.0 },
        )))
        .with_combiner(ScoreCombiner::Multiplicative)
}

fn create_aggressive_move_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("Move", 0.4)  // Lower base - aggression prioritizes shooting
        .with_consideration(Box::new(ExposedDangerConsideration::new(
            ResponseCurve::Linear,  // Less concerned with danger
        )))
        .with_consideration(Box::new(TacticalAdvantageConsideration::new(
            ResponseCurve::Polynomial { exponent: 2.0 },  // Aggressive seeks flanking
        )))
        .with_consideration(Box::new(ForceBalanceConsideration::new(
            ResponseCurve::Inverse,  // Less concerned about being outnumbered
        )))
        .with_consideration(Box::new(SupportProximityConsideration::new(
            ResponseCurve::Inverse,  // Doesn't need to stay close to allies
        )))
        .with_consideration(Box::new(ObjectivePressureConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_consideration(Box::new(RetreatNecessityConsideration::new(
            ResponseCurve::Polynomial { exponent: 2.0 },  // Only retreats when very hurt
        )))
        .with_combiner(ScoreCombiner::Average)
}

fn create_aggressive_seek_cover_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("SeekCover", 0.4)
        .with_consideration(Box::new(HealthLevelConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(CoverQualityConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_consideration(Box::new(ThreatLevelConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_combiner(ScoreCombiner::Multiplicative)
}

fn create_aggressive_seek_objective_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("SeekObjective", 0.3)
        .with_consideration(Box::new(ObjectiveProximityConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(AlliesNearbyConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_combiner(ScoreCombiner::Average)
}

fn create_aggressive_wait_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("Wait", 0.05)
        .with_consideration(Box::new(AmmoLevelConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_combiner(ScoreCombiner::Minimum)
}

fn create_defensive_shoot_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("Shoot", 0.7)  // Lower priority for defensive
        .with_consideration(Box::new(HasLineOfSightConsideration::new(
            ResponseCurve::Boolean { threshold: 0.5 },
        )))
        .with_consideration(Box::new(DistanceToTargetConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(AmmoLevelConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_consideration(Box::new(ThreatLevelConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_consideration(Box::new(CoverQualityConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_combiner(ScoreCombiner::Average)
}

fn create_defensive_reload_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("Reload", 0.7)
        .with_consideration(Box::new(AmmoLevelConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(DistanceToTargetConsideration::new(
            ResponseCurve::Polynomial { exponent: 2.0 },
        )))
        .with_combiner(ScoreCombiner::Multiplicative)
}

fn create_defensive_move_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("Move", 0.6)  // Higher base - defensive prefers repositioning
        .with_consideration(Box::new(ExposedDangerConsideration::new(
            ResponseCurve::Polynomial { exponent: 2.0 },  // Very concerned with danger
        )))
        .with_consideration(Box::new(TacticalAdvantageConsideration::new(
            ResponseCurve::Polynomial { exponent: 2.0 },  // Seeks good cover strongly
        )))
        .with_consideration(Box::new(ForceBalanceConsideration::new(
            ResponseCurve::Polynomial { exponent: 2.0 },  // Retreats when outnumbered
        )))
        .with_consideration(Box::new(SupportProximityConsideration::new(
            ResponseCurve::Linear,  // Wants to stay near allies
        )))
        .with_consideration(Box::new(ObjectivePressureConsideration::new(
            ResponseCurve::Inverse,  // Less objective-focused
        )))
        .with_consideration(Box::new(RetreatNecessityConsideration::new(
            ResponseCurve::Linear,  // Retreats readily when hurt
        )))
        .with_combiner(ScoreCombiner::Average)
}

fn create_defensive_seek_cover_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("SeekCover", 0.9)
        .with_consideration(Box::new(HealthLevelConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(CoverQualityConsideration::new(
            ResponseCurve::Polynomial { exponent: 2.0 },
        )))
        .with_consideration(Box::new(ThreatLevelConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_combiner(ScoreCombiner::Multiplicative)
}

fn create_defensive_seek_objective_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("SeekObjective", 0.3)
        .with_consideration(Box::new(ObjectiveProximityConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(AlliesNearbyConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_combiner(ScoreCombiner::Average)
}

fn create_defensive_wait_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("Wait", 0.2)
        .with_consideration(Box::new(AmmoLevelConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_combiner(ScoreCombiner::Minimum)
}

fn create_objective_shoot_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("Shoot", 0.6)  // Moderate - objectives more important
        .with_consideration(Box::new(HasLineOfSightConsideration::new(
            ResponseCurve::Boolean { threshold: 0.5 },
        )))
        .with_consideration(Box::new(DistanceToTargetConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(AmmoLevelConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_consideration(Box::new(ThreatLevelConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_combiner(ScoreCombiner::Average)
}

fn create_objective_reload_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("Reload", 0.5)
        .with_consideration(Box::new(AmmoLevelConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(DistanceToTargetConsideration::new(
            ResponseCurve::Polynomial { exponent: 2.0 },
        )))
        .with_combiner(ScoreCombiner::Multiplicative)
}

fn create_objective_move_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("Move", 0.5)
        .with_consideration(Box::new(ExposedDangerConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_consideration(Box::new(TacticalAdvantageConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_consideration(Box::new(ForceBalanceConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_consideration(Box::new(SupportProximityConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(ObjectivePressureConsideration::new(
            ResponseCurve::Polynomial { exponent: 2.0 },  // Strongly prioritizes objectives
        )))
        .with_consideration(Box::new(RetreatNecessityConsideration::new(
            ResponseCurve::Polynomial { exponent: 2.0 },  // Mission-focused, retreats reluctantly
        )))
        .with_combiner(ScoreCombiner::Average)
}

fn create_objective_seek_cover_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("SeekCover", 0.6)
        .with_consideration(Box::new(HealthLevelConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(CoverQualityConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_consideration(Box::new(ThreatLevelConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_combiner(ScoreCombiner::Multiplicative)
}

fn create_objective_seek_objective_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("SeekObjective", 0.8)
        .with_consideration(Box::new(ObjectiveProximityConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(AlliesNearbyConsideration::new(
            ResponseCurve::Polynomial { exponent: 2.0 },
        )))
        .with_combiner(ScoreCombiner::Average)
}

fn create_objective_wait_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("Wait", 0.1)
        .with_consideration(Box::new(AmmoLevelConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_combiner(ScoreCombiner::Minimum)
}

fn create_scout_shoot_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("Shoot", 0.5)
        .with_consideration(Box::new(HasLineOfSightConsideration::new(
            ResponseCurve::Boolean { threshold: 0.5 },
        )))
        .with_consideration(Box::new(DistanceToTargetConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(AmmoLevelConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_consideration(Box::new(ThreatLevelConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_combiner(ScoreCombiner::Average)
}

fn create_scout_reload_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("Reload", 0.4)
        .with_consideration(Box::new(AmmoLevelConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(DistanceToTargetConsideration::new(
            ResponseCurve::Polynomial { exponent: 2.0 },
        )))
        .with_combiner(ScoreCombiner::Multiplicative)
}

fn create_scout_move_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("Move", 0.8)
        .with_consideration(Box::new(ExposedDangerConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(TacticalAdvantageConsideration::new(
            ResponseCurve::Polynomial { exponent: 2.0 },
        )))
        .with_consideration(Box::new(ForceBalanceConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(SupportProximityConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(ObjectivePressureConsideration::new(
            ResponseCurve::Polynomial { exponent: 2.0 },
        )))
        .with_consideration(Box::new(RetreatNecessityConsideration::new(
            ResponseCurve::Polynomial { exponent: 2.0 },
        )))
        .with_combiner(ScoreCombiner::Average)
}

fn create_scout_seek_cover_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("SeekCover", 0.3)
        .with_consideration(Box::new(HealthLevelConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(CoverQualityConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_consideration(Box::new(ThreatLevelConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_combiner(ScoreCombiner::Multiplicative)
}

fn create_scout_seek_objective_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("SeekObjective", 0.9)
        .with_consideration(Box::new(ObjectiveProximityConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(AlliesNearbyConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_combiner(ScoreCombiner::Average)
}

fn create_scout_wait_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("Wait", 0.05)
        .with_consideration(Box::new(AmmoLevelConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_combiner(ScoreCombiner::Minimum)
}

fn create_rearguard_shoot_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("Shoot", 0.6)
        .with_consideration(Box::new(HasLineOfSightConsideration::new(
            ResponseCurve::Boolean { threshold: 0.5 },
        )))
        .with_consideration(Box::new(DistanceToTargetConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(AmmoLevelConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_consideration(Box::new(ThreatLevelConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_combiner(ScoreCombiner::Average)
}

fn create_rearguard_reload_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("Reload", 0.7)
        .with_consideration(Box::new(AmmoLevelConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(DistanceToTargetConsideration::new(
            ResponseCurve::Polynomial { exponent: 2.0 },
        )))
        .with_combiner(ScoreCombiner::Multiplicative)
}

fn create_rearguard_move_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("Move", 0.3)
        .with_consideration(Box::new(ExposedDangerConsideration::new(
            ResponseCurve::Polynomial { exponent: 2.0 },
        )))
        .with_consideration(Box::new(TacticalAdvantageConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_consideration(Box::new(ForceBalanceConsideration::new(
            ResponseCurve::Polynomial { exponent: 2.0 },
        )))
        .with_consideration(Box::new(SupportProximityConsideration::new(
            ResponseCurve::Polynomial { exponent: 2.0 },
        )))
        .with_consideration(Box::new(ObjectivePressureConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(RetreatNecessityConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_combiner(ScoreCombiner::Average)
}

fn create_rearguard_seek_cover_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("SeekCover", 0.8)
        .with_consideration(Box::new(HealthLevelConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(CoverQualityConsideration::new(
            ResponseCurve::Polynomial { exponent: 2.0 },
        )))
        .with_consideration(Box::new(ThreatLevelConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_combiner(ScoreCombiner::Multiplicative)
}

fn create_rearguard_seek_objective_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("SeekObjective", 0.7)
        .with_consideration(Box::new(ObjectiveProximityConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_consideration(Box::new(AlliesNearbyConsideration::new(
            ResponseCurve::Polynomial { exponent: 2.0 },
        )))
        .with_combiner(ScoreCombiner::Average)
}

fn create_rearguard_wait_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("Wait", 0.3)
        .with_consideration(Box::new(AmmoLevelConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_combiner(ScoreCombiner::Minimum)
}
