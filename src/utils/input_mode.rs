// Input mode management for modal UI (vim-style)

/// Defines the different interaction modes in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// Command mode: direct character control
    /// - hjkl/arrows move the player character
    /// - Other keys trigger actions
    /// - Camera follows player
    Command,

    /// Look/Targeting mode: free cursor for examination and targeting
    /// - hjkl/arrows move the cursor
    /// - Enter selects target
    /// - ESC returns to Command mode
    /// - Camera can pan independently
    Look,
}

impl InputMode {
    pub fn name(&self) -> &'static str {
        match self {
            InputMode::Command => "COMMAND",
            InputMode::Look => "LOOK",
        }
    }

    pub fn help_text(&self) -> &'static str {
        match self {
            InputMode::Command => "hjkl/arrows: move | x: look | c: center | q: quit",
            InputMode::Look => "hjkl/arrows: pan camera | c: center | Enter: select | ESC: exit",
        }
    }
}

impl Default for InputMode {
    fn default() -> Self {
        InputMode::Command
    }
}
