# Time Budget System Implementation Plan

## Overview
Implement a turn-based time budget system where actions cost time (in seconds) and players have a configurable time budget per turn (5-30 seconds). Actions commit immediately on input and can carry time debt to future turns.

## Design Decisions (from user)
- **Action Commitment:** Immediate per-action (commits when player presses key)
- **Budget Overflow:** Actions carry to next turn with time debt
- **Multi-turn Actions:** Cannot be cancelled once committed
- **Turn Order:** Start with player-first, architect for future experimentation (simultaneous, initiative-based)
- **Timescale Range:** 5-30 seconds per turn (user configurable)
- **Entity Time Budgets:** Every entity (player, allies, enemies) has individual time budgets and acts independently

## Architecture Overview

### Phase 1: Core Data Structures

#### 1.1 Action System (`src/components/action.rs`)
```rust
// Action types with time costs
pub enum ActionType {
    Move { dx: i32, dy: i32, terrain_cost: f32 },
    Shoot { target: Entity },
    Reload,
    ThrowGrenade { target_pos: Position },
    Wait,
}

impl ActionType {
    pub fn base_time_cost(&self) -> f32 {
        match self {
            ActionType::Move { terrain_cost, .. } => 2.0 * terrain_cost,
            ActionType::Shoot { .. } => 3.0,
            ActionType::Reload => 5.0,
            ActionType::ThrowGrenade { .. } => 4.0,
            ActionType::Wait => 1.0,
        }
    }
}

// Component: queued action on an entity
pub struct QueuedAction {
    pub action_type: ActionType,
    pub time_cost: f32,
    pub committed: bool,
}

// Component: ongoing multi-turn action
pub struct OngoingAction {
    pub action_type: ActionType,
    pub total_time: f32,
    pub time_completed: f32,
    pub locked: bool,  // Cannot cancel
}
```

Register components:
- `QueuedAction` (VecStorage)
- `OngoingAction` (VecStorage)

#### 1.2 Time Budget Tracking (`src/components/time_budget.rs`)
```rust
// Component: per-entity time tracking
pub struct TimeBudget {
    pub base_duration: f32,      // Configurable 5-30 seconds
    pub time_debt: f32,           // Negative = owe time, positive = extra time
    pub time_spent_this_turn: f32,
}

impl TimeBudget {
    pub fn available_time(&self) -> f32 {
        self.base_duration - self.time_debt
    }

    pub fn consume_time(&mut self, cost: f32) -> bool {
        self.time_spent_this_turn += cost;
        self.time_debt += cost - self.base_duration;
        true  // Always succeeds, may create debt
    }

    pub fn can_afford(&self, cost: f32) -> bool {
        // For UI display purposes
        self.available_time() >= cost
    }

    pub fn reset_for_new_turn(&mut self) {
        self.time_spent_this_turn = 0.0;
        // Keep time_debt to carry forward
    }
}
```

Register component: `TimeBudget` (VecStorage)

#### 1.3 Turn State Management (`src/game_logic/turn_state.rs`)
```rust
// Resource: global turn state
pub struct TurnState {
    pub current_turn: u32,
    pub phase: TurnPhase,
    pub turn_order_mode: TurnOrderMode,
    pub entities_ready: HashSet<Entity>,  // Entities that have finished their turn
}

pub enum TurnPhase {
    Planning,          // Entities planning/committing actions
    Execution,         // All committed actions resolve
    Resolution,        // Post-execution cleanup, damage application, etc.
}

pub enum TurnOrderMode {
    PlayerFirst,       // Player-controlled entity acts first, then all NPCs (start with this)
    Simultaneous,      // All entities act, resolve together (future)
    InitiativeBased,   // Speed stat determines action order (future)
}

impl TurnState {
    pub fn is_entity_ready(&self, entity: Entity) -> bool {
        self.entities_ready.contains(&entity)
    }

    pub fn mark_entity_ready(&mut self, entity: Entity) {
        self.entities_ready.insert(entity);
    }

    pub fn reset_for_new_turn(&mut self) {
        self.current_turn += 1;
        self.entities_ready.clear();
        self.phase = TurnPhase::Planning;
    }
}
```

