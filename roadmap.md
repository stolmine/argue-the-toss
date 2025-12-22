# Argue the Toss - Development Roadmap

## Project Status: Phase 3 - 100% Complete, Phase 6 - 50% Complete

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
**Status:** 95% Complete

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
- [x] Last-seen entity markers (ghost markers at last known positions)
- [x] Shared ally vision (players see what allies see)

#### Remaining Tasks
- [ ] Action/event subdivision for animation support
- [ ] Visual action indicators (muzzle flashes, movement trails)
- [ ] FOW mode options (configurable vision modes)

#### Implementation Details - Core Systems
- **Pathfinding:** A* with terrain costs, diagonal support, visual path preview (numbered steps)
- **Turn System:** 12s budget, time-based actions, multi-turn action tracking
- **UI:** Split-pane layout, event log (wrapping text), context info panel

#### Implementation Details - Vision & Tactical Systems
- **Vision Cone:** 120° main cone, 60° peripheral (dimmed), 180° blind spot
- **8-Direction Movement:** qweasdzxc layout with auto-facing
- **Time Scale:** Tiles = ~2m, movement 1.5s, rotation 0.3s, turn budget 12s
- **Last-Seen Markers:** Ghost markers at enemy last known positions (10 turn expiry)
- **Shared Vision:** Players see combined allied FOV with spotter attribution
- **Combat:** Hitscan weapons, 70% base accuracy, ammo/reload mechanics

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
- ✓ Manual movement overrides planned paths
- ✓ Automatic path execution (Space advances turn)
- ✓ Combat with LOS checks, shooting, reloading, ammo tracking
- ✓ 8-direction movement (qweasdzxc layout)
- ✓ Vision cone with directional awareness (120° main, 60° peripheral, 180° blind)
- ✓ Peripheral vision dimmed (50% brightness)
- ✓ Manual facing control (rotation with , . keys)
- ✓ Dead entities cannot move or act
- ✓ Last-seen enemy markers (ghost positions)
- ✓ Shared ally vision (combined FOV)
- ✓ Targeting mode with visual validation feedback
- Action subdivision for animations (not implemented)
- Visual action indicators (not implemented)
- Configurable FOW modes (not implemented)

---

### Phase 3: Environmental Systems
**Target:** Procedural battlefield generation, terrain types, and weather mechanics
**Status:** 100% Complete

#### Completed Tasks
- [x] Procedural battlefield generation system
  - [x] Intelligent trench network placement (front lines, support trenches, communication trenches)
  - [x] Town/village placement with building clusters
  - [x] Vegetation distribution (forests, hedgerows, individual trees)
  - [x] No-man's-land generation (shell craters, wire, debris)
  - [x] Spawn point placement logic (faction-specific starting positions)
  - [x] **Proper spawn zones with faction separation (75+ tiles apart)**
  - [x] **Dynamic spawn radius scaling with map size**
- [x] Barbed wire obstacles (movement impediment, vision blocking)
  - Note: **Needs tuning - barbed wire density too high**
- [x] Static emplacements
  - [x] Machine gun nests (defensive positions)
  - [x] Bunkers (defensive structures)
  - [x] Sandbag positions
