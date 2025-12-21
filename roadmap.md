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

#### In Progress Tasks
- [ ] Movement system with pathfinding (bracket-pathfinding)
- [ ] Line-of-sight calculations (bracket-lib FOV)
- [ ] Basic combat (hitscan weapons)
- [ ] Ammunition mechanic (reloading, running out of ammo)
- [ ] Time budget system (seconds per turn instead of abstract AP)
- [ ] Turn-based game loop
- [ ] Turn timescale determination (how much real-time per turn)
- [ ] Action/event subdivision for animation support (sub-turn phases)
- [ ] Basic UI panels (event log, stats)
- [ ] Targeting system completion (object pickup, enemy selection actions)
- [ ] Visual indication of entity actions (movement trails, firing indicators, grenade throws)
- [ ] Player vision cone system (CDDA-style directional FOV)
- [ ] Terrain dimming outside vision cone
- [ ] Last-seen entity markers (static ghosts of last known positions)
- [ ] FOW mode options (no FOW, friendly vision, player-only vision)

#### Success Criteria
- ✓ Viewport adapts to terminal size
- ✓ Modal UI with Command and Look modes
- ✓ Basic player movement with camera following
- ✓ Targeting cursor foundation (free cursor in Look mode)
- Player can move units with pathfinding
- Combat resolves with line-of-sight checks
- Weapons require reloading and can run out of ammunition
- Time budget system limits actions (e.g., 10 sec/turn budget)
- Actions have time costs (move: 2s, shoot: 3s, reload: 5s, etc.)
- Turn timescale defined (seconds/minutes per turn) and consistent
- Actions subdivided into phases for smooth animation
- Time remaining displayed clearly in UI
- Event log displays combat results
- Targeting cursor allows object/enemy selection actions
- Prior turn actions are visually indicated (who moved, fired, threw grenades)
- Vision cone emanates from player with directional awareness
- Terrain outside vision is dimmed/obscured
- Last-seen enemy positions marked with static indicators
- Multiple FOW modes available (configurable)

---

### Phase 3: Environmental Systems
**Target:** Terrain and weather mechanics
**Status:** Not Started

#### Planned Tasks
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
