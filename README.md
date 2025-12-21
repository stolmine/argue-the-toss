# Argue the Toss
## WW1 Trench Warfare Roguelike Battle Simulator

A turn-based TUI roguelike inspired by Running with Rifles, focusing on individual soldier perspective in massive-scale WWI trench warfare.

## Core Concept

- **Turn-based tactical combat** (potential real-time mode - feasibility TBD)
- **Massive scale simulation** - each character represents a single soldier
- **Individual perspective** with squad command and tactical elements
- **Text User Interface** built with ratatui

## Feature Set

### Combat & Warfare Systems
- **Projectiles**
  - Hitscan weapons (rifles, machine guns)
  - Physics-based projectiles (mortars, grenades, artillery)
- **Environmental Hazards**
  - Smoke effects
  - Gas warfare
  - Fire
- **Cover & Terrain**
  - Cover system mechanics
  - Multiple z-levels (trenches, elevated positions)
  - Movement cost based on terrain
  - Terrain deformation (shell craters, destroyed fortifications)
- **Armaments & Equipment**
  - Period-appropriate weapons
  - Inventory management
  - Equipment malfunction debuffs
- **Artillery & Vehicles**
  - Artillery support mechanics
  - Vehicle units (tanks, transports)
- **Emplacements**
  - Mortars
  - Machine Guns
  - Bunkers
  - Foxholes

### Tactical & Strategic Systems
- **Squad Management**
  - Formation control
  - Squad tactics
  - Rank system
- **Action Point System**
  - Turn-based action economy
  - Movement and combat costs
- **Strategic/Logistic Layers**
  - Higher-scale command options
  - Supply and reinforcement mechanics

### Simulation & AI
- **AI Opponents & Allies**
  - Individual soldier AI
  - Squad-level tactics
  - Command hierarchy simulation
- **Morale System**
  - Individual morale tracking
  - Aggregate unit morale
  - Morale-based behavior changes
- **Fog of War & Visibility**
  - Line-of-sight calculations
  - Lighting effects (day/night, flares, explosions)
  - Weather effects (rain, fog, snow)

### Character Systems
- **Stats**
  - Speed (movement rate)
  - Accuracy (shooting precision)
  - Constitution (health, resistance to disease)
  - Strength (melee, carrying capacity)
  - Will (morale resistance, shock recovery)
  - Endurance (fatigue resistance)
- **Debuffs**
  - Wounds (bleeding, broken bones)
  - Disease (trench foot, infections)
  - Shell shock
  - Equipment malfunction
- **Progression**
  - Experience and leveling
  - Skill improvements
  - Promotion through ranks

### Information & Events
- **Event System**
  - Local events (immediate vicinity)
  - Global events (sector/battlefield-wide)
  - Text readout log

### Technical Requirements
- **Object Tracking**
  - All extant entities (NPCs, objects)
  - Environmental state
  - Terrain modifications
  - Active effects (smoke, fire, gas)
- **Gameplay Balance**
  - Combat lethality vs. progression
  - Scale vs. control
  - Realism vs. fun

## UI/UX Design (Ratatui)

### Feasible Features
- Main battlefield viewport (ASCII/Unicode grid)
- Fog of war (cell-based visibility)
- Event log panel (scrollable text)
- Stats/inventory widgets (lists, tables)
- Action points & morale displays (bars/colors)
- Cover visualization (symbols/colors)
- Weather (Unicode overlays: ░▒▓ for rain, ❄ for snow)

### Challenging but Doable
- Lighting effects (layered style/brightness)
- Multiple z-levels (current level view or split elevation)
- Smoke/gas (animated ░▒▓ overlays with colors)
- Projectile animations (frame-by-frame mortars, tracer lines)
- Squad formations (unit highlighting, formation indicators)
- Minimap (compressed corner view)

### Potential Issues & Solutions
- **Massive scale vs terminal resolution** (~200x50 typical)
  - Solution: Viewport/camera system that pans across larger logical battlefield
  - Similar to Dwarf Fortress visible window approach

- **Multiple simultaneous animations** (projectiles + smoke + weather)
  - Solution: Coordinated rendering pipeline with frame update system

- **Visual layering** (terrain + cover + units + projectiles + fog + lighting + weather + smoke/gas)
  - Solution: Clear visual hierarchy to prevent clutter
  - Suggested layer order: base terrain → lighting → fog → cover → units → effects → projectiles

