// Input mode management for modal UI (vim-style)

/// Defines the different interaction modes in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// Command mode: direct character control
    /// - qweasdzxc move the player in 8 directions
    /// - s waits in place
    /// - ,/. rotate facing CCW/CW
    /// - Other keys trigger actions
    /// - Camera follows player
    Command,

    /// Look mode: free cursor for examination and path planning
    /// - hjkl/arrows move the cursor
    /// - Enter selects destination for pathfinding
    /// - ESC returns to Command mode
    /// - Camera can pan independently
    Look,

    /// Targeting mode: select target for shooting
    /// - hjkl/arrows move the cursor
    /// - Enter shoots at target
    /// - ESC cancels and returns to Command mode
    /// - Camera can pan independently
    Targeting,
}

impl InputMode {
    pub fn name(&self) -> &'static str {
        match self {
            InputMode::Command => "COMMAND",
            InputMode::Look => "LOOK",
            InputMode::Targeting => "TARGETING",
        }
    }

    pub fn help_text(&self) -> &'static str {
        match self {
            InputMode::Command => "qweasdzxc: 8-dir move | s: wait | ,/.: rotate | Space: advance | f: fire | r: reload | l: look | v: center | Shift+Q/Ctrl+C: quit",
            InputMode::Look => "hjkl/arrows: pan camera | c: center | Enter: select destination | ESC: exit",
            InputMode::Targeting => "hjkl/arrows: pan camera | c: center | Enter: shoot target | ESC: cancel",
        }
    }
}

impl Default for InputMode {
    fn default() -> Self {
        InputMode::Command
    }
}
