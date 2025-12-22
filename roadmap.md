# Argue the Toss - Development Roadmap

## Project Status: Phase 2 In Progress

### Current Phase: Phase 0 - Project Setup
**Status:** Completed

#### Completed
- [x] Initial project concept and design documentation
- [x] Technology stack selection (ratatui, bracket-lib, Specs ECS)
- [x] Feature set definition
- [x] Development phase planning
- [x] Project conventions established
- [x] Documentation structure created
- [x] Project initialization (cargo init)
- [x] Development environment setup and verification
- [x] Initial Cargo.toml configuration with dependencies
- [x] Basic project structure creation (modular organization)

---

## Development Phases

### Phase 1: Core Foundation
**Target:** Basic rendering and viewport system
**Status:** Completed

#### Completed Tasks
- [x] Basic ratatui viewport rendering
- [x] Simple unit representation (ASCII characters)
- [x] Camera/viewport system for panning
- [x] Basic fog of war implementation
- [x] Input handling system

#### Success Criteria - All Met ✓
- ✓ Viewport displays grid-based battlefield (100x100 grid with 60x40 viewport)
- ✓ Camera can pan across larger logical battlefield (arrow keys / hjkl)
- ✓ Units render as distinct characters (@ for Allies, Ӝ for Central Powers)
- ✓ Basic fog of war obscures unseen areas (visible/explored/unexplored states)

#### Implementation Details
- Created `battlefield.rs` with grid structure, terrain types, and fog of war
- Created `viewport.rs` with Camera system for panning and viewport management
- Created `widgets.rs` with BattlefieldWidget for ratatui rendering
- Created ECS components: Position and Soldier
- Integrated Specs ECS for entity management
- Full game loop with input handling in main.rs

---

### Phase 2: Essential Mechanics
**Target:** Core gameplay loop
**Status:** In Progress

#### Completed Tasks
- [x] Modal UI system (Command mode vs Look/Targeting mode)
- [x] Basic player movement system (boundary-checked, camera following)
- [x] Adaptable battlefield viewport (dynamic sizing, terminal resize support)
- [x] Deadzone camera system (33% adaptive ratio, smooth following)
- [x] Targeting cursor foundation (Look mode with free cursor)
- [x] Player highlighting (bright green for easy identification)
- [x] Time budget system (seconds per turn instead of abstract AP)
- [x] Multi-turn action tracking (actions that span multiple turns)
- [x] Action commitment system (locked-in actions, partial time tracking)
- [x] Turn-based game loop
- [x] Turn timescale determination (how much real-time per turn)
- [x] Basic UI panels (event log, stats)
- [x] Line-of-sight calculations (bracket-lib FOV with symmetric shadowcasting)
- [x] Vision stat system (per-entity vision range)
- [x] Terrain-based LOS blocking (Fortifications, Trees, Buildings)
- [x] Fog of war (visible/explored/unexplored states)
- [x] Pathfinding system (A* with terrain costs, visual path preview)
- [x] Player path planning (Look mode + Enter to select destination)
- [x] AI pathfinding (NPCs move toward player using pathfinding)
- [x] Manual movement override (hjkl cancels planned paths)
- [x] Automatic path execution (Space key advances turn and executes planned path steps)
- [x] Basic combat (hitscan weapons)
- [x] Ammunition mechanic (reloading, running out of ammo)
- [x] 8-direction movement system (qweasdzxc keyboard layout)
- [x] Player vision cone system (CDCA-style directional FOV)
- [x] Terrain dimming outside vision cone (peripheral vision)

#### In Progress Tasks
- [ ] Action/event subdivision for animation support (sub-turn phases)
- [ ] Targeting system completion (object pickup, enemy selection actions)
- [ ] Visual indication of entity actions (movement trails, firing indicators, grenade throws)
- [ ] Last-seen entity markers (static ghosts of last known positions)
- [ ] FOW mode options (no FOW, friendly vision, player-only vision)

