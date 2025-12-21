// Argue the Toss - WWI Trench Warfare Roguelike
// Main entry point

use argue_the_toss::{
    components::{
        action::{OngoingAction, QueuedAction},
        player::Player,
        position::Position,
        soldier::{Faction, Rank, Soldier},
        time_budget::TimeBudget,
    },
    config::game_config::GameConfig,
    game_logic::{
        battlefield::{Battlefield, Position as BattlefieldPos, TerrainType},
        turn_state::TurnState,
    },
    rendering::{viewport::Camera, widgets::BattlefieldWidget},
    systems::{
        action_execution::ActionExecutionSystem, ai_action_planner::AIActionPlannerSystem,
        turn_manager::TurnManagerSystem,
    },
    utils::{event_log::EventLog, input_mode::InputMode},
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
use specs::{Builder, DispatcherBuilder, Join, World, WorldExt};
use std::io;

/// Main game state
struct GameState {
    world: World,
    battlefield: Battlefield,
    camera: Camera,
    running: bool,
    input_mode: InputMode,
    cursor_pos: BattlefieldPos, // For Look mode
    config: GameConfig,
}

impl GameState {
    fn new(viewport_width: usize, viewport_height: usize) -> Self {
        let mut world = World::new();

        // Register components
        world.register::<Position>();
        world.register::<Soldier>();
        world.register::<Player>();
        world.register::<TimeBudget>();
        world.register::<QueuedAction>();
        world.register::<OngoingAction>();

        // Create game config
        let config = GameConfig::default();

        // Create event log
        let mut event_log = EventLog::new();
        event_log.add("Welcome to Argue the Toss!".to_string());
        event_log.add("WWI Trench Warfare Roguelike".to_string());

        // Insert resources
        world.insert(TurnState::new());
        world.insert(event_log);

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

        // Player starting position
        let player_start_pos = BattlefieldPos::new(50, 50);

        // Create camera centered at player position with adaptive viewport
        let camera = Camera::new(player_start_pos, viewport_width, viewport_height);

        // Create some test soldiers
        // First soldier is player-controlled
        world
            .create_entity()
            .with(Position::new(player_start_pos.x, player_start_pos.y))
            .with(Soldier {
                name: "Pvt. Smith".to_string(),
                faction: Faction::Allies,
                rank: Rank::Private,
            })
            .with(Player)
            .with(TimeBudget::new(config.time_budget_seconds))
            .build();

        world
            .create_entity()
            .with(Position::new(55, 52))
            .with(Soldier {
                name: "Sgt. Jones".to_string(),
                faction: Faction::Allies,
                rank: Rank::Sergeant,
            })
            .with(TimeBudget::new(config.time_budget_seconds))
            .build();

        world
            .create_entity()
            .with(Position::new(45, 48))
            .with(Soldier {
                name: "Pvt. Mueller".to_string(),
                faction: Faction::CentralPowers,
                rank: Rank::Private,
            })
            .with(TimeBudget::new(config.time_budget_seconds))
            .build();

        Self {
            world,
            battlefield,
            camera,
            running: true,
            input_mode: InputMode::default(),
            cursor_pos: player_start_pos,
            config,
        }
    }

    /// Update viewport size based on terminal dimensions
    fn update_viewport_size(&mut self, area: Rect) {
        // Account for borders (2 chars horizontal, 2 vertical) and status panel
        let new_width = (area.width.saturating_sub(2)) as usize;
        let new_height = (area.height.saturating_sub(7)) as usize; // -2 for borders, -5 for status panel

        // Only update if size actually changed
        if new_width != self.camera.viewport_width || new_height != self.camera.viewport_height {
            self.camera.viewport_width = new_width;
            self.camera.viewport_height = new_height;
        }
    }

    fn handle_input(&mut self, key: KeyEvent) {
        match self.input_mode {
            InputMode::Command => self.handle_command_mode(key),
            InputMode::Look => self.handle_look_mode(key),
        }
    }

    fn handle_command_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.running = false,
            KeyCode::Char('x') => {
                // Enter Look mode
                self.input_mode = InputMode::Look;
                // Set cursor to player position
                if let Some(player_pos) = self.get_player_position() {
                    self.cursor_pos = player_pos;
                }
            }
            KeyCode::Char('c') => {
                // Center camera on player
                if let Some(player_pos) = self.get_player_position() {
                    self.camera.center_on(player_pos);
                    self.camera
                        .constrain(self.battlefield.width(), self.battlefield.height());
                }
            }
            // Movement keys - commit movement actions
            KeyCode::Up | KeyCode::Char('k') => self.commit_player_action(0, -1),
            KeyCode::Down | KeyCode::Char('j') => self.commit_player_action(0, 1),
            KeyCode::Left | KeyCode::Char('h') => self.commit_player_action(-1, 0),
            KeyCode::Right | KeyCode::Char('l') => self.commit_player_action(1, 0),
            _ => {}
        }
    }

    fn handle_look_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                // Exit Look mode back to Command
                self.input_mode = InputMode::Command;
            }
            KeyCode::Enter => {
                // Select target at cursor (future: targeting logic)
                self.input_mode = InputMode::Command;
            }
            KeyCode::Char('c') => {
                // Center camera on player
                if let Some(player_pos) = self.get_player_position() {
                    self.camera.center_on(player_pos);
                    self.camera
                        .constrain(self.battlefield.width(), self.battlefield.height());
                }
            }
            // Movement keys - move cursor AND camera in Look mode
            KeyCode::Up | KeyCode::Char('k') => {
                self.cursor_pos.y -= 1;
                self.constrain_cursor();
                self.camera.pan(0, -1);
                self.camera
                    .constrain(self.battlefield.width(), self.battlefield.height());
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.cursor_pos.y += 1;
                self.constrain_cursor();
                self.camera.pan(0, 1);
                self.camera
                    .constrain(self.battlefield.width(), self.battlefield.height());
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.cursor_pos.x -= 1;
                self.constrain_cursor();
                self.camera.pan(-1, 0);
                self.camera
                    .constrain(self.battlefield.width(), self.battlefield.height());
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.cursor_pos.x += 1;
                self.constrain_cursor();
                self.camera.pan(1, 0);
                self.camera
                    .constrain(self.battlefield.width(), self.battlefield.height());
            }
            _ => {}
        }
    }

    fn commit_player_action(&mut self, dx: i32, dy: i32) {
        use argue_the_toss::components::action::{ActionType, QueuedAction};
        use argue_the_toss::game_logic::turn_state::TurnState;
        use specs::WorldExt;

        let player_entity = match self.get_player_entity() {
            Some(e) => e,
            None => return,
        };

        // Get current position to calculate terrain cost
        let positions = self.world.read_storage::<Position>();
        let current_pos = match positions.get(player_entity) {
            Some(pos) => *pos,
            None => return,
        };
        drop(positions);

        // Calculate target position and get terrain cost
        let new_x = current_pos.x() + dx;
        let new_y = current_pos.y() + dy;
        let new_pos = BattlefieldPos::new(new_x, new_y);

        // Check if new position is valid
        if !self.battlefield.in_bounds(&new_pos) {
            self.world.write_resource::<EventLog>().add("Cannot move out of bounds!".to_string());
            return;
        }

        let terrain_cost = self
            .battlefield
            .get_tile(&new_pos)
            .map(|t| t.terrain.movement_cost())
            .unwrap_or(1.0);

        // Create movement action
        let action_type = ActionType::Move {
            dx,
            dy,
            terrain_cost,
        };
        let time_cost = action_type.base_time_cost();

        // Commit action
        let mut time_budgets = self.world.write_storage::<TimeBudget>();
        let mut queued_actions = self.world.write_storage::<QueuedAction>();

        if let Some(budget) = time_budgets.get_mut(player_entity) {
            budget.consume_time(time_cost);

            queued_actions
                .insert(player_entity, QueuedAction::new(action_type))
                .ok();

            self.world.write_resource::<EventLog>()
                .add(format!("Movement queued ({:.1}s)", time_cost));

            // Check if turn should end (budget exhausted or in debt)
            if budget.available_time() <= 0.0 {
                let mut turn_state = self.world.write_resource::<TurnState>();
                turn_state.mark_entity_ready(player_entity);

                self.world.write_resource::<EventLog>()
                    .add("Time budget exhausted. Waiting for others...".to_string());
            }
        }
    }

    #[allow(dead_code)]
    fn move_player(&mut self, dx: i32, dy: i32) {
        let mut positions = self.world.write_storage::<Position>();
        let players = self.world.read_storage::<Player>();

        for (_player, pos) in (&players, &mut positions).join() {
            let new_x = pos.x() + dx;
            let new_y = pos.y() + dy;
            let new_pos = BattlefieldPos::new(new_x, new_y);

            // Check if new position is valid
            if self.battlefield.in_bounds(&new_pos) {
                *pos = Position::new(new_x, new_y);

                // Update camera to follow player with deadzone in Command mode
                if self.input_mode == InputMode::Command {
                    self.camera.follow_target(&new_pos);
                    self.camera
                        .constrain(self.battlefield.width(), self.battlefield.height());
                }
            }
            break; // Only one player
        }
    }

    fn get_player_position(&self) -> Option<BattlefieldPos> {
        let positions = self.world.read_storage::<Position>();
        let players = self.world.read_storage::<Player>();

        for (_player, pos) in (&players, &positions).join() {
            return Some(*pos.as_battlefield_pos());
        }
        None
    }

    fn get_player_entity(&self) -> Option<specs::Entity> {
        use specs::Join;
        let players = self.world.read_storage::<Player>();
        let entities = self.world.entities();

        for (entity, _) in (&entities, &players).join() {
            return Some(entity);
        }
        None
    }

    fn constrain_cursor(&mut self) {
        self.cursor_pos.x = self
            .cursor_pos
            .x
            .max(0)
            .min(self.battlefield.width() as i32 - 1);
        self.cursor_pos.y = self
            .cursor_pos
            .y
            .max(0)
            .min(self.battlefield.height() as i32 - 1);
    }

    /// Get terrain description at a position
    fn get_terrain_info(&self, pos: &BattlefieldPos) -> String {
        if let Some(tile) = self.battlefield.get_tile(pos) {
            let terrain_name = match tile.terrain {
                TerrainType::Trench => "Trench",
                TerrainType::NoMansLand => "No Man's Land",
                TerrainType::Mud => "Mud",
                TerrainType::Fortification => "Fortification",
            };
            let visibility = if tile.visible {
                "visible"
            } else if tile.explored {
                "explored"
            } else {
                "unexplored"
            };
            format!("{} ({})", terrain_name, visibility)
        } else {
            "Out of bounds".to_string()
        }
    }

    /// Get entity info at cursor position (for Look mode)
    fn get_entity_info(&self, pos: &BattlefieldPos) -> Option<String> {
        let positions = self.world.read_storage::<Position>();
        let soldiers = self.world.read_storage::<Soldier>();
        let players = self.world.read_storage::<Player>();
        let entities = self.world.entities();

        for (entity, soldier_pos, soldier) in (&entities, &positions, &soldiers).join() {
            if soldier_pos.as_battlefield_pos() == pos {
                let is_player = players.contains(entity);
                let player_marker = if is_player { " (YOU)" } else { "" };
                return Some(format!(
                    "{}{} - {:?} {}",
                    soldier.name, player_marker, soldier.faction, soldier.rank.as_str()
                ));
            }
        }
        None
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
    // Main layout: Top (battlefield + event log) and Bottom (info panel)
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),      // Top: battlefield + event log
            Constraint::Length(7),    // Bottom: info panel
        ])
        .split(f.area());

    // Top split: Battlefield (left) and Event Log (right)
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(75),  // Battlefield
            Constraint::Percentage(25),  // Event log
        ])
        .split(main_chunks[0]);

    // Render battlefield
    let battlefield_block = Block::default()
        .title("Battlefield")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    let inner_area = battlefield_block.inner(top_chunks[0]);
    f.render_widget(battlefield_block, top_chunks[0]);

    let battlefield_widget = BattlefieldWidget::new(&state.battlefield, &state.camera);
    f.render_widget(battlefield_widget, inner_area);

    // Render soldiers on top
    render_soldiers(f, inner_area, state);

    // Render cursor in Look mode
    if state.input_mode == InputMode::Look {
        render_cursor(f, inner_area, state);
    }

    // Render event log
    let event_log_block = Block::default()
        .title("Event Log")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let event_lines: Vec<Line> = {
        let event_log = state.world.fetch::<EventLog>();
        event_log
            .recent(15)
            .iter()
            .map(|e| Line::from(e.to_string()))
            .collect()
    };

    let event_paragraph = Paragraph::new(Text::from(event_lines)).block(event_log_block);
    f.render_widget(event_paragraph, top_chunks[1]);

    // Render info panel
    let mode_color = match state.input_mode {
        InputMode::Command => Color::Green,
        InputMode::Look => Color::Yellow,
    };

    let info_block = Block::default()
        .title(format!("Mode: {} | Info", state.input_mode.name()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(mode_color));

    let mut info_lines = vec![
        Line::from(state.input_mode.help_text()),
        Line::from(""),
    ];

    // Show time budget for player
    if let Some(player_entity) = state.get_player_entity() {
        let time_budgets = state.world.read_storage::<TimeBudget>();
        let turn_state = state.world.fetch::<TurnState>();

        if let Some(budget) = time_budgets.get(player_entity) {
            let available = budget.available_time();
            let budget_color = if budget.time_debt < 0.0 {
                "DEBT"
            } else if available > 3.0 {
                "Good"
            } else if available > 1.0 {
                "Low"
            } else {
                "Critical"
            };

            let time_info = if budget.time_debt < 0.0 {
                format!(
                    "Turn {} | Time: {:.1}s ({}) | Debt: {:.1}s",
                    turn_state.current_turn, available, budget_color, -budget.time_debt
                )
            } else {
                format!(
                    "Turn {} | Time: {:.1}s ({})",
                    turn_state.current_turn, available, budget_color
                )
            };

            info_lines.push(Line::from(time_info));
            info_lines.push(Line::from(""));
        }
    }

    // Show terrain info for player position or cursor position
    let inspect_pos = if state.input_mode == InputMode::Look {
        state.cursor_pos
    } else {
        state.get_player_position().unwrap_or(state.cursor_pos)
    };

    info_lines.push(Line::from(format!(
        "Position: ({}, {})",
        inspect_pos.x, inspect_pos.y
    )));
    info_lines.push(Line::from(format!(
        "Terrain: {}",
        state.get_terrain_info(&inspect_pos)
    )));

    // In Look mode, show entity info if any
    if state.input_mode == InputMode::Look {
        if let Some(entity_info) = state.get_entity_info(&state.cursor_pos) {
            info_lines.push(Line::from(""));
            info_lines.push(Line::from(format!("Entity: {}", entity_info)));
        }
    }

    let info_paragraph = Paragraph::new(Text::from(info_lines)).block(info_block);
    f.render_widget(info_paragraph, main_chunks[1]);
}

