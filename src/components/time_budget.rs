// Time budget component for turn-based time management

use specs::{Component, VecStorage};

/// Component: Per-entity time tracking for turn-based gameplay
#[derive(Debug, Clone)]
pub struct TimeBudget {
    /// Base duration per turn in seconds (configurable 5-30)
    pub base_duration: f32,
    /// Time debt from previous turns (negative = owe time, positive = extra time)
    pub time_debt: f32,
    /// Time spent this turn
    pub time_spent_this_turn: f32,
}

impl Component for TimeBudget {
    type Storage = VecStorage<Self>;
}

impl TimeBudget {
    pub fn new(base_duration: f32) -> Self {
        Self {
            base_duration,
            time_debt: 0.0,
            time_spent_this_turn: 0.0,
        }
    }

    /// Get available time remaining this turn
    pub fn available_time(&self) -> f32 {
        self.base_duration - self.time_spent_this_turn - self.time_debt
    }

    /// Consume time for an action (may create debt)
    pub fn consume_time(&mut self, cost: f32) -> bool {
        self.time_spent_this_turn += cost;

        // Update debt if we've gone over budget
        let total_spent = self.time_spent_this_turn + self.time_debt;
        if total_spent > self.base_duration {
            self.time_debt = total_spent - self.base_duration;
        }

        true // Always succeeds, may create debt
    }

    /// Check if we can afford an action (for UI display)
    pub fn can_afford(&self, cost: f32) -> bool {
        self.available_time() >= cost
    }

    /// Reset for new turn (keeps debt)
    pub fn reset_for_new_turn(&mut self) {
        self.time_spent_this_turn = 0.0;
        // Keep time_debt to carry forward
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_available_time() {
        let budget = TimeBudget::new(10.0);
        assert_eq!(budget.available_time(), 10.0);
    }

    #[test]
    fn test_consume_time() {
        let mut budget = TimeBudget::new(10.0);
        budget.consume_time(3.0);
        assert_eq!(budget.time_spent_this_turn, 3.0);
        assert_eq!(budget.available_time(), 7.0);
    }

    #[test]
    fn test_time_debt() {
        let mut budget = TimeBudget::new(10.0);
        budget.consume_time(12.0);
        assert_eq!(budget.time_debt, 2.0);
        assert_eq!(budget.available_time(), -2.0);
    }

    #[test]
    fn test_reset_keeps_debt() {
        let mut budget = TimeBudget::new(10.0);
        budget.consume_time(12.0);
        budget.reset_for_new_turn();
        assert_eq!(budget.time_spent_this_turn, 0.0);
        assert_eq!(budget.time_debt, 2.0);
        assert_eq!(budget.available_time(), 8.0);
    }
}