#### Implementation Details - Pathfinding System
- Created `pathfinding.rs` with A* implementation using bracket-pathfinding
- Created `PlannedPath` component to store multi-step paths
- Created `PathExecutionSystem` to convert path steps into Move actions
- Implemented terrain-aware cost calculation (Trench 1.0x, Mud 2.0x, etc.)
- Added diagonal movement penalty (1.414x for diagonal steps)
- Integrated path visualization with numbered step display (1-9, then +)
- **Critical Bug Fix:** Added `get_pathing_distance()` heuristic implementation
  - Without proper heuristic, A* degraded to exhaustive search
  - Caused 245-step paths for 7-tile moves (touring entire map)
  - Fix: Implemented Euclidean distance heuristic for optimal pathfinding
- AI integration: NPCs pathfind toward player instead of waiting
- Manual override: hjkl movement cancels planned paths
- Space key turn advancement: Triggers PathExecutionSystem to execute next path step
- Time budget integration: PathExecutionSystem properly consumes time when creating actions

#### Implementation Details - Vision Cone System
- Created `facing.rs` component with Direction8 enum (N, NE, E, SE, S, SW, W, NW)
- Created `vision_cone.rs` with directional FOV calculation
- Implemented 8-direction movement with qweasdzxc keyboard layout
- 120° main vision cone (±60° from facing direction)
- 60° peripheral vision on each side (dimmed to 50% brightness)
- 180° rear blind spot (explored tiles only)
- Auto-facing on movement (facing updates to match movement direction)
- Manual rotation with , (CCW) and . (CW) keys (0.3s cost)
- Time scale adjustments for trench warfare feel (~2m per tile):
  - Movement: 1.5s per tile (was 2.0s)
  - Rotation: 0.3s per 45° (was 0.5s)
  - Turn budget: 12s default (was 10s)
- Vision cone respects LOS and terrain blocking
- Peripheral vision rendered with dimmed colors (gray tint)
- Dead entities cannot move or take actions (bug fix)

#### Success Criteria
- ✓ Viewport adapts to terminal size
- ✓ Modal UI with Command and Look modes
- ✓ Basic player movement with camera following
- ✓ Targeting cursor foundation (free cursor in Look mode)
- ✓ Time budget system limits actions (12 sec/turn budget default, configurable)
- ✓ Actions have time costs (move: 1.5s × terrain, shoot: 3s, reload: 5s, rotate: 0.3s)
- ✓ Multi-turn actions tracked correctly (partial completion, locked-in state)
- ✓ Over-budget actions handle gracefully (time debt carries to next turn)
- ✓ Turn timescale defined (seconds per turn) and consistent
- ✓ Time remaining displayed clearly in UI
- ✓ Event log displays action feedback and turn transitions
- ✓ Line-of-sight calculated with symmetric shadowcasting
- ✓ Terrain blocks LOS (Fortifications, Trees, Buildings)
- ✓ Vision stat per entity (allows progression)
- ✓ Fog of war with visible/explored/unexplored states
- ✓ Pathfinding calculates optimal terrain-aware paths
- ✓ Path preview shows numbered steps visually
- ✓ Player can plan paths via Look mode cursor
- ✓ AI uses pathfinding to move toward player
- ✓ Manual movement (hjkl) overrides planned paths
- ✓ Automatic path execution (Space advances turn, path step executes automatically)
- ✓ Combat resolves with line-of-sight checks
- ✓ Weapons require reloading and can run out of ammunition
- ✓ 8-direction movement system (qweasdzxc layout)
- ✓ Vision cone emanates from player with directional awareness (120° main, 60° peripheral)
- ✓ Terrain outside vision cone is dimmed (peripheral vision at 50% brightness)
- ✓ Manual facing control (rotation keys for tactical positioning)
- ✓ Dead entities cannot move or act
- Actions subdivided into phases for smooth animation
- Committed actions shown with progress indicators
- Targeting cursor allows object/enemy selection actions
- Prior turn actions are visually indicated (who moved, fired, threw grenades)
- Last-seen enemy positions marked with static indicators
- Multiple FOW modes available (configurable)

---

### Phase 3: Environmental Systems
**Target:** Procedural battlefield generation, terrain types, and weather mechanics
**Status:** Not Started

