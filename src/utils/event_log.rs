// Event log system for displaying game events

use std::collections::VecDeque;

/// Maximum number of events to keep in the log
const MAX_EVENTS: usize = 100;

/// Event log for tracking game events
pub struct EventLog {
    events: VecDeque<String>,
}

impl EventLog {
    pub fn new() -> Self {
        Self {
            events: VecDeque::with_capacity(MAX_EVENTS),
        }
    }

    /// Add a new event to the log
    pub fn add(&mut self, message: String) {
        self.events.push_front(message);
        if self.events.len() > MAX_EVENTS {
            self.events.pop_back();
        }
    }

    /// Get recent events (newest first)
    pub fn recent(&self, count: usize) -> Vec<&String> {
        self.events.iter().take(count).collect()
    }

    /// Get all events
    pub fn all(&self) -> &VecDeque<String> {
        &self.events
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

impl Default for EventLog {
    fn default() -> Self {
        Self::new()
    }
}
