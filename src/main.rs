// Argue the Toss - WWI Trench Warfare Roguelike
// Main entry point

use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

fn main() -> Result<(), io::Error> {
    // Initialize terminal
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Clear terminal
    terminal.clear()?;

    println!("Argue the Toss - WWI Trench Warfare Roguelike");
    println!("Project initialized successfully!");
    println!("\nPress any key to exit...");

    Ok(())
}
