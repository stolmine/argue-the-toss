// Argue the Toss - WWI Trench Warfare Roguelike
// Main entry point

use argue_the_toss::{
    components::{
        action::{OngoingAction, QueuedAction},
        dead::Dead,
        health::Health,
        pathfinding::PlannedPath,
        player::Player,
        position::Position,
        soldier::{Faction, Rank, Soldier},
        time_budget::TimeBudget,
        vision::Vision,
        weapon::Weapon,
    },
    config::game_config::GameConfig,
    game_logic::{
        battlefield::{Battlefield, Position as BattlefieldPos, TerrainType},
        pathfinding::calculate_path,
        turn_state::TurnState,
    },
    rendering::{viewport::Camera, widgets::BattlefieldWidget},
    systems::{
        action_execution::ActionExecutionSystem, ai_action_planner::AIActionPlannerSystem,
        path_execution::PathExecutionSystem, turn_manager::TurnManagerSystem,
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
        world.register::<Vision>();
        world.register::<PlannedPath>();
        world.register::<Weapon>();
        world.register::<Health>();
        world.register::<Dead>();

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

        // Add some trees (blocks LOS)
        for y in 30..35 {
            for x in 45..50 {
                battlefield.set_terrain(
                    BattlefieldPos::new(x, y),
                    TerrainType::Tree,
                );
            }
        }

        // Add a civilian building (blocks LOS)
        for y in 60..65 {
            for x in 50..55 {
                battlefield.set_terrain(
                    BattlefieldPos::new(x, y),
                    TerrainType::CivilianBuilding,
                );
            }
        }

        // Insert battlefield as a world resource for systems to access
        world.insert(battlefield.clone());

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
            .with(Vision::new(10))
            .with(Weapon::rifle())
            .with(Health::soldier())
            .build();

        // Allied NPC
        world
            .create_entity()
            .with(Position::new(55, 52))
            .with(Soldier {
                name: "Sgt. Jones".to_string(),
                faction: Faction::Allies,
                rank: Rank::Sergeant,
            })
            .with(TimeBudget::new(config.time_budget_seconds))
            .with(Vision::new(10))
            .with(Weapon::rifle())
            .with(Health::soldier())
            .build();

        // Enemy NPC 1
        world
            .create_entity()
            .with(Position::new(45, 48))
            .with(Soldier {
                name: "Pvt. Mueller".to_string(),
                faction: Faction::CentralPowers,
                rank: Rank::Private,
            })
            .with(TimeBudget::new(config.time_budget_seconds))
            .with(Vision::new(10))
            .with(Weapon::rifle())
            .with(Health::soldier())
            .build();

        // Enemy NPC 2 (for testing)
        world
            .create_entity()
            .with(Position::new(43, 50))
            .with(Soldier {
                name: "Cpl. Schmidt".to_string(),
                faction: Faction::CentralPowers,
                rank: Rank::Corporal,
            })
            .with(TimeBudget::new(config.time_budget_seconds))
            .with(Vision::new(10))
            .with(Weapon::rifle())
            .with(Health::soldier())
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
            InputMode::Targeting => self.handle_targeting_mode(key),
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
            KeyCode::Char(' ') => {
                // Space: Advance turn (PathExecutionSystem will handle path step execution)
                self.advance_turn();
            }
            KeyCode::Char('f') => {
                // Enter targeting mode for shooting
                self.input_mode = InputMode::Targeting;
                // Set cursor to player position
                if let Some(player_pos) = self.get_player_position() {
                    self.cursor_pos = player_pos;
                }
            }
            KeyCode::Char('r') => {
                // Reload weapon
                self.player_reload();
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
                // Calculate path from player to cursor position
                if let Some(player_pos) = self.get_player_position() {
                    if let Some(player_entity) = self.get_player_entity() {
                        let path = calculate_path(&player_pos, &self.cursor_pos, &self.battlefield);

                        if let Some(steps) = path {
                            // Calculate total estimated time cost
                            let total_cost: f32 = steps
                                .iter()
                                .map(|pos| {
                                    self.battlefield
                                        .get_tile(pos)
                                        .map(|t| 2.0 * t.terrain.movement_cost())
                                        .unwrap_or(2.0)
                                })
                                .sum();

                            // Insert PlannedPath component for player
                            let mut paths = self.world.write_storage::<PlannedPath>();
                            paths
                                .insert(
                                    player_entity,
                                    PlannedPath::new(steps, total_cost, true),
                                )
                                .ok();

                            self.world
                                .write_resource::<EventLog>()
                                .add(format!("Path planned ({:.1}s)", total_cost));
                        } else {
                            let mut log = self.world.write_resource::<EventLog>();
                            log.add("No path to destination!".to_string());
                        }
                    }
                }

                // Return to Command mode
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

    fn handle_targeting_mode(&mut self, key: KeyEvent) {
        use argue_the_toss::components::action::{ActionType, QueuedAction};
        use specs::{Join, WorldExt};

        match key.code {
            KeyCode::Esc => {
                // Cancel targeting and return to Command mode
                self.input_mode = InputMode::Command;
            }
            KeyCode::Enter => {
                // Find entity at cursor position and shoot at it
                let target_entity = {
                    let positions = self.world.read_storage::<Position>();
                    let soldiers = self.world.read_storage::<Soldier>();
                    let entities = self.world.entities();

                    (&entities, &positions, &soldiers)
                        .join()
                        .find(|(_, pos, _)| pos.x() == self.cursor_pos.x && pos.y() == self.cursor_pos.y)
                        .map(|(entity, _, _)| entity)
                };

                if let Some(target) = target_entity {
                    // Queue shoot action for player
                    if let Some(player_entity) = self.get_player_entity() {
                        let action = QueuedAction::new(ActionType::Shoot { target });
                        let mut actions = self.world.write_storage::<QueuedAction>();
                        actions.insert(player_entity, action).ok();

                        let mut log = self.world.write_resource::<EventLog>();
                        log.add("Shoot action queued.".to_string());
                    }
                } else {
                    let mut log = self.world.write_resource::<EventLog>();
                    log.add("No target at cursor position!".to_string());
                }

                // Return to Command mode
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
            // Movement keys - move cursor AND camera in Targeting mode
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

    fn player_reload(&mut self) {
        use argue_the_toss::components::action::{ActionType, QueuedAction};
        use specs::WorldExt;

        if let Some(player_entity) = self.get_player_entity() {
            let action = QueuedAction::new(ActionType::Reload);
            let mut actions = self.world.write_storage::<QueuedAction>();
            actions.insert(player_entity, action).ok();

            let mut log = self.world.write_resource::<EventLog>();
            log.add("Reload action queued.".to_string());
        }
    }

    fn advance_turn(&mut self) {
        use argue_the_toss::game_logic::turn_state::TurnState;
        use specs::WorldExt;

        let player_entity = match self.get_player_entity() {
            Some(e) => e,
            None => return,
        };

        // Mark player as ready to advance turn
        // PathExecutionSystem will automatically create action from PlannedPath (if exists)
        // Otherwise, this just ends the player's turn
        let mut turn_state = self.world.write_resource::<TurnState>();
        turn_state.mark_entity_ready(player_entity);
        drop(turn_state);

        // Check if player has a planned path
        let paths = self.world.read_storage::<PlannedPath>();
        let has_path = paths.get(player_entity).is_some();
        drop(paths);

        if has_path {
            self.world
                .write_resource::<EventLog>()
                .add("Advancing along path...".to_string());
        } else {
            self.world
                .write_resource::<EventLog>()
                .add("Waiting...".to_string());
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

        // Clear any existing planned path when manually moving
        {
            let mut paths = self.world.write_storage::<PlannedPath>();
            if paths.remove(player_entity).is_some() {
                self.world
                    .write_resource::<EventLog>()
                    .add("Planned path cancelled".to_string());
            }
        }

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
                TerrainType::Tree => "Tree",
                TerrainType::CivilianBuilding => "Civilian Building",
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
        use argue_the_toss::game_logic::line_of_sight::calculate_fov;
        use specs::Join;

        // Reset all visibility flags
        self.battlefield.reset_visibility();

        // Calculate FOV for player
        if let Some(player_pos) = self.get_player_position() {
            // Get player's vision range
            let vision_range = {
                let players = self.world.read_storage::<Player>();
                let visions = self.world.read_storage::<Vision>();
                let entities = self.world.entities();

                (&entities, &players, &visions)
                    .join()
                    .next()
                    .map(|(_, _, vision)| vision.range)
                    .unwrap_or(10) // Default 10 if no vision component
            };

            // Calculate visible tiles
            let visible_tiles = calculate_fov(&player_pos, vision_range, &self.battlefield);

            // Mark all visible tiles
            for pos in visible_tiles {
                self.battlefield.set_visible(pos, true);
            }
        }
    }
}

fn ui(f: &mut Frame, state: &GameState) {
    // Main layout: Top (battlefield + right pane) and Bottom (info panel)
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),      // Top: battlefield + right pane
            Constraint::Length(7),    // Bottom: info panel
        ])
        .split(f.area());

    // Top split: Battlefield (left), Event Log + Context Info (right)
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(75),  // Battlefield
            Constraint::Percentage(25),  // Right pane (event log + context)
        ])
        .split(main_chunks[0]);

    // Split right pane vertically into Event Log (top) and Context Info (bottom)
    let right_pane_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60),  // Event Log
            Constraint::Percentage(40),  // Context Info (cursor/target)
        ])
        .split(top_chunks[1]);

    // Render battlefield
    let battlefield_block = Block::default()
        .title("Battlefield")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    let inner_area = battlefield_block.inner(top_chunks[0]);
    f.render_widget(battlefield_block, top_chunks[0]);

    let battlefield_widget = BattlefieldWidget::new(&state.battlefield, &state.camera);
    f.render_widget(battlefield_widget, inner_area);

    // Render planned paths (before soldiers so they appear underneath)
    render_paths(f, inner_area, state);

    // Render soldiers on top
    render_soldiers(f, inner_area, state);

    // Render cursor in Look mode or Targeting mode
    if state.input_mode == InputMode::Look {
        render_cursor(f, inner_area, state);
    } else if state.input_mode == InputMode::Targeting {
        render_targeting_cursor(f, inner_area, state);
    }

    // Render event log (top of right pane)
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
    f.render_widget(event_paragraph, right_pane_chunks[0]);

    // Render context info (bottom of right pane)
    render_context_info(f, right_pane_chunks[1], state);

    // Render player info panel (bottom)
    let mode_color = match state.input_mode {
        InputMode::Command => Color::Green,
        InputMode::Look => Color::Yellow,
        InputMode::Targeting => Color::Red,
    };

    let info_block = Block::default()
        .title(format!("Mode: {} | Player Info", state.input_mode.name()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(mode_color));

    let mut info_lines = vec![
        Line::from(state.input_mode.help_text()),
        Line::from(""),
    ];

    // Show player info
    if let Some(player_entity) = state.get_player_entity() {
        let positions = state.world.read_storage::<Position>();
        let time_budgets = state.world.read_storage::<TimeBudget>();
        let weapons = state.world.read_storage::<Weapon>();
        let healths = state.world.read_storage::<Health>();
        let turn_state = state.world.fetch::<TurnState>();

        // Player position
        if let Some(pos) = positions.get(player_entity) {
            info_lines.push(Line::from(format!(
                "Position: ({}, {})",
                pos.x(), pos.y()
            )));
        }

        // Player HP with color coding
        if let Some(health) = healths.get(player_entity) {
            let hp_percent = health.percentage();
            let hp_color_name = if hp_percent > 0.7 {
                "GREEN"
            } else if hp_percent > 0.3 {
                "YELLOW"
            } else {
                "RED"
            };

            info_lines.push(Line::from(format!(
                "HP: {}/{} ({}%) [{}]",
                health.current,
                health.maximum,
                health.percentage_display(),
                hp_color_name
            )));
        }

        // Weapon info
        if let Some(weapon) = weapons.get(player_entity) {
            info_lines.push(Line::from(format!(
                "Weapon: {} | Ammo: {}/{} ({:.0}%)",
                weapon.stats.name,
                weapon.ammo.current,
                weapon.ammo.max_capacity,
                weapon.ammo.percentage()
            )));
        }

        // Time budget
        if let Some(budget) = time_budgets.get(player_entity) {
            let available = budget.available_time();
            let budget_status = if budget.time_debt < 0.0 {
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
                    turn_state.current_turn, available, budget_status, -budget.time_debt
                )
            } else {
                format!(
                    "Turn {} | Time: {:.1}s ({})",
                    turn_state.current_turn, available, budget_status
                )
            };

            info_lines.push(Line::from(time_info));
        }
    }

    let info_paragraph = Paragraph::new(Text::from(info_lines)).block(info_block);
    f.render_widget(info_paragraph, main_chunks[1]);
}

