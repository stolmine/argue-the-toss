# Argue the Toss - Development Roadmap

## Project Status: Planning Phase

### Current Phase: Phase 0 - Project Setup
**Status:** In Progress

#### Completed
- [x] Initial project concept and design documentation
- [x] Technology stack selection (ratatui, bracket-lib, Specs ECS)
- [x] Feature set definition
- [x] Development phase planning
- [x] Project conventions established
- [x] Documentation structure created

#### In Progress
- [ ] Project initialization
- [ ] Development environment setup
- [ ] Initial Cargo.toml configuration
- [ ] Basic project structure creation

#### Upcoming
- [ ] Set up ratatui basic rendering
- [ ] Initialize Specs ECS framework
- [ ] Integrate bracket-lib utilities

---

## Development Phases

### Phase 1: Core Foundation
**Target:** Basic rendering and viewport system
**Status:** Not Started

#### Planned Tasks
- [ ] Basic ratatui viewport rendering
- [ ] Simple unit representation (ASCII characters)
- [ ] Camera/viewport system for panning
- [ ] Basic fog of war implementation
- [ ] Input handling system

#### Success Criteria
- Viewport displays grid-based battlefield
- Camera can pan across larger logical battlefield
- Units render as distinct characters
- Basic fog of war obscures unseen areas

---

### Phase 2: Essential Mechanics
**Target:** Core gameplay loop
**Status:** Not Started

#### Planned Tasks
- [ ] Movement system with pathfinding (bracket-pathfinding)
- [ ] Line-of-sight calculations (bracket-lib FOV)
- [ ] Basic combat (hitscan weapons)
- [ ] Action point system
- [ ] Turn-based game loop
- [ ] Basic UI panels (event log, stats)

#### Success Criteria
- Player can move units with pathfinding
- Combat resolves with line-of-sight checks
- Action points limit player actions per turn
- Event log displays combat results

---

### Phase 3: Environmental Systems
**Target:** Terrain and weather mechanics
**Status:** Not Started

#### Planned Tasks
- [ ] Terrain types (mud, trenches, no-man's-land)
- [ ] Movement cost based on terrain
- [ ] Cover mechanics and calculations
- [ ] Weather effects (rain, fog, snow)
- [ ] Lighting system (day/night, flares)
- [ ] Z-level foundation (trenches, elevated positions)

#### Success Criteria
- Different terrain affects movement speed
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

#### Success Criteria
- AI soldiers make tactical decisions
- Morale affects unit behavior
- Characters gain experience and improve
- Squad formations provide tactical benefits

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