Register as Specs resource with `world.insert(TurnState::new())`

**Note on Turn Order Modes:**
- **PlayerFirst:** Player entity plans/commits first. Once player's budget is exhausted or turn submitted, AI takes over for all NPCs (allies and enemies). All actions then execute together.
- **Simultaneous:** All entities (including player) act within their time budgets simultaneously. Turn ends when all entities have exhausted budgets or submitted.
- **InitiativeBased:** Actions execute in initiative order. Each entity acts when their initiative comes up, spending from their time budget.

#### 1.4 Configuration (`src/config/game_config.rs`)
```rust
pub struct GameConfig {
    pub time_budget_seconds: f32,  // Default: 10.0, range 5.0-30.0
    pub turn_order_mode: TurnOrderMode,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            time_budget_seconds: 10.0,
            turn_order_mode: TurnOrderMode::PlayerFirst,
        }
    }
}
```

Add config to GameState in main.rs

### Phase 2: Input Processing Changes

#### 2.1 Modify `handle_command_mode()` in `main.rs`
Current: Calls `move_player()` immediately
New: Creates and commits action immediately

```rust
fn handle_command_mode(&mut self, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') => self.running = false,
        KeyCode::Char('x') => self.input_mode = InputMode::Look,
        KeyCode::Char('c') => self.center_camera_on_player(),

        // Movement keys now commit actions instead of moving directly
        KeyCode::Char('h') | KeyCode::Left => self.commit_player_action(ActionType::Move { dx: -1, dy: 0 }),
        KeyCode::Char('j') | KeyCode::Down => self.commit_player_action(ActionType::Move { dx: 0, dy: 1 }),
        KeyCode::Char('k') | KeyCode::Up => self.commit_player_action(ActionType::Move { dx: 0, dy: -1 }),
        KeyCode::Char('l') | KeyCode::Right => self.commit_player_action(ActionType::Move { dx: 1, dy: 1 }),

        // Future: other action keys
        KeyCode::Char('f') => self.enter_targeting_mode(ActionType::Shoot),
        KeyCode::Char('r') => self.commit_player_action(ActionType::Reload),
        KeyCode::Char('.') => self.commit_player_action(ActionType::Wait),

        _ => {}
    }
}

fn commit_player_action(&mut self, action_type: ActionType) {
    // Get player entity
    let player_entity = self.get_player_entity();

    // Calculate time cost (factor in terrain for movement)
    let time_cost = action_type.base_time_cost();

    // Commit action (immediate commitment as per user choice)
    let mut time_budgets = self.world.write_storage::<TimeBudget>();
    let mut queued_actions = self.world.write_storage::<QueuedAction>();

    if let Some(budget) = time_budgets.get_mut(player_entity) {
        budget.consume_time(time_cost);

        queued_actions.insert(player_entity, QueuedAction {
            action_type,
            time_cost,
            committed: true,
        }).ok();

        // Check if turn should end (budget exhausted or in debt)
        if budget.available_time() <= 0.0 {
            // Mark player as ready (finished their turn)
            let mut turn_state = self.world.write_resource::<TurnState>();
            turn_state.mark_entity_ready(player_entity);

            self.event_log.add_message("Time budget exhausted. Waiting for others...".to_string());
        }
    }
}
```

#### 2.2 Remove `move_player()`
The old immediate movement function becomes obsolete - movement now goes through action system.

### Phase 3: ECS Systems