/// Render context-sensitive information (cursor/target details)
fn render_context_info(f: &mut Frame, area: Rect, state: &GameState) {
    use specs::{Join, WorldExt};

    let title = match state.input_mode {
        InputMode::Look => "Cursor Info",
        InputMode::Targeting => "Target Info",
        InputMode::Command => "Context",
    };

    let context_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let mut context_lines = vec![];

    // Get the position to inspect
    let inspect_pos = if state.input_mode == InputMode::Look || state.input_mode == InputMode::Targeting {
        state.cursor_pos
    } else {
        state.get_player_position().unwrap_or(state.cursor_pos)
    };

    // Show position
    context_lines.push(Line::from(format!(
        "Position: ({}, {})",
        inspect_pos.x, inspect_pos.y
    )));

    // Show terrain info
    context_lines.push(Line::from(format!(
        "Terrain: {}",
        state.get_terrain_info(&inspect_pos)
    )));

    context_lines.push(Line::from(""));

    // Show entity info at cursor/target position
    let positions = state.world.read_storage::<Position>();
    let soldiers = state.world.read_storage::<Soldier>();
    let healths = state.world.read_storage::<Health>();
    let weapons = state.world.read_storage::<Weapon>();
    let players = state.world.read_storage::<Player>();
    let entities = state.world.entities();

    // Find entity at inspect position
    let entity_at_pos = (&entities, &positions, &soldiers)
        .join()
        .find(|(_, pos, _)| pos.x() == inspect_pos.x && pos.y() == inspect_pos.y);

    if let Some((entity, _, soldier)) = entity_at_pos {
        let is_player = players.contains(entity);
        let player_marker = if is_player { " (YOU)" } else { "" };

        context_lines.push(Line::from(format!(
            "Unit: {}{}",
            soldier.name, player_marker
        )));
        context_lines.push(Line::from(format!(
            "Faction: {:?}",
            soldier.faction
        )));
        context_lines.push(Line::from(format!(
            "Rank: {}",
            soldier.rank.as_str()
        )));

        // Show HP if entity is visible (or if it's the player)
        if is_player {
            if let Some(health) = healths.get(entity) {
                let hp_percent = health.percentage();
                let hp_status = if hp_percent > 0.7 {
                    "Healthy"
                } else if hp_percent > 0.3 {
                    "Wounded"
                } else {
                    "Critical"
                };

                context_lines.push(Line::from(format!(
                    "HP: {}/{} ({}%)",
                    health.current,
                    health.maximum,
                    health.percentage_display()
                )));
                context_lines.push(Line::from(format!("Status: {}", hp_status)));
            }
        } else {
            // For other entities, check if tile is visible
            if let Some(tile) = state.battlefield.get_tile(&inspect_pos) {
                if tile.visible {
                    if let Some(health) = healths.get(entity) {
                        context_lines.push(Line::from(format!(
                            "HP: {}/{} ({}%)",
                            health.current,
                            health.maximum,
                            health.percentage_display()
                        )));
                    }
                } else {
                    context_lines.push(Line::from("HP: ???"));
                }
            }
        }

        // Show weapon info if visible or player
        if is_player {
            if let Some(weapon) = weapons.get(entity) {
                context_lines.push(Line::from(format!(
                    "Weapon: {}",
                    weapon.stats.name
                )));
            }
        } else if let Some(tile) = state.battlefield.get_tile(&inspect_pos) {
            if tile.visible {
                if let Some(weapon) = weapons.get(entity) {
                    context_lines.push(Line::from(format!(
                        "Weapon: {}",
                        weapon.stats.name
                    )));
                }
            }
        }
    } else {
        context_lines.push(Line::from("No entity here"));
    }

    // In Targeting mode, show additional targeting info
    if state.input_mode == InputMode::Targeting {
        context_lines.push(Line::from(""));
        context_lines.push(Line::from("--- Targeting ---"));

        let validation = validate_target(state);
        let status_msg = match validation {
            TargetValidation::Valid => "VALID TARGET (X)",
            TargetValidation::NoTarget => "No target (+)",
            TargetValidation::Friendly => "FRIENDLY (!)",
            TargetValidation::OutOfRange => "OUT OF RANGE (?)",
            TargetValidation::NoLineOfSight => "NO LINE OF SIGHT (/)",
        };
        context_lines.push(Line::from(format!("Status: {}", status_msg)));

        // Show weapon range
        if let Some(player_entity) = state.get_player_entity() {
            let weapons = state.world.read_storage::<Weapon>();
            if let Some(weapon) = weapons.get(player_entity) {
                context_lines.push(Line::from(format!(
                    "Range: {}/{} tiles",
                    weapon.stats.effective_range,
                    weapon.stats.max_range
                )));
            }
        }
    }

    let context_paragraph = Paragraph::new(Text::from(context_lines)).block(context_block);
    f.render_widget(context_paragraph, area);
}

