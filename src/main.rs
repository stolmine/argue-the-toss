// Argue the Toss - WWI Trench Warfare Roguelike
// Main entry point

use argue_the_toss::{
    components::{position::Position, soldier::{Faction, Rank, Soldier}},
    game_logic::battlefield::{Battlefield, Position as BattlefieldPos, TerrainType},
    rendering::{viewport::Camera, widgets::BattlefieldWidget},
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use specs::{Builder, World, WorldExt};
use std::io;

/// Main game state
struct GameState {
    world: World,
    battlefield: Battlefield,
    camera: Camera,
    running: bool,
}

impl GameState {
    fn new() -> Self {
        let mut world = World::new();

        // Register components
        world.register::<Position>();
        world.register::<Soldier>();

        // Create battlefield (100x100 grid)
        let mut battlefield = Battlefield::new(100, 100);

        // Add some terrain variety
        for x in 10..90 {
            battlefield.set_terrain(
                BattlefieldPos::new(x, 20),
                TerrainType::Trench,
            );
            battlefield.set_terrain(
                BattlefieldPos::new(x, 80),
                TerrainType::Trench,
            );
        }

        for y in 25..75 {
            for x in 40..60 {
                battlefield.set_terrain(
                    BattlefieldPos::new(x, y),
                    TerrainType::Mud,
                );
            }
        }

        // Create camera centered at (50, 50) with 60x40 viewport
        let camera = Camera::new(BattlefieldPos::new(50, 50), 60, 40);

        // Create some test soldiers
        world
            .create_entity()
            .with(Position::new(50, 50))
            .with(Soldier {
                name: "Pvt. Smith".to_string(),
                faction: Faction::Allies,
                rank: Rank::Private,
            })
            .build();

        world
            .create_entity()
            .with(Position::new(55, 52))
            .with(Soldier {
                name: "Sgt. Jones".to_string(),
                faction: Faction::Allies,
                rank: Rank::Sergeant,
            })
            .build();

        world
            .create_entity()
            .with(Position::new(45, 48))
            .with(Soldier {
                name: "Pvt. Mueller".to_string(),
                faction: Faction::CentralPowers,
                rank: Rank::Private,
            })
            .build();

        Self {
            world,
            battlefield,
            camera,
            running: true,
        }
    }

    fn handle_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.running = false,
            KeyCode::Up | KeyCode::Char('k') => self.camera.pan(0, -1),
            KeyCode::Down | KeyCode::Char('j') => self.camera.pan(0, 1),
            KeyCode::Left | KeyCode::Char('h') => self.camera.pan(-1, 0),
            KeyCode::Right | KeyCode::Char('l') => self.camera.pan(1, 0),
            _ => {}
        }

        // Constrain camera to battlefield bounds
        self.camera
            .constrain(self.battlefield.width(), self.battlefield.height());
    }

    fn update_visibility(&mut self) {
        // Reset all visibility
        self.battlefield.reset_visibility();

        // Make all tiles within viewport visible (simplified FOV)
        let top_left = self.camera.top_left();
        let bottom_right = self.camera.bottom_right();

        for y in top_left.y..=bottom_right.y {
            for x in top_left.x..=bottom_right.x {
                let pos = BattlefieldPos::new(x, y);
                if self.battlefield.in_bounds(&pos) {
                    self.battlefield.set_visible(pos, true);
                }
            }
        }
    }
}

fn ui(f: &mut Frame, state: &GameState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),      // Main battlefield view
            Constraint::Length(5),    // Status/info panel
        ])
        .split(f.area());

    // Render battlefield
    let battlefield_block = Block::default()
        .title("Battlefield")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    let inner_area = battlefield_block.inner(chunks[0]);
    f.render_widget(battlefield_block, chunks[0]);

    let battlefield_widget = BattlefieldWidget::new(&state.battlefield, &state.camera);
    f.render_widget(battlefield_widget, inner_area);

    // Render soldiers on top
    render_soldiers(f, inner_area, state);

    // Render status panel
    let status_block = Block::default()
        .title("Controls")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    let status_text = vec![
        Line::from("Arrow Keys / hjkl: Pan camera"),
        Line::from("q: Quit"),
        Line::from(format!(
            "Camera: ({}, {})",
            state.camera.center.x, state.camera.center.y
        )),
    ];

    let status_paragraph = Paragraph::new(Text::from(status_text)).block(status_block);
    f.render_widget(status_paragraph, chunks[1]);
}

fn render_soldiers(f: &mut Frame, area: Rect, state: &GameState) {
    use specs::Join;

    let positions = state.world.read_storage::<Position>();
    let soldiers = state.world.read_storage::<Soldier>();

    let top_left = state.camera.top_left();

    for (pos, soldier) in (&positions, &soldiers).join() {
        let screen_x = pos.x() - top_left.x;
        let screen_y = pos.y() - top_left.y;

        // Only render if within viewport
        if screen_x >= 0
            && screen_x < area.width as i32
            && screen_y >= 0
            && screen_y < area.height as i32
        {
            let buf_x = area.x + screen_x as u16;
            let buf_y = area.y + screen_y as u16;

            if buf_x < area.right() && buf_y < area.bottom() {
                let ch = soldier.faction.to_char();
                let color = match soldier.faction {
                    Faction::Allies => Color::Blue,
                    Faction::CentralPowers => Color::Red,
                };

                f.buffer_mut()[(buf_x, buf_y)]
                    .set_char(ch)
                    .set_style(Style::default().fg(color));
            }
        }
    }
}

fn main() -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create game state
    let mut game_state = GameState::new();

    // Main game loop
    while game_state.running {
        // Update visibility
        game_state.update_visibility();

        // Render
        terminal.draw(|f| ui(f, &game_state))?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                game_state.handle_input(key);
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