#### 3.1 AI Action Planning System (`src/systems/ai_action_planner.rs`)
```rust
// NEW: System to generate actions for NPC entities
pub struct AIActionPlannerSystem;

impl<'a> System<'a> for AIActionPlannerSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Soldier>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, TimeBudget>,
        WriteStorage<'a, QueuedAction>,
        Read<'a, TurnState>,
    );

    fn run(&mut self, (entities, positions, soldiers, players, mut budgets, mut queued, turn_state): Self::SystemData) {
        // Only plan during Planning phase
        if !matches!(turn_state.phase, TurnPhase::Planning) {
            return;
        }

        // For PlayerFirst mode: only plan for NPCs after player is ready
        if matches!(turn_state.turn_order_mode, TurnOrderMode::PlayerFirst) {
            // Check if player entity is ready
            let player_ready = (&entities, &players).join()
                .any(|(e, _)| turn_state.is_entity_ready(e));

            if !player_ready {
                return;  // Wait for player to finish
            }
        }

        // Plan actions for all NPCs (non-player entities with time budgets)
        for (entity, pos, _soldier, budget) in (&entities, &positions, &soldiers, &mut budgets).join() {
            // Skip if this is the player
            if players.get(entity).is_some() {
                continue;
            }

            // Skip if entity already has action queued or is out of time
            if queued.get(entity).is_some() || budget.available_time() <= 0.0 {
                continue;
            }

            // Simple AI: just wait for now (future: pathfinding, combat decisions)
            let action = ActionType::Wait;
            let time_cost = action.base_time_cost();

            budget.consume_time(time_cost);
            queued.insert(entity, QueuedAction {
                action_type: action,
                time_cost,
                committed: true,
            }).ok();
        }
    }
}
```

**Note:** This is a placeholder AI system. Future iterations will implement:
- Pathfinding-based movement decisions
- Target selection for shooting
- Tactical decision-making based on morale, cover, etc.

#### 3.2 Action Execution System (`src/systems/action_execution.rs`)
```rust
pub struct ActionExecutionSystem;

impl<'a> System<'a> for ActionExecutionSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, QueuedAction>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, OngoingAction>,
        Write<'a, EventLog>,
        Read<'a, TurnState>,
    );

    fn run(&mut self, (entities, queued, mut positions, mut ongoing, mut log, turn_state): Self::SystemData) {
        // Only execute during Execution phase
        if !matches!(turn_state.phase, TurnPhase::Execution) {
            return;
        }

        // Execute ALL committed actions (player, allies, enemies)
        for (entity, action, pos) in (&entities, &queued, &mut positions).join() {
            if !action.committed {
                continue;
            }

            match &action.action_type {
                ActionType::Move { dx, dy, .. } => {
                    let new_x = pos.x() + dx;
                    let new_y = pos.y() + dy;
                    // Boundary check (from battlefield size)
                    if new_x >= 0 && new_x < 100 && new_y >= 0 && new_y < 100 {
                        *pos = Position::new(new_x, new_y);
                        log.add_message(format!("Entity {:?} moved to ({}, {})", entity, new_x, new_y));
                    }
                },
                ActionType::Wait => {
                    // Waiting is a no-op execution
                },
                // Other action types handled here
                _ => {}
            }
        }

        // Clear executed actions after processing
        // (done by cleanup system in turn manager)
    }
}
```

#### 3.3 Turn Management System (`src/systems/turn_manager.rs`)
```rust
pub struct TurnManagerSystem;

impl<'a> System<'a> for TurnManagerSystem {
    type SystemData = (
        Entities<'a>,
        Write<'a, TurnState>,
        WriteStorage<'a, TimeBudget>,
        WriteStorage<'a, QueuedAction>,
        ReadStorage<'a, Player>,
        Write<'a, EventLog>,
    );

    fn run(&mut self, (entities, mut turn_state, mut budgets, mut actions, players, mut log): Self::SystemData) {
        match turn_state.phase {
            TurnPhase::Planning => {
                // Check if all entities are ready to execute
                let all_ready = match turn_state.turn_order_mode {
                    TurnOrderMode::PlayerFirst => {
                        // In PlayerFirst: check if player is ready, then if all NPCs are ready
                        let player_ready = (&entities, &players).join()
                            .any(|(e, _)| turn_state.is_entity_ready(e));

                        if !player_ready {
                            return;  // Wait for player
                        }

                        // Player ready, check if all NPCs have actions or are out of budget
                        (&entities, &budgets).join()
                            .filter(|(e, _)| players.get(*e).is_none())  // NPCs only
                            .all(|(e, budget)| {
                                actions.get(e).is_some() || budget.available_time() <= 0.0
                            })
                    },
                    TurnOrderMode::Simultaneous => {
                        // All entities must be ready
                        (&entities, &budgets).join()
                            .all(|(e, budget)| {
                                turn_state.is_entity_ready(e) || budget.available_time() <= 0.0
                            })
                    },
                    _ => false,  // Other modes not implemented yet
                };

                if all_ready {
                    turn_state.phase = TurnPhase::Execution;
                    log.add_message("=== Executing Turn ===".to_string());
                }
            },

            TurnPhase::Execution => {
                // Actions have been executed, transition to Resolution
                turn_state.phase = TurnPhase::Resolution;
            },

            TurnPhase::Resolution => {
                // Clear executed actions
                actions.clear();

                // Reset time budgets for new turn (keep debt)
                for budget in (&mut budgets).join() {
                    budget.reset_for_new_turn();
                }

                // Start new turn
                turn_state.reset_for_new_turn();
                log.add_message(format!("=== Turn {} ===", turn_state.current_turn));
            },
        }
    }
}
```