fn render_soldiers(f: &mut Frame, area: Rect, state: &GameState) {
    let entities = state.world.entities();
    let positions = state.world.read_storage::<Position>();
    let soldiers = state.world.read_storage::<Soldier>();
    let players = state.world.read_storage::<Player>();

    let top_left = state.camera.top_left();

    for (entity, pos, soldier) in (&entities, &positions, &soldiers).join() {
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

                // Player character is bright green, others use faction colors
                let color = if players.contains(entity) {
                    Color::LightGreen
                } else {
                    match soldier.faction {
                        Faction::Allies => Color::Blue,
                        Faction::CentralPowers => Color::Red,
                    }
                };

                f.buffer_mut()[(buf_x, buf_y)]
                    .set_char(ch)
                    .set_style(Style::default().fg(color));
            }
        }
    }
}

fn render_cursor(f: &mut Frame, area: Rect, state: &GameState) {
    let top_left = state.camera.top_left();
    let screen_x = state.cursor_pos.x - top_left.x;
    let screen_y = state.cursor_pos.y - top_left.y;

    // Only render if within viewport
    if screen_x >= 0
        && screen_x < area.width as i32
        && screen_y >= 0
        && screen_y < area.height as i32
    {
        let buf_x = area.x + screen_x as u16;
        let buf_y = area.y + screen_y as u16;

        if buf_x < area.right() && buf_y < area.bottom() {
            // Render cursor as a highlighted square
            f.buffer_mut()[(buf_x, buf_y)]
                .set_style(Style::default().bg(Color::Yellow));
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

    // Get initial terminal size
    let size = terminal.size()?;
    let initial_width = (size.width.saturating_sub(2)) as usize;
    let initial_height = (size.height.saturating_sub(7)) as usize;

    // Create game state with adaptive viewport
    let mut game_state = GameState::new(initial_width, initial_height);

    // Create ECS dispatcher for turn-based systems
    let mut dispatcher = DispatcherBuilder::new()
        .with(AIActionPlannerSystem, "ai_planner", &[])
        .with(ActionExecutionSystem, "action_execution", &[])
        .with(TurnManagerSystem, "turn_manager", &["action_execution"])
        .build();

    // Main game loop
    while game_state.running {
        // Update visibility
        game_state.update_visibility();

        // Render
        terminal.draw(|f| {
            // Update viewport size if terminal was resized
            game_state.update_viewport_size(f.area());
            ui(f, &game_state)
        })?;

        // Run ECS systems (process turns, AI, actions)
        dispatcher.dispatch(&game_state.world);
        game_state.world.maintain();

        // Handle input (only in planning phase)
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                let turn_state = game_state.world.fetch::<TurnState>();
                let can_input = matches!(turn_state.phase, argue_the_toss::game_logic::turn_state::TurnPhase::Planning);

                // In PlayerFirst mode, only accept input if player hasn't finished
                let player_can_act = if matches!(
                    turn_state.turn_order_mode,
                    argue_the_toss::game_logic::turn_state::TurnOrderMode::PlayerFirst
                ) {
                    if let Some(player_entity) = game_state.get_player_entity() {
                        !turn_state.is_entity_ready(player_entity)
                    } else {
                        false
                    }
                } else {
                    true // In other modes, player can always act during Planning
                };

                drop(turn_state); // Release borrow

                if can_input && player_can_act {
                    game_state.handle_input(key);
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