#### Planned Tasks
- [ ] Procedural battlefield generation system
  - [ ] Intelligent trench network placement (front lines, support trenches, communication trenches)
  - [ ] Town/village placement with building clusters
  - [ ] Vegetation distribution (forests, hedgerows, individual trees)
  - [ ] No-man's-land generation (shell craters, wire, debris)
  - [ ] Spawn point placement logic (faction-specific starting positions)
- [ ] Barbed wire obstacles (movement impediment, vision blocking)
- [ ] Static emplacements
  - [ ] Machine gun nests (defensive positions)
  - [ ] Mortar pits
  - [ ] Artillery positions
  - [ ] Observation posts
- [ ] Terrain types (mud, trenches, no-man's-land)
- [ ] Additional terrain types (vegetation, tree, concrete, water)
- [ ] Color coding for all terrain types
- [ ] Buildings (multi-tile structures spanning multiple positions)
- [ ] Movement cost based on terrain
- [ ] Cover mechanics and calculations
- [ ] Weather effects (rain, fog, snow)
- [ ] Lighting system (day/night, flares)
- [ ] Z-level foundation (trenches, elevated positions)

#### Success Criteria
- Procedural generation creates realistic WWI battlefields
- Trench networks have logical layout (front/support/communication trenches)
- Towns and vegetation placed with strategic considerations
- Spawn points positioned appropriately for each faction
- Barbed wire creates tactical obstacles
- Static emplacements provide defensive positions
- Different terrain affects movement speed
- All terrain types have distinct colors and characters
- Buildings render as multi-tile structures
- Cover provides combat bonuses
- Weather affects visibility
- Lighting changes visibility ranges

---

### Phase 4: Advanced Combat
**Target:** Complex weapon systems
**Status:** Not Started

#### Planned Tasks
- [ ] Physics-based projectiles (mortars, grenades)
- [ ] Artillery system (off-map support)
- [ ] Smoke effects
- [ ] Gas warfare mechanics
- [ ] Fire mechanics
- [ ] Projectile animations

#### Success Criteria
- Artillery strikes designated areas
- Grenades follow arc trajectories
- Environmental hazards affect units
- Visual feedback for projectile motion

---

### Phase 5: Simulation Depth
**Target:** AI and character systems
**Status:** Not Started

#### Planned Tasks
- [ ] Morale system (individual and unit)
- [ ] AI opponents (individual soldier behavior)
- [ ] AI allies (squad coordination)
- [ ] Squad formations and tactics
- [ ] Character stats and progression
- [ ] Debuff system (wounds, shell shock, disease)
- [ ] Rank and promotion system
- [ ] Medic units with healing abilities
- [ ] Simulated inventories for all NPCs (weapons, ammo, items)

#### Success Criteria
- AI soldiers make tactical decisions
- Morale affects unit behavior
- Characters gain experience and improve
- Squad formations provide tactical benefits
- Medics can heal wounded soldiers
- All NPCs track and manage their own inventories

---

### Phase 6: Scale & Polish
**Target:** Full-scale battles and optimization
**Status:** Not Started

#### Planned Tasks
- [ ] Multi-z-level support (full implementation)
- [ ] Strategic/logistic layers
- [ ] Massive scale optimization (hundreds of units)
- [ ] Balance tuning
- [ ] Performance profiling and optimization
- [ ] Visual polish and clarity improvements
- [ ] Minimap implementation

#### Success Criteria
- Game handles 200+ active units smoothly
- Strategic layer provides command options
- Balanced gameplay between lethality and progression
- Smooth performance at target scale

---

## Long-term Considerations

### Potential Future Features
- Real-time mode (feasibility to be evaluated)
- Vehicle units (tanks, transports)
- Expanded historical scenarios
- Save/load system
- Campaign mode

### Technical Debt & Refactoring
_(To be tracked as development progresses)_

---

## Notes
- Prioritize core gameplay loop before advanced features
- Maintain modular code structure throughout
- Regular performance testing as scale increases
- Balance realism with fun factor

---

*Last updated: 2025-12-21*