### Phase 4: UI Updates

#### 4.1 Time Budget Display (`src/ui/widgets.rs` or new file)
Add widget to show:
- Current turn number
- Time budget remaining (colored: green > 3s, yellow 1-3s, red < 1s, purple if debt)
- Time debt indicator if negative
- Current committed action (if any)

```rust
pub fn render_time_budget(f: &mut Frame, area: Rect, time_budget: &TimeBudget, turn: u32) {
    let available = time_budget.available_time();
    let color = if time_budget.time_debt < 0.0 {
        Color::Magenta  // In debt
    } else if available > 3.0 {
        Color::Green
    } else if available > 1.0 {
        Color::Yellow
    } else {
        Color::Red
    };

    let text = if time_budget.time_debt < 0.0 {
        format!("Turn {} | Time: {:.1}s (debt: {:.1}s)", turn, available, -time_budget.time_debt)
    } else {
        format!("Turn {} | Time: {:.1}s", turn, available)
    };

    let block = Block::default()
        .title("Time Budget")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color));

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(color))
        .block(block);

    f.render_widget(paragraph, area);
}
```

Add to main UI layout in `ui()` function.

#### 4.2 Action Feedback in Event Log
When action commits, log it:
- "Movement queued (2.0s)"
- "Reload committed (5.0s) - will complete next turn"
- "Time debt: -2.5s - action continues..."

### Phase 5: Game Loop Integration

#### 5.1 Modified Game Loop (`main.rs`)
```rust
// Initialize turn state
world.insert(TurnState::new());

// Dispatcher for turn processing
let mut dispatcher = DispatcherBuilder::new()
    .with(AIActionPlannerSystem, "ai_planner", &[])
    .with(ActionExecutionSystem, "action_execution", &[])
    .with(TurnManagerSystem, "turn_manager", &["action_execution"])
    .build();

// Main loop
while game_state.running {
    // Update visibility
    game_state.update_visibility();

    // Render
    terminal.draw(|f| {
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
            // Only accept player input during Planning phase
            let can_input = matches!(turn_state.phase, TurnPhase::Planning);

            // In PlayerFirst mode, only accept input if player hasn't finished
            let player_can_act = if matches!(turn_state.turn_order_mode, TurnOrderMode::PlayerFirst) {
                let player_entity = game_state.get_player_entity();
                !turn_state.is_entity_ready(player_entity)
            } else {
                true  // In other modes, player can always act during Planning
            };

            drop(turn_state);  // Release borrow

            if can_input && player_can_act {
                game_state.handle_input(key);
            }
        }
    }
}
```

### Phase 6: Initialization

#### 6.1 Entity Initialization

**Player Initialization:**
When creating the player entity, add TimeBudget component:

```rust
world.create_entity()
    .with(Position::new(50, 50))
    .with(Soldier::new(Faction::Allies, Rank::Private))
    .with(Player)
    .with(TimeBudget {
        base_duration: config.time_budget_seconds,
        time_debt: 0.0,
        time_spent_this_turn: 0.0,
    })
    .build();
```

**NPC Initialization:**
ALL NPC entities (allies and enemies) must also have TimeBudget components:

