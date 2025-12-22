use crate::ai::{
    considerations::{
        ActionContext, AlliesNearbyConsideration, AmmoLevelConsideration, Consideration,
        CoverQualityConsideration, DistanceToTargetConsideration, HasLineOfSightConsideration,
        HealthLevelConsideration, NearbyOfficerConsideration, ObjectiveProximityConsideration,
        ThreatLevelConsideration,
    },
    response_curves::ResponseCurve,
};
use crate::components::action::ActionType;
use specs::Entity;

#[derive(Debug, Clone)]
pub struct ScoredAction {
    pub action_type: ActionType,
    pub target: Option<Entity>,
    pub position: Option<crate::game_logic::battlefield::Position>,
    pub score: f32,
    pub debug_info: Option<String>,
}

impl ScoredAction {
    pub fn new(action_type: ActionType, score: f32) -> Self {
        Self {
            action_type,
            target: None,
            position: None,
            score,
            debug_info: None,
        }
    }

    pub fn with_target(mut self, target: Entity) -> Self {
        self.target = Some(target);
        self
    }

    pub fn with_position(mut self, position: crate::game_logic::battlefield::Position) -> Self {
        self.position = Some(position);
        self
    }

    pub fn with_debug(mut self, info: String) -> Self {
        self.debug_info = Some(info);
        self
    }
}

#[derive(Debug, Clone)]
pub enum ScoreCombiner {
    Multiplicative,
    Average,
    WeightedSum { weights: Vec<f32> },
    Minimum,
}

impl ScoreCombiner {
    pub fn combine(&self, base_score: f32, consideration_scores: &[f32]) -> f32 {
        match self {
            ScoreCombiner::Multiplicative => {
                let mut result = base_score;
                for &score in consideration_scores {
                    result *= score;
                }
                result
            }

            ScoreCombiner::Average => {
                let sum: f32 = consideration_scores.iter().sum();
                (base_score + sum) / (1.0 + consideration_scores.len() as f32)
            }

            ScoreCombiner::WeightedSum { weights } => {
                let mut result = base_score;
                for (i, &score) in consideration_scores.iter().enumerate() {
                    let weight = weights.get(i).copied().unwrap_or(1.0);
                    result += weight * score;
                }
                result
            }

            ScoreCombiner::Minimum => {
                let mut min_score = base_score;
                for &score in consideration_scores {
                    if score < min_score {
                        min_score = score;
                    }
                }
                min_score
            }
        }
    }
}

pub struct ActionEvaluator {
    pub name: String,
    pub base_score: f32,
    pub considerations: Vec<Box<dyn Consideration>>,
    pub combiner: ScoreCombiner,
}

impl ActionEvaluator {
    pub fn new(name: impl Into<String>, base_score: f32) -> Self {
        Self {
            name: name.into(),
            base_score,
            considerations: Vec::new(),
            combiner: ScoreCombiner::Multiplicative,
        }
    }

    pub fn with_consideration(mut self, consideration: Box<dyn Consideration>) -> Self {
        self.considerations.push(consideration);
        self
    }

    pub fn with_combiner(mut self, combiner: ScoreCombiner) -> Self {
        self.combiner = combiner;
        self
    }

    pub fn evaluate(&self, context: &ActionContext) -> f32 {
        let scores: Vec<f32> = self
            .considerations
            .iter()
            .map(|c| c.evaluate(context))
            .collect();

        self.combiner.combine(self.base_score, &scores)
    }
}

pub fn create_shoot_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("Shoot", 0.8)
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
        .with_combiner(ScoreCombiner::Multiplicative)
}

pub fn create_reload_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("Reload", 0.5)
        .with_consideration(Box::new(AmmoLevelConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(DistanceToTargetConsideration::new(
            ResponseCurve::Polynomial { exponent: 2.0 },
        )))
        .with_combiner(ScoreCombiner::Multiplicative)
}

pub fn create_move_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("Move", 0.75)
        .with_consideration(Box::new(DistanceToTargetConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_consideration(Box::new(HealthLevelConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_consideration(Box::new(NearbyOfficerConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_combiner(ScoreCombiner::Average)
}

pub fn create_seek_cover_evaluator() -> ActionEvaluator {
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

pub fn create_seek_objective_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("SeekObjective", 0.6)
        .with_consideration(Box::new(ObjectiveProximityConsideration::new(
            ResponseCurve::Inverse,
        )))
        .with_consideration(Box::new(AlliesNearbyConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_combiner(ScoreCombiner::Average)
}

pub fn create_wait_evaluator() -> ActionEvaluator {
    ActionEvaluator::new("Wait", 0.05)
        .with_consideration(Box::new(AmmoLevelConsideration::new(
            ResponseCurve::Linear,
        )))
        .with_combiner(ScoreCombiner::Minimum)
}