- [x] Terrain types (mud, trenches, no-man's-land)
- [x] Additional terrain types (vegetation, tree, concrete, water)
- [x] Color coding for all terrain types
- [x] Buildings (multi-tile structures spanning multiple positions)
- [x] Movement cost based on terrain
- [x] Cover mechanics and calculations
- [x] **Full map configuration UI with all terrain parameters**
- [x] **Custom preset system with manual parameter override**

#### Remaining Tasks (Deferred to Phase 7)
- [ ] Static emplacements (continued)
  - [ ] Mortar pits
  - [ ] Artillery positions
  - [ ] Observation posts
- [ ] Weather effects (rain, fog, snow)
- [ ] Lighting system (day/night, flares)
- [ ] Z-level foundation (trenches, elevated positions)

#### Implementation Details - Terrain Generation
- **30+ Terrain Types:** NoMansLand, Grass, Mud, Water, TrenchFloor/Parapet/Ramp, Sandbags, Bunker, MgNest, BarbedWire, Tree, Forest, Hedge, Rubble, ShellCrater, CraterWater, BuildingWall/Floor/Door/Window, Ruins, Road, Path, CommTrench
- **Multi-tile Trenches:** 3-tile wide structures (Floor, Parapet, Ramp) with realistic layout
- **7-Phase Generation Algorithm:**
  1. Base layout (Perlin noise for natural terrain variation)
  2. Trench networks (multi-tile structures, front lines)
  3. Fortifications (bunkers, MG nests, sandbags)
  4. Environmental features (craters, forests)
  5. Buildings (procedural multi-tile structures)
  6. Tactical balancing (cover density analysis)
  7. Spawn point placement (faction-specific zones)
- **7 Historical Presets:** Verdun, Somme, Ypres, Tannenberg, Village, Urban, Open Field
- **Cover Bonus Integration:** Terrain provides combat bonuses (trenches, sandbags, buildings)
- **Configurable Parameters:** Seed, density levels, coverage percentages, side placement

#### Success Criteria - All Core Criteria Met ✓
- ✓ Procedural generation creates realistic WWI battlefields
- ✓ Trench networks have logical layout (front/support/communication trenches)
- ✓ Towns and vegetation placed with strategic considerations
- ✓ Spawn points positioned appropriately for each faction
- ✓ **Factions spawn at proper distance (75+ tiles apart)**
- ✓ **Spawn zones avoid water and impassable terrain**
- ✓ **Spawn distribution handles up to 500 soldiers per faction**
- ✓ Barbed wire creates tactical obstacles
- ✓ Static emplacements provide defensive positions
- ✓ Different terrain affects movement speed
- ✓ All terrain types have distinct colors and characters
- ✓ Buildings render as multi-tile structures
- ✓ Cover provides combat bonuses
- ✓ **All map parameters configurable in UI**
- ✓ **Custom preset for manual configuration**
- Weather affects visibility (deferred to Phase 7)
- Lighting changes visibility ranges (deferred to Phase 7)

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

### Phase 6: Game Structure & Objectives
**Target:** Mission system, game modes, and menu infrastructure
**Status:** 50% Complete

#### Completed Tasks
- [x] **Menu system**
  - [x] **Main menu (New Game, Load Game placeholder, Settings, Quit)**
  - [x] **New game configuration menu with presets**
  - [x] **Settings menu (turn order, time budget)**
  - [x] **Pause menu (in-game with ESC)**
  - [x] **Full map configuration interface**
  - [x] **Soldier count configuration (5-500 per team)**
- [x] **Basic objective system**
  - [x] **Flag capture mechanics (occupy for 5 turns)**
  - [x] **Two flags per map (one per faction)**
  - [x] **Visual flag rendering with faction colors**
  - [x] **Capture progress tracking**
  - [x] **Victory condition (capture all flags)**
  - [x] **AI seeks objectives when no enemies visible**

#### Remaining Tasks
- [ ] Objective system (continued)
  - [ ] Elimination objectives (kill all enemies, kill specific target)
  - [ ] Survival objectives (last X turns, protect unit)
  - [ ] Escort objectives (move unit to extraction point)
  - [ ] Intel objectives (reach location, investigate area)
- [ ] Game modes
  - [ ] Skirmish mode (single battle, customizable settings) - partially complete
  - [ ] Mission mode (objective-based scenarios)
  - [ ] Survival mode (endless waves, high score)
  - [ ] Historical scenarios (predefined WWI battles)
  - [ ] Custom mode (player-defined rules and victory conditions)
- [ ] Menu system (continued)
  - [ ] Load/Save game functionality
  - [ ] Mission select screen
  - [ ] After-action report (statistics, performance, casualties)
- [ ] Victory/defeat conditions (continued)
  - [ ] Loss condition detection and handling
  - [ ] Mission success/failure screens
  - [ ] Performance ratings (optional)
- [ ] Meta-progression (optional)
  - [ ] Persistent soldier roster across missions
  - [ ] Campaign structure linking missions
  - [ ] Unlockable scenarios or units

#### Success Criteria
- ✓ **Players have clear objectives to complete (flag capture)**
- ✓ **Basic victory condition implemented and detected**
- ✓ **Full menu system for navigation and settings**
- ✓ **Settings are configurable (turn order, time budget, map params)**
- ✓ **AI has tactical objectives beyond just hunting enemies**
- ✓ **Game launches through menus, not directly into gameplay**
- Multiple game modes available for variety (in progress)
- After-action reports show mission results (not implemented)
- Loss condition handling (not implemented)
- Meta-progression system (not implemented)

---

### Phase 7: Scale & Polish
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

*Last updated: 2025-12-21 (evening)*