```rust
// Example: Creating allied NPC
world.create_entity()
    .with(Position::new(x, y))
    .with(Soldier::new(Faction::Allies, Rank::Private))
    .with(TimeBudget {
        base_duration: config.time_budget_seconds,
        time_debt: 0.0,
        time_spent_this_turn: 0.0,
    })
    .build();

// Example: Creating enemy NPC
world.create_entity()
    .with(Position::new(x, y))
    .with(Soldier::new(Faction::CentralPowers, Rank::Private))
    .with(TimeBudget {
        base_duration: config.time_budget_seconds,
        time_debt: 0.0,
        time_spent_this_turn: 0.0,
    })
    .build();
```

**Note:** Currently in main.rs, only the player and some test entities are created. You'll need to ensure all spawned NPCs (current and future) receive TimeBudget components.

## Implementation Order

1. **Create data structures** (Phase 1)
   - `components/action.rs` - Action types and components
   - `components/time_budget.rs` - Time tracking
   - `game_logic/turn_state.rs` - Turn phases
   - `config/game_config.rs` - Configuration

2. **Register components and resources** (Phase 6.1)
   - Register Action, TimeBudget components
   - Insert TurnState resource
   - Add TimeBudget to player entity creation

3. **Create ECS systems** (Phase 3)
   - `systems/ai_action_planner.rs` - AI planning for NPCs
   - `systems/action_execution.rs` - Execute actions
   - `systems/turn_manager.rs` - Manage turn flow
   - Create dispatcher in main.rs

4. **Update input handling** (Phase 2)
   - Modify `handle_command_mode()` to commit actions
   - Remove old `move_player()` function
   - Add action commitment logic

5. **Update UI** (Phase 4)
   - Add time budget widget
   - Update event log for action feedback
   - Modify layout to include time display

6. **Integrate game loop** (Phase 5)
   - Add dispatcher to main loop
   - Gate input on turn phase
   - Test turn flow

## Critical Files

### New Files to Create
- `src/components/action.rs` - Action types and components (~100 lines)
- `src/components/time_budget.rs` - Time tracking (~50 lines)
- `src/game_logic/turn_state.rs` - Turn management (~120 lines)
- `src/systems/ai_action_planner.rs` - AI action planning (~100 lines)
- `src/systems/action_execution.rs` - Action processor (~150 lines)
- `src/systems/turn_manager.rs` - Turn flow (~120 lines)
- `src/config/game_config.rs` - Configuration (~30 lines)

### Files to Modify
- `src/main.rs` - Input handling, game loop, initialization (~50 lines changed)
- `src/ui/mod.rs` - Add time budget display (~80 lines added)
- `src/components/mod.rs` - Register new components (~3 lines)
- `src/systems/mod.rs` - Register new systems (~2 lines)
- `src/game_logic/mod.rs` - Add turn_state module (~1 line)

## Testing Strategy

1. **Unit tests** for time budget math (debt, overflow, reset)
2. **Manual testing** of immediate action commitment
3. **Test turn flow**: Planning → Execution → Enemy → New turn
4. **Test time debt**: Action overflow into next turn
5. **Test UI display**: Budget colors, debt indicator
6. **Test edge cases**: Exactly 0 time, multiple debt turns

## Future Extensions (Mentioned for Architecture)

- **Turn order modes**: Add Simultaneous and InitiativeBased implementations
- **Action preview**: Show projected time cost before commit
- **Smarter AI**: Replace Wait action with pathfinding, combat, tactical decisions
- **Time scaling**: Allow runtime adjustment of timescale
- **Per-entity time budget configs**: Different budgets for different unit types/ranks
- **Action visualization**: Show who's acting, movement trails, etc.

## Notes

- **All entities** (player, allies, enemies) have individual time budgets and act independently
- System is designed to be flexible for future turn order experimentation
- Immediate commitment means no "undo" - keep this in mind for UX
- Time debt can theoretically accumulate indefinitely - may need cap
- Camera should follow player during execution phase, not just planning
- Event log should clearly distinguish planning vs execution messages
- AI system starts simple (Wait action) - pathfinding and combat AI come later
- In PlayerFirst mode: player acts first, then all NPCs plan simultaneously before execution
- Turn ends when all entities have either committed actions or exhausted budgets