- **Real-time mode complexity**
  - Solution: Start turn-based, evaluate real-time feasibility later

### Proposed Layout
```
┌─ Battlefield (60x40) ─┬─ Status ─┐
│ [main viewport]       │ AP: ████  │
│ fog/lighting/weather  │ Morale: ↑ │
│ units/terrain/cover   │ Rank: Sgt │
│                       ├─ Squad ──┤
│                       │ Unit list │
│                       │ Formation │
├─ Event Log ───────────┴──────────┤
│ > Artillery incoming...           │
│ > Cpl. Smith wounded              │
└───────────────────────────────────┘
```

## Technology Stack

### Recommended Architecture
```
ratatui          // TUI rendering layer
   ↓
bracket-lib      // FOV, pathfinding, roguelike utilities
   ↓
specs ECS        // Entity/component management
   ↓
Custom sim logic // Combat, morale, physics, AI
```

### Core Libraries

**ratatui** - TUI Framework
- Complex multi-panel layouts
- Custom rendering control
- Real-time update capability
- Terminal input handling

**bracket-lib** - Roguelike Toolkit
- `bracket-pathfinding`: A*, Dijkstra algorithms (multi-threaded)
- FOV algorithms (multiple implementations)
- Multiple rendering backends
- Map generation utilities
- Terminal/console mode compatible

**Specs ECS** - Entity Component System
- Parallel entity processing
- Handles massive scale (hundreds/thousands of entities)
- Perfect for tracking all game objects
- Excellent integration with bracket-lib
- Proven in roguelike development

### Why This Stack

1. **Specs ECS** handles massive scale elegantly
   - Entities are just data with component tags
   - Systems process components in parallel
   - Easy to add/remove entities (spawn soldiers, projectiles, effects)

2. **bracket-pathfinding** solves critical algorithms
   - Unit movement pathfinding
   - Line-of-sight for shooting
   - Artillery arc calculations
   - Dijkstra maps for AI tactical decisions

3. **bracket-lib FOV** handles visibility
   - Fog of war implementation
   - Lighting effects
   - Multiple algorithm options (speed vs. precision trade-offs)

4. **ratatui** provides rendering flexibility
   - Full control over complex layered rendering
   - Custom widgets for specialized displays
   - Efficient update system for animations

5. **All pure Rust**
   - Good interoperability
   - Performance benefits
   - Strong type safety for complex simulation

### Alternative Minimal Approach
If bracket-lib feels too opinionated:
- **ratatui** - Rendering
- **hecs** or **bevy_ecs** - Lighter ECS alternatives
- **pathfinding** crate - Just algorithms, no framework
- Custom FOV implementation

## Development Approach

### Phase 1: Core Foundation
- Basic ratatui viewport rendering
- Simple unit representation
- Camera/viewport system
- Basic fog of war

### Phase 2: Essential Mechanics
- Movement with pathfinding
- Line-of-sight
- Basic combat (hitscan)
- Action point system

### Phase 3: Environmental Systems
- Terrain types and movement cost
- Cover mechanics
- Weather effects
- Lighting system

### Phase 4: Advanced Combat
- Physics projectiles (mortars, grenades)
- Artillery system
- Smoke and gas
- Fire mechanics

### Phase 5: Simulation Depth
- Morale system
- AI opponents and allies
- Squad formations
- Debuffs and progression

### Phase 6: Scale & Polish
- Multi-z-level support
- Strategic/logistic layers
- Balance tuning
- Performance optimization

## Design Philosophy

- **Start simple, layer complexity** - Get core gameplay loop working first
- **Avoid over-engineering** - Don't build abstractions before they're needed
- **Prioritize readability** - Visual clarity is critical in a TUI
- **Performance matters** - Massive scale requires efficient algorithms
- **Realism vs. fun** - Historical accuracy serves gameplay, not vice versa

## References

- [bracket-lib GitHub](https://github.com/amethyst/bracket-lib)
- [Roguelike Tutorial - In Rust](https://bfnightly.bracketproductions.com/)
- [bracket-pathfinding Documentation](https://lib.rs/crates/bracket-pathfinding)
- [Specs ECS](https://gitpiper.com/resources/rust/gamedevelopment/amethyst-specs)
- [ratatui Documentation](https://ratatui.rs/)