fn render_paths(f: &mut Frame, area: Rect, state: &GameState) {
    let entities = state.world.entities();
    let paths = state.world.read_storage::<PlannedPath>();

    let top_left = state.camera.top_left();

    for (_entity, path) in (&entities, &paths).join() {
        // Only render paths with preview enabled
        if !path.show_preview {
            continue;
        }

        for (i, pos) in path.steps.iter().enumerate() {
            let screen_x = pos.x - top_left.x;
            let screen_y = pos.y - top_left.y;

            // Only render if within viewport
            if screen_x >= 0
                && screen_x < area.width as i32
                && screen_y >= 0
                && screen_y < area.height as i32
            {
                let buf_x = area.x + screen_x as u16;
                let buf_y = area.y + screen_y as u16;

                if buf_x < area.right() && buf_y < area.bottom() {
                    // Show numbered path (1-9, then +)
                    let ch = if i < 9 {
                        char::from_digit((i + 1) as u32, 10).unwrap()
                    } else {
                        '+'
                    };

                    f.buffer_mut()[(buf_x, buf_y)]
                        .set_char(ch)
                        .set_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray));
                }
            }
        }
    }
}

fn render_soldiers(f: &mut Frame, area: Rect, state: &GameState) {
    let entities = state.world.entities();
    let positions = state.world.read_storage::<Position>();
    let soldiers = state.world.read_storage::<Soldier>();
    let players = state.world.read_storage::<Player>();
    let dead_markers = state.world.read_storage::<Dead>();

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
                // Check if entity is dead
                let is_dead = dead_markers.contains(entity);

                let ch = if is_dead {
                    'X' // Dead bodies shown as X
                } else {
                    soldier.faction.to_char()
                };

                // Color based on status
                let color = if is_dead {
                    Color::DarkGray // Dead entities are dark gray
                } else if players.contains(entity) {
                    Color::LightGreen // Player is bright green
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

/// Validation result for targeting
enum TargetValidation {
    Valid,          // Enemy in range with LOS
    NoTarget,       // No entity at cursor
    Friendly,       // Friendly/self at cursor
    OutOfRange,     // Target exists but out of weapon range
    NoLineOfSight,  // Target exists but no LOS
}

/// Check if the cursor position is a valid target for shooting
fn validate_target(state: &GameState) -> TargetValidation {
    use argue_the_toss::game_logic::line_of_sight::calculate_fov;
    use specs::{Join, WorldExt};

    // Get player info
    let player_entity = match state.get_player_entity() {
        Some(e) => e,
        None => return TargetValidation::NoTarget,
    };

    let positions = state.world.read_storage::<Position>();
    let soldiers = state.world.read_storage::<Soldier>();
    let weapons = state.world.read_storage::<Weapon>();
    let visions = state.world.read_storage::<Vision>();
    let entities = state.world.entities();

    // Get player position, weapon, and vision
    let player_pos = match positions.get(player_entity) {
        Some(p) => p,
        None => return TargetValidation::NoTarget,
    };

    let player_weapon = match weapons.get(player_entity) {
        Some(w) => w,
        None => return TargetValidation::NoTarget,
    };

    let player_faction = match soldiers.get(player_entity) {
        Some(s) => s.faction,
        None => return TargetValidation::NoTarget,
    };

    let player_vision = visions.get(player_entity)
        .map(|v| v.range)
        .unwrap_or(10);

    // Check if there's an entity at cursor position
    let target_at_cursor = (&entities, &positions, &soldiers)
        .join()
        .find(|(_, pos, _)| pos.x() == state.cursor_pos.x && pos.y() == state.cursor_pos.y);

    let (target_entity, target_pos, target_soldier) = match target_at_cursor {
        Some((e, p, s)) => (e, p, s),
        None => return TargetValidation::NoTarget,
    };

    // Don't allow shooting self or friendlies
    if target_entity == player_entity || target_soldier.faction == player_faction {
        return TargetValidation::Friendly;
    }

    // Calculate distance
    let dx = (player_pos.x() - target_pos.x()) as f32;
    let dy = (player_pos.y() - target_pos.y()) as f32;
    let distance = (dx * dx + dy * dy).sqrt().ceil() as i32;

    // Check range
    if distance > player_weapon.stats.max_range {
        return TargetValidation::OutOfRange;
    }

    // Check line of sight using FOV calculation
    let player_battlefield_pos = BattlefieldPos::new(player_pos.x(), player_pos.y());
    let visible_tiles = calculate_fov(&player_battlefield_pos, player_vision, &state.battlefield);
    let target_battlefield_pos = BattlefieldPos::new(target_pos.x(), target_pos.y());

    if !visible_tiles.contains(&target_battlefield_pos) {
        return TargetValidation::NoLineOfSight;
    }

    TargetValidation::Valid
}

fn render_targeting_cursor(f: &mut Frame, area: Rect, state: &GameState) {
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
            // Validate the target and choose style accordingly
            let validation = validate_target(state);

            let (cursor_char, cursor_style) = match validation {
                TargetValidation::Valid => {
                    // Valid target: bright red background with crosshair
                    ('X', Style::default().fg(Color::White).bg(Color::Red))
                }
                TargetValidation::NoTarget => {
                    // No target: dim red background
                    ('+', Style::default().fg(Color::White).bg(Color::DarkGray))
                }
                TargetValidation::Friendly => {
                    // Friendly: yellow/amber warning
                    ('!', Style::default().fg(Color::Black).bg(Color::Yellow))
                }
                TargetValidation::OutOfRange => {
                    // Out of range: orange/amber
                    ('?', Style::default().fg(Color::White).bg(Color::Rgb(255, 140, 0)))
                }
                TargetValidation::NoLineOfSight => {
                    // No LOS: magenta/purple
                    ('/', Style::default().fg(Color::White).bg(Color::Magenta))
                }
            };

            // Set both character and style for clear visual feedback
            f.buffer_mut()[(buf_x, buf_y)]
                .set_char(cursor_char)
                .set_style(cursor_style);
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
        .with(PathExecutionSystem, "path_execution", &[])
        .with(AIActionPlannerSystem, "ai_planner", &["path_execution"])
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
